const EXAMPLE_NAME: &str = "multiple_render_targets";

/// Renderer that draws its outputs to two output texture targets at the same time.
struct MultiTargetRenderer {
    pipeline: wgpu::RenderPipeline,
    bindgroup: wgpu::BindGroup,
}

fn create_ball_texture_data(width: usize, height: usize) -> Vec<u8> {
    // Creates black and white pixel data for the texture to sample.
    let mut img_data = Vec::with_capacity(width * height);
    let center: glam::Vec2 = glam::Vec2::new(width as f32 * 0.5, height as f32 * 0.5);
    let half_distance = width as f32 * 0.5;
    for y in 0..width {
        for x in 0..height {
            let cur_pos = glam::Vec2::new(x as f32, y as f32);
            let distance_to_center_normalized = 1.0 - (cur_pos - center).length() / half_distance;
            let val: u8 = (u8::MAX as f32 * distance_to_center_normalized) as u8;
            img_data.push(val)
        }
    }
    img_data
}

impl MultiTargetRenderer {
    fn create_image_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        const WIDTH: usize = 256;
        const HEIGHT: usize = 256;

        let size = wgpu::Extent3d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            ..
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("data texture"),
            size,
            format: wgpu::TextureFormat::R8Unorm, // we need only the red channel for black/white image,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
            ..
        });

        let ball_texture_data = &create_ball_texture_data(WIDTH, HEIGHT);

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                ..
            },
            ball_texture_data,
            wgpu::TexelCopyBufferLayout {
                bytes_per_row: Some(WIDTH as u32),
                rows_per_image: Some(HEIGHT as u32),
                ..
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("view"),
            ..
        });

        (texture, view)
    }

    fn init(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader: &wgpu::ShaderModule,
        target_states: &[Option<wgpu::ColorTargetState>],
    ) -> MultiTargetRenderer {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { .. },
                        ..
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(Default::default()),
                        ..
                    },
                ],
                label: None,
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[Some(&texture_bind_group_layout)],
            ..
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let (_, texture_view) = Self::create_image_texture(device, queue);

        let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            ..
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                ..
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_multi_main"),
                // IMPORTANT: specify the color states for the outputs:
                targets: target_states,
                ..
            }),
            ..
        });

        Self {
            pipeline,
            bindgroup,
        }
    }

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        targets: &[Option<wgpu::RenderPassColorAttachment>],
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: targets,
            ..
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bindgroup, &[]);
        rpass.draw(0..3, 0..1);
    }
}

/// Renderer that displays results on the screen.
struct TargetRenderer {
    pipeline: wgpu::RenderPipeline,
    bindgroup_layout: wgpu::BindGroupLayout,
    bindgroup_left: wgpu::BindGroup,
    bindgroup_right: wgpu::BindGroup,
    sampler: wgpu::Sampler,
}

impl TargetRenderer {
    fn init(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        targets: &TextureTargets,
    ) -> TargetRenderer {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { .. },
                        ..
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(Default::default()),
                        ..
                    },
                ],
                label: None,
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[Some(&texture_bind_group_layout)],
            ..
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                ..
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_display_main"),
                targets: &[Some(wgpu::ColorTargetState { format, .. })],
                ..
            }),
            ..
        });

        let (bg_left, bg_right) =
            Self::create_bindgroups(device, &texture_bind_group_layout, targets, &sampler);
        Self {
            pipeline: render_pipeline,
            bindgroup_layout: texture_bind_group_layout,
            bindgroup_left: bg_left,
            bindgroup_right: bg_right,
            sampler,
        }
    }
    fn create_bindgroups(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture_targets: &TextureTargets,
        sampler: &wgpu::Sampler,
    ) -> (wgpu::BindGroup, wgpu::BindGroup) {
        let left = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_targets.red_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: None,
        });

        let right = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_targets.green_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: None,
        });
        (left, right)
    }

    fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
                ..
            })],
            ..
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.bindgroup_left, &[]);

        let height = height as f32;
        let half_w = width as f32 * 0.5;

        // draw results in two separate viewports that split the screen:

        rpass.set_viewport(0.0, 0.0, half_w, height, 0.0, 1.0);
        rpass.draw(0..3, 0..1);

        rpass.set_viewport(half_w, 0.0, half_w, height, 0.0, 1.0);
        rpass.set_bind_group(0, &self.bindgroup_right, &[]);
        rpass.draw(0..3, 0..1);
    }

    fn rebuild_resources(&mut self, device: &wgpu::Device, texture_targets: &TextureTargets) {
        (self.bindgroup_left, self.bindgroup_right) = Self::create_bindgroups(
            device,
            &self.bindgroup_layout,
            texture_targets,
            &self.sampler,
        )
    }
}

struct TextureTargets {
    red_view: wgpu::TextureView,
    green_view: wgpu::TextureView,
}

impl TextureTargets {
    fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> TextureTargets {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let red_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            format,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[format],
            ..
        });
        let green_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            format,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[format],
            ..
        });
        let red_view = red_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..wgpu::TextureViewDescriptor::default()
        });
        let green_view = green_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            ..wgpu::TextureViewDescriptor::default()
        });
        TextureTargets {
            red_view,
            green_view,
        }
    }
}

struct Example {
    drawer: TargetRenderer,
    multi_target_renderer: MultiTargetRenderer,
    texture_targets: TextureTargets,
    screen_width: u32,
    screen_height: u32,
}

impl crate::framework::Example for Example {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
            ..
        });
        // Renderer that draws to 2 textures at the same time:
        let multi_target_renderer = MultiTargetRenderer::init(
            device,
            queue,
            &shader,
            // ColorTargetStates specify how the data will be written to the
            // output textures:
            &[
                Some(wgpu::ColorTargetState {
                    format: config.view_formats[0],
                    ..
                }),
                Some(wgpu::ColorTargetState {
                    format: config.view_formats[0],
                    ..
                }),
            ],
        );

        // create our target textures that will receive the simultaneous rendering:
        let texture_targets =
            TextureTargets::new(device, config.view_formats[0], config.width, config.height);

        // helper renderer that displays the results in 2 separate viewports:
        let drawer =
            TargetRenderer::init(device, &shader, config.view_formats[0], &texture_targets);

        Self {
            texture_targets,
            multi_target_renderer,
            drawer,
            screen_width: config.width,
            screen_height: config.height,
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.screen_width = config.width;
        self.screen_height = config.height;
        self.texture_targets =
            TextureTargets::new(device, config.view_formats[0], config.width, config.height);
        self.drawer.rebuild_resources(device, &self.texture_targets);
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {}

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // draw to 2 textures at the same time:
        self.multi_target_renderer.draw(
            &mut encoder,
            &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_targets.red_view,
                    ops: Default::default(),
                    ..
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_targets.green_view,
                    ops: Default::default(),
                    ..
                }),
            ],
        );

        // display results of the both drawn textures on screen:
        self.drawer
            .draw(&mut encoder, view, self.screen_width, self.screen_height);

        queue.submit(Some(encoder.finish()));
    }
}

pub fn main() {
    crate::framework::run::<Example>(EXAMPLE_NAME);
}

#[cfg(test)]
#[wgpu_test::gpu_test]
pub static TEST: crate::framework::ExampleTestParams = crate::framework::ExampleTestParams {
    name: EXAMPLE_NAME,
    image_path: "/examples/features/src/multiple_render_targets/screenshot.png",
    width: 1024,
    height: 768,
    // Bounded by lavapipe
    comparisons: &[wgpu_test::ComparisonType::Mean(0.014)], // Bounded by Apple A9
    _phantom: std::marker::PhantomData::<Example>,
    ..
};
