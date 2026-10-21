use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: usize,
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

static LOG_BUFFER: Mutex<VecDeque<LogEntry>> = Mutex::new(VecDeque::new());
static LOG_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn current_time_str() -> String {
    if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        let total_secs = duration.as_secs();
        let secs = total_secs % 60;
        let mins = (total_secs / 60) % 60;
        let hours = (total_secs / 3600) % 24;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        "00:00:00".to_string()
    }
}

pub fn push_log(level: &str, message: String) {
    let id = LOG_COUNTER.fetch_add(1, Ordering::SeqCst);
    let entry = LogEntry {
        id,
        timestamp: current_time_str(),
        level: level.to_string(),
        message,
    };
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        if buf.len() >= 150 {
            buf.pop_front();
        }
        buf.push_back(entry);
    }
}

pub fn get_logs_since(last_id: usize) -> (Vec<LogEntry>, usize) {
    if let Ok(buf) = LOG_BUFFER.lock() {
        let items: Vec<LogEntry> = buf.iter().filter(|l| l.id > last_id).cloned().collect();
        let new_last_id = buf.back().map_or(last_id, |l| l.id).max(last_id);
        (items, new_last_id)
    } else {
        (Vec::new(), last_id)
    }
}

pub fn get_latest_log_id() -> usize {
    if let Ok(buf) = LOG_BUFFER.lock() {
        buf.back().map_or(0, |l| l.id)
    } else {
        0
    }
}

pub fn clear_logs() {
    if let Ok(mut buf) = LOG_BUFFER.lock() {
        buf.clear();
    }
}

#[macro_export]
macro_rules! term_log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        println!("{}", msg);
        $crate::config::push_log("info", msg);
    }};
}

#[macro_export]
macro_rules! term_err {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        eprintln!("{}", msg);
        $crate::config::push_log("error", msg);
    }};
}

fn default_vad_threshold() -> f32 {
    0.15
}

fn default_true() -> bool {
    true
}

fn default_lang() -> String {
    "id".to_string()
}

fn default_voice() -> String {
    "F4".to_string()
}

fn default_tts_volume() -> f32 {
    1.6
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default, alias = "assistantName")]
    pub assistant_name: String,
    #[serde(default, alias = "userCallsign")]
    pub user_callsign: String,
    #[serde(default, alias = "personalityRole")]
    pub personality_role: String,
    #[serde(default, alias = "assistantGender")]
    pub assistant_gender: String,
    #[serde(default, alias = "llmProvider")]
    pub llm_provider: String,
    #[serde(default, alias = "llmBaseUrl")]
    pub llm_base_url: String,
    #[serde(default, alias = "llmModel")]
    pub llm_model: String,
    #[serde(default, alias = "llmApiKey")]
    pub llm_api_key: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default = "default_voice", alias = "ttsVoiceName", alias = "voiceName")]
    pub tts_voice_name: String,
    #[serde(default, alias = "ttsSpeed")]
    pub tts_speed: f32,
    #[serde(default = "default_tts_volume", alias = "ttsVolume", alias = "voiceVolume")]
    pub tts_volume: f32,
    #[serde(default, alias = "avatarType")]
    pub avatar_type: String,
    #[serde(default, alias = "alwaysOnTop")]
    pub always_on_top: bool,
    #[serde(default, alias = "voiceInterruption")]
    pub voice_interruption: bool,
    #[serde(default = "default_vad_threshold", alias = "vadThreshold")]
    pub vad_threshold: f32,
    #[serde(default = "default_true", alias = "vadNoiseSuppression")]
    pub vad_noise_suppression: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            assistant_name: "".to_string(),
            user_callsign: "".to_string(),
            personality_role: "".to_string(),
            assistant_gender: "".to_string(),
            llm_provider: "".to_string(),
            llm_base_url: "".to_string(),
            llm_model: "".to_string(),
            llm_api_key: "".to_string(),
            lang: "id".to_string(),
            tts_voice_name: "F4".to_string(),
            tts_speed: 1.0,
            tts_volume: 1.6,
            avatar_type: "2d".to_string(),
            always_on_top: true,
            voice_interruption: false,
            vad_threshold: 0.15,
            vad_noise_suppression: true,
        }
    }
}

pub fn get_music_dir() -> PathBuf {
    // 1. If running in development mode (Cargo.toml exists) and local assets/music exists, use it
    if PathBuf::from("Cargo.toml").exists() {
        let local = PathBuf::from("assets").join("music");
        if local.exists() {
            return local;
        }

        let parent_local = PathBuf::from("../assets").join("music");
        if parent_local.exists() {
            return parent_local;
        }
    }

    // 2. Persistent user data folder: ~/.local/share/ragasukma/music (Linux),
    //    ~/Library/Application Support/ragasukma/music (macOS), %LOCALAPPDATA%\ragasukma\music (Windows)
    let user_music = get_app_data_dir().join("music");
    if user_music.exists() {
        return user_music;
    }

    // 3. Backward compatibility: legacy config assets folder
    let legacy_music = get_app_config_dir().join("assets").join("music");
    if legacy_music.exists() {
        return legacy_music;
    }

    let _ = fs::create_dir_all(&user_music);
    user_music
}

pub fn get_app_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_appdata) = env::var("LOCALAPPDATA").or_else(|_| env::var("APPDATA")) {
            let p = PathBuf::from(local_appdata).join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            // Respect existing config in ~/.config/ragasukma if already present
            let legacy_p = PathBuf::from(&home).join(".config").join("ragasukma");
            if legacy_p.exists() {
                return legacy_p;
            }
            let p = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            if !xdg_config.trim().is_empty() {
                let p = PathBuf::from(xdg_config).join("ragasukma");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
    }

    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let p = PathBuf::from(home).join(".config").join("ragasukma");
        let _ = fs::create_dir_all(&p);
        p
    } else {
        let p = PathBuf::from(".config").join("ragasukma");
        let _ = fs::create_dir_all(&p);
        p
    }
}

