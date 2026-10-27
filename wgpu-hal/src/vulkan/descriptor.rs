use alloc::vec::Vec;
use arrayvec::ArrayVec;
use ash::vk;
use hashbrown::{HashMap, HashSet};

const POOL_MIN_SETS: u32 = 64;
const POOL_MAX_SETS: u32 = 512;

/// The dynamic counts are tracked separately since dynamic textures/buffers
/// have distinct descriptor types compared to their non-dynamic counterparts.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct DescriptorCounts {
    pub sampler: u32,
    pub sampled_image: u32,
    pub storage_image: u32,
    pub uniform_buffer: u32,
    pub uniform_buffer_dynamic: u32,
    pub storage_buffer: u32,
    pub storage_buffer_dynamic: u32,
    pub acceleration_structure: u32,
}

impl DescriptorCounts {
    fn total(&self) -> u32 {
        self.sampler
            + self.sampled_image
            + self.storage_image
            + self.uniform_buffer
            + self.uniform_buffer_dynamic
            + self.storage_buffer
            + self.storage_buffer_dynamic
            + self.acceleration_structure
    }
}

#[derive(Debug)]
pub struct DescriptorSet {
    raw: vk::DescriptorSet,
    bucket_key: BucketKey,
    pool_index: usize,
}

impl DescriptorSet {
    pub fn raw(&self) -> vk::DescriptorSet {
        self.raw
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct BucketKey {
    counts: DescriptorCounts,
    update_after_bind: bool,
}

/// Whether a [`Pool`] can serve allocations, and why not when it cannot.
///
/// Every state other than [`Ready`] exists for a driver that does not behave the way the Vulkan
/// specification requires. They are named for the workarounds that motivate them rather than for
/// some general notion of an unusable pool, because what the allocator may still do with the pool
/// differs between them.
///
/// [`Ready`]: PoolState::Ready
#[derive(Clone, Copy, Debug)]
enum PoolState {
    /// The pool can serve allocations.
    Ready,

    /// `vkAllocateDescriptorSets` failed with `VK_ERROR_FRAGMENTED_POOL` or
    /// `VK_ERROR_OUT_OF_POOL_MEMORY` even though every set allocated from this pool requests the
    /// same descriptor counts, which the specification says must not fail that way. Seen on Mali,
    /// where freeing sets does not make the pool usable again but resetting it does.
    ///
    /// [`DescriptorAllocator::free`] returns the pool to [`Ready`] once it has no live sets,
    /// which is the only point at which resetting it is safe.
    ///
    /// See <https://github.com/gfx-rs/wgpu/pull/10264>.
    ///
    /// [`Ready`]: PoolState::Ready
    FragmentedUntilReset,
}

struct Pool {
    raw: vk::DescriptorPool,
    capacity: u32,
    available: u32,
    state: PoolState,
}

impl Pool {
    /// Whether [`DescriptorAllocator::alloc`] may take a set from this pool.
    fn is_allocatable(&self) -> bool {
        matches!(self.state, PoolState::Ready) && self.available != 0
    }

    /// Whether this pool has no live sets, and so may be reset.
    fn is_empty(&self) -> bool {
        self.available == self.capacity
    }

    /// Whether this pool has been retired from allocation but could be returned to service by
    /// [`Pool::try_reset`].
    fn needs_reset(&self) -> bool {
        matches!(self.state, PoolState::FragmentedUntilReset)
    }

    /// This pool's handle, or `None` if the driver must never see it again.
    ///
    /// Every Vulkan entry point that takes this pool goes through here, so that a state in which
    /// the handle is unusable only has to be honored in one place rather than at each call site.
    /// No current state withholds the handle.
    fn raw_for_driver(&self) -> Option<vk::DescriptorPool> {
        match self.state {
            PoolState::Ready | PoolState::FragmentedUntilReset => Some(self.raw),
        }
    }

    /// How many of this pool's free sets count towards [`Bucket::available_sets`].
    ///
    /// A pool that cannot serve allocations contributes nothing, however many of its sets are
    /// free.
    fn available_contribution(&self) -> u32 {
        match self.state {
            PoolState::Ready => self.available,
            PoolState::FragmentedUntilReset => 0,
        }
    }

    /// Retire this pool because the driver reported a fragmentation failure that the Vulkan
    /// specification says cannot happen for these pools.
    fn mark_fragmented(&mut self) {
        assert!(matches!(self.state, PoolState::Ready));
        log::debug!(
            "Retiring fragmented descriptor pool ({}/{} sets free), retrying on another pool",
            self.available,
            self.capacity,
        );
        self.state = PoolState::FragmentedUntilReset;
    }

    /// Try to reset this pool. If resetting the pool succeeded, transition to [`PoolState::Ready`]
    /// and return `true`. Otherwise, return `false`.
    ///
    /// Resetting implicitly frees every set allocated from the pool, so this must be empty.
    ///
    /// A pool the driver must not see again can never become ready, so this reports `false` for
    /// one without calling into the driver.
    ///
    /// # Panics
    ///
    /// Panics if [`Pool::is_empty`] is `false`.
    unsafe fn try_reset(&mut self, device: &ash::Device) -> bool {
        assert!(self.is_empty());
        let Some(raw) = self.raw_for_driver() else {
            return false;
        };
        let reset =
            unsafe { device.reset_descriptor_pool(raw, vk::DescriptorPoolResetFlags::empty()) };
        if let Err(err) = reset {
            log::error!(
                "`vkResetDescriptorPool` failed unexpectedly, leaving the pool retired: {err}"
            );
            return false;
        }
        self.state = PoolState::Ready;
        true
    }

    /// Free `set` back to this pool.
    unsafe fn free_set(&mut self, device: &ash::Device, set: vk::DescriptorSet) {
        if let Some(raw) = self.raw_for_driver() {
            let result = unsafe { device.free_descriptor_sets(raw, core::slice::from_ref(&set)) };
            if let Err(err) = result {
                // vkFreeDescriptorSets is documented to return:
                // - VK_ERROR_UNKNOWN
                // - VK_ERROR_VALIDATION_FAILED (we shouldn't encounter this one)
                // wgpu-hal doesn't currently report errors in destroy functions.
                // Panic here for now. It might be ok to ignore the error but the
                // proper solution would probably be to lose the device.
                panic!("vkFreeDescriptorSets error: {err}, please report this error");
            }
        }
        self.available += 1;
    }

    unsafe fn destroy(self, device: &ash::Device) {
        if let Some(raw) = self.raw_for_driver() {
            unsafe { device.destroy_descriptor_pool(raw, None) };
        }
    }
}

/// Keeps track of all pools created with this bucket's [`BucketKey`].
#[derive(Default)]
struct Bucket {
    /// We keep track of all descriptor set layouts that might use this bucket.
    /// If the set becomes empty, the bucket is destroyed.
    layouts: HashSet<vk::DescriptorSetLayout>,
    pools: Vec<Pool>,
    allocated_sets: u32,
}

impl Bucket {
    fn create_pool(
        &mut self,
        device: &ash::Device,
        key: &BucketKey,
        capacity_hint: u32,
    ) -> Result<(usize, &mut Pool), crate::DeviceError> {
        let index = self.pools.len();
        let pool = create_descriptor_pool(device, key, capacity_hint)?;
        self.pools.push(pool);
        Ok((index, self.pools.last_mut().unwrap()))
    }

    /// How many sets [`DescriptorAllocator::alloc`] could serve without creating a new pool.
    ///
    /// Derived rather than cached, so that it cannot drift from the state of the pools it
    /// summarizes.
    fn available_sets(&self) -> u32 {
        self.pools.iter().map(Pool::available_contribution).sum()
    }
}

pub struct DescriptorAllocator {
    buckets: HashMap<BucketKey, Bucket>,
    max_update_after_bind_descriptors_in_all_pools: u32,
    update_after_bind_descriptors_in_all_pools: u32,
}

impl super::BindGroupLayout {
    fn bucket_key(&self) -> BucketKey {
        let update_after_bind = self.contains_binding_arrays;
        let counts = self.desc_count.clone();
        BucketKey {
            counts,
            update_after_bind,
        }
    }
}

impl DescriptorAllocator {
    pub fn new(max_update_after_bind_descriptors_in_all_pools: u32) -> Self {
        DescriptorAllocator {
            buckets: HashMap::default(),
            max_update_after_bind_descriptors_in_all_pools,
            update_after_bind_descriptors_in_all_pools: 0,
        }
    }

    pub fn register_layout(
        &mut self,
        device: &ash::Device,
        layout: &super::BindGroupLayout,
    ) -> Result<(), crate::DeviceError> {
        let key = layout.bucket_key();
        let bucket = match self.buckets.entry(key) {
            hashbrown::hash_map::Entry::Occupied(occupied_entry) => occupied_entry.into_mut(),
            hashbrown::hash_map::Entry::Vacant(vacant_entry) => {
                let mut bucket = Bucket::default();
                // Create at least 1 pool upfront instead of doing this when
                // creating the first bind group.
                bucket.create_pool(device, vacant_entry.key(), POOL_MIN_SETS)?;
                vacant_entry.insert(bucket)
            }
        };

        assert!(bucket.layouts.insert(layout.raw));

        Ok(())
    }

    pub fn unregister_layout(&mut self, device: &ash::Device, layout: &super::BindGroupLayout) {
        let key = layout.bucket_key();
        let bucket = self.buckets.get_mut(&key).unwrap();

        assert!(bucket.layouts.remove(&layout.raw));

        if bucket.layouts.is_empty() {
            // Remove the bucket and destroy any remaining pools.
            let bucket = self.buckets.remove(&key).unwrap();
            for pool in bucket.pools {
                assert_eq!(
                    pool.available, pool.capacity,
                    "pool is not empty, at least one DescriptorSet has not been freed"
                );
                unsafe { pool.destroy(device) };
            }
        }
    }

    pub unsafe fn alloc(
        &mut self,
        device: &ash::Device,
        layout: &super::BindGroupLayout,
    ) -> Result<DescriptorSet, crate::DeviceError> {
        let update_after_bind = layout.contains_binding_arrays;
        let total_descriptors = layout.desc_count.total();

        if update_after_bind
            && self.max_update_after_bind_descriptors_in_all_pools
                - self.update_after_bind_descriptors_in_all_pools
                < total_descriptors
        {
            return Err(crate::DeviceError::OutOfMemory);
        }

        let key = layout.bucket_key();
        let bucket = self.buckets.get_mut(&key).unwrap();
        let capacity_hint = bucket.allocated_sets;

        // Prefer smaller/older/fuller pools for new allocations to prevent
        // fragmentation and possibly fragmentation of hardware resources
        // (VK_ERROR_FRAGMENTATION)
        let mut candidate = bucket.pools.iter().position(Pool::is_allocatable);

        let (raw, pool_index) = loop {
            let index = match candidate {
                Some(index) => index,
                None => bucket.create_pool(device, &key, capacity_hint)?.0,
            };
            let created_pool = candidate.is_none();

            let Some(raw_pool) = bucket.pools[index].raw_for_driver() else {
                // `candidate` only ever names an allocatable pool, and a pool this function just
                // created is always `Ready`.
                unreachable!("tried to allocate from a descriptor pool the driver cannot be given");
            };
            let vk_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(raw_pool)
                .set_layouts(core::slice::from_ref(&layout.raw));

            // From https://docs.vulkan.org/refpages/latest/refpages/source/VkDescriptorPoolCreateInfo.html#_description
            // > if all sets allocated from the pool since it was created or
            // > most recently reset use the same number of descriptors
            // > (of each type) and the requested allocation also uses that
            // > same number of descriptors (of each type), then
            // > fragmentation must not cause an allocation failure
            //
            // These pools satisfy that condition, but some drivers (e.g.
            // Mali) fail such allocations anyway once a pool has served
            // a number of allocations; freeing sets does not make the
            // pool usable again, only resetting it does.
            //
            // Recover instead of failing: reset a pool that has no live sets,
            // otherwise mark the pool `fragmented` so that allocations skip
            // it. A fragmented pool is reset once its last set is freed.
            //
            // See https://github.com/gfx-rs/wgpu/pull/10264
            match unsafe { device.allocate_descriptor_sets(&vk_info) } {
                Ok(sets) => break (sets[0], index),
                Err(vk::Result::ERROR_FRAGMENTED_POOL | vk::Result::ERROR_OUT_OF_POOL_MEMORY) => {
                    if created_pool {
                        // A fresh pool failing to allocate even a single set leaves
                        // no pool left to fall back to, so drop it and give up.
                        let pool = bucket.pools.pop().unwrap();
                        unsafe { pool.destroy(device) };
                        log::error!(
                            "descriptor set allocation from a fresh pool failed unexpectedly"
                        );
                        return Err(crate::DeviceError::OutOfMemory);
                    }

                    // Resetting a pool restores the spec guarantee that
                    // fragmentation cannot cause an allocation failure, but it
                    // implicitly frees all of the pool's sets, so only reset a
                    // pool that has no live sets.
                    let pool = &mut bucket.pools[index];
                    if pool.is_empty() && unsafe { pool.try_reset(device) } {
                        match unsafe { device.allocate_descriptor_sets(&vk_info) } {
                            Ok(sets) => break (sets[0], index),
                            Err(
                                vk::Result::ERROR_FRAGMENTED_POOL
                                | vk::Result::ERROR_OUT_OF_POOL_MEMORY,
                            ) => {}
                            Err(err) => return Err(super::map_host_device_oom_err(err)),
                        }
                    }

                    bucket.pools[index].mark_fragmented();
                    candidate = bucket.pools.iter().position(Pool::is_allocatable);
                }
                Err(err) => return Err(super::map_host_device_oom_err(err)),
            }
        };

        let pool = &mut bucket.pools[pool_index];
        pool.available -= 1;
        bucket.allocated_sets += 1;
        if update_after_bind {
            self.update_after_bind_descriptors_in_all_pools += total_descriptors;
        }

        Ok(DescriptorSet {
            raw,
            bucket_key: key,
            pool_index,
        })
    }

    pub unsafe fn free(&mut self, device: &ash::Device, set: DescriptorSet) {
        let bucket = self.buckets.get_mut(&set.bucket_key).unwrap();
        let pool = bucket.pools.get_mut(set.pool_index).unwrap();

        unsafe { pool.free_set(device, set.raw()) };

        if pool.needs_reset() && pool.is_empty() {
            unsafe { pool.try_reset(device) };
        }
        bucket.allocated_sets -= 1;
        if set.bucket_key.update_after_bind {
            self.update_after_bind_descriptors_in_all_pools -= set.bucket_key.counts.total();
        }

        // Do not immediately destroy empty pools since they might have
        // been recently created.
        // Destroy a pool only if it's empty and we have 1/4th its capacity
        // in other pools.
        // Note that this logic will never destroy the last pool.
        let last = bucket.pools.last().unwrap();
        let available_in_other_pools = bucket.available_sets() - last.available_contribution();
        if last.available == last.capacity && available_in_other_pools > last.capacity / 4 {
            let pool = bucket.pools.pop().unwrap();
            unsafe { pool.destroy(device) };
        }
    }
}

impl Drop for DescriptorAllocator {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(
                self.buckets.is_empty(),
                "buckets are not empty, at least one BGL has not been unregistered"
            )
        }
    }
}

