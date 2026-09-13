use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::pose_system::PoseManager;
use crate::vrm_loader::{Vertex, VrmModelData};

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

        let alpha_mode = surface_caps.alpha_modes[0];

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

        let depth_texture = Self::create_depth_view(&device, &config);

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
                count: 1,
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            depth_texture,
            camera_buffer,
            camera_bind_group,
            bone_buffer,
            bone_bind_group,
            model_data: None,
            gpu_primitives: Vec::new(),
            gpu_materials: Vec::new(),
            model_min_y: 0.0,
            model_max_y: 1.55,
            model_max_extent_x: 0.38,
        })
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
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Self::create_depth_view(&self.device, &self.config);
        }
    }

    fn create_texture_from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img_opt: &Option<image::RgbaImage>,
        fallback_rgba: [u8; 4],
        label: &str,
    ) -> wgpu::TextureView {
        if let Some(ref img) = img_opt {
            let dimensions = img.dimensions();
            let texture_size = wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: texture_size,
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
                    bytes_per_row: Some(4 * dimensions.0),
                    rows_per_image: Some(dimensions.1),
                },
                texture_size,
            );
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

    pub fn load_vrm_model(&mut self, model: VrmModelData) {
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

        let mut gpu_materials = Vec::new();
        for mat in &model.materials {
            let base_view = Self::create_texture_from_rgba(
                &self.device,
                &self.queue,
                &mat.base_color_texture,
                [255, 255, 255, 255],
                &format!("Base_{}", mat.name),
            );

            let shade_view = Self::create_texture_from_rgba(
                &self.device,
                &self.queue,
                &mat.shade_texture,
                [255, 255, 255, 255],
                &format!("Shade_{}", mat.name),
            );

            let emissive_view = Self::create_texture_from_rgba(
                &self.device,
                &self.queue,
                &mat.emissive_texture,
                [0, 0, 0, 255],
                &format!("Emissive_{}", mat.name),
            );

            let matcap_view = Self::create_texture_from_rgba(
                &self.device,
                &self.queue,
                &mat.matcap_texture,
                [0, 0, 0, 255],
                &format!("Matcap_{}", mat.name),
            );

            let is_emissive_active = mat.emissive_texture.is_some()
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
                has_matcap: if mat.matcap_texture.is_some() { 1 } else { 0 },
                has_shade_tex: if mat.shade_texture.is_some() { 1 } else { 0 },
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
                        resource: wgpu::BindingResource::TextureView(&base_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::TextureView(&shade_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&emissive_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::TextureView(&matcap_view),
                    },
                ],
                label: Some("material_bind_group"),
            });

            gpu_materials.push(GpuMaterial { bind_group });
        }

        let mut gpu_primitives = Vec::new();
        for prim in &model.primitives {
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

            gpu_primitives.push(GpuPrimitive {
                vertex_buffer,
                index_buffer,
                index_count: prim.indices.len() as u32,
                material_index: prim.material_index.unwrap_or(0),
            });
        }

        self.gpu_materials = gpu_materials;
        self.gpu_primitives = gpu_primitives;
        self.model_data = Some(model);
    }

    pub fn update_camera_and_bones(&mut self, pose_manager: &mut PoseManager, delta_seconds: f32) {
        pose_manager.update_poses(delta_seconds);

        let width = self.config.width as f32;
        let height = (self.config.height as f32).max(1.0);
        let aspect = width / height;

        // STANDARISASI AUTO-FRAMING 1:1 THREE.JS / UNITY
        // Dihitung otomatis dari batas geometris aktual model VRoid
        let fov_rad = 28.0_f32.to_radians();
        let half_fov_tan = (fov_rad / 2.0).tan();

        // 1. Titik tengah dan tinggi aktual model (Min Y: -0.022, Max Y: 1.618)
        let center_y = (self.model_min_y + self.model_max_y) * 0.5;
        let model_height = (self.model_max_y - self.model_min_y).max(1.2);

        // 2. Margin standar 12% agar rambut teratas (ahoge) & sepatu memiliki padding lega
        let safe_margin = 1.12_f32;
        let fit_height = model_height * safe_margin;
        let fit_width = fit_height * (9.0 / 16.0); // Standarisasi rasio 9:16

        // 3. Jarak kamera vertikal & horizontal
        let dist_v = (fit_height / 2.0) / half_fov_tan;
        let dist_h = (fit_width / 2.0) / (half_fov_tan * aspect);
        let camera_z = dist_v.max(dist_h);

        // 4. Transformasi Kamera RH (Right-Handed)
        let camera_pos = Vec3::new(0.0, center_y, camera_z);
        let view = Mat4::look_at_rh(camera_pos, Vec3::new(0.0, center_y, 0.0), Vec3::Y);
        let proj = Mat4::perspective_rh(fov_rad, aspect, 0.05, 50.0);
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

        if let Some((bone_name, _)) = model.bone_mapping.iter().find(|(_, &idx)| idx == node_idx) {
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
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
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

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
