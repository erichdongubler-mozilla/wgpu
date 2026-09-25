use wgpu_test::{
    apply, gpu_test, image::ReadbackBuffers, GpuTestConfiguration, GpuTestInitializer,
    TestParameters, TestingContext,
};

pub fn all_tests(vec: &mut Vec<GpuTestInitializer>) {
    vec.push(DISCARD_THEN_LOOPS_NAN);
    vec.push(DISCARD_THEN_LOOPS_NAN_UNBOUNDED);
}

const SIZE: u32 = 256;
const FRAMES: u32 = 8;

/// WebKit found that six copies never reproduced the fault and nine always did.
const LOOPS: usize = 16;

const PROLOGUE: &str = r#"
@group(0) @binding(0) var<uniform> frame: u32;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let xy = vec2f(f32(i & 1u) * 4.0 - 1.0, f32(i >> 1u) * 4.0 - 1.0);
    return vec4f(xy, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    if (fract(pos.x * 0.37 + f32(frame) * 0.11) > 0.9) {
        discard;
    }
    // The second discard has to be nested for the fault to appear.
    if (pos.x > 1.0) {
        if (fract(pos.y * 0.29 + f32(frame) * 0.07) > 0.6) {
            discard;
        }
    }
    var p = pos.xy * 0.01;
    var sum = 0.0;
"#;

const LOOP_SNIPPET: &str = r#"    for (var k = 0; k < 3; k++) {
        sum += p.x * 0.001;
        p = p * 2.0 + 1.0;
    }
"#;

// `sum` reaches ~2e12, far outside f16 range. Scaling it back in keeps a
// non-finite texel attributable to corruption rather than to a legitimate
// overflow on backends that do not clamp when converting f32 to f16.
const EPILOGUE: &str = r#"    return vec4f(vec3f(sum * 1e-9), 1.0);
}
"#;

fn shader_source() -> String {
    let mut source = String::from(PROLOGUE);
    source.push_str(&LOOP_SNIPPET.repeat(LOOPS));
    source.push_str(EPILOGUE);
    source
}

/// A fragment shader with two `discard`s, the second nested inside an `if`, followed by a
/// series of small loops. Every texel must either keep the clear value, because the fragment
/// was discarded, or hold the finite colour the shader returns. On M1 the loops' scaffolding
/// interacts with the already-discarded fragment such that the colour comes out of stale
/// register contents, so NaN and Inf land in the attachment.
///
/// The arithmetic is only `+` and `*` over finite inputs, so the shader cannot produce a NaN
/// on its own: NaN requires `Inf - Inf`, `0 * Inf`, `Inf / Inf`, or an already-NaN operand.
/// Any NaN read back is therefore data that did not come from this shader.
///
/// See <https://github.com/gfx-rs/wgpu/issues/10446>.
#[apply(gpu_test!)]
static DISCARD_THEN_LOOPS_NAN: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(TestParameters::default())
    .run_async(|ctx| discard_then_loops_nan(ctx, true));

/// As [`DISCARD_THEN_LOOPS_NAN`], but with naga's loop bounding disabled.
///
/// WebKit's fix for the same symptom was to stop emitting their forward-progress counter on
/// Apple GPU family 7 and 8 (M1 and M2), which is the same lever as `force_loop_bounding`.
/// Comparing the two tests says whether that lever is the one that matters for wgpu.
#[apply(gpu_test!)]
static DISCARD_THEN_LOOPS_NAN_UNBOUNDED: GpuTestConfiguration = GpuTestConfiguration::new()
    .parameters(TestParameters::default())
    .run_async(|ctx| discard_then_loops_nan(ctx, false));

async fn discard_then_loops_nan(ctx: TestingContext, force_loop_bounding: bool) {
    let descriptor = wgpu::ShaderModuleDescriptor {
        label: Some("issue_10446"),
        source: wgpu::ShaderSource::Wgsl(shader_source().into()),
    };

    let shader = if force_loop_bounding {
        ctx.device.create_shader_module(descriptor)
    } else {
        // SAFETY: every loop in the shader has a constant trip count, so it cannot hang the
        // GPU with loop bounding disabled.
        unsafe {
            ctx.device.create_shader_module_trusted(
                descriptor,
                wgpu::ShaderRuntimeChecks {
                    force_loop_bounding: false,
                    ..wgpu::ShaderRuntimeChecks::checked()
                },
            )
        }
    };

    let uniform = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("frame"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let pipeline = ctx
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }],
    });

    let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
        size: wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let buffers = ReadbackBuffers::new(&ctx.device, &texture);

    for frame in 0..FRAMES {
        ctx.queue
            .write_buffer(&uniform, 0, bytemuck::bytes_of(&frame));

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rpass.set_pipeline(&pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..3, 0..1);
        }
        buffers.copy_from(&ctx.device, &mut encoder, &texture);
        ctx.queue.submit([encoder.finish()]);

        let data = buffers.retrieve(&ctx, wgpu::TextureAspect::All).await;

        let mut nan = 0_u32;
        let mut inf = 0_u32;
        let mut first_bad = None;
        // Rgba16Float, and the shader writes the same value to r, g and b.
        for (index, texel) in data.all().chunks_exact(8).enumerate() {
            let red = half::f16::from_bits(u16::from_le_bytes([texel[0], texel[1]]));
            if red.is_nan() {
                nan += 1;
            } else if red.is_infinite() {
                inf += 1;
            } else {
                continue;
            }
            first_bad.get_or_insert_with(|| {
                let index = index as u32;
                (index % SIZE, index / SIZE, red)
            });
        }

        assert!(
            nan == 0 && inf == 0,
            "frame {frame}: {nan} NaN and {inf} Inf texels out of {}, \
             first at (x: {}, y: {}) with value {}",
            SIZE * SIZE,
            first_bad.unwrap().0,
            first_bad.unwrap().1,
            first_bad.unwrap().2,
        );
    }
}
