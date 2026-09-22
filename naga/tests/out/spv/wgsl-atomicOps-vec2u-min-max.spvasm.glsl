#version 460
#if defined(GL_ARB_gpu_shader_int64)
#extension GL_ARB_gpu_shader_int64 : require
#else
#error No extension available for 64-bit integers.
#endif
#extension GL_EXT_shader_atomic_int64 : require
layout(local_size_x = 2, local_size_y = 1, local_size_z = 1) in;

struct _8
{
    uint64_t _m0;
    uint64_t _m1[2];
};

layout(set = 0, binding = 0, std430) buffer _11_10
{
    uint64_t _m0;
} _10;

layout(set = 0, binding = 1, std430) buffer _14_13
{
    uint64_t _m0[2];
} _13;

layout(set = 0, binding = 2, std430) buffer _17_16
{
    _8 _m0;
} _16;

layout(set = 0, binding = 3, std140) uniform _20_19
{
    uvec2 _m0;
} _19;

void main()
{
    uint64_t _41 = atomicMax(_10._m0, packUint2x32(_19._m0));
    uint64_t _47 = atomicMax(_13._m0[1u], packUint2x32(_19._m0 + uvec2(1u, 0u)));
    uint64_t _50 = atomicMax(_16._m0._m0, packUint2x32(uvec2(1u, 0u)));
    uint64_t _55 = atomicMax(_16._m0._m1[1u], packUint2x32(uvec2(gl_LocalInvocationID.x, 0u)));
    barrier();
    uint64_t _60 = atomicMin(_10._m0, packUint2x32(_19._m0));
    uint64_t _64 = atomicMin(_13._m0[1u], packUint2x32(_19._m0 + uvec2(1u, 0u)));
    uint64_t _67 = atomicMin(_16._m0._m0, packUint2x32(uvec2(1u, 0u)));
    uint64_t _72 = atomicMin(_16._m0._m1[1u], packUint2x32(uvec2(gl_LocalInvocationID.x, 0u)));
}

