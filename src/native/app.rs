use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::{
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowLevel},
};

use super::pose_system::{AssistantVisualState, PoseManager};
use super::renderer::Renderer;
use super::server::{self, ServerSharedState};
use super::vrm_loader::VrmModelData;

pub struct App {
    pub window: Arc<Window>,
    pub renderer: Renderer,
    pub pose_manager: PoseManager,
    pub last_frame_time: Instant,
    pub shared_server_state: Arc<Mutex<ServerSharedState>>,
    pub is_right_mouse_down: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
    pub is_window_focused: bool,
    pub active_resize_dir: Option<winit::window::ResizeDirection>,
    pub prev_win_size: (f32, f32),
    pub was_maximized: bool,
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

        // Load icon window dari icons/icon.png
        let window_icon = {
            let icon_bytes = include_bytes!("../../icons/icon.png");
            if let Ok(img) = image::load_from_memory(icon_bytes) {
                let rgba = img.to_rgba8();
                let (w, h) = rgba.dimensions();
                winit::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
            } else {
                None
            }
        };

        // Window ukuran kompak portrait (270x480) di pojok kanan bawah desktop
        let win_w = 270.0;
        let win_h = 480.0;
        let pos_x = (screen_w - win_w - 40.0).max(0.0);
        let pos_y = (screen_h - win_h - 60.0).max(0.0);

        let mut win_builder = WindowBuilder::new()
            .with_title("Ragasukma Avatar Overlay (Native)")
            .with_inner_size(winit::dpi::LogicalSize::new(win_w, win_h))
            .with_position(winit::dpi::LogicalPosition::new(pos_x, pos_y))
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(initial_level)
            .with_resizable(true);

        if let Some(icon) = window_icon {
            win_builder = win_builder.with_window_icon(Some(icon));
        }

        let window = win_builder.build(window_target)?;

        let window = Arc::new(window);
        let mut renderer = Renderer::new(window.clone()).await?;
        renderer.avatar_type = initial_cfg.avatar_type.clone();
        // Simpan dimensi awal window untuk mengunci posisi karakter di layar
        renderer.anchor_center_x = (win_w * 0.5) as f32;
        renderer.anchor_center_y = (win_h * 0.5) as f32;

