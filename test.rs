use pollster::FutureExt;

fn main() {
    let instance = wgpu::Instance::new(Default::default());

    let adapter1 = instance
        .request_adapter(&Default::default())
        .block_on()
        .unwrap();

    let (device1, queue1) = adapter1
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::TIMESTAMP_QUERY,
                ..Default::default()
            },
            None,
        )
        .block_on()
        .unwrap();

    let adapter2 = instance
        .request_adapter(&Default::default())
        .block_on()
        .unwrap();

    let (device2, queue2) = adapter2
        .request_device(&Default::default(), None)
        .block_on()
        .unwrap();

    let texture2 = device2.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            ..Default::default()
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let texture_view = texture2.create_view(&Default::default());

    let mut encoder2 = device2.create_command_encoder(&Default::default());

    let query_set1 = device1.create_query_set(&wgpu::QuerySetDescriptor {
        label: None,
        ty: wgpu::QueryType::Timestamp,
        count: 1,
    });

    {
        let render_pass = encoder2.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Discard,
                },
                resolve_target: None,
            })],
            timestamp_writes: Some(wgpu::RenderPassTimestampWrites {
                query_set: &query_set1,
                beginning_of_pass_write_index: None,
                end_of_pass_write_index: None,
            }),
            ..Default::default()
        });
    }
    // encoder.beginRenderPass({ colorAttachments: [{ view: texture_view, loadOp: "clear", storeOp: "discard" }], timestampWrites: { querySet: device2.createQuerySet({ type: "timestamp", count: 1 }) } })
}
