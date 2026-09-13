use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder},
};

use crate::pose_system::{AssistantVisualState, PoseManager};
use crate::renderer::Renderer;
use crate::vrm_loader::VrmModelData;

pub struct App {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub pose_manager: PoseManager,
    pub last_frame_time: Instant,
}

impl App {
    pub async fn new(window_target: &EventLoopWindowTarget<()>) -> anyhow::Result<Self> {
        let window = WindowBuilder::new()
            .with_title("Ragasukma Avatar Overlay (Native)")
            .with_inner_size(winit::dpi::LogicalSize::new(270.0, 480.0)) // Rasio 9:16 standar portrait
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true)
            .build(window_target)?;

        let window = Arc::new(window);
        let mut renderer = Renderer::new(window.clone()).await?;

        // Cari path model VRoid/VRM di berbagai kemungkinan direktori (relative/parent)
        let candidates = [
            PathBuf::from("assets/char.glb"),
            PathBuf::from("../assets/char.glb"),
            PathBuf::from("C:/Project/ragasukma/assets/char.glb"),
            PathBuf::from("assets/char.vrm"),
            PathBuf::from("../assets/char.vrm"),
            PathBuf::from("C:/Project/ragasukma/assets/char.vrm"),
        ];

        let mut loaded = false;
        for path in &candidates {
            if path.exists() {
                println!("📂 [Asset Loader] Memuat model avatar dari: {:?}", path);
                if let Ok(model) = VrmModelData::load_from_file(path) {
                    renderer.load_vrm_model(model);
                    loaded = true;
                    break;
                }
            }
        }

        if !loaded {
            eprintln!("⚠️ Gagal memuat avatar dari daftar path kandidat {:?}", candidates);
        }

        let mut pose_manager = PoseManager::default();
        pose_manager.current_state = AssistantVisualState::Idle;

        Ok(Self {
            window,
            renderer,
            pose_manager,
            last_frame_time: Instant::now(),
        })
    }

    pub fn handle_window_event(&mut self, event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        match event {
            WindowEvent::CloseRequested => {
                elwt.exit();
            }
            WindowEvent::Resized(physical_size) => {
                self.renderer.resize(physical_size);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let physical_size = self.window.inner_size();
                self.renderer.resize(physical_size);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    // Native Windows OS Dragging (Sangat smooth, 0 latency, GPU-accelerated)
                    let _ = self.window.drag_window();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 15.0,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };

                let current_size = self.window.inner_size();
                let aspect = current_size.width as f32 / current_size.height as f32;
                let new_height = (current_size.height as f32 + scroll_y).clamp(160.0, 1400.0);
                let new_width = (new_height * aspect).max(100.0);

                let _ = self.window.request_inner_size(winit::dpi::PhysicalSize::new(
                    new_width as u32,
                    new_height as u32,
                ));
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = (now - self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;

                self.pose_manager.elapsed_time += delta;
                self.renderer.update_camera_and_bones(&mut self.pose_manager, delta);

                match self.renderer.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.renderer.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprintln!("Render error: {:?}", e),
                }

                self.window.request_redraw();
            }
            _ => {}
        }
    }
}