pub fn get_app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_appdata) = env::var("LOCALAPPDATA").or_else(|_| env::var("APPDATA")) {
            let p = PathBuf::from(local_appdata).join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            let p = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            if !xdg_data.trim().is_empty() {
                let p = PathBuf::from(xdg_data).join("ragasukma");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
        if let Ok(home) = env::var("HOME") {
            let p = PathBuf::from(home).join(".local").join("share").join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    get_app_config_dir()
}

pub fn get_app_cache_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(local_appdata) = env::var("LOCALAPPDATA").or_else(|_| env::var("APPDATA")) {
            let p = PathBuf::from(local_appdata).join("ragasukma").join("cache");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            let p = PathBuf::from(home).join("Library").join("Caches").join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
            if !xdg_cache.trim().is_empty() {
                let p = PathBuf::from(xdg_cache).join("ragasukma");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
        if let Ok(home) = env::var("HOME") {
            let p = PathBuf::from(home).join(".cache").join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    get_app_config_dir().join("cache")
}

pub fn get_models_dir() -> PathBuf {
    // 1. Portable / local development folder (only in repo/dev mode)
    if PathBuf::from("Cargo.toml").exists() {
        let local = PathBuf::from("models");
        if local.exists() {
            return local;
        }
        let parent = PathBuf::from("../models");
        if parent.exists() {
            return parent;
        }
    }

    // 2. Standard persistent data directory (~/.local/share/ragasukma/models, etc.)
    let data_models = get_app_data_dir().join("models");
    if data_models.exists() {
        return data_models;
    }

    // 3. Backward compatibility: legacy config directory (~/.config/ragasukma/models)
    let legacy_models = get_app_config_dir().join("models");
    if legacy_models.exists() {
        return legacy_models;
    }

    let _ = fs::create_dir_all(&data_models);
    data_models
}

pub fn get_config_file_path() -> PathBuf {
    // 1. Check via standard candidate (Local project assets/ -> beside exe -> Resources macOS -> AppData/Roaming -> ~/.config)
    let candidate = crate::assets::find_file_in_candidates("config-ai.json");
    if candidate.exists() {
        return candidate;
    }

    // 2. Default target if it doesn't already exist: ~/.config/ragasukma/config-ai.json (or config dir)
    let p = get_app_config_dir().join("config-ai.json");
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    p
}

static CONFIG_MUTEX: Mutex<()> = Mutex::new(());

pub fn get_config_modified_time() -> Option<SystemTime> {
    let path = get_config_file_path();
    fs::metadata(&path).and_then(|m| m.modified()).ok()
}

pub fn load_user_config() -> UserConfig {
    let _guard = CONFIG_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let path = get_config_file_path();
    if path.exists() {
        // Try reading up to 3 times with short pauses to overcome temporary file lock in Windows
        for _ in 0..3 {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut cfg) = serde_json::from_str::<UserConfig>(&content) {
                    if cfg.lang.trim().is_empty() {
                        cfg.lang = "id".to_string();
                    }
                    if cfg.tts_voice_name.trim().is_empty() || cfg.tts_voice_name.contains("Google")
                    {
                        cfg.tts_voice_name = "F4".to_string();
                    }
                    return cfg;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        crate::term_err!(
            "[Config Load] Failed to read config-ai.json (locked/unreadable), using in-memory fallback."
        );
        return UserConfig::default();
    }

    // Default configuration ONLY if the config-ai.json file has never existed on disk
    let default_cfg = UserConfig::default();
    let _ = save_user_config_locked(&default_cfg);
    default_cfg
}

pub fn save_user_config(cfg: &UserConfig) -> Result<(), String> {
    let _guard = CONFIG_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    save_user_config_locked(cfg)
}

fn save_user_config_locked(cfg: &UserConfig) -> Result<(), String> {
    let path = get_config_file_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;

    // Atomic write uses temporary files + retry rename to be safe in the Windows filesystem
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, &json).map_err(|e| e.to_string())?;

    let mut rename_success = false;
    for _ in 0..5 {
        if fs::rename(&tmp_path, &path).is_ok() {
            rename_success = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }

    if !rename_success {
        // Fallback: direct write if rename is blocked by file lock in Windows
        fs::write(&path, json).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&tmp_path);
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub llm_provider: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: String,
    pub lang: String,
    pub tts_voice_name: String,
    pub tts_speed: f32,
    pub tts_volume: f32,
    pub assistant_name: String,
    pub assistant_gender: String,
    pub user_callsign: String,
    pub personality_role: String,
    pub avatar_type: String,
    pub always_on_top: bool,
    pub voice_interruption: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        let u = load_user_config();

        Self {
            llm_provider: u.llm_provider,
            llm_base_url: u.llm_base_url,
            llm_model: u.llm_model,
            llm_api_key: u.llm_api_key,
            lang: u.lang,
            tts_voice_name: u.tts_voice_name,
            tts_speed: u.tts_speed,
            tts_volume: u.tts_volume,
            assistant_name: u.assistant_name,
            assistant_gender: u.assistant_gender,
            user_callsign: u.user_callsign,
            personality_role: u.personality_role,
            avatar_type: u.avatar_type,
            always_on_top: u.always_on_top,
            voice_interruption: u.voice_interruption,
        }
    }
}