        // Cari path model VRoid/VRM di berbagai kemungkinan direktori
        let candidates = [
            server::find_file_in_candidates("char.vrm"),
            server::find_file_in_candidates("char.glb"),
            PathBuf::from("assets/char.vrm"),
            PathBuf::from("assets/char.glb"),
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
            current_emotion: "neutral".to_string(),
            is_running: false,
            is_paused: false,
            voice_muted: false,
            mic_muted: false,
            poses_updated: false,
            config_updated: false,
            vrm_updated: false,
            tuning_mode: false,
            live_tuning_bones: std::collections::HashMap::new(),
            state_version: 1,
        }));

        server::register_shared_state(Arc::clone(&shared_server_state));
        server::start_http_server(Arc::clone(&shared_server_state));

        Ok(Self {
            window,
            renderer,
            pose_manager,
            last_frame_time: Instant::now(),
            shared_server_state,
            is_right_mouse_down: false,
            last_cursor_pos: None,
            is_window_focused: false,
            active_resize_dir: None,
            prev_win_size: (win_w as f32, win_h as f32),
            was_maximized: false,
        })
    }

    pub fn handle_window_event(&mut self, event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        match event {
            WindowEvent::Focused(focused) => {
                self.is_window_focused = focused;
                self.renderer.is_header_visible = focused;
            }
            WindowEvent::CursorEntered { .. } => {
                self.renderer.is_header_visible = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.renderer.cursor_pixel_pos = [-100.0, -100.0];
                self.renderer.is_header_visible = self.is_window_focused;
            }
            WindowEvent::CloseRequested => {
                println!("🛑 Menutup aplikasi Ragasukma...");
                std::process::exit(0);
            }
            WindowEvent::Resized(physical_size) => {
                let new_w = physical_size.width as f32;
                let new_h = physical_size.height as f32;
                self.prev_win_size = (new_w, new_h);
                self.renderer.resize(physical_size);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let physical_size = self.window.inner_size();
                self.renderer.resize(physical_size);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let winit::keyboard::Key::Character(c) = &event.logical_key {
                        if c.eq_ignore_ascii_case("m") {
                            // Minimize window
                            self.window.set_minimized(true);
                        } else if c.eq_ignore_ascii_case("f") {
                            // Toggle Maximize / Fullscreen
                            let is_max = self.window.is_maximized();
                            self.window.set_maximized(!is_max);
                        } else if c.eq_ignore_ascii_case("q") {
                            // Exit / Close
                            println!("🛑 Menutup aplikasi Ragasukma (Key Q)...");
                            std::process::exit(0);
                        }
                    } else if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape) = &event.logical_key {
                        println!("🛑 Menutup aplikasi Ragasukma (Key ESC)...");
                        std::process::exit(0);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    let win_size = self.window.inner_size();
                    let w = win_size.width as f64;
                    let h = win_size.height as f64;
                    let border_margin = 10.0;

                    if let Some((cur_x, cur_y)) = self.last_cursor_pos {
                        // 1. Cek Resize Border di seluruh 4 Sisi & 4 Sudut Window
                        let on_left = cur_x <= border_margin;
                        let on_right = cur_x >= (w - border_margin);
                        let on_top = cur_y <= border_margin;
                        let on_bottom = cur_y >= (h - border_margin);

                        let resize_dir = if on_top && on_left {
                            Some(winit::window::ResizeDirection::NorthWest)
                        } else if on_top && on_right {
                            Some(winit::window::ResizeDirection::NorthEast)
                        } else if on_bottom && on_left {
                            Some(winit::window::ResizeDirection::SouthWest)
                        } else if on_bottom && on_right {
                            Some(winit::window::ResizeDirection::SouthEast)
                        } else if on_left {
                            Some(winit::window::ResizeDirection::West)
                        } else if on_right {
                            Some(winit::window::ResizeDirection::East)
                        } else if on_bottom {
                            Some(winit::window::ResizeDirection::South)
                        } else if on_top && cur_x < (w - 90.0) {
                            Some(winit::window::ResizeDirection::North)
                        } else {
                            None
                        };

                        if let Some(dir) = resize_dir {
                            self.active_resize_dir = Some(dir);
                            // Tarik border / sudut window: Resize jendela secara native OS
                            let _ = self.window.drag_resize_window(dir);
                            return;
                        }

                        // 2. Cek Klik pada Header Bar di bagian atas (y <= 28.0)
                        if cur_y <= 28.0 {
                            // Tombol Close (✕)
                            if cur_x >= (w - 28.0) && cur_x <= (w - 4.0) {
                                println!("🛑 Menutup aplikasi Ragasukma (Header Close)...");
                                std::process::exit(0);
                            }
                            // Tombol Maximize / Fullscreen (⛶)
                            if cur_x >= (w - 56.0) && cur_x <= (w - 32.0) {
                                let is_max = self.window.is_maximized();
                                self.window.set_maximized(!is_max);
                                return;
                            }
                            // Tombol Minimize (—)
                            if cur_x >= (w - 84.0) && cur_x <= (w - 60.0) {
                                self.window.set_minimized(true);
                                return;
                            }

                            // Klik di header bar lainnya -> Drag window
                            let _ = self.window.drag_window();
                            return;
                        }
                    }

                    // Klik Kiri Drag pada body avatar: Pindahkan posisi jendela avatar secara native OS
                    let _ = self.window.drag_window();
                } else if button == MouseButton::Left && state == ElementState::Released {
                    self.active_resize_dir = None;
                } else if button == MouseButton::Right {
                    self.is_right_mouse_down = state == ElementState::Pressed;
                } else if button == MouseButton::Middle && state == ElementState::Pressed {
                    // Klik Tengah: Reset rotasi & zoom kamera avatar
                    self.renderer.reset_camera();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.renderer.is_header_visible = true;
                self.renderer.cursor_pixel_pos = [position.x as f32, position.y as f32];
                let win_size = self.window.inner_size();
                let w = win_size.width as f64;
                let h = win_size.height as f64;
                let border_margin = 10.0;

                // Update Icon Kursor sesuai posisi tepi border
                let on_left = position.x <= border_margin;
                let on_right = position.x >= (w - border_margin);
                let on_top = position.y <= border_margin;
                let on_bottom = position.y >= (h - border_margin);

                if (on_top && on_left) || (on_bottom && on_right) {
                    self.window.set_cursor_icon(winit::window::CursorIcon::NwseResize);
                } else if (on_top && on_right) || (on_bottom && on_left) {
                    self.window.set_cursor_icon(winit::window::CursorIcon::NeswResize);
                } else if on_left || on_right {
                    self.window.set_cursor_icon(winit::window::CursorIcon::EwResize);
                } else if on_bottom || (on_top && position.x < (w - 90.0)) {
                    self.window.set_cursor_icon(winit::window::CursorIcon::NsResize);
                } else {
                    self.window.set_cursor_icon(winit::window::CursorIcon::Default);
                }

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
                    self.renderer.current_emotion = lock.current_emotion.clone();
                    is_tuning = lock.tuning_mode;

                    if is_tuning {
                        // Freeze animasi siklus, aplikasikan langsung live bone rotation dari tuner
                        self.pose_manager.apply_tuning_pose(&lock.live_tuning_bones);
                    } else {
                        if lock.poses_updated {
                            let path = server::find_file_in_candidates("poses.json");
                            self.pose_manager.load_from_file(&path);
                            lock.poses_updated = false;
                        }

                        if lock.vrm_updated {
                            let vrm_candidates = [
                                server::find_file_in_candidates("char.vrm"),
                                PathBuf::from("assets/char.vrm"),
                                PathBuf::from("../assets/char.vrm"),
                                PathBuf::from("assets/char.glb"),
                                PathBuf::from("../assets/char.glb"),
                            ];
                            for p in &vrm_candidates {
                                if p.exists() {
                                    println!("📂 [Asset Loader] Memuat ulang avatar dari: {:?}", p);
                                    if let Ok(model) = VrmModelData::load_from_file(p) {
                                        self.renderer.load_vrm_model(model);
                                        break;
                                    }
                                }
                            }
                            self.renderer.reload_2d_avatar_textures();
                            lock.vrm_updated = false;
                        }

                        if lock.config_updated {
                            let cfg = server::load_config();
                            self.renderer.avatar_type = cfg.avatar_type.clone();
                            let lvl = if cfg.always_on_top {
                                WindowLevel::AlwaysOnTop
                            } else {
                                WindowLevel::Normal
                            };
                            self.window.set_window_level(lvl);
                            lock.config_updated = false;
                        }
                    }
                }

                let delta_sim = if is_tuning { 0.0 } else { delta };
                self.renderer.current_visual_state = self.pose_manager.current_state;
                self.renderer.elapsed_time += delta;
                self.renderer.update_camera_and_bones(&mut self.pose_manager, delta_sim, is_tuning);

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
