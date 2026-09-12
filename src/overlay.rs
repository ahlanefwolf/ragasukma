use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

use crate::ats_webview::*;
use crate::skills::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PositionConfig {
    pub corner: String,
    pub margin_x: i32,
    pub margin_y: i32,
    pub size: i32,
}

impl Default for PositionConfig {
    fn default() -> Self {
        Self {
            corner: "top-right".to_string(),
            margin_x: 24,
            margin_y: 24,
            size: 320,
        }
    }
}

fn get_user_home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

fn config_path() -> PathBuf {
    if let Some(home) = get_user_home_dir() {
        home.join(".config/ai-voice/overlay_pos.json")
    } else {
        PathBuf::from("overlay_pos.json")
    }
}

fn load_position_config() -> PositionConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<PositionConfig>(&content) {
                return cfg;
            }
        }
    }
    PositionConfig::default()
}

fn save_position_config(cfg: &PositionConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(path, json);
    }
}

fn poses_config_path() -> PathBuf {
    let local = PathBuf::from("poses.json");
    if local.exists() {
        return local;
    }
    if let Some(home) = get_user_home_dir() {
        let cfg_path = home.join(".config/ai-voice/poses.json");
        if cfg_path.exists() {
            return cfg_path;
        }
    }
    local
}

#[tauri::command]
fn get_poses_config() -> serde_json::Value {
    let path = poses_config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            return v;
        }
    }
    serde_json::json!({})
}

#[tauri::command]
fn save_poses_config(app: tauri::AppHandle, config_json: String) -> Result<(), String> {
    let path = poses_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&path, &config_json).map_err(|e| e.to_string())?;

    // Simpan juga ke home config sebagai cadangan
    if let Some(home) = get_user_home_dir() {
        let backup = home.join(".config/ai-voice/poses.json");
        if let Some(parent) = backup.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(backup, &config_json);
    }
    println!("💾 [Pose Tuner] Berhasil menyimpan poses.json ke {:?}", path);

    // Kirim sinyal hanya saat tombol Save ditekan
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.eval("if (window.loadExternalPoses) { window.loadExternalPoses(); }");
    }

    Ok(())
}

fn get_asset_file_path(filename: &str) -> Option<PathBuf> {
    // 1. Cek di ./assets/<filename>
    let local = PathBuf::from("assets").join(filename);
    if local.exists() {
        return Some(local);
    }
    // 2. Cek relatif terhadap lokasi binary yang sedang berjalan
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let p1 = parent.join("assets").join(filename);
            if p1.exists() { return Some(p1); }
            if let Some(parent2) = parent.parent() {
                let p2 = parent2.join("assets").join(filename);
                if p2.exists() { return Some(p2); }
                if let Some(parent3) = parent2.parent() {
                    let p3 = parent3.join("assets").join(filename);
                    if p3.exists() { return Some(p3); }
                }
            }
        }
    }
    // 3. Cek di ~/.config/ai-voice/assets/<filename>
    if let Some(home) = get_user_home_dir() {
        let home_asset = home.join(".config/ai-voice/assets").join(filename);
        if home_asset.exists() {
            return Some(home_asset);
        }
    }
    // 4. Cek di ./ui/assets/<filename>
    let ui_asset = PathBuf::from("ui/assets").join(filename);
    if ui_asset.exists() {
        return Some(ui_asset);
    }
    None
}

#[tauri::command]
fn log_frontend(msg: String) {
    println!("🖥️ [Overlay WebKit]: {}", msg);
}

#[tauri::command]
fn get_external_asset_bytes(filename: String) -> Result<Vec<u8>, String> {
    let path = get_asset_file_path(&filename);
    println!("🔍 [Overlay Asset Request] filename: '{}', resolved path: {:?}", filename, path);
    let path = path.ok_or_else(|| {
        eprintln!("❌ [Overlay Asset Error] File aset '{}' tidak ditemukan di folder assets eksternal!", filename);
        format!("File aset '{}' tidak ditemukan di folder assets eksternal", filename)
    })?;
    let bytes = fs::read(&path).map_err(|e| {
        eprintln!("❌ [Overlay Asset Read Error] Gagal baca {:?}: {}", path, e);
        e.to_string()
    })?;
    println!("✅ [Overlay Asset Success] Berhasil membaca {:?} ({} bytes)", path, bytes.len());
    Ok(bytes)
}

