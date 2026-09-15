use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::pose_system::{AssistantVisualState, PosesConfig};
use crate::config::UserConfig;
use crate::skills::{
    execute_command_by_os, get_installed_applications, parse_skills_markdown,
    save_skills_markdown, SkillItem,
};

#[derive(RustEmbed)]
#[folder = "web/dist"]
pub struct EmbeddedWebDist;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppStatePayload {
    pub state: String,
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
}

pub struct ServerSharedState {
    pub current_state: AssistantVisualState,
    pub current_emotion: String,
    pub is_running: bool,
    pub is_paused: bool,
    pub voice_muted: bool,
    pub mic_muted: bool,
    pub poses_updated: bool,
    pub config_updated: bool,
    pub vrm_updated: bool,
    pub tuning_mode: bool,
    pub live_tuning_bones: HashMap<String, [f32; 3]>,
    /// Live preview pose in-memory tanpa overwrite poses.json di disk
    pub preview_poses: Option<PosesConfig>,
    pub state_version: u64,
}

static GLOBAL_SHARED_STATE: Mutex<Option<Arc<Mutex<ServerSharedState>>>> = Mutex::new(None);

struct ActiveWebSession {
    token: String,
    last_seen: std::time::Instant,
}
static ACTIVE_WEB_SESSION: Mutex<Option<ActiveWebSession>> = Mutex::new(None);

pub fn register_shared_state(shared: Arc<Mutex<ServerSharedState>>) {
    if let Ok(mut lock) = GLOBAL_SHARED_STATE.lock() {
        *lock = Some(shared);
    }
}

pub fn update_session_running(running: bool) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                if s.is_running != running {
                    s.is_running = running;
                    if !running {
                        s.current_state = AssistantVisualState::Idle;
                    }
                    s.state_version = s.state_version.wrapping_add(1);
                }
            }
        }
    }
}

pub fn update_visual_state(state: AssistantVisualState) {
    update_visual_state_with_emotion(state, "neutral");
}

pub fn update_visual_state_with_emotion(state: AssistantVisualState, emotion: &str) {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(mut s) = shared.lock() {
                let changed = s.current_state != state || s.current_emotion != emotion;
                if changed {
                    s.current_state = state;
                    s.current_emotion = emotion.to_string();
                    s.state_version = s.state_version.wrapping_add(1);
                }
            }
        }
    }
}

pub fn is_session_running() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.is_running;
            }
        }
    }
    false
}

pub fn is_tuning_mode() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.tuning_mode;
            }
        }
    }
    false
}

pub fn is_voice_muted() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.voice_muted;
            }
        }
    }
    false
}

pub fn is_mic_muted() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.mic_muted;
            }
        }
    }
    false
}

pub fn is_paused() -> bool {
    if let Ok(lock) = GLOBAL_SHARED_STATE.lock() {
        if let Some(shared) = lock.as_ref() {
            if let Ok(s) = shared.lock() {
                return s.is_paused;
            }
        }
    }
    false
}

pub fn get_assets_dir() -> PathBuf {
    // 1. Cek di samping binary saat diinstall (Windows, Linux Portable, AppImage)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let beside = exe_dir.join("assets");
            if beside.exists() {
                return beside;
            }
            // macOS .app bundle
            let mac_resources = exe_dir.join("../Resources/assets");
            if mac_resources.exists() {
                return mac_resources;
            }
        }
    }

    let local = PathBuf::from("assets");
    if local.exists() || PathBuf::from("Cargo.toml").exists() {
        let _ = fs::create_dir_all(&local);
        return local;
    }
    let parent = PathBuf::from("../assets");
    if parent.exists() {
        return parent;
    }
    let cfg_dir = crate::config::get_app_config_dir().join("assets");
    let _ = fs::create_dir_all(&cfg_dir);
    cfg_dir
}

pub fn find_file_in_candidates(filename: &str) -> PathBuf {
    let mut candidates = Vec::new();

    // 1. Cek di samping executable / bundle installer
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("assets").join(filename));
            candidates.push(exe_dir.join(filename));
            candidates.push(exe_dir.join("../Resources/assets").join(filename));
            candidates.push(exe_dir.join("../Resources").join(filename));
        }
    }

    // 2. Working directory & project folder
    candidates.push(PathBuf::from(filename));
    candidates.push(PathBuf::from(format!("assets/{}", filename)));
    candidates.push(PathBuf::from(format!("../assets/{}", filename)));
    candidates.push(PathBuf::from(format!("../{}", filename)));

    // 3. User config & appdata folder
    candidates.push(crate::config::get_app_config_dir().join("assets").join(filename));
    candidates.push(crate::config::get_app_config_dir().join(filename));

    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }

    // Fallback AppData Windows
    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        let appdata_path = PathBuf::from(appdata);
        let p = appdata_path.join("ragasukma").join("assets").join(filename);
        if p.exists() {
            return p;
        }
        let p_root = appdata_path.join("ragasukma").join(filename);
        if p_root.exists() {
            return p_root;
        }
    }

    PathBuf::from(filename)
}

