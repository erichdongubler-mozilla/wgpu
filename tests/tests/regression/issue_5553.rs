use wgpu_test::{gpu_test, GpuTestConfiguration};

use wgpu::*;

/// Previously, for every user-defined vertex output a fragment shader had to have a corresponding
/// user-defined input. This would generate `StageError::InputNotConsumed`.
///
/// This requirement was removed from the WebGPU spec. Now, when generating hlsl, wgpu will
/// automatically remove any user-defined outputs from the vertex shader that are not present in
/// the fragment inputs. This is necessary for generating correct hlsl:
/// https://github.com/gfx-rs/wgpu/issues/5553
#[gpu_test]
static ALLOW_INPUT_NOT_CONSUMED: GpuTestConfiguration =
    GpuTestConfiguration::new().run_async(|ctx| async move {
        let module = ctx
            .device
            .create_shader_module(include_wgsl!("issue_5553.wgsl"));

        let pipeline_layout = ctx.device.create_pipeline_layout(
            &PipelineLayoutDescriptor::builder()
                .label("Pipeline Layout")
                .bind_group_layouts(&[])
                .build(),
        );

        let targets = &[Some(
            ColorTargetState::builder()
                .format(TextureFormat::Rgba8Unorm)
                .build(),
        )];
        let _ = ctx.device.create_render_pipeline(
            &RenderPipelineDescriptor::builder()
                .label("Pipeline")
                .layout(&pipeline_layout)
                .vertex(
                    VertexState::from_module(&module)
                        .entry_point("vs_main")
                        .build(),
                )
                .primitive(Default::default())
                .maybe_depth_stencil(None)
                .multisample(Default::default())
                .fragment(
                    FragmentState::from_module(&module)
                        .entry_point("fs_main")
                        .targets(targets)
                        .build(),
                )
                .maybe_multiview(None)
                .maybe_cache(None)
                .build(),
        );
    });