fn create_descriptor_pool(
    device: &ash::Device,
    key: &BucketKey,
    capacity_hint: u32,
) -> Result<Pool, crate::DeviceError> {
    let counts = &key.counts;

    const NR_OF_DESCRIPTOR_TYPES: usize = 8;

    use vk::DescriptorType as Dt;
    let counts: [_; NR_OF_DESCRIPTOR_TYPES] = [
        (Dt::SAMPLER, counts.sampler),
        (Dt::SAMPLED_IMAGE, counts.sampled_image),
        (Dt::STORAGE_IMAGE, counts.storage_image),
        (Dt::UNIFORM_BUFFER, counts.uniform_buffer),
        (Dt::UNIFORM_BUFFER_DYNAMIC, counts.uniform_buffer_dynamic),
        (Dt::STORAGE_BUFFER, counts.storage_buffer),
        (Dt::STORAGE_BUFFER_DYNAMIC, counts.storage_buffer_dynamic),
        (
            Dt::ACCELERATION_STRUCTURE_KHR,
            counts.acceleration_structure,
        ),
    ];

    // bounded doubling growth strategy
    let mut capacity = capacity_hint
        .clamp(POOL_MIN_SETS, POOL_MAX_SETS)
        .next_power_of_two();

    // Lower capacity to avoid overflowing `count * capacity` calculations
    // below. The resulting `capacity` might not be a power of 2 but that's
    // ok.
    for (_, count) in counts {
        capacity = (u32::MAX / count.max(1)).min(capacity);
    }

    let pool_sizes = counts
        .into_iter()
        .filter(|&(_, count)| count != 0)
        .map(|(ty, count)| vk::DescriptorPoolSize {
            ty,
            descriptor_count: count * capacity,
        })
        .collect::<ArrayVec<_, NR_OF_DESCRIPTOR_TYPES>>();

    // Prior to Vulkan 1.3.215 `poolSizeCount` was required to be non-zero, so
    // add a single small sampler entry if `pool_sizes` would otherwise be
    // empty.
    let mut pool_sizes = pool_sizes;
    if pool_sizes.is_empty() {
        pool_sizes.push(vk::DescriptorPoolSize {
            ty: Dt::SAMPLER,
            descriptor_count: 1,
        });
    }

    let mut flags = vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET;
    if key.update_after_bind {
        flags |= vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND;
    };

    let vk_info = vk::DescriptorPoolCreateInfo::default()
        .flags(flags)
        .max_sets(capacity)
        .pool_sizes(&pool_sizes);

    let raw = unsafe { device.create_descriptor_pool(&vk_info, None) }
        .map_err(super::map_host_device_oom_and_fragmentation_err)?;

    Ok(Pool {
        raw,
        capacity,
        available: capacity,
        state: PoolState::Ready,
    })
}