pub fn load_config() -> UserConfig {
    crate::config::load_user_config()
}

pub fn save_config(cfg: &UserConfig) -> Result<(), String> {
    crate::config::save_user_config(cfg)
}

pub fn load_poses() -> PosesConfig {
    let path = find_file_in_candidates("poses.json");
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(p) = serde_json::from_str::<PosesConfig>(&content) {
            return p;
        }
    }
    PosesConfig::default()
}

pub fn save_poses(poses: &PosesConfig) -> Result<(), String> {
    let path = find_file_in_candidates("poses.json");
    let json = serde_json::to_string_pretty(poses).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn open_browser(url: &str) {
    let url = url.to_string();
    thread::spawn(move || {
        // Beri sedikit jeda agar listener TCP siap menerima koneksi
        thread::sleep(std::time::Duration::from_millis(150));

        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("cmd")
                .args(["/C", "start", &url])
                .spawn();
        }

        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg(&url)
                .spawn();
        }

        #[cfg(target_os = "linux")]
        {
            let _ = std::process::Command::new("xdg-open")
                .arg(&url)
                .spawn();
        }
    });
}

pub fn start_http_server(shared: Arc<Mutex<ServerSharedState>>) {
    thread::spawn(move || {
        let addr = "127.0.0.1:42128";
        let listener = match TcpListener::bind(addr) {
            Ok(l) => {
                println!("🌐 [Web Dashboard & API Server] Berjalan di http://{}", addr);
                open_browser(&format!("http://{}", addr));
                l
            }
            Err(e) => {
                eprintln!("⚠️ [Web Server] Gagal bind ke {}: {}", addr, e);
                return;
            }
        };

        for stream in listener.incoming() {
            if let Ok(stream) = stream {
                let shared_clone = Arc::clone(&shared);
                thread::spawn(move || {
                    handle_client(stream, shared_clone);
                });
            }
        }
    });
}

fn read_http_request(stream: &mut TcpStream) -> Result<(String, Vec<u8>), std::io::Error> {
    let mut header_bytes = Vec::new();
    let mut buf = [0u8; 4096];
    let mut header_end_pos = None;

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        header_bytes.extend_from_slice(&buf[..n]);
        if let Some(pos) = header_bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end_pos = Some(pos);
            break;
        }
        if header_bytes.len() > 1024 * 1024 {
            break;
        }
    }

    let end_pos = match header_end_pos {
        Some(pos) => pos,
        None => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Incomplete HTTP headers")),
    };

    let header_str = String::from_utf8_lossy(&header_bytes[..end_pos]).to_string();
    let mut content_length = 0usize;
    for line in header_str.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    let initial_body = header_bytes[end_pos + 4..].to_vec();
    let mut body = initial_body;
    while body.len() < content_length {
        let to_read = (content_length - body.len()).min(65536);
        let mut chunk = vec![0u8; to_read];
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..n]);
    }

    Ok((header_str, body))
}

