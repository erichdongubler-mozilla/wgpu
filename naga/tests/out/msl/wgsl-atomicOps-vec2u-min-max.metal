// language: metal2.4
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct type_2 {
    metal::atomic_ulong inner[2];
};
struct Struct {
    metal::atomic_ulong atomic_scalar;
    type_2 atomic_arr;
};

struct cs_mainInput {
};
[[max_total_threads_per_threadgroup(2)]] kernel void cs_main(
  metal::uint3 id [[thread_position_in_threadgroup]]
, device metal::atomic_ulong& storage_atomic_scalar [[user(fake0)]]
, device type_2& storage_atomic_arr [[user(fake0)]]
, device Struct& storage_struct [[user(fake0)]]
, constant metal::uint2& input [[user(fake0)]]
) {
    metal::uint2 _e3 = input;
    metal::atomic_max_explicit(&storage_atomic_scalar, as_type<ulong>(_e3), metal::memory_order_relaxed);
    metal::uint2 _e7 = input;
    metal::atomic_max_explicit(&storage_atomic_arr.inner[1], as_type<ulong>(_e7 + metal::uint2(1u, 0u)), metal::memory_order_relaxed);
    metal::atomic_max_explicit(&storage_struct.atomic_scalar, as_type<ulong>(metal::uint2(1u, 0u)), metal::memory_order_relaxed);
    metal::atomic_max_explicit(&storage_struct.atomic_arr.inner[1], as_type<ulong>(metal::uint2(id.x, 0u)), metal::memory_order_relaxed);
    metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    metal::uint2 _e25 = input;
    metal::atomic_min_explicit(&storage_atomic_scalar, as_type<ulong>(_e25), metal::memory_order_relaxed);
    metal::uint2 _e29 = input;
    metal::atomic_min_explicit(&storage_atomic_arr.inner[1], as_type<ulong>(_e29 + metal::uint2(1u, 0u)), metal::memory_order_relaxed);
    metal::atomic_min_explicit(&storage_struct.atomic_scalar, as_type<ulong>(metal::uint2(1u, 0u)), metal::memory_order_relaxed);
    metal::atomic_min_explicit(&storage_struct.atomic_arr.inner[1], as_type<ulong>(metal::uint2(id.x, 0u)), metal::memory_order_relaxed);
    return;
}
