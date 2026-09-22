enable atomic_vec2u_min_max;

struct Struct {
    atomic_scalar: atomic<vec2<u32>>,
    atomic_arr: array<atomic<vec2u>, 2>,
}

@group(0) @binding(0)
var<storage, read_write> storage_atomic_scalar: atomic<vec2u>;
@group(0) @binding(1)
var<storage, read_write> storage_atomic_arr: array<atomic<vec2u>, 2>;
@group(0) @binding(2)
var<storage, read_write> storage_struct: Struct;
@group(0) @binding(3)
var<uniform> input: vec2u;

@compute
@workgroup_size(2)
fn cs_main(@builtin(local_invocation_id) id: vec3<u32>) {
    atomicStoreMax(&storage_atomic_scalar, input);
    atomicStoreMax(&storage_atomic_arr[1], input + vec2u(1u, 0u));
    atomicStoreMax(&storage_struct.atomic_scalar, vec2u(1, 0));
    atomicStoreMax(&storage_struct.atomic_arr[1], vec2u(id.x, 0u));

    workgroupBarrier();

    atomicStoreMin(&storage_atomic_scalar, input);
    atomicStoreMin(&storage_atomic_arr[1], input + vec2u(1u, 0u));
    atomicStoreMin(&storage_struct.atomic_scalar, vec2u(1, 0));
    atomicStoreMin(&storage_struct.atomic_arr[1], vec2u(id.x, 0u));
}