#[tauri::command]
fn get_external_asset(filename: String) -> Result<String, String> {
    use base64::Engine;
    let path = get_asset_file_path(&filename)
        .ok_or_else(|| format!("File aset '{}' tidak ditemukan di folder assets eksternal", filename))?;

    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    let mime = if filename.ends_with(".vrm") || filename.ends_with(".glb") {
        "model/gltf-binary"
    } else if filename.ends_with(".svg") {
        "image/svg+xml"
    } else if filename.ends_with(".png") {
        "image/png"
    } else {
        "application/octet-stream"
    };

    Ok(format!("data:{};base64,{}", mime, b64))
}

#[tauri::command]
fn open_tuner_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("pose-tuner") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "pose-tuner", tauri::WebviewUrl::App("tuner.html".into()))
        .title("VRM Pose & Motion Tuner")
        .inner_size(880.0, 680.0)
        .resizable(true)
        .transparent(false)
        .decorations(true)
        .always_on_top(false)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}



#[tauri::command]
fn open_menu_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("assistant-menu") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "assistant-menu", tauri::WebviewUrl::App("menu.html".into()))
        .title("Assistant Menu & Settings")
        .inner_size(380.0, 590.0)
        .resizable(true)
        .transparent(false)
        .decorations(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn open_skills_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("skills") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(&app, "skills", tauri::WebviewUrl::App("skills.html".into()))
        .title("Skill Commands Manager")
        .inner_size(680.0, 560.0)
        .resizable(true)
        .transparent(false)
        .decorations(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub static APP_HANDLE: std::sync::LazyLock<Mutex<Option<tauri::AppHandle>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

pub struct OverlayState {
    pub assistant_name: String,
    pub avatar_type: String,
    pub tts_engine: String,
    pub position_config: Mutex<PositionConfig>,
}

#[tauri::command]
fn get_app_config(state: tauri::State<'_, OverlayState>) -> serde_json::Value {
    let cfg = match state.position_config.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    serde_json::json!({
        "assistant_name": state.assistant_name,
        "avatar_type": state.avatar_type,
        "tts_engine": state.tts_engine,
        "size": cfg.size,
        "corner": cfg.corner,
    })
}

#[tauri::command]
fn get_app_state() -> Option<serde_json::Value> {
    crate::tray::TrayState::get_state()
}

#[tauri::command]
fn send_command(cmd: String) {
    if cmd == "test_voice" {
        std::thread::spawn(|| {
            let user_callsign = std::env::var("USER_CALLSIGN")
                .or_else(|_| std::env::var("USER_NAME"))
                .unwrap_or_else(|_| "Kak".to_string());

            let tts = crate::tts::TextToSpeech::new();
            let is_interrupted = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            let text = format!("Halo {}! Ini adalah pengujian suara asisten. Suara saya sudah terdengar dengan jelas kan?", user_callsign);
            let _ = tts.speak_interruptible(&text, is_interrupted);
        });
        return;
    }
    crate::tray::TrayState::push_command(cmd);
}

#[tauri::command]
fn start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn drag_window_delta(
    window: tauri::Window,
    dx: i32,
    dy: i32,
    state: tauri::State<'_, OverlayState>,
) {
    if let Ok(pos) = window.outer_position() {
        let new_x = pos.x + dx;
        let new_y = pos.y + dy;
        let _ = window.set_position(tauri::Position::Physical(PhysicalPosition {
            x: new_x,
            y: new_y,
        }));

        if let Ok(Some(monitor)) = window.current_monitor() {
            let m_size = monitor.size();
            let mut cfg = match state.position_config.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let corner = cfg.corner.clone();

            match corner.as_str() {
                "top-right" => {
                    let w = if state.avatar_type.eq_ignore_ascii_case("vrm") { (cfg.size * 9) / 16 } else { cfg.size };
                    cfg.margin_x = (m_size.width as i32 - (new_x + w)).max(-200);
                    cfg.margin_y = new_y.max(-200);
                }
                "top-left" => {
                    cfg.margin_x = new_x.max(-200);
                    cfg.margin_y = new_y.max(-200);
                }
                "bottom-right" => {
                    let w = if state.avatar_type.eq_ignore_ascii_case("vrm") { (cfg.size * 9) / 16 } else { cfg.size };
                    let h = cfg.size;
                    cfg.margin_x = (m_size.width as i32 - (new_x + w)).max(-200);
                    cfg.margin_y = (m_size.height as i32 - (new_y + h)).max(-200);
                }
                "bottom-left" => {
                    let h = cfg.size;
                    cfg.margin_x = new_x.max(-200);
                    cfg.margin_y = (m_size.height as i32 - (new_y + h)).max(-200);
                }
                _ => {}
            }
        }
    }
}

#[tauri::command]
fn save_current_position(state: tauri::State<'_, OverlayState>) {
    let cfg = match state.position_config.lock() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    save_position_config(&cfg);
}

#[tauri::command]
fn change_size_delta(
    window: tauri::Window,
    delta: i32,
    state: tauri::State<'_, OverlayState>,
) -> i32 {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let old_size = cfg.size;
    let new_size = (old_size + delta).clamp(60, 1800);

    if new_size != old_size {
        cfg.size = new_size;
        let w = if state.avatar_type.eq_ignore_ascii_case("vrm") {
            (new_size * 9) / 16
        } else {
            new_size
        };
        let h = new_size;

        let _ = window.set_size(tauri::Size::Physical(PhysicalSize {
            width: w as u32,
            height: h as u32,
        }));
        save_position_config(&cfg);
    }
    cfg.size
}

#[tauri::command]
fn reset_size(window: tauri::Window, state: tauri::State<'_, OverlayState>) {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    cfg.size = 320;
    apply_position_to_window(&window, &cfg, &state.avatar_type);
    save_position_config(&cfg);
}

#[tauri::command]
fn set_corner(window: tauri::Window, corner: String, state: tauri::State<'_, OverlayState>) {
    let mut cfg = match state.position_config.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    cfg.corner = corner.clone();
    cfg.margin_x = 24;
    cfg.margin_y = 24;
    apply_position_to_window(&window, &cfg, &state.avatar_type);
    save_position_config(&cfg);
}

/// Ambil geometri output aktif dari swaymsg (logical pixels, bukan physical)
/// Return: (width, height, x, y) dalam logical px
fn get_sway_output_geometry() -> Option<(i32, i32, i32, i32)> {
    let out = std::process::Command::new("swaymsg")
        .args(["-t", "get_outputs"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    // Parse JSON minimal — cari output yang "active": true
    // Format: [{"name":"...","active":true,"rect":{"x":0,"y":0,"width":1920,"height":1080},...}]
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let arr = v.as_array()?;
    for o in arr {
        if o.get("active").and_then(|a| a.as_bool()).unwrap_or(false) {
            if let Some(rect) = o.get("rect") {
                let x = rect.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let y = rect.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let w = rect.get("width").and_then(|v| v.as_i64()).unwrap_or(1920) as i32;
                let h = rect.get("height").and_then(|v| v.as_i64()).unwrap_or(1080) as i32;
                return Some((w, h, x, y));
            }
        }
    }
    // Fallback: ambil yang pertama
    if let Some(o) = arr.first() {
        if let Some(rect) = o.get("rect") {
            let x = rect.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let y = rect.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let w = rect.get("width").and_then(|v| v.as_i64()).unwrap_or(1920) as i32;
            let h = rect.get("height").and_then(|v| v.as_i64()).unwrap_or(1080) as i32;
            return Some((w, h, x, y));
        }
    }
    None
}

fn apply_position_to_window(window: &tauri::Window, cfg: &PositionConfig, avatar_type: &str) {
    let w = if avatar_type.eq_ignore_ascii_case("vrm") {
        (cfg.size * 9) / 16
    } else {
        cfg.size
    };
    let h = cfg.size;

    // Di Sway/Wayland: gunakan swaymsg get_outputs untuk logical px yang akurat
    // Di X11/fallback: gunakan Tauri monitor API
    let (screen_w, screen_h, screen_x, screen_y) = if std::env::var("SWAYSOCK").is_ok() {
        get_sway_output_geometry().unwrap_or_else(|| {
            // Fallback Tauri
            if let Ok(Some(m)) = window.primary_monitor() {
                (m.size().width as i32, m.size().height as i32, m.position().x, m.position().y)
            } else {
                (1920, 1080, 0, 0)
            }
        })
    } else if let Ok(Some(m)) = window.current_monitor() {
        (m.size().width as i32, m.size().height as i32, m.position().x, m.position().y)
    } else if let Ok(Some(m)) = window.primary_monitor() {
        (m.size().width as i32, m.size().height as i32, m.position().x, m.position().y)
    } else {
        (1920, 1080, 0, 0)
    };

    // Pastikan window tidak melebihi ukuran layar
    let w = w.min(screen_w);
    let h = h.min(screen_h);

    let (x, y) = match cfg.corner.as_str() {
        "top-left" => (
            screen_x + cfg.margin_x,
            screen_y + cfg.margin_y,
        ),
        "top-right" => (
            screen_x + (screen_w - w - cfg.margin_x),
            screen_y + cfg.margin_y,
        ),
        "bottom-left" => (
            screen_x + cfg.margin_x,
            screen_y + (screen_h - h - cfg.margin_y),
        ),
        "bottom-right" => (
            screen_x + (screen_w - w - cfg.margin_x),
            screen_y + (screen_h - h - cfg.margin_y),
        ),
        _ => (
            screen_x + (screen_w - w - cfg.margin_x),
            screen_y + cfg.margin_y,
        ),
    };

    let _ = window.set_size(tauri::Size::Physical(PhysicalSize {
        width: w as u32,
        height: h as u32,
    }));
    let _ = window.set_position(tauri::Position::Physical(PhysicalPosition { x, y }));
    let _ = window.set_always_on_top(true);
    let _ = window.show();

    // Sway (Wayland tiling WM): gerakkan via swaymsg dengan kriteria app_id/title yang tepat
    if std::env::var("SWAYSOCK").is_ok() {
        let sway_cmd = format!(
            r#"[title="(?i).*overlay.*|ahlan-voice"] floating enable, sticky enable, border none, resize set {} px {} px, move position {} px {} px"#,
            w, h, x, y
        );
        thread::spawn(move || {
            let _ = std::process::Command::new("swaymsg")
                .args([&sway_cmd])
                .output();
        });
    }
}

pub struct NativeOverlay;

impl NativeOverlay {
    pub fn run(avatar_type: &str, assistant_name: &str, tts_engine: &str) {
        let av_type = avatar_type.to_lowercase();
        let pos_cfg = load_position_config();
        let overlay_state = OverlayState {
            assistant_name: assistant_name.to_string(),
            avatar_type: av_type.clone(),
            tts_engine: tts_engine.to_string(),
            position_config: Mutex::new(pos_cfg.clone()),
        };

        let app = tauri::Builder::default()
            .manage(overlay_state)
            .invoke_handler(tauri::generate_handler![
                get_app_config,
                get_app_state,
                send_command,
                start_drag,
                drag_window_delta,
                save_current_position,
                change_size_delta,
                reset_size,
                set_corner,
                get_skills,
                save_skill,
                delete_skill,
                execute_skill,
                open_skills_window,
                open_menu_window,
                open_tuner_window,
                get_poses_config,
                save_poses_config,
                get_external_asset,
                get_external_asset_bytes,
                log_frontend,
                open_anytospeech_window,
                ats_report_status
            ])
            .setup(move |app| {
                if let Ok(mut guard) = APP_HANDLE.lock() {
                    *guard = Some(app.handle().clone());
                }

                // Otomatis buka jendela WebKit AnyToSpeech / KikiVoice jika TTS_ENGINE diset
                let tts_engine_env = std::env::var("TTS_ENGINE").unwrap_or_default().to_lowercase();
                if tts_engine_env == "anytospeech" || tts_engine_env == "ats"
                    || tts_engine_env == "natural_readers" || tts_engine_env == "naturalreaders"
                    || tts_engine_env == "kiki" || tts_engine_env == "kikivoice" {
                    let app_h = app.handle().clone();
                    thread::spawn(move || {
                        thread::sleep(std::time::Duration::from_millis(500));
                        let _ = open_anytospeech_window(app_h);
                    });
                }
                if let Some(window) = app.get_webview_window("main") {
                    let state = app.state::<OverlayState>();
                    let cfg = match state.position_config.lock() {
                        Ok(guard) => guard.clone(),
                        Err(poisoned) => poisoned.into_inner().clone(),
                    };
                    apply_position_to_window(&window.as_ref().window(), &cfg, &av_type);
                    println!("🪟 Tauri Overlay Window berhasil ditampilkan di layar.");
                } else {
                    eprintln!("⚠️ Warning: Window 'main' tidak ditemukan di Tauri.");
                }

                // Background watcher untuk sinyal quit (100% in-memory thread-safe)
                let handle = app.handle().clone();
                thread::spawn(move || {
                    loop {
                        thread::sleep(std::time::Duration::from_millis(100));
                        if crate::tray::TrayState::is_quit_requested() {
                            handle.exit(0);
                            break;
                        }
                    }
                });

                Ok(())
            })
            .build(tauri::generate_context!());

        match app {
            Ok(app) => {
                app.run(|_app_handle, _event| {});
            }
            Err(err) => {
                eprintln!("⚠️ Gagal menginisialisasi Tauri overlay: {}", err);
            }
        }
    }


}
