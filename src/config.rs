use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

pub use ragasukma_core::config::{
    get_app_cache_dir, get_app_config_dir, get_app_data_dir, get_config_modified_time,
    get_music_dir, load_user_config, save_user_config, UserConfig,
};

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

    let _ = std::fs::create_dir_all(&data_models);
    data_models
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub llm_model: String,
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
            llm_model: u.llm_model,
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
