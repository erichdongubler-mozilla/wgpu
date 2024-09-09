use bytemuck::{Pod, Zeroable};
use std::mem::size_of;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
}

fn vertex(x: f32, y: f32) -> Vertex {
    Vertex {
        _pos: [x, y, 0.0, 1.0],
    }
}

struct Example {
    outer_vertex_buffer: wgpu::Buffer,
    mask_vertex_buffer: wgpu::Buffer,
    outer_pipeline: wgpu::RenderPipeline,
    mask_pipeline: wgpu::RenderPipeline,
    stencil_buffer: wgpu::Texture,
}

impl crate::framework::Example for Example {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_size = size_of::<Vertex>();
        let outer_vertices = [vertex(-1.0, -1.0), vertex(1.0, -1.0), vertex(0.0, 1.0)];
        let mask_vertices = [vertex(-0.5, 0.0), vertex(0.0, -1.0), vertex(0.5, 0.0)];

        let outer_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Outer Vertex Buffer"),
            contents: bytemuck::cast_slice(&outer_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mask_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mask Vertex Buffer"),
            contents: bytemuck::cast_slice(&mask_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor::builder()
                .bind_group_layouts(&[])
                .build(),
        );

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let vertex_buffers = [wgpu::VertexBufferLayout::builder()
            .array_stride(vertex_size as wgpu::BufferAddress)
            .attributes(&[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }])
            .build()];

        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState::from_module(&shader)
                .entry_point("vs_main")
                .buffers(&vertex_buffers)
                .build(),
            fragment: Some(
                wgpu::FragmentState::from_module(&shader)
                    .entry_point("fs_main")
                    .targets(&[Some(
                        wgpu::ColorTargetState::builder()
                            .format(config.view_formats[0])
                            .write_mask(wgpu::ColorWrites::empty())
                            .build(),
                    )])
                    .build(),
            ),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Always,
                        pass_op: wgpu::StencilOperation::Replace,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    ..Default::default()
                },
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        let outer_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState::from_module(&shader)
                .entry_point("vs_main")
                .buffers(&vertex_buffers)
                .build(),
            fragment: Some(
                wgpu::FragmentState::from_module(&shader)
                    .entry_point("fs_main")
                    .targets(&[Some(config.view_formats[0].into())])
                    .build(),
            ),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                stencil: wgpu::StencilState {
                    front: wgpu::StencilFaceState {
                        compare: wgpu::CompareFunction::Greater,
                        ..Default::default()
                    },
                    back: wgpu::StencilFaceState::IGNORE,
                    ..Default::default()
                },
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        let stencil_buffer = device.create_texture(
            &wgpu::TextureDescriptor::builder()
                .label("Stencil buffer")
                .size(wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                })
                .format(wgpu::TextureFormat::Stencil8)
                .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
                .build(),
        );

        // Done
        Example {
            outer_vertex_buffer,
            mask_vertex_buffer,
            outer_pipeline,
            mask_pipeline,
            stencil_buffer,
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        // empty
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.stencil_buffer = device.create_texture(
            &wgpu::TextureDescriptor::builder()
                .label("Stencil buffer")
                .size(wgpu::Extent3d {
                    width: config.width,
                    height: config.height,
                    depth_or_array_layers: 1,
                })
                .format(wgpu::TextureFormat::Stencil8)
                .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
                .build(),
        );
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let depth_view = self.stencil_buffer.create_view(&Default::default());
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: None,
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_stencil_reference(1);

            rpass.set_pipeline(&self.mask_pipeline);
            rpass.set_vertex_buffer(0, self.mask_vertex_buffer.slice(..));
            rpass.draw(0..3, 0..1);

            rpass.set_pipeline(&self.outer_pipeline);
            rpass.set_vertex_buffer(0, self.outer_vertex_buffer.slice(..));
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

pub fn main() {
    crate::framework::run::<Example>("stencil-triangles");
}

#[cfg(test)]
#[wgpu_test::gpu_test]
static TEST: crate::framework::ExampleTestParams = crate::framework::ExampleTestParams {
    name: "stencil-triangles",
    image_path: "/examples/src/stencil_triangles/screenshot.png",
    width: 1024,
    height: 768,
    optional_features: wgpu::Features::default(),
    base_test_parameters: wgpu_test::TestParameters::default(),
    comparisons: &[wgpu_test::ComparisonType::Mean(0.03)],
    _phantom: std::marker::PhantomData::<Example>,
};
