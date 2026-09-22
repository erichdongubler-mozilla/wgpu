enable atomic_vec2u_min_max;

struct Struct {
    atomic_scalar: atomic<vec2<u32>>,
    atomic_arr: array<atomic<vec2<u32>>, 2>,
}

@group(0) @binding(0)
var<storage, read_write> storage_atomic_scalar: atomic<vec2<u32>>;
@group(0) @binding(1)
var<storage, read_write> storage_atomic_arr: array<atomic<vec2<u32>>, 2>;
@group(0) @binding(2)
var<storage, read_write> storage_struct: Struct;
@group(0) @binding(3)
var<uniform> input: vec2<u32>;

@compute @workgroup_size(2, 1, 1)
fn cs_main(@builtin(local_invocation_id) id: vec3<u32>) {
    let _e3 = input;
    atomicStoreMax((&storage_atomic_scalar), _e3);
    let _e7 = input;
    atomicStoreMax((&storage_atomic_arr[1]), (_e7 + vec2<u32>(1u, 0u)));
    atomicStoreMax((&storage_struct.atomic_scalar), vec2<u32>(1u, 0u));
    atomicStoreMax((&storage_struct.atomic_arr[1]), vec2<u32>(id.x, 0u));
    workgroupBarrier();
    let _e25 = input;
    atomicStoreMin((&storage_atomic_scalar), _e25);
    let _e29 = input;
    atomicStoreMin((&storage_atomic_arr[1]), (_e29 + vec2<u32>(1u, 0u)));
    atomicStoreMin((&storage_struct.atomic_scalar), vec2<u32>(1u, 0u));
    atomicStoreMin((&storage_struct.atomic_arr[1]), vec2<u32>(id.x, 0u));
    return;
}
