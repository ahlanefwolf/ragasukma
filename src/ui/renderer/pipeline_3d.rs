use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use super::types::{
    BoneMatricesUniform, CameraUniform, GpuMaterial, GpuPrimitive, MaterialUniform,
};
use crate::ui::vrm_loader::{Vertex, VrmModelData};
use glam::Mat4;

pub struct Pipeline3dComponents {
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub bone_buffer: wgpu::Buffer,
    pub bone_bind_group: wgpu::BindGroup,
}

pub fn create_3d_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> Pipeline3dComponents {
    let camera_bind_group_layout =
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
            label: Some("camera_bind_group_layout"),
        });

    let bone_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let material_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let shader = device.create_shader_module(wgpu::include_wgsl!("../shader.wgsl"));

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

    let initial_bones = Box::new(BoneMatricesUniform {
        matrices: [Mat4::IDENTITY.to_cols_array_2d(); 512],
    });
    let bone_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Bone Buffer"),
        contents: bytemuck::cast_slice(&[*initial_bones]),
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

    Pipeline3dComponents {
        render_pipeline,
        camera_buffer,
        camera_bind_group,
        bone_buffer,
        bone_bind_group,
    }
}

pub fn fast_downscale_half(src: &[u8], w: usize, h: usize) -> Vec<u8> {
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
                let sum = src[i00 + c] as u32
                    + src[i10 + c] as u32
                    + src[i01 + c] as u32
                    + src[i11 + c] as u32;
                dst[out_idx + c] = (sum / 4) as u8;
            }
        }
    }
    dst
}

pub fn create_texture_from_rgba(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    img_opt: Option<&image::RgbaImage>,
    fallback_rgba: [u8; 4],
    label: &str,
) -> wgpu::TextureView {
    if let Some(img) = img_opt {
        let dimensions = img.dimensions();
        let max_dim = dimensions.0.max(dimensions.1);
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

        let mut current_bytes = img.as_raw().clone();
        let mut cur_w = dimensions.0 as usize;
        let mut cur_h = dimensions.1 as usize;

        for mip in 1..mip_level_count {
            current_bytes = fast_downscale_half(&current_bytes, cur_w, cur_h);
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
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
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
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

pub struct LoadedVrmGpu {
    pub gpu_materials: Vec<GpuMaterial>,
    pub gpu_primitives: Vec<GpuPrimitive>,
    pub min_y: f32,
    pub max_y: f32,
    pub max_extent_x: f32,
    pub model: Box<VrmModelData>,
}

pub fn load_vrm_model_to_gpu(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    mut model: Box<VrmModelData>,
) -> LoadedVrmGpu {
    let material_bind_group_layout = pipeline.get_bind_group_layout(2);
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

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

    let mut model_min_y = 0.0;
    let mut model_max_y = 1.6;
    let mut model_max_extent_x = 0.7;

    if min_y < max_y && min_x < max_x {
        model_min_y = min_y;
        model_max_y = max_y;
        model_max_extent_x = min_x.abs().max(max_x.abs());
        println!(
            "[Model Bounds] Y: [{:.3} .. {:.3}], Extent X: {:.3}",
            min_y, max_y, model_max_extent_x
        );
    }

    let dummy_white_view = create_texture_from_rgba(
        device,
        queue,
        None,
        [255, 255, 255, 255],
        "Shared_Dummy_White",
    );

    let dummy_black_view =
        create_texture_from_rgba(device, queue, None, [0, 0, 0, 255], "Shared_Dummy_Black");

    let mut gpu_materials = Vec::new();
    let mut gpu_texture_cache: HashMap<usize, Arc<wgpu::TextureView>> = HashMap::new();

    let get_or_create_gpu_view = |dev: &wgpu::Device,
                                  q: &wgpu::Queue,
                                  img_opt: Option<Arc<image::RgbaImage>>,
                                  default_color: [u8; 4],
                                  label: &str,
                                  cache: &mut HashMap<usize, Arc<wgpu::TextureView>>|
     -> Option<Arc<wgpu::TextureView>> {
        if let Some(arc_img) = img_opt {
            let key = Arc::as_ptr(&arc_img) as usize;
            if let Some(view) = cache.get(&key) {
                return Some(view.clone());
            }
            let view = Arc::new(create_texture_from_rgba(
                dev,
                q,
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
            device,
            queue,
            base_img,
            [255, 255, 255, 255],
            &format!("Base_{}", mat.name),
            &mut gpu_texture_cache,
        );
        let base_view_ref: &wgpu::TextureView =
            custom_base_view.as_deref().unwrap_or(&dummy_white_view);

        let custom_shade_view = get_or_create_gpu_view(
            device,
            queue,
            shade_img,
            [255, 255, 255, 255],
            &format!("Shade_{}", mat.name),
            &mut gpu_texture_cache,
        );
        let shade_view_ref: &wgpu::TextureView =
            custom_shade_view.as_deref().unwrap_or(&dummy_white_view);

        let custom_emissive_view = get_or_create_gpu_view(
            device,
            queue,
            emissive_img,
            [0, 0, 0, 255],
            &format!("Emissive_{}", mat.name),
            &mut gpu_texture_cache,
        );
        let emissive_view_ref: &wgpu::TextureView =
            custom_emissive_view.as_deref().unwrap_or(&dummy_black_view);

        let custom_matcap_view = get_or_create_gpu_view(
            device,
            queue,
            matcap_img,
            [0, 0, 0, 255],
            &format!("Matcap_{}", mat.name),
            &mut gpu_texture_cache,
        );
        let matcap_view_ref: &wgpu::TextureView =
            custom_matcap_view.as_deref().unwrap_or(&dummy_black_view);

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

        let mat_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::cast_slice(&[mat_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        let has_morph = !prim.morph_targets.is_empty();
        let usage = if has_morph {
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
        } else {
            wgpu::BufferUsages::VERTEX
        };

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&prim.vertices),
            usage,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&prim.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let index_count = prim.indices.len() as u32;

        if !has_morph {
            prim.base_vertices.clear();
            prim.base_vertices.shrink_to_fit();
            prim.vertices.clear();
            prim.vertices.shrink_to_fit();
        }
        prim.indices.clear();
        prim.indices.shrink_to_fit();

        gpu_primitives.push(GpuPrimitive {
            vertex_buffer,
            index_buffer,
            index_count,
            material_index: prim.material_index.unwrap_or(0),
        });
    }

    model.materials.clear();
    model.materials.shrink_to_fit();

    LoadedVrmGpu {
        gpu_materials,
        gpu_primitives,
        min_y: model_min_y,
        max_y: model_max_y,
        max_extent_x: model_max_extent_x,
        model,
    }
}
