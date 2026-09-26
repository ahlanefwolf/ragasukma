pub mod camera;
pub mod expressions;
pub mod overlay;
pub mod pipeline_2d;
pub mod pipeline_3d;
pub mod types;

use std::collections::HashMap;
use std::sync::Arc;
use winit::window::Window;

use crate::ui::pose_system::{AssistantVisualState, PoseManager};
use crate::ui::vrm_loader::VrmModelData;

pub use camera::CameraState;
pub use types::{
    Avatar2dUniform, BoneMatricesUniform, CameraUniform, GpuMaterial, GpuPrimitive,
    MaterialUniform, OverlayUniform,
};

pub struct Renderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,

    msaa_texture: wgpu::Texture,
    msaa_texture_view: wgpu::TextureView,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    bone_buffer: wgpu::Buffer,
    bone_bind_group: wgpu::BindGroup,
    cached_global_transforms: Vec<glam::Mat4>,
    cached_bone_uniform: Box<BoneMatricesUniform>,

    model_data: Option<Box<VrmModelData>>,
    gpu_primitives: Vec<GpuPrimitive>,
    gpu_materials: Vec<GpuMaterial>,

    // Referensi bounding box model
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
    pub current_emotion: String,
    pub elapsed_time: f32,
    avatar_2d_pipeline: wgpu::RenderPipeline,
    avatar_2d_uniform_buffer: wgpu::Buffer,
    avatar_2d_uniform_bind_group: wgpu::BindGroup,
    avatar_2d_bind_group_layout: wgpu::BindGroupLayout,
    avatar_2d_sampler: wgpu::Sampler,
    avatar_2d_textures:
        HashMap<AssistantVisualState, (wgpu::TextureView, wgpu::BindGroup, [f32; 2])>,
    pub loading_texture: Option<(wgpu::TextureView, wgpu::BindGroup, [f32; 2])>,
    pub is_loading: bool,

    // CSD Header & Controls
    pub is_header_visible: bool,
    pub cursor_pixel_pos: [f32; 2],
    overlay_pipeline: wgpu::RenderPipeline,
    overlay_buffer: wgpu::Buffer,
    overlay_bind_group: wgpu::BindGroup,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.msaa_texture.destroy();
        self.depth_texture.destroy();
    }
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

        println!(
            "🎮 [WGPU Adapter] Menggunakan GPU: {}",
            adapter.get_info().name
        );

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
            .find(|m| *m == wgpu::CompositeAlphaMode::PreMultiplied)
            .or_else(|| {
                surface_caps
                    .alpha_modes
                    .iter()
                    .copied()
                    .find(|m| *m == wgpu::CompositeAlphaMode::Auto)
            })
            .unwrap_or(surface_caps.alpha_modes[0]);

        println!(
            "🔍 [WGPU Surface] Formats: {:?}, Alpha Modes: {:?}",
            surface_caps.formats, surface_caps.alpha_modes
        );
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

        // 1. Pipeline 3D VRM MToon
        let pipeline_3d = pipeline_3d::create_3d_pipeline(&device, &config);

        // 2. Overlay Pipeline (Border & Header Buttons)
        let overlay = overlay::create_overlay_pipeline(&device, &config);

        // 3. Pipeline 2D Sprite Quad
        let pipeline_2d = pipeline_2d::create_2d_pipeline(&device, &queue, &config);

        let (msaa_texture, msaa_texture_view) = Self::create_msaa_texture(&device, &config);
        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline: pipeline_3d.render_pipeline,
            msaa_texture,
            msaa_texture_view,
            depth_texture,
            depth_texture_view,
            camera_buffer: pipeline_3d.camera_buffer,
            camera_bind_group: pipeline_3d.camera_bind_group,
            bone_buffer: pipeline_3d.bone_buffer,
            bone_bind_group: pipeline_3d.bone_bind_group,
            cached_global_transforms: Vec::with_capacity(128),
            cached_bone_uniform: Box::new(BoneMatricesUniform {
                matrices: [glam::Mat4::IDENTITY.to_cols_array_2d(); 256],
            }),
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
            current_emotion: "neutral".to_string(),
            elapsed_time: 0.0,
            avatar_2d_pipeline: pipeline_2d.pipeline,
            avatar_2d_uniform_buffer: pipeline_2d.uniform_buffer,
            avatar_2d_uniform_bind_group: pipeline_2d.uniform_bind_group,
            avatar_2d_bind_group_layout: pipeline_2d.bind_group_layout,
            avatar_2d_sampler: pipeline_2d.sampler,
            avatar_2d_textures: pipeline_2d.textures,
            loading_texture: pipeline_2d.loading_texture,
            is_loading: false,
            is_header_visible: false,
            cursor_pixel_pos: [-100.0, -100.0],
            overlay_pipeline: overlay.pipeline,
            overlay_buffer: overlay.buffer,
            overlay_bind_group: overlay.bind_group,
        })
    }

    pub fn reload_2d_avatar_textures(&mut self) {
        self.avatar_2d_textures = pipeline_2d::load_2d_avatar_textures(
            &self.device,
            &self.queue,
            &self.avatar_2d_bind_group_layout,
            &self.avatar_2d_sampler,
        );
    }

    pub fn update_anchor_on_resize(
        &mut self,
        old_w: f32,
        old_h: f32,
        new_w: f32,
        new_h: f32,
        dir: Option<winit::window::ResizeDirection>,
    ) {
        if let Some(d) = dir {
            use winit::window::ResizeDirection::*;
            match d {
                West | NorthWest | SouthWest => {
                    self.anchor_center_x += new_w - old_w;
                }
                East | NorthEast | SouthEast => {
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
        self.anchor_center_x = self.anchor_center_x.clamp(new_w * 0.1, new_w * 0.9);
        self.anchor_center_y = self.anchor_center_y.clamp(new_h * 0.1, new_h * 0.9);
    }

    fn create_msaa_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView) {
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
        let view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        (depth_texture, view)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            if self.size == new_size
                && self.config.width == new_size.width
                && self.config.height == new_size.height
            {
                return;
            }
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Bebaskan alokasi VRAM / GTT lama seketika sebelum membuat texture baru
            self.msaa_texture.destroy();
            self.depth_texture.destroy();

            let (msaa_tex, msaa_view) = Self::create_msaa_texture(&self.device, &self.config);
            let (depth_tex, depth_view) = Self::create_depth_texture(&self.device, &self.config);

            self.msaa_texture = msaa_tex;
            self.msaa_texture_view = msaa_view;
            self.depth_texture = depth_tex;
            self.depth_texture_view = depth_view;
        }
    }

    pub fn has_model(&self) -> bool {
        self.model_data.is_some()
    }

    pub fn load_vrm_model(&mut self, model: Box<VrmModelData>) {
        // Bebaskan GPU buffers (vertex & index) dari model VRM sebelumnya jika ada
        for prim in &self.gpu_primitives {
            prim.vertex_buffer.destroy();
            prim.index_buffer.destroy();
        }
        self.gpu_primitives.clear();
        self.gpu_materials.clear();

        let loaded = pipeline_3d::load_vrm_model_to_gpu(
            &self.device,
            &self.queue,
            &self.render_pipeline,
            model,
        );

        self.model_min_y = loaded.min_y;
        self.model_max_y = loaded.max_y;
        self.model_max_extent_x = loaded.max_extent_x;
        self.gpu_materials = loaded.gpu_materials;
        self.gpu_primitives = loaded.gpu_primitives;
        self.model_data = Some(loaded.model);
    }

    pub fn update_camera_and_bones(
        &mut self,
        pose_manager: &mut PoseManager,
        delta_seconds: f32,
        is_tuning: bool,
    ) {
        let cam = CameraState {
            camera_yaw: self.camera_yaw,
            camera_pitch: self.camera_pitch,
            camera_zoom: self.camera_zoom,
            camera_pan_x: self.camera_pan_x,
            camera_pan_y: self.camera_pan_y,
        };

        camera::update_camera_and_bones(
            &self.queue,
            &self.camera_buffer,
            &self.bone_buffer,
            self.model_data.as_deref(),
            self.model_min_y,
            self.model_max_y,
            self.config.width as f32,
            self.config.height as f32,
            &cam,
            pose_manager,
            delta_seconds,
            &mut self.cached_global_transforms,
            &mut self.cached_bone_uniform,
        );

        if let Some(model) = self.model_data.as_mut() {
            expressions::update_facial_expressions_and_lip_sync(
                &self.queue,
                model,
                &self.gpu_primitives,
                self.current_visual_state,
                &self.current_emotion,
                self.elapsed_time,
                is_tuning,
            );
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
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let overlay_uniform = OverlayUniform {
            window_size: [self.config.width as f32, self.config.height as f32],
            cursor_pos: self.cursor_pixel_pos,
            is_visible: if self.is_header_visible { 1.0 } else { 0.0 },
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        self.queue.write_buffer(
            &self.overlay_buffer,
            0,
            bytemuck::cast_slice(&[overlay_uniform]),
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_texture_view,
                    resolve_target: Some(&view),
                    ops: wgpu::Operations {
                        // Background 100% transparan
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if self.is_loading {
                if let Some((_, bind_group, img_size)) = &self.loading_texture {
                    let bounce = (self.elapsed_time * 6.0).sin().abs() * 28.0;
                    let u = Avatar2dUniform {
                        window_size: [self.config.width as f32, self.config.height as f32],
                        image_size: *img_size,
                        camera_pan: [0.0, -bounce],
                        camera_zoom: 0.65,
                        breath_offset: 0.0,
                        _pad0: 0.0,
                        _pad1: 0.0,
                    };
                    self.queue.write_buffer(
                        &self.avatar_2d_uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[u]),
                    );

                    render_pass.set_pipeline(&self.avatar_2d_pipeline);
                    render_pass.set_bind_group(0, &self.avatar_2d_uniform_bind_group, &[]);
                    render_pass.set_bind_group(1, bind_group, &[]);
                    render_pass.draw(0..3, 0..1);
                }
            } else if self.avatar_type.eq_ignore_ascii_case("2d") {
                let state = self.current_visual_state;

                // Untuk 2D Avatar saat Speaking: buat animasi mouth flapping (buka-tutup mulut)
                // tersinkronisasi dengan gelombang vokal AIUEO yang sama dengan 3D VRM (~5.5 Hz)
                let active_2d_state = if state == AssistantVisualState::Speaking {
                    let mouth_open = ((self.elapsed_time * 5.5).sin().abs()).powf(1.4);
                    if mouth_open > 0.28 {
                        AssistantVisualState::Speaking
                    } else {
                        AssistantVisualState::Idle
                    }
                } else {
                    state
                };

                if let Some((_, bind_group, img_size)) = self
                    .avatar_2d_textures
                    .get(&active_2d_state)
                    .or_else(|| self.avatar_2d_textures.get(&AssistantVisualState::Idle))
                {
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
                    self.queue.write_buffer(
                        &self.avatar_2d_uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[u]),
                    );

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
                        render_pass.set_index_buffer(
                            prim.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
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
        let mut cam = CameraState {
            camera_yaw: self.camera_yaw,
            camera_pitch: self.camera_pitch,
            camera_zoom: self.camera_zoom,
            camera_pan_x: self.camera_pan_x,
            camera_pan_y: self.camera_pan_y,
        };
        cam.rotate(delta_yaw, delta_pitch);
        self.camera_yaw = cam.camera_yaw;
        self.camera_pitch = cam.camera_pitch;
    }

    pub fn pan_camera(&mut self, delta_x: f32, delta_y: f32) {
        let mut cam = CameraState {
            camera_yaw: self.camera_yaw,
            camera_pitch: self.camera_pitch,
            camera_zoom: self.camera_zoom,
            camera_pan_x: self.camera_pan_x,
            camera_pan_y: self.camera_pan_y,
        };
        cam.pan(
            delta_x,
            delta_y,
            self.config.height as f32,
            self.model_min_y,
            self.model_max_y,
        );
        self.camera_pan_x = cam.camera_pan_x;
        self.camera_pan_y = cam.camera_pan_y;
    }

    pub fn zoom_camera(&mut self, scroll_delta: f32) {
        let mut cam = CameraState {
            camera_yaw: self.camera_yaw,
            camera_pitch: self.camera_pitch,
            camera_zoom: self.camera_zoom,
            camera_pan_x: self.camera_pan_x,
            camera_pan_y: self.camera_pan_y,
        };
        cam.zoom(scroll_delta);
        self.camera_zoom = cam.camera_zoom;
    }

    pub fn reset_camera(&mut self) {
        let mut cam = CameraState::default();
        cam.reset();
        self.camera_yaw = cam.camera_yaw;
        self.camera_pitch = cam.camera_pitch;
        self.camera_zoom = cam.camera_zoom;
        self.camera_pan_x = cam.camera_pan_x;
        self.camera_pan_y = cam.camera_pan_y;
    }

    pub fn reset_camera_rotation(&mut self) {
        self.reset_camera();
    }
}
