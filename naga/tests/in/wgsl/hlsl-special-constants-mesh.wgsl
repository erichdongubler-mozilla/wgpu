// `num_workgroups` in task/mesh entry points, which use a nested wrapper in HLSL.

enable wgpu_mesh_shader;

struct TaskPayload {
    dummy: u32,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}
struct PrimitiveOutput {
    @builtin(triangle_indices) indices: vec3<u32>,
}

var<task_payload> taskPayload: TaskPayload;

@task
@payload(taskPayload)
@workgroup_size(64)
fn ts_main(@builtin(num_workgroups) n: vec3<u32>) -> @builtin(mesh_task_size) vec3<u32> {
    return n;
}

struct MeshOutput {
    @builtin(vertices) vertices: array<VertexOutput, 3>,
    @builtin(primitives) primitives: array<PrimitiveOutput, 1>,
    @builtin(vertex_count) vertex_count: u32,
    @builtin(primitive_count) primitive_count: u32,
}

var<workgroup> mesh_output: MeshOutput;

@mesh(mesh_output)
@payload(taskPayload)
@workgroup_size(64)
fn ms_main(@builtin(num_workgroups) n: vec3<u32>) { mesh_output.vertex_count = n.x; }