fn handle_client(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    let (header_str, body_bytes) = match read_http_request(&mut stream) {
        Ok(res) => res,
        Err(_) => return,
    };

    let mut lines = header_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let full_url = parts[1];
    let url = full_url.split('?').next().unwrap_or(full_url);

    // CORS preflight OPTIONS
    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
                    Access-Control-Allow-Headers: Content-Type, Content-Length\r\n\
                    Content-Length: 0\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let body_str = match std::str::from_utf8(&body_bytes) {
        Ok(s) => s,
        Err(_) => "",
    };

    match (method, url) {
        ("GET", "/api/assets/info") => {
            let assets_dir = get_assets_dir();
            let vrm_path = find_file_in_candidates("char.vrm");
            let vrm_exists = vrm_path.exists();
            let vrm_size = if vrm_exists {
                fs::metadata(&vrm_path).map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };

            let mut images = Vec::new();
            if let Ok(entries) = fs::read_dir(&assets_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        let ext_lower = ext.to_lowercase();
                        if ["svg", "png", "jpg", "jpeg", "webp"].contains(&ext_lower.as_str()) {
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                images.push(name.to_string());
                            }
                        }
                    }
                }
            }

            #[derive(Serialize)]
            struct AssetInfoResponse {
                vrm_exists: bool,
                vrm_size_bytes: u64,
                vrm_size_mb: f32,
                vrm_path: String,
                images: Vec<String>,
            }

            let resp = AssetInfoResponse {
                vrm_exists,
                vrm_size_bytes: vrm_size,
                vrm_size_mb: (vrm_size as f32) / (1024.0 * 1024.0),
                vrm_path: vrm_path.to_string_lossy().to_string(),
                images,
            };

            let json = serde_json::to_string(&resp).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/upload/vrm") => {
            if body_bytes.is_empty() {
                send_json_response(&mut stream, 400, "{\"error\":\"Empty file payload\"}");
                return;
            }

            let assets_dir = get_assets_dir();
            let _ = fs::create_dir_all(&assets_dir);
            let target_path = assets_dir.join("char.vrm");

            match fs::write(&target_path, &body_bytes) {
                Ok(_) => {
                    crate::term_log!("✨ [Asset Manager] Model VRM berhasil diunggah ({:.2} MB) ke {:?}", (body_bytes.len() as f32) / (1024.0 * 1024.0), target_path);
                    if let Ok(mut lock) = shared.lock() {
                        lock.vrm_updated = true;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true,\"message\":\"VRM model uploaded and reloaded successfully\"}");
                }
                Err(e) => {
                    crate::term_err!("⚠️ [Asset Manager] Gagal menyimpan file VRM: {}", e);
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                }
            }
        }
        ("POST", "/api/upload/avatar") => {
            if body_bytes.is_empty() {
                send_json_response(&mut stream, 400, "{\"error\":\"Empty image payload\"}");
                return;
            }

            // Ekstrak nama slot dari query string, misal ?slot=idle, ?slot=listening, ?slot=talking, ?slot=thinking
            let slot = full_url.split('?').nth(1).and_then(|q| {
                for pair in q.split('&') {
                    let mut kv = pair.split('=');
                    if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                        if k == "slot" {
                            return Some(v);
                        }
                    }
                }
                None
            }).unwrap_or("idle");

            // Deteksi format file sederhana dari magic bytes
            let ext = if body_bytes.starts_with(b"\x89PNG") {
                "png"
            } else if body_bytes.starts_with(b"\xFF\xD8\xFF") {
                "jpg"
            } else if body_bytes.starts_with(b"RIFF") && body_bytes.len() > 12 && &body_bytes[8..12] == b"WEBP" {
                "webp"
            } else if body_bytes.starts_with(b"<?xml") || body_bytes.starts_with(b"<svg") || body_str.contains("<svg") {
                "svg"
            } else {
                "png"
            };

            let filename = format!("char-avatar-{}.{}", slot, ext);
            let assets_dir = get_assets_dir();
            let _ = fs::create_dir_all(&assets_dir);
            let target_path = assets_dir.join(&filename);

            match fs::write(&target_path, &body_bytes) {
                Ok(_) => {
                    crate::term_log!("🎨 [Asset Manager] Gambar avatar slot '{}' berhasil disimpan ke {:?}", slot, target_path);
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
                Err(e) => {
                    crate::term_err!("⚠️ [Asset Manager] Gagal menyimpan avatar: {}", e);
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                }
            }
        }
        ("GET", "/api/config") => {
            let cfg = load_config();
            let json = serde_json::to_string(&cfg).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/config") => {
            if let Ok(cfg) = serde_json::from_str::<UserConfig>(body_str) {
                if let Err(e) = save_config(&cfg) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    if let Ok(mut lock) = shared.lock() {
                        lock.config_updated = true;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid JSON config\"}");
            }
        }
        ("POST", "/api/tuner/preview") => {
            #[derive(Deserialize)]
            struct TunerPreviewPayload {
                poses: PosesConfig,
                #[serde(default)]
                emotion: Option<String>,
                #[serde(default)]
                state: Option<String>,
            }
            if let Ok(payload) = serde_json::from_str::<TunerPreviewPayload>(body_str) {
                if let Ok(mut lock) = shared.lock() {
                    lock.tuning_mode = false;
                    lock.live_tuning_bones.clear();
                    lock.preview_poses = Some(payload.poses);
                    if let Some(st_str) = payload.state {
                        lock.current_state = match st_str.to_lowercase().as_str() {
                            "listening" => crate::native::pose_system::AssistantVisualState::Listening,
                            "speaking" => crate::native::pose_system::AssistantVisualState::Speaking,
                            "thinking" => crate::native::pose_system::AssistantVisualState::Thinking,
                            _ => crate::native::pose_system::AssistantVisualState::Idle,
                        };
                    }
                    if let Some(emo) = payload.emotion {
                        lock.current_emotion = emo;
                    }
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid preview payload\"}");
            }
        }
        ("POST", "/api/tuner/live") => {
            #[derive(Deserialize)]
            struct TunerLivePayload {
                active: bool,
                #[serde(default)]
                bones: HashMap<String, [f32; 3]>,
                #[serde(default)]
                emotion: Option<String>,
                #[serde(default)]
                state: Option<String>,
            }
            if let Ok(payload) = serde_json::from_str::<TunerLivePayload>(body_str) {
                if let Ok(mut lock) = shared.lock() {
                    if payload.active {
                        lock.tuning_mode = true;
                        if let Some(st_str) = payload.state {
                            lock.current_state = match st_str.to_lowercase().as_str() {
                                "listening" => crate::native::pose_system::AssistantVisualState::Listening,
                                "speaking" => crate::native::pose_system::AssistantVisualState::Speaking,
                                "thinking" => crate::native::pose_system::AssistantVisualState::Thinking,
                                _ => crate::native::pose_system::AssistantVisualState::Idle,
                            };
                        }
                        lock.live_tuning_bones = payload.bones;
                        if let Some(emo) = payload.emotion {
                            lock.current_emotion = emo;
                        }
                    } else {
                        // KELUAR DARI TUNER / BATAL: Kembalikan 100% ke real runtime state & source of truth poses.json di disk
                        lock.tuning_mode = false;
                        lock.live_tuning_bones.clear();
                        lock.preview_poses = None;
                        lock.poses_updated = true;

                        let (real_state, real_emotion) = crate::tray::TrayState::get_current_runtime_state();
                        lock.current_state = real_state;
                        lock.current_emotion = real_emotion;
                    }
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid tuner payload\"}");
            }
        }
        ("GET", "/api/poses") => {
            let poses = load_poses();
            let json = serde_json::to_string(&poses).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/poses") => {
            if let Ok(poses) = serde_json::from_str::<PosesConfig>(body_str) {
                if let Err(e) = save_poses(&poses) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    if let Ok(mut lock) = shared.lock() {
                        lock.preview_poses = None;
                        lock.poses_updated = true;
                    }
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid JSON poses\"}");
            }
        }
        ("GET", "/api/skills") => {
            let skills = parse_skills_markdown();
            let json = serde_json::to_string(&skills).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("GET", "/api/system/apps") => {
            let apps = get_installed_applications();
            let json = serde_json::to_string(&apps).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/skills") => {
            #[derive(Deserialize)]
            struct AddSkillPayload {
                name: String,
                description: String,
                command: String,
            }
            if let Ok(item) = serde_json::from_str::<AddSkillPayload>(body_str) {
                let mut skills = parse_skills_markdown();
                if let Some(pos) = skills.iter().position(|s| s.name.eq_ignore_ascii_case(&item.name)) {
                    skills[pos] = SkillItem {
                        name: item.name,
                        description: item.description,
                        command: item.command,
                    };
                } else {
                    skills.push(SkillItem {
                        name: item.name,
                        description: item.description,
                        command: item.command,
                    });
                }
                if let Err(e) = save_skills_markdown(&skills) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid skill format\"}");
            }
        }
        ("POST", "/api/skills/delete") => {
            #[derive(Deserialize)]
            struct DeleteSkillPayload {
                name: String,
            }
            if let Ok(item) = serde_json::from_str::<DeleteSkillPayload>(body_str) {
                let mut skills = parse_skills_markdown();
                skills.retain(|s| !s.name.eq_ignore_ascii_case(&item.name));
                if let Err(e) = save_skills_markdown(&skills) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid delete payload\"}");
            }
        }
        ("POST", "/api/skills/run") => {
            #[derive(Deserialize)]
            struct RunSkillPayload {
                command: String,
            }
            if let Ok(item) = serde_json::from_str::<RunSkillPayload>(body_str) {
                if let Err(e) = execute_command_by_os(&item.command) {
                    send_json_response(&mut stream, 500, &format!("{{\"error\":\"{}\"}}", e));
                } else {
                    send_json_response(&mut stream, 200, "{\"success\":true}");
                }
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid run payload\"}");
            }
        }
        ("POST", "/api/session/heartbeat") => {
            #[derive(Deserialize)]
            struct HeartbeatPayload {
                token: String,
            }
            if let Ok(payload) = serde_json::from_str::<HeartbeatPayload>(body_str) {
                let now = std::time::Instant::now();
                let mut is_active = false;
                if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
                    if let Some(sess) = lock.as_mut() {
                        if sess.token == payload.token {
                            sess.last_seen = now;
                            is_active = true;
                        } else if now.duration_since(sess.last_seen) > Duration::from_secs(4) {
                            // Sesi lama sudah offline > 4 detik -> ambil alih otomatis
                            *sess = ActiveWebSession {
                                token: payload.token,
                                last_seen: now,
                            };
                            is_active = true;
                        }
                    } else {
                        *lock = Some(ActiveWebSession {
                            token: payload.token,
                            last_seen: now,
                        });
                        is_active = true;
                    }
                }
                send_json_response(&mut stream, 200, &format!("{{\"is_active\":{}}}", is_active));
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid payload\"}");
            }
        }
        ("POST", "/api/session/takeover") => {
            #[derive(Deserialize)]
            struct TakeoverPayload {
                token: String,
            }
            if let Ok(payload) = serde_json::from_str::<TakeoverPayload>(body_str) {
                if let Ok(mut lock) = ACTIVE_WEB_SESSION.lock() {
                    *lock = Some(ActiveWebSession {
                        token: payload.token,
                        last_seen: std::time::Instant::now(),
                    });
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid payload\"}");
            }
        }
        ("GET", "/api/logs") => {
            let logs = crate::config::get_all_logs();
            let json = serde_json::to_string(&logs).unwrap_or_default();
            send_json_response(&mut stream, 200, &json);
        }
        ("POST", "/api/logs/clear") => {
            crate::config::clear_logs();
            send_json_response(&mut stream, 200, "{\"success\":true}");
        }
        ("POST", "/api/cmd") => {
            #[derive(Deserialize)]
            struct CmdPayload {
                cmd: String,
            }
            if let Ok(payload) = serde_json::from_str::<CmdPayload>(body_str) {
                crate::tray::TrayState::push_command(payload.cmd);
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid cmd payload\"}");
            }
        }
        ("GET", "/api/events") => {
            handle_sse_client(stream, shared);
        }
        ("GET", "/api/state") => {
            if let Ok(lock) = shared.lock() {
                let st_str = match lock.current_state {
                    AssistantVisualState::Idle => "idle",
                    AssistantVisualState::Listening => "listening",
                    AssistantVisualState::Speaking => "speaking",
                    AssistantVisualState::Thinking => "thinking",
                };
                let payload = AppStatePayload {
                    state: st_str.to_string(),
                    is_running: lock.is_running,
                    is_paused: lock.is_paused,
                    voice_muted: lock.voice_muted,
                    mic_muted: lock.mic_muted,
                };
                let json = serde_json::to_string(&payload).unwrap_or_default();
                send_json_response(&mut stream, 200, &json);
            }
        }
        ("POST", "/api/state") => {
            if let Ok(payload) = serde_json::from_str::<AppStatePayload>(body_str) {
                let mut cmd_to_send = None;
                if let Ok(mut lock) = shared.lock() {
                    let prev_running = lock.is_running;
                    let prev_paused = lock.is_paused;

                    lock.is_running = payload.is_running;
                    lock.is_paused = payload.is_paused;
                    lock.voice_muted = payload.voice_muted;
                    lock.mic_muted = payload.mic_muted;
                    lock.state_version = lock.state_version.wrapping_add(1);

                    if payload.is_running && !prev_running {
                        cmd_to_send = Some("start_chat".to_string());
                    } else if !payload.is_running && prev_running {
                        cmd_to_send = Some("stop_chat".to_string());
                    } else if payload.is_paused != prev_paused {
                        cmd_to_send = Some("toggle_pause".to_string());
                    }
                }
                if let Some(cmd) = cmd_to_send {
                    crate::tray::TrayState::push_command(cmd);
                }
                send_json_response(&mut stream, 200, "{\"success\":true}");
            } else {
                send_json_response(&mut stream, 400, "{\"error\":\"Invalid state payload\"}");
            }
        }
        ("GET", path) => {
            serve_static_file(&mut stream, path);
        }
        _ => {
            send_json_response(&mut stream, 404, "{\"error\":\"Not Found\"}");
        }
    }
}

