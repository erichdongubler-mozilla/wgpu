struct NagaConstants {
    int first_vertex;
    int first_instance;
    uint other;
};
ConstantBuffer<NagaConstants> _NagaConstants: register(b0, space1);

struct Struct {
    uint64_t atomic_scalar;
    uint64_t atomic_arr[2];
};

RWByteAddressBuffer storage_atomic_scalar : register(u0);
RWByteAddressBuffer storage_atomic_arr : register(u1);
RWByteAddressBuffer storage_struct : register(u2);
cbuffer input : register(b3) { uint2 input; }

[numthreads(2, 1, 1)]
void cs_main(uint3 id : SV_GroupThreadID)
{
    uint2 _e3 = input;
    uint2 atomic_value = _e3; storage_atomic_scalar.InterlockedMax64(0, (uint64_t(atomic_value.y) << 32) | uint64_t(atomic_value.x));
    uint2 _e7 = input;
    uint2 atomic_value_1 = (_e7 + uint2(1u, 0u)); storage_atomic_arr.InterlockedMax64(8, (uint64_t(atomic_value_1.y) << 32) | uint64_t(atomic_value_1.x));
    uint2 atomic_value_2 = uint2(1u, 0u); storage_struct.InterlockedMax64(0, (uint64_t(atomic_value_2.y) << 32) | uint64_t(atomic_value_2.x));
    uint2 atomic_value_3 = uint2(id.x, 0u); storage_struct.InterlockedMax64(8+8, (uint64_t(atomic_value_3.y) << 32) | uint64_t(atomic_value_3.x));
    GroupMemoryBarrierWithGroupSync();
    uint2 _e25 = input;
    uint2 atomic_value_4 = _e25; storage_atomic_scalar.InterlockedMin64(0, (uint64_t(atomic_value_4.y) << 32) | uint64_t(atomic_value_4.x));
    uint2 _e29 = input;
    uint2 atomic_value_5 = (_e29 + uint2(1u, 0u)); storage_atomic_arr.InterlockedMin64(8, (uint64_t(atomic_value_5.y) << 32) | uint64_t(atomic_value_5.x));
    uint2 atomic_value_6 = uint2(1u, 0u); storage_struct.InterlockedMin64(0, (uint64_t(atomic_value_6.y) << 32) | uint64_t(atomic_value_6.x));
    uint2 atomic_value_7 = uint2(id.x, 0u); storage_struct.InterlockedMin64(8+8, (uint64_t(atomic_value_7.y) << 32) | uint64_t(atomic_value_7.x));
    return;
}
