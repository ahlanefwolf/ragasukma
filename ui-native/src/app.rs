use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowLevel},
};

use crate::pose_system::{AssistantVisualState, PoseManager};
use crate::renderer::Renderer;
use crate::server::{self, ServerSharedState};
use crate::vrm_loader::VrmModelData;

pub struct App {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub pose_manager: PoseManager,
    pub last_frame_time: Instant,
    pub shared_server_state: Arc<Mutex<ServerSharedState>>,
    pub is_right_mouse_down: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
}

impl App {
    pub async fn new(window_target: &EventLoopWindowTarget<()>) -> anyhow::Result<Self> {
        let initial_cfg = server::load_config();

        let initial_level = if initial_cfg.always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };

        let monitor = window_target.primary_monitor().or_else(|| window_target.available_monitors().next());
        let (screen_w, screen_h) = if let Some(m) = monitor {
            let s = m.size();
            (s.width as f64, s.height as f64)
        } else {
            (1920.0, 1080.0)
        };

        // Window ukuran kompak portrait (270x480) di pojok kanan bawah desktop
        let win_w = 270.0;
        let win_h = 480.0;
        let pos_x = (screen_w - win_w - 40.0).max(0.0);
        let pos_y = (screen_h - win_h - 60.0).max(0.0);

        let window = WindowBuilder::new()
            .with_title("Ragasukma Avatar Overlay (Native)")
            .with_inner_size(winit::dpi::LogicalSize::new(win_w, win_h))
            .with_position(winit::dpi::LogicalPosition::new(pos_x, pos_y))
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(initial_level)
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

        let shared_server_state = Arc::new(Mutex::new(ServerSharedState {
            current_state: AssistantVisualState::Idle,
            is_paused: false,
            voice_muted: false,
            mic_muted: false,
            poses_updated: false,
            config_updated: false,
            tuning_mode: false,
            live_tuning_bones: std::collections::HashMap::new(),
        }));

        server::start_http_server(Arc::clone(&shared_server_state));

        Ok(Self {
            window,
            renderer,
            pose_manager,
            last_frame_time: Instant::now(),
            shared_server_state,
            is_right_mouse_down: false,
            last_cursor_pos: None,
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
                    // Klik Kiri Drag: Pindahkan posisi jendela avatar di desktop secara native OS (super smooth, 0 CPU overhead)
                    let _ = self.window.drag_window();
                } else if button == MouseButton::Right {
                    self.is_right_mouse_down = state == ElementState::Pressed;
                } else if button == MouseButton::Middle && state == ElementState::Pressed {
                    // Klik Tengah: Reset rotasi & zoom kamera avatar
                    self.renderer.reset_camera();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((prev_x, prev_y)) = self.last_cursor_pos {
                    if self.is_right_mouse_down {
                        // Drag Klik Kanan: Orbit rotasi kamera 3D
                        let delta_x = (position.x - prev_x) as f32;
                        let delta_y = (position.y - prev_y) as f32;
                        let sensitivity = 0.008;
                        let delta_yaw = -delta_x * sensitivity;
                        let delta_pitch = delta_y * sensitivity;

                        self.renderer.rotate_camera(delta_yaw, delta_pitch);
                    }
                }
                self.last_cursor_pos = Some((position.x, position.y));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => (pos.y as f32) / 40.0,
                };

                // Scroll Mouse: POV Kamera Zoom In / Out halus
                self.renderer.zoom_camera(scroll_y);
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = (now - self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;

                let mut is_tuning = false;
                // Sync state & live poses reload dari Web HTTP Dashboard
                if let Ok(mut lock) = self.shared_server_state.lock() {
                    self.pose_manager.current_state = lock.current_state;
                    is_tuning = lock.tuning_mode;

                    if is_tuning {
                        // Freeze animasi siklus, aplikasikan langsung live bone rotation dari tuner
                        self.pose_manager.apply_tuning_pose(&lock.live_tuning_bones);
                    } else if lock.poses_updated {
                        let path = server::find_file_in_candidates("poses.json");
                        self.pose_manager.load_from_file(&path);
                        lock.poses_updated = false;
                    }

                    if lock.config_updated {
                        let cfg = server::load_config();
                        let lvl = if cfg.always_on_top {
                            WindowLevel::AlwaysOnTop
                        } else {
                            WindowLevel::Normal
                        };
                        self.window.set_window_level(lvl);
                        lock.config_updated = false;
                    }
                }

                let delta_sim = if is_tuning { 0.0 } else { delta };
                self.renderer.update_camera_and_bones(&mut self.pose_manager, delta_sim);

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
