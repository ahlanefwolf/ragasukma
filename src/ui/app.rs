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
use super::vrm_loader::VrmModelData;
use crate::app_state::ServerSharedState;
use crate::assets;
use crate::server;

pub fn open_browser(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

pub struct App {
    pub window: Arc<Window>,
    pub renderer: Box<Renderer>,
    pub pose_manager: PoseManager,
    pub last_frame_time: Instant,
    pub shared_server_state: Arc<Mutex<ServerSharedState>>,
    pub vrm_loader_rx: Option<std::sync::mpsc::Receiver<Box<VrmModelData>>>,
    pub is_right_mouse_down: bool,
    pub is_left_mouse_down: bool,
    pub is_ctrl_down: bool,
    pub right_press_pos: Option<(f64, f64)>,
    pub right_dragged: bool,
    pub last_cursor_pos: Option<(f64, f64)>,
    pub is_window_focused: bool,
    pub active_resize_dir: Option<winit::window::ResizeDirection>,
    pub prev_win_size: (f32, f32),
    pub was_maximized: bool,
    pub last_config_check: Instant,
    pub last_config_mtime: Option<std::time::SystemTime>,
    pub has_opened_browser: bool,
}

impl App {
    pub async fn new(window_target: &EventLoopWindowTarget<()>) -> anyhow::Result<Self> {
        let initial_cfg = assets::load_config();

        let initial_level = if initial_cfg.always_on_top {
            WindowLevel::AlwaysOnTop
        } else {
            WindowLevel::Normal
        };

        let monitor = window_target
            .primary_monitor()
            .or_else(|| window_target.available_monitors().next());
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
            .with_visible(true)
            .with_window_level(initial_level)
            .with_resizable(true);

        if let Some(icon) = window_icon {
            win_builder = win_builder.with_window_icon(Some(icon));
        }

        let window = win_builder.build(window_target)?;

        let window = Arc::new(window);
        let mut renderer = Box::new(Renderer::new(window.clone()).await?);
        renderer.avatar_type = initial_cfg.avatar_type.clone();
        renderer.anchor_center_x = (win_w * 0.5) as f32;
        renderer.anchor_center_y = (win_h * 0.5) as f32;

        // Lazy load asinkron via background thread: jika avatar_type adalah VRM
        let is_vrm = initial_cfg.avatar_type.eq_ignore_ascii_case("vrm");
        let vrm_loader_rx = if is_vrm {
            renderer.is_loading = true;
            let (vrm_tx, vrm_rx) = std::sync::mpsc::channel::<Box<VrmModelData>>();
            std::thread::spawn(move || {
                let candidates = [
                    assets::find_file_in_candidates("char.vrm"),
                    assets::find_file_in_candidates("char.glb"),
                    PathBuf::from("assets/char.vrm"),
                    PathBuf::from("assets/char.glb"),
                ];
                for p in &candidates {
                    if p.exists() {
                        println!(
                            "📂 [Async Asset Loader] Memuat model avatar di background: {:?}",
                            p
                        );
                        if let Ok(model) = VrmModelData::load_from_file(p) {
                            let _ = vrm_tx.send(model);
                            break;
                        }
                    }
                }
            });
            Some(vrm_rx)
        } else {
            renderer.is_loading = false;
            None
        };

        let pose_manager = PoseManager::default();

        let shared_server_state = Arc::new(Mutex::new(ServerSharedState {
            current_state: AssistantVisualState::Idle,
            current_emotion: "neutral".to_string(),
            is_running: false,
            is_paused: false,
            is_shutdown: false,
            voice_muted: false,
            mic_muted: false,
            mic_level: 0,
            poses_updated: false,
            config_updated: false,
            vrm_updated: false,
            tuning_mode: false,
            live_tuning_bones: std::collections::HashMap::new(),
            preview_poses: None,
            state_version: 1,
        }));

        crate::app_state::register_shared_state(Arc::clone(&shared_server_state));
        server::start_http_server(Arc::clone(&shared_server_state));

        Ok(Self {
            window,
            renderer,
            pose_manager,
            last_frame_time: Instant::now(),
            shared_server_state,
            vrm_loader_rx,
            is_right_mouse_down: false,
            is_left_mouse_down: false,
            is_ctrl_down: false,
            right_press_pos: None,
            right_dragged: false,
            last_cursor_pos: None,
            is_window_focused: false,
            active_resize_dir: None,
            prev_win_size: (win_w as f32, win_h as f32),
            was_maximized: false,
            last_config_check: Instant::now(),
            last_config_mtime: crate::config::get_config_modified_time(),
            has_opened_browser: false,
        })
    }

    pub fn handle_window_event(&mut self, event: WindowEvent, elwt: &EventLoopWindowTarget<()>) {
        match event {
            WindowEvent::Focused(focused) => {
                self.is_window_focused = focused;
                self.renderer.is_header_visible = focused;
                if !focused {
                    self.is_left_mouse_down = false;
                    self.is_right_mouse_down = false;
                    self.is_ctrl_down = false;
                }
            }
            WindowEvent::CursorEntered { .. } => {
                self.renderer.is_header_visible = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.renderer.cursor_pixel_pos = [-100.0, -100.0];
                self.renderer.is_header_visible = self.is_window_focused;
                self.is_left_mouse_down = false;
                self.is_right_mouse_down = false;
            }
            WindowEvent::CloseRequested => {
                println!("🛑 Menutup aplikasi Ragasukma (Window Close)...");
                crate::app_state::trigger_app_shutdown();
                crate::tray::TrayState::push_command("quit".to_string());
                // Jeda 200ms agar browser menerima SSE event shutdown dan membersihkan seluruh resource/player
                std::thread::sleep(std::time::Duration::from_millis(200));
                elwt.exit();
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
            WindowEvent::ModifiersChanged(modifiers) => {
                self.is_ctrl_down = modifiers.state().control_key();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Control) =
                    &event.logical_key
                {
                    self.is_ctrl_down = event.state == ElementState::Pressed;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    self.is_left_mouse_down = true;

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
                                crate::app_state::trigger_app_shutdown();
                                crate::tray::TrayState::push_command("quit".to_string());
                                // Jeda 200ms agar browser menerima SSE event shutdown dan membersihkan seluruh resource/player
                                std::thread::sleep(std::time::Duration::from_millis(200));
                                elwt.exit();
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
                    self.is_left_mouse_down = false;
                    self.active_resize_dir = None;
                } else if button == MouseButton::Right {
                    if state == ElementState::Pressed {
                        self.is_right_mouse_down = true;
                        self.right_press_pos = self.last_cursor_pos;
                        self.right_dragged = false;
                    } else {
                        // Released
                        self.is_right_mouse_down = false;
                    }
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
                    self.window
                        .set_cursor_icon(winit::window::CursorIcon::NwseResize);
                } else if (on_top && on_right) || (on_bottom && on_left) {
                    self.window
                        .set_cursor_icon(winit::window::CursorIcon::NeswResize);
                } else if on_left || on_right {
                    self.window
                        .set_cursor_icon(winit::window::CursorIcon::EwResize);
                } else if on_bottom || (on_top && position.x < (w - 90.0)) {
                    self.window
                        .set_cursor_icon(winit::window::CursorIcon::NsResize);
                } else {
                    self.window
                        .set_cursor_icon(winit::window::CursorIcon::Default);
                }

                if let Some((prev_x, prev_y)) = self.last_cursor_pos {
                    if self.is_right_mouse_down {
                        let dx = (position.x - prev_x).abs();
                        let dy = (position.y - prev_y).abs();
                        if dx > 1.5 || dy > 1.5 {
                            self.right_dragged = true;
                        }

                        let delta_x = (position.x - prev_x) as f32;
                        let delta_y = (position.y - prev_y) as f32;

                        if self.is_ctrl_down {
                            // Ctrl + Klik Kanan Drag: Pan / Geser Posisi Karakter 3D
                            self.renderer.pan_camera(delta_x, delta_y);
                        } else {
                            // Klik Kanan Drag: Orbit rotasi kamera 3D
                            let sensitivity = 0.008;
                            let delta_yaw = -delta_x * sensitivity;
                            let delta_pitch = delta_y * sensitivity;
                            self.renderer.rotate_camera(delta_yaw, delta_pitch);
                        }
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
                let mut is_previewing = false;
                let mut live_tuner_emotion = String::new();

                // Cek background loader channel untuk loading awal asinkron
                if let Some(rx) = &self.vrm_loader_rx {
                    if let Ok(model) = rx.try_recv() {
                        self.renderer.load_vrm_model(model);
                        self.renderer.is_loading = false;
                        self.vrm_loader_rx = None;
                        // Buka browser dashboard hanya sekali setelah avatar/model 3D pertama kali selesai dimuat ke GPU!
                        if !self.has_opened_browser {
                            self.has_opened_browser = true;
                            assets::open_browser("http://127.0.0.1:42128");
                        }
                    }
                } else if !self.has_opened_browser {
                    // Jika mode 2D (tanpa perlu tunggu VRM loader), langsung buka browser dashboard sekali saat awal start
                    self.has_opened_browser = true;
                    assets::open_browser("http://127.0.0.1:42128");
                }

                // Sync state & live poses reload dari Web HTTP Dashboard
                if let Ok(mut lock) = self.shared_server_state.lock() {
                    self.pose_manager.current_state = lock.current_state;
                    is_tuning = lock.tuning_mode;
                    is_previewing = lock.preview_poses.is_some();
                    live_tuner_emotion = lock.current_emotion.clone();

                    if is_tuning {
                        // Freeze animasi siklus, aplikasikan langsung live bone rotation dari tuner
                        self.pose_manager.apply_tuning_pose(&lock.live_tuning_bones);
                    } else {
                        if let Some(preview) = &lock.preview_poses {
                            // Gunakan preview poses in-memory tanpa overwrite poses.json di disk
                            self.pose_manager.config = preview.clone();
                        } else if lock.poses_updated {
                            // Reload 100% dari file poses.json resmi di disk (Source of Truth)
                            let path = assets::find_file_in_candidates("poses.json");
                            self.pose_manager.load_from_file(&path);
                            lock.poses_updated = false;
                        }

                        if lock.vrm_updated {
                            self.renderer.is_loading = true;
                            let (vrm_tx, vrm_rx) = std::sync::mpsc::channel::<Box<VrmModelData>>();
                            std::thread::spawn(move || {
                                let vrm_candidates = [
                                    assets::find_file_in_candidates("char.vrm"),
                                    PathBuf::from("assets/char.vrm"),
                                    PathBuf::from("../assets/char.vrm"),
                                    PathBuf::from("assets/char.glb"),
                                    PathBuf::from("../assets/char.glb"),
                                ];
                                for p in &vrm_candidates {
                                    if p.exists() {
                                        println!(
                                            "📂 [Async Asset Loader] Memuat ulang model avatar di background: {:?}",
                                            p
                                        );
                                        if let Ok(model) = VrmModelData::load_from_file(p) {
                                            let _ = vrm_tx.send(model);
                                            break;
                                        }
                                    }
                                }
                            });
                            self.vrm_loader_rx = Some(vrm_rx);
                            self.renderer.reload_2d_avatar_textures();
                            lock.vrm_updated = false;
                        }

                        let check_file = self.last_config_check.elapsed() >= std::time::Duration::from_millis(500);
                        let file_changed = if check_file {
                            self.last_config_check = Instant::now();
                            let current_mtime = crate::config::get_config_modified_time();
                            if current_mtime != self.last_config_mtime {
                                self.last_config_mtime = current_mtime;
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if lock.config_updated || file_changed {
                            let cfg = assets::load_config();
                            let old_avatar_type = self.renderer.avatar_type.clone();
                            self.renderer.avatar_type = cfg.avatar_type.clone();

                            if !self.renderer.avatar_type.eq_ignore_ascii_case(&old_avatar_type) {
                                if self.renderer.avatar_type.eq_ignore_ascii_case("2d") {
                                    self.renderer.reload_2d_avatar_textures();
                                } else if self.renderer.avatar_type.eq_ignore_ascii_case("vrm") {
                                    if !self.renderer.has_model() && self.vrm_loader_rx.is_none() {
                                        self.renderer.is_loading = true;
                                        let (vrm_tx, vrm_rx) = std::sync::mpsc::channel::<Box<VrmModelData>>();
                                        std::thread::spawn(move || {
                                            let vrm_candidates = [
                                                assets::find_file_in_candidates("char.vrm"),
                                                PathBuf::from("assets/char.vrm"),
                                                PathBuf::from("../assets/char.vrm"),
                                                PathBuf::from("assets/char.glb"),
                                                PathBuf::from("../assets/char.glb"),
                                            ];
                                            for p in &vrm_candidates {
                                                if p.exists() {
                                                    println!(
                                                        "📂 [Async Asset Loader] Memuat model avatar di background: {:?}",
                                                        p
                                                    );
                                                    if let Ok(model) = VrmModelData::load_from_file(p) {
                                                        let _ = vrm_tx.send(model);
                                                        break;
                                                    }
                                                }
                                            }
                                        });
                                        self.vrm_loader_rx = Some(vrm_rx);
                                    }
                                }
                            }

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

                // Hirarki Emosi (diupdate SEBELUM update_camera_and_bones agar diaplikasikan pada frame ini):
                // 1. Jika sedang di Pose Tuner (Tuning edit mode atau Preview loop), gunakan emosi realtime yang dipilih user di UI
                // 2. Jika sedang Thinking, tampilkan ekspresi thinking
                // 3. Saat Speaking, Idle, atau Listening, pertahankan emosi terakhir (misal tetap tersipu/senang/cemberut saat idle)!
                if is_tuning || is_previewing {
                    self.renderer.current_emotion = live_tuner_emotion;
                } else if self.pose_manager.current_state
                    == crate::ui::pose_system::AssistantVisualState::Thinking
                {
                    self.renderer.current_emotion = "thinking".to_string();
                } else {
                    self.renderer.current_emotion = live_tuner_emotion;
                }

                self.renderer
                    .update_camera_and_bones(&mut self.pose_manager, delta_sim, is_tuning);

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