fn handle_sse_client(mut stream: TcpStream, shared: Arc<Mutex<ServerSharedState>>) {
    let header = "HTTP/1.1 200 OK\r\n\
                  Content-Type: text/event-stream\r\n\
                  Cache-Control: no-cache\r\n\
                  Connection: keep-alive\r\n\
                  Access-Control-Allow-Origin: *\r\n\r\n";
    if stream.write_all(header.as_bytes()).is_err() {
        return;
    }

    let mut last_seen_version: u64 = 0;
    let mut initial_sent = false;

    loop {
        let (should_send, payload_json) = if let Ok(lock) = shared.lock() {
            let ver = lock.state_version;
            if !initial_sent || ver != last_seen_version {
                last_seen_version = ver;
                initial_sent = true;

                let st_str = match lock.current_state {
                    AssistantVisualState::Idle => "idle",
                    AssistantVisualState::Listening => "listening",
                    AssistantVisualState::Speaking => "speaking",
                    AssistantVisualState::Thinking => "thinking",
                };
                let payload = AppStatePayload {
                    state: st_str.to_string(),
                    is_running: lock.is_running,
                    is_paused: lock.is_paused,
                    voice_muted: lock.voice_muted,
                    mic_muted: lock.mic_muted,
                };
                (true, serde_json::to_string(&payload).unwrap_or_default())
            } else {
                (false, String::new())
            }
        } else {
            (false, String::new())
        };

        if should_send {
            let event_msg = format!("data: {}\n\n", payload_json);
            if stream.write_all(event_msg.as_bytes()).is_err() {
                break;
            }
            let _ = stream.flush();
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn send_json_response(stream: &mut TcpStream, status: u16, json: &str) {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {}\r\n\
        Content-Type: application/json; charset=utf-8\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
        Access-Control-Allow-Headers: Content-Type\r\n\
        Content-Length: {}\r\n\
        Connection: close\r\n\r\n\
        {}",
        status,
        status_text,
        json.as_bytes().len(),
        json
    );
    let _ = stream.write_all(response.as_bytes());
}

fn serve_static_file(stream: &mut TcpStream, req_path: &str) {
    let clean_path = if req_path == "/" { "index.html" } else { req_path.trim_start_matches('/') };

    // 1. Coba layani dari Embedded Web Asset (compiled 1:1 inside binary)
    if let Some(file) = EmbeddedWebDist::get(clean_path).or_else(|| EmbeddedWebDist::get("index.html")) {
        let content_type = if clean_path.ends_with(".html") || !clean_path.contains('.') {
            "text/html; charset=utf-8"
        } else if clean_path.ends_with(".js") {
            "application/javascript; charset=utf-8"
        } else if clean_path.ends_with(".css") {
            "text/css; charset=utf-8"
        } else if clean_path.ends_with(".svg") {
            "image/svg+xml"
        } else if clean_path.ends_with(".png") {
            "image/png"
        } else if clean_path.ends_with(".ico") {
            "image/x-icon"
        } else {
            "application/octet-stream"
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: {}\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n",
            content_type,
            file.data.len()
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(&file.data);
        return;
    }

    // 2. Fallback baca langsung dari disk filesystem (untuk mode dev / live reload)
    let dist_dirs = [
        PathBuf::from("web/dist"),
        PathBuf::from("../web/dist"),
        crate::config::get_app_config_dir().join("web/dist"),
    ];

    for base in &dist_dirs {
        let file_path = base.join(clean_path);
        if file_path.exists() && file_path.is_file() {
            if let Ok(bytes) = fs::read(&file_path) {
                let content_type = if clean_path.ends_with(".html") {
                    "text/html; charset=utf-8"
                } else if clean_path.ends_with(".js") {
                    "application/javascript; charset=utf-8"
                } else if clean_path.ends_with(".css") {
                    "text/css; charset=utf-8"
                } else if clean_path.ends_with(".svg") {
                    "image/svg+xml"
                } else if clean_path.ends_with(".png") {
                    "image/png"
                } else if clean_path.ends_with(".ico") {
                    "image/x-icon"
                } else {
                    "application/octet-stream"
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                    Content-Type: {}\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Content-Length: {}\r\n\
                    Connection: close\r\n\r\n",
                    content_type,
                    bytes.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&bytes);
                return;
            }
        }
    }

    send_json_response(stream, 404, "{\"error\":\"Static file not found\"}");
}
