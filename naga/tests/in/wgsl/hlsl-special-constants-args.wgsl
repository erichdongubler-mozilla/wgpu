// Special-constant built-ins must be offset exactly once, including when
// read through helper functions or `let` bindings (wgpu#4564).

struct VIn {
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) ii: u32,
}

fn helper(v: VIn) -> u32 {
    return v.ii + v.vi;
}

@vertex
fn vs_helper(v: VIn) -> @builtin(position) vec4<f32> {
    return vec4<f32>(f32(helper(v)));
}

@vertex
fn vs_let(v: VIn) -> @builtin(position) vec4<f32> {
    let ii = v.ii;
    let a = ii + 1u;
    let b = ii + 2u;
    return vec4<f32>(f32(a), f32(b), 0.0, 1.0);
}

fn helper_wg(n: vec3<u32>) -> u32 { return n.x; }

@compute @workgroup_size(1)
fn cs(@builtin(num_workgroups) n: vec3<u32>) {
    _ = helper_wg(n);
}

struct CsIn {
    @builtin(num_workgroups) n: vec3<u32>,
    @builtin(local_invocation_index) lii: u32,
}

fn helper_cs_in(i: CsIn) -> u32 { return i.n.x + i.lii; }

@compute @workgroup_size(1)
fn cs_struct(i: CsIn) {
    _ = helper_cs_in(i);
}
