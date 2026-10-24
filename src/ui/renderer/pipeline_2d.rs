use std::collections::HashMap;
use std::path::Path;

use super::types::Avatar2dUniform;
use crate::ui::pose_system::AssistantVisualState;

pub struct Pipeline2dComponents {
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub textures: HashMap<AssistantVisualState, (wgpu::TextureView, wgpu::BindGroup, [f32; 2])>,
    pub loading_texture: Option<(wgpu::TextureView, wgpu::BindGroup, [f32; 2])>,
}

pub fn create_2d_pipeline(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    config: &wgpu::SurfaceConfiguration,
) -> Pipeline2dComponents {
    let avatar_2d_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("avatar2d.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../avatar2d.wgsl").into()),
    });

    let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        label: Some("avatar_2d_uniform_layout"),
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Avatar 2D Uniform Buffer"),
        size: std::mem::size_of::<Avatar2dUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("avatar_2d_uniform_bind_group"),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("avatar_2d_bind_group_layout"),
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Avatar 2D Pipeline Layout"),
        bind_group_layouts: &[&uniform_layout, &bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Avatar 2D Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &avatar_2d_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &avatar_2d_shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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

    let textures = load_2d_avatar_textures(device, queue, &bind_group_layout, &sampler);
    let loading_texture = load_loading_icon_texture(device, queue, &bind_group_layout, &sampler);

    Pipeline2dComponents {
        pipeline,
        uniform_buffer,
        uniform_bind_group,
        bind_group_layout,
        sampler,
        textures,
        loading_texture,
    }
}

pub fn load_loading_icon_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
) -> Option<(wgpu::TextureView, wgpu::BindGroup, [f32; 2])> {
    // 1. Try reading from file candidates on disk
    let candidates = [
        crate::assets::find_file_in_candidates("icon.png"),
        std::path::PathBuf::from("icons/icon.png"),
        std::path::PathBuf::from("icons/128x128.png"),
        std::path::PathBuf::from("assets/icon.png"),
    ];
    let mut image_data = None;
    for p in &candidates {
        if p.exists() {
            if let Ok(bytes) = std::fs::read(p) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    image_data = Some(img.to_rgba8());
                    break;
                }
            }
        }
    }

    // 2. Fallback: Embed icon direct from rust-embed or byte build
    if image_data.is_none() {
        if let Some(file) = crate::assets::EmbeddedAssets::get("icon.png") {
            if let Ok(img) = image::load_from_memory(file.data.as_ref()) {
                image_data = Some(img.to_rgba8());
            }
        }
    }

    if let Some(img) = image_data {
        let dims = img.dimensions();
        let tex_size = wgpu::Extent3d {
            width: dims.0,
            height: dims.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Loading_Icon_Texture"),
            size: tex_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            img.as_raw(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dims.0),
                rows_per_image: Some(dims.1),
            },
            tex_size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some("loading_icon_bind_group"),
        });
        return Some((view, bind_group, [dims.0 as f32, dims.1 as f32]));
    }
    None
}

pub fn load_image_to_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &Path,
    label: &str,
) -> Option<(wgpu::TextureView, [f32; 2])> {
    if !path.exists() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let dims = img.dimensions();
    let tex_size = wgpu::Extent3d {
        width: dims.0,
        height: dims.1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: tex_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        img.as_raw(),
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dims.0),
            rows_per_image: Some(dims.1),
        },
        tex_size,
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Some((view, [dims.0 as f32, dims.1 as f32]))
}

pub fn load_2d_avatar_textures(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
) -> HashMap<AssistantVisualState, (wgpu::TextureView, wgpu::BindGroup, [f32; 2])> {
    let mut map = HashMap::new();

    let states = [
        (
            AssistantVisualState::Idle,
            vec![
                "char-avatar-idle.png",
                "char-avatar-idle.jpg",
                "char-avatar-idle.svg",
            ],
        ),
        (
            AssistantVisualState::Listening,
            vec![
                "char-avatar-listening.png",
                "char-avatar-listening.jpg",
                "char-avatar-listening.svg",
            ],
        ),
        (
            AssistantVisualState::Speaking,
            vec![
                "char-avatar-speaking.png",
                "char-avatar-speaking.jpg",
                "char-avatar-speaking.svg",
            ],
        ),
        (
            AssistantVisualState::Thinking,
            vec![
                "char-avatar-thinking.png",
                "char-avatar-thinking.jpg",
                "char-avatar-thinking.svg",
            ],
        ),
    ];

    for (state, candidates) in states {
        let mut loaded = None;
        for c in &candidates {
            let path = crate::assets::find_file_in_candidates(c);
            if path.exists() {
                if let Some((view, size)) =
                    load_image_to_texture(device, queue, &path, &format!("Avatar2D_{:?}", state))
                {
                    loaded = Some((view, size));
                    break;
                }
            }
        }

        if loaded.is_none() && state != AssistantVisualState::Idle {
            let path = crate::assets::find_file_in_candidates("char-avatar-idle.png");
            if path.exists() {
                loaded = load_image_to_texture(
                    device,
                    queue,
                    &path,
                    &format!("Avatar2D_Fallback_{:?}", state),
                );
            }
        }

        if let Some((view, size)) = loaded {
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
                label: Some(&format!("avatar_2d_bind_group_{:?}", state)),
            });
            map.insert(state, (view, bind_group, size));
        }
    }

    map
}
