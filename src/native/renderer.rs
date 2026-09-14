use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::pose_system::{AssistantVisualState, PoseManager};
use super::vrm_loader::{Vertex, VrmModelData};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 3],
    _padding: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BoneMatricesUniform {
    matrices: [[[f32; 4]; 4]; 256],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaterialUniform {
    base_color_factor: [f32; 4],
    shade_color_factor: [f32; 4], // rgb + 0.0
    emissive_factor: [f32; 4],    // rgb + 0.0
    matcap_factor: [f32; 4],      // rgb + 0.0
    shading_shift: f32,
    shading_toony: f32,
    alpha_cutoff: f32,
    has_emissive: u32,
    has_matcap: u32,
    has_shade_tex: u32,
    _pad0: u32,
    _pad1: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Avatar2dUniform {
    window_size: [f32; 2],
    image_size: [f32; 2],
    camera_pan: [f32; 2],
    camera_zoom: f32,
    breath_offset: f32,
    _pad0: f32,
    _pad1: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayUniform {
    window_size: [f32; 2],
    cursor_pos: [f32; 2],
    is_visible: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

struct GpuPrimitive {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    material_index: usize,
}

struct GpuMaterial {
    bind_group: wgpu::BindGroup,
}

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    msaa_texture: wgpu::TextureView,
    depth_texture: wgpu::TextureView,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    bone_buffer: wgpu::Buffer,
    bone_bind_group: wgpu::BindGroup,

    model_data: Option<VrmModelData>,
    gpu_primitives: Vec<GpuPrimitive>,
    gpu_materials: Vec<GpuMaterial>,

    // Referensi bounding box & framing persis Three.js
    model_min_y: f32,
    model_max_y: f32,
    model_max_extent_x: f32,

    pub anchor_center_x: f32,
    pub anchor_center_y: f32,

    // Kontrol POV & Posisi Kamera di Layar Fullscreen
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_zoom: f32,
    pub camera_pan_x: f32,
    pub camera_pan_y: f32,

    // Mode Avatar & Visual State
    pub avatar_type: String,
    pub current_visual_state: AssistantVisualState,
    pub elapsed_time: f32,
    avatar_2d_pipeline: wgpu::RenderPipeline,
    avatar_2d_uniform_buffer: wgpu::Buffer,
    avatar_2d_uniform_bind_group: wgpu::BindGroup,
    avatar_2d_bind_group_layout: wgpu::BindGroupLayout,
    avatar_2d_sampler: wgpu::Sampler,
    avatar_2d_textures: HashMap<AssistantVisualState, (wgpu::TextureView, wgpu::BindGroup, [f32; 2])>,

    // CSD Header & Controls
    pub is_header_visible: bool,
    pub cursor_pixel_pos: [f32; 2],
    overlay_pipeline: wgpu::RenderPipeline,
    overlay_buffer: wgpu::Buffer,
    overlay_bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Gagal menemukan adapter GPU yang cocok"))?;

        println!("🎮 [WGPU Adapter] Menggunakan GPU: {}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: Some("Ragasukma Device"),
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let alpha_mode = surface_caps
            .alpha_modes
            .iter()
            .copied()
            .find(|m| {
                *m == wgpu::CompositeAlphaMode::PreMultiplied
                    || *m == wgpu::CompositeAlphaMode::PostMultiplied
                    || *m == wgpu::CompositeAlphaMode::Inherit
            })
            .unwrap_or(surface_caps.alpha_modes[0]);

        println!("🔍 [WGPU Surface] Formats: {:?}, Alpha Modes: {:?}", surface_caps.formats, surface_caps.alpha_modes);
        println!("✨ [WGPU Surface] Alpha Mode Aktif: {:?}", alpha_mode);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("camera_bind_group_layout"),
        });

        let bone_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("bone_bind_group_layout"),
        });

        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                // 0: MaterialUniform
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 1: Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // 2: Base Color Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 3: Shade Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 4: Emissive Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                // 5: Matcap Texture
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
            ],
            label: Some("material_bind_group_layout"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_bind_group_layout,
                &bone_bind_group_layout,
                &material_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("MToon Anime Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: 24,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 32,
                            shader_location: 3,
                            format: wgpu::VertexFormat::Uint32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: 48,
                            shader_location: 4,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
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
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
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

        let initial_camera = CameraUniform {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0, 0.85, 2.5],
            _padding: 0.0,
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[initial_camera]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let initial_bones = BoneMatricesUniform {
            matrices: [Mat4::IDENTITY.to_cols_array_2d(); 256],
        };
        let bone_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bone Buffer"),
            contents: bytemuck::cast_slice(&[initial_bones]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bone_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bone_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: bone_buffer.as_entire_binding(),
            }],
            label: Some("bone_bind_group"),
        });

        // Overlay Pipeline untuk Top Titlebar & Tombol Minimize, Maximize, Close
        let overlay_shader = device.create_shader_module(wgpu::include_wgsl!("overlay.wgsl"));
        let overlay_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let overlay_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Overlay Uniform Buffer"),
            size: std::mem::size_of::<OverlayUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let overlay_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &overlay_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: overlay_buffer.as_entire_binding(),
            }],
            label: Some("overlay_bind_group"),
        });

        let overlay_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Overlay Pipeline Layout"),
            bind_group_layouts: &[&overlay_bind_group_layout],
            push_constant_ranges: &[],
        });

        let overlay_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        // -------------------------------------------------------------
        // Setup 2D Avatar Sprite Pipeline & Uniforms
        // -------------------------------------------------------------
        let avatar_2d_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("avatar2d.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("avatar2d.wgsl").into()),
        });

        let avatar_2d_uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let avatar_2d_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Avatar 2D Uniform Buffer"),
            size: std::mem::size_of::<Avatar2dUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let avatar_2d_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &avatar_2d_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: avatar_2d_uniform_buffer.as_entire_binding(),
            }],
            label: Some("avatar_2d_uniform_bind_group"),
        });

        let avatar_2d_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let avatar_2d_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let avatar_2d_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Avatar 2D Pipeline Layout"),
            bind_group_layouts: &[&avatar_2d_uniform_layout, &avatar_2d_bind_group_layout],
            push_constant_ranges: &[],
        });

        let avatar_2d_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Avatar 2D Pipeline"),
            layout: Some(&avatar_2d_pipeline_layout),
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

        let avatar_2d_textures = Self::load_2d_avatar_textures(
            &device,
            &queue,
            &avatar_2d_bind_group_layout,
            &avatar_2d_sampler,
        );

        let msaa_texture = Self::create_msaa_view(&device, &config);
        let depth_texture = Self::create_depth_view(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            msaa_texture,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            bone_buffer,
            bone_bind_group,
            model_data: None,
            gpu_primitives: Vec::new(),
            gpu_materials: Vec::new(),
            model_min_y: 0.0,
            model_max_y: 1.6,
            model_max_extent_x: 0.7,
            anchor_center_x: 135.0,
            anchor_center_y: 240.0,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_zoom: 1.0,
            camera_pan_x: 0.0,
            camera_pan_y: 0.0,
            avatar_type: "vrm".to_string(),
            current_visual_state: AssistantVisualState::Idle,
            elapsed_time: 0.0,
            avatar_2d_pipeline,
            avatar_2d_uniform_buffer,
            avatar_2d_uniform_bind_group,
            avatar_2d_bind_group_layout,
            avatar_2d_sampler,
            avatar_2d_textures,
            is_header_visible: false,
            cursor_pixel_pos: [-100.0, -100.0],
            overlay_pipeline,
            overlay_buffer,
            overlay_bind_group,
        })
    }

    pub fn reload_2d_avatar_textures(&mut self) {
        self.avatar_2d_textures = Self::load_2d_avatar_textures(
            &self.device,
            &self.queue,
            &self.avatar_2d_bind_group_layout,
            &self.avatar_2d_sampler,
        );
    }

    fn load_image_to_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &std::path::Path,
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

    fn load_2d_avatar_textures(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
    ) -> HashMap<AssistantVisualState, (wgpu::TextureView, wgpu::BindGroup, [f32; 2])> {
        let mut map = HashMap::new();

        let states = [
            (AssistantVisualState::Idle, vec!["char-avatar-idle.png", "char-avatar-idle.jpg", "char-avatar-idle.svg"]),
            (AssistantVisualState::Listening, vec!["char-avatar-listening.png", "char-avatar-listening.jpg", "char-avatar-listening.svg"]),
            (AssistantVisualState::Speaking, vec!["char-avatar-talking.png", "char-avatar-talking.jpg", "char-avatar-talking.svg", "char-avatar-speaking.png"]),
            (AssistantVisualState::Thinking, vec!["char-avatar-thinking.png", "char-avatar-thinking.jpg", "char-avatar-thinking.svg"]),
        ];

        for (state, candidates) in states {
            let mut loaded = None;
            for c in &candidates {
                let path = super::server::find_file_in_candidates(c);
                if path.exists() {
                    if let Some((view, size)) = Self::load_image_to_texture(device, queue, &path, &format!("Avatar2D_{:?}", state)) {
                        loaded = Some((view, size));
                        break;
                    }
                }
            }
            // Fallback ke Idle jika slot tertentu belum ada gambarnya
            if loaded.is_none() && state != AssistantVisualState::Idle {
                let path = super::server::find_file_in_candidates("char-avatar-idle.png");
                if path.exists() {
                    loaded = Self::load_image_to_texture(device, queue, &path, &format!("Avatar2D_Fallback_{:?}", state));
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

    pub fn update_anchor_on_resize(&mut self, old_w: f32, old_h: f32, new_w: f32, new_h: f32, dir: Option<winit::window::ResizeDirection>) {
        if let Some(d) = dir {
            use winit::window::ResizeDirection::*;
            match d {
                West | NorthWest | SouthWest => {
                    self.anchor_center_x += new_w - old_w;
                }
                East | NorthEast | SouthEast => {
                    // Tarik sisi kanan: geser anchor secara halus agar karakter mengikuti posisi tengah horizontal
                    self.anchor_center_x += (new_w - old_w) * 0.5;
                }
                _ => {}
            }
            match d {
                North | NorthWest | NorthEast => {
                    self.anchor_center_y += new_h - old_h;
                }
                _ => {}
            }
        }
        // Pastikan anchor tidak pernah lari ke luar batas window
        self.anchor_center_x = self.anchor_center_x.clamp(new_w * 0.1, new_w * 0.9);
        self.anchor_center_y = self.anchor_center_y.clamp(new_h * 0.1, new_h * 0.9);
    }

    fn create_msaa_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("MSAA Multisample Texture"),
            size: wgpu::Extent3d {
                width: config.width.max(1),
                height: config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn create_depth_view(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width.max(1),
                height: config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if self.size == new_size && self.config.width == new_size.width && self.config.height == new_size.height {
                return;
            }
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.msaa_texture = Self::create_msaa_view(&self.device, &self.config);
            self.depth_texture = Self::create_depth_view(&self.device, &self.config);
        }
    }

    fn fast_downscale_half(src: &[u8], w: usize, h: usize) -> Vec<u8> {
        let new_w = (w / 2).max(1);
        let new_h = (h / 2).max(1);
        let mut dst = vec![0u8; new_w * new_h * 4];

        for y in 0..new_h {
            let src_y1 = (y * 2).min(h - 1);
            let src_y2 = (y * 2 + 1).min(h - 1);
            for x in 0..new_w {
                let src_x1 = (x * 2).min(w - 1);
                let src_x2 = (x * 2 + 1).min(w - 1);

                let i00 = (src_y1 * w + src_x1) * 4;
                let i10 = (src_y1 * w + src_x2) * 4;
                let i01 = (src_y2 * w + src_x1) * 4;
                let i11 = (src_y2 * w + src_x2) * 4;

                let out_idx = (y * new_w + x) * 4;
                for c in 0..4 {
                    let sum = src[i00 + c] as u32 + src[i10 + c] as u32 + src[i01 + c] as u32 + src[i11 + c] as u32;
                    dst[out_idx + c] = (sum / 4) as u8;
                }
            }
        }
        dst
    }

    fn create_texture_from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img_opt: Option<&image::RgbaImage>,
        fallback_rgba: [u8; 4],
        label: &str,
    ) -> wgpu::TextureView {
        if let Some(img) = img_opt {
            let dimensions = img.dimensions();
            let max_dim = dimensions.0.max(dimensions.1);
            // Batasi hingga 5 level mipmap (cukup dari 2048px hingga 128px) untuk performa & memori instan
            let calculated_mips = (max_dim as f32).log2().floor() as u32 + 1;
            let mip_level_count = calculated_mips.min(6);

            let texture_size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: texture_size,
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            // Tulis Mip Level 0 (Ukuran Asli)
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
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                texture_size,
            );

            // Generate rantai Mipmap dengan Fast 2x2 Box Filter instan
            let mut current_bytes = img.as_raw().clone();
            let mut cur_w = dimensions.0 as usize;
            let mut cur_h = dimensions.1 as usize;

            for mip in 1..mip_level_count {
                current_bytes = Self::fast_downscale_half(&current_bytes, cur_w, cur_h);
                cur_w = (cur_w / 2).max(1);
                cur_h = (cur_h / 2).max(1);

                let mip_size = wgpu::Extent3d {
                    width: cur_w as u32,
                    height: cur_h as u32,
                    depth_or_array_layers: 1,
                };

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &texture,
                        mip_level: mip,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &current_bytes,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * cur_w as u32),
                        rows_per_image: Some(cur_h as u32),
                    },
                    mip_size,
                );
            }

            texture.create_view(&wgpu::TextureViewDescriptor::default())
        } else {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
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
                &fallback_rgba,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            );
            texture.create_view(&wgpu::TextureViewDescriptor::default())
        }
    }

    pub fn load_vrm_model(&mut self, mut model: VrmModelData) {
        let material_bind_group_layout = self.render_pipeline.get_bind_group_layout(2);
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Hitung bounding box akurat model (X dan Y)
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for prim in &model.primitives {
            for v in &prim.vertices {
                min_x = min_x.min(v.position[0]);
                max_x = max_x.max(v.position[0]);
                min_y = min_y.min(v.position[1]);
                max_y = max_y.max(v.position[1]);
            }
        }

        if min_y < max_y && min_x < max_x {
            self.model_min_y = min_y;
            self.model_max_y = max_y;
            self.model_max_extent_x = min_x.abs().max(max_x.abs());
            println!("📐 [Model Bounds] Y: [{:.3} .. {:.3}], Extent X: {:.3}", min_y, max_y, self.model_max_extent_x);
        }

        let dummy_white_view = Self::create_texture_from_rgba(
            &self.device,
            &self.queue,
            None,
            [255, 255, 255, 255],
            "Shared_Dummy_White",
        );

        let dummy_black_view = Self::create_texture_from_rgba(
            &self.device,
            &self.queue,
            None,
            [0, 0, 0, 255],
            "Shared_Dummy_Black",
        );

        let mut gpu_materials = Vec::new();
        let mut gpu_texture_cache: HashMap<usize, std::sync::Arc<wgpu::TextureView>> = HashMap::new();

        let get_or_create_gpu_view = |device: &wgpu::Device,
                                      queue: &wgpu::Queue,
                                      img_opt: Option<std::sync::Arc<image::RgbaImage>>,
                                      default_color: [u8; 4],
                                      label: &str,
                                      cache: &mut HashMap<usize, std::sync::Arc<wgpu::TextureView>>|
         -> Option<std::sync::Arc<wgpu::TextureView>> {
            if let Some(arc_img) = img_opt {
                let key = std::sync::Arc::as_ptr(&arc_img) as usize;
                if let Some(view) = cache.get(&key) {
                    return Some(view.clone());
                }
                let view = std::sync::Arc::new(Self::create_texture_from_rgba(
                    device,
                    queue,
                    Some(&arc_img),
                    default_color,
                    label,
                ));
                cache.insert(key, view.clone());
                Some(view)
            } else {
                None
            }
        };

        for mat in &mut model.materials {
            let base_img = mat.base_color_texture.take();
            let shade_img = mat.shade_texture.take();
            let emissive_img = mat.emissive_texture.take();
            let matcap_img = mat.matcap_texture.take();

            let has_shade_tex = shade_img.is_some();
            let has_emissive_tex = emissive_img.is_some();
            let has_matcap_tex = matcap_img.is_some();

            let custom_base_view = get_or_create_gpu_view(
                &self.device,
                &self.queue,
                base_img,
                [255, 255, 255, 255],
                &format!("Base_{}", mat.name),
                &mut gpu_texture_cache,
            );
            let base_view_ref: &wgpu::TextureView = custom_base_view.as_deref().unwrap_or(&dummy_white_view);

            let custom_shade_view = get_or_create_gpu_view(
                &self.device,
                &self.queue,
                shade_img,
                [255, 255, 255, 255],
                &format!("Shade_{}", mat.name),
                &mut gpu_texture_cache,
            );
            let shade_view_ref: &wgpu::TextureView = custom_shade_view.as_deref().unwrap_or(&dummy_white_view);

            let custom_emissive_view = get_or_create_gpu_view(
                &self.device,
                &self.queue,
                emissive_img,
                [0, 0, 0, 255],
                &format!("Emissive_{}", mat.name),
                &mut gpu_texture_cache,
            );
            let emissive_view_ref: &wgpu::TextureView = custom_emissive_view.as_deref().unwrap_or(&dummy_black_view);

            let custom_matcap_view = get_or_create_gpu_view(
                &self.device,
                &self.queue,
                matcap_img,
                [0, 0, 0, 255],
                &format!("Matcap_{}", mat.name),
                &mut gpu_texture_cache,
            );
            let matcap_view_ref: &wgpu::TextureView = custom_matcap_view.as_deref().unwrap_or(&dummy_black_view);

            let is_emissive_active = has_emissive_tex
                && (mat.emissive_factor[0] > 0.001
                    || mat.emissive_factor[1] > 0.001
                    || mat.emissive_factor[2] > 0.001);

            let mat_uniform = MaterialUniform {
                base_color_factor: mat.base_color_factor,
                shade_color_factor: [
                    mat.shade_color_factor[0],
                    mat.shade_color_factor[1],
                    mat.shade_color_factor[2],
                    0.0,
                ],
                emissive_factor: [
                    mat.emissive_factor[0],
                    mat.emissive_factor[1],
                    mat.emissive_factor[2],
                    0.0,
                ],
                matcap_factor: [
                    mat.matcap_factor[0],
                    mat.matcap_factor[1],
                    mat.matcap_factor[2],
                    0.0,
                ],
                shading_shift: mat.shading_shift,
                shading_toony: mat.shading_toony,
                alpha_cutoff: mat.alpha_cutoff,
                has_emissive: if is_emissive_active { 1 } else { 0 },
                has_matcap: if has_matcap_tex { 1 } else { 0 },
                has_shade_tex: if has_shade_tex { 1 } else { 0 },
                _pad0: 0,
                _pad1: 0,
            };

            let mat_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Material Uniform Buffer"),
                contents: bytemuck::cast_slice(&[mat_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: mat_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(base_view_ref),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(shade_view_ref),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(emissive_view_ref),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(matcap_view_ref),
                    },
                ],
                label: Some("material_bind_group"),
            });

            gpu_materials.push(GpuMaterial { bind_group });
        }

        let mut gpu_primitives = Vec::new();
        for prim in &mut model.primitives {
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&prim.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&prim.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let index_count = prim.indices.len() as u32;

            // Free CPU vertex data karena sudah di VRAM GPU
            prim.vertices.clear();
            prim.vertices.shrink_to_fit();
            prim.indices.clear();
            prim.indices.shrink_to_fit();

            gpu_primitives.push(GpuPrimitive {
                vertex_buffer,
                index_buffer,
                index_count,
                material_index: prim.material_index.unwrap_or(0),
            });
        }

        // Free semua CPU data material & primitive karena sudah berada di VRAM GPU
        model.materials.clear();
        model.materials.shrink_to_fit();
        model.primitives.clear();
        model.primitives.shrink_to_fit();

        self.gpu_materials = gpu_materials;
        self.gpu_primitives = gpu_primitives;
        self.model_data = Some(model);
    }

    pub fn update_camera_and_bones(&mut self, pose_manager: &mut PoseManager, delta_seconds: f32) {
        pose_manager.update_poses(delta_seconds);

        let width = self.config.width as f32;
        let height = (self.config.height as f32).max(1.0);
        let aspect = width / height;

        let fov_rad = 28.0_f32.to_radians();

        // 1. Titik tengah dan tinggi aktual model
        let center_y = (self.model_min_y + self.model_max_y) * 0.5;
        let model_height = (self.model_max_y - self.model_min_y).max(1.2);

        // Margin standar agar avatar pas di dalam frame viewport
        let safe_margin = 1.12_f32;
        let fit_height = model_height * safe_margin;

        let dist_v = (fit_height / 2.0) / (fov_rad / 2.0).tan();
        let radius = dist_v / self.camera_zoom.clamp(0.15, 6.0);

        // 2. Transformasi Kamera Orbit RH Standar (Yaw & Pitch)
        let cos_pitch = self.camera_pitch.cos();
        let sin_pitch = self.camera_pitch.sin();
        let sin_yaw = self.camera_yaw.sin();
        let cos_yaw = self.camera_yaw.cos();

        let target = Vec3::new(self.camera_pan_x, center_y + self.camera_pan_y, 0.0);
        let camera_pos = Vec3::new(
            target.x + radius * cos_pitch * sin_yaw,
            target.y + radius * sin_pitch,
            target.z + radius * cos_pitch * cos_yaw,
        );
        let view = Mat4::look_at_rh(camera_pos, target, Vec3::Y);

        // Standar Perspective Matrix 3D
        let near = 0.05_f32;
        let far = 50.0_f32;
        let proj = Mat4::perspective_rh(fov_rad, aspect, near, far);
        let view_proj = proj * view;

        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            _padding: 0.0,
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        // Update Matrix Tulang (Hierarki Pose Three.js)
        if let Some(ref model) = self.model_data {
            let mut global_transforms = vec![Mat4::IDENTITY; model.nodes.len()];
            let mut bone_matrices = [Mat4::IDENTITY.to_cols_array_2d(); 256];

            for &root_idx in &model.root_nodes {
                self.compute_global_transforms(root_idx, Mat4::IDENTITY, pose_manager, model, &mut global_transforms);
            }

            if let Some(skin) = model.skins.first() {
                for (joint_i, &node_idx) in skin.joints.iter().enumerate() {
                    if joint_i < 256 {
                        let inv_bind = skin.inverse_bind_matrices.get(joint_i).copied().unwrap_or(Mat4::IDENTITY);
                        let global_joint = global_transforms[node_idx];
                        let final_mat = global_joint * inv_bind;
                        bone_matrices[joint_i] = final_mat.to_cols_array_2d();
                    }
                }
            }

            let bone_uniform = BoneMatricesUniform {
                matrices: bone_matrices,
            };
            self.queue.write_buffer(&self.bone_buffer, 0, bytemuck::cast_slice(&[bone_uniform]));
        }
    }

    fn compute_global_transforms(
        &self,
        node_idx: usize,
        parent_transform: Mat4,
        pose_manager: &PoseManager,
        model: &VrmModelData,
        globals: &mut [Mat4],
    ) {
        let node = &model.nodes[node_idx];

        let mut rotation = node.initial_rotation;

        if let Some((bone_name, _)) = model.bone_mapping.iter().find(|&(_, &idx)| idx == node_idx) {
            if let Some(pose_rot) = pose_manager.get_bone_quat(bone_name) {
                rotation = rotation * pose_rot;
            }
        }

        let local_transform = Mat4::from_scale_rotation_translation(
            node.initial_scale,
            rotation,
            node.initial_translation,
        );

        let global_transform = parent_transform * local_transform;
        globals[node_idx] = global_transform;

        for &child_idx in &node.children {
            self.compute_global_transforms(child_idx, global_transform, pose_manager, model, globals);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                self.surface.get_current_texture()?
            }
            Err(e) => return Err(e),
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let overlay_uniform = OverlayUniform {
            window_size: [self.config.width as f32, self.config.height as f32],
            cursor_pos: self.cursor_pixel_pos,
            is_visible: if self.is_header_visible { 1.0 } else { 0.0 },
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        self.queue.write_buffer(&self.overlay_buffer, 0, bytemuck::cast_slice(&[overlay_uniform]));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_texture,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        // Background 100% transparan
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if self.avatar_type.eq_ignore_ascii_case("2d") {
                let state = self.current_visual_state;
                if let Some((_, bind_group, img_size)) = self.avatar_2d_textures.get(&state).or_else(|| self.avatar_2d_textures.get(&AssistantVisualState::Idle)) {
                    let breath = if state == AssistantVisualState::Idle {
                        (self.elapsed_time * 2.4).sin() * 4.0
                    } else if state == AssistantVisualState::Speaking {
                        (self.elapsed_time * 8.0).sin() * 3.0
                    } else {
                        0.0
                    };

                    let u = Avatar2dUniform {
                        window_size: [self.config.width as f32, self.config.height as f32],
                        image_size: *img_size,
                        camera_pan: [self.camera_pan_x * 80.0, self.camera_pan_y * 80.0],
                        camera_zoom: self.camera_zoom,
                        breath_offset: breath,
                        _pad0: 0.0,
                        _pad1: 0.0,
                    };
                    self.queue.write_buffer(&self.avatar_2d_uniform_buffer, 0, bytemuck::cast_slice(&[u]));

                    render_pass.set_pipeline(&self.avatar_2d_pipeline);
                    render_pass.set_bind_group(0, &self.avatar_2d_uniform_bind_group, &[]);
                    render_pass.set_bind_group(1, bind_group, &[]);
                    render_pass.draw(0..3, 0..1);
                }
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.bone_bind_group, &[]);

                for prim in &self.gpu_primitives {
                    if let Some(mat) = self.gpu_materials.get(prim.material_index) {
                        render_pass.set_bind_group(2, &mat.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, prim.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(prim.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..prim.index_count, 0, 0..1);
                    }
                }
            }

            if self.is_header_visible {
                render_pass.set_pipeline(&self.overlay_pipeline);
                render_pass.set_bind_group(0, &self.overlay_bind_group, &[]);
                render_pass.draw(0..3, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn rotate_camera(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.camera_yaw += delta_yaw;
        self.camera_pitch = (self.camera_pitch + delta_pitch).clamp(-1.2, 1.2);
    }

    pub fn pan_camera(&mut self, delta_x: f32, delta_y: f32) {
        let fov_rad = 28.0_f32.to_radians();
        let half_fov_tan = (fov_rad / 2.0).tan();
        let height = (self.config.height as f32).max(1.0);
        let model_height = (self.model_max_y - self.model_min_y).max(1.2);
        let base_radius = (model_height * 1.12 / 2.0) / half_fov_tan;
        let current_radius = base_radius / self.camera_zoom.clamp(0.15, 6.0);

        // Faktor konversi pixel layar ke koordinat 3D dunia
        let pan_factor = (2.0 * half_fov_tan * current_radius) / height;
        self.camera_pan_x -= delta_x * pan_factor;
        self.camera_pan_y += delta_y * pan_factor;
    }

    pub fn zoom_camera(&mut self, scroll_delta: f32) {
        let zoom_factor = 1.0 + (scroll_delta * 0.08);
        self.camera_zoom = (self.camera_zoom * zoom_factor).clamp(0.2, 5.0);
    }

    pub fn reset_camera(&mut self) {
        self.camera_yaw = 0.0;
        self.camera_pitch = 0.0;
        self.camera_zoom = 1.0;
        self.camera_pan_x = 0.0;
        self.camera_pan_y = 0.0;
    }

    pub fn reset_camera_rotation(&mut self) {
        self.reset_camera();
    }
}
