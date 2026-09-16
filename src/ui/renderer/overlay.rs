use super::types::OverlayUniform;

pub struct OverlayComponents {
    pub pipeline: wgpu::RenderPipeline,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub fn create_overlay_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> OverlayComponents {
    let overlay_shader = device.create_shader_module(wgpu::include_wgsl!("../overlay.wgsl"));
    let overlay_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("overlay_bind_group_layout"),
        });

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Overlay Uniform Buffer"),
        size: std::mem::size_of::<OverlayUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &overlay_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("overlay_bind_group"),
    });

    let overlay_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Overlay Pipeline Layout"),
        bind_group_layouts: &[&overlay_bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Overlay Pipeline"),
        layout: Some(&overlay_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &overlay_shader,
            entry_point: "vs_overlay",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &overlay_shader,
            entry_point: "fs_overlay",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    OverlayComponents {
        pipeline,
        buffer,
        bind_group,
    }
}
