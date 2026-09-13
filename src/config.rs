use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

static TERMINAL_LOG_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn set_terminal_log_enabled(enabled: bool) {
    TERMINAL_LOG_ENABLED.store(enabled, Ordering::SeqCst);
}

pub fn is_terminal_log_enabled() -> bool {
    TERMINAL_LOG_ENABLED.load(Ordering::SeqCst)
}

#[macro_export]
macro_rules! term_log {
    ($($arg:tt)*) => {
        if $crate::config::is_terminal_log_enabled() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! term_err {
    ($($arg:tt)*) => {
        if $crate::config::is_terminal_log_enabled() {
            eprintln!($($arg)*);
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub assistant_name: String,
    pub user_callsign: String,
    pub personality_role: String,
    pub stt_engine: String,
    pub groq_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: String,
    pub voice_mode: String,
    pub mic_sensitivity: String,
    pub vad_silence_limit: String,
    pub noise_reduction_level: u8,
    pub tts_engine: String,
    #[serde(default = "default_tts_lang")]
    pub tts_lang: String,
    pub tts_speed: f32,
    pub tts_eleven_labs_key: String,
    pub elevenlabs_voice_id: String,
    pub avatar_type: String,
    #[serde(default = "default_always_on_top")]
    pub always_on_top: bool,
    #[serde(default)]
    pub enable_terminal_log: bool,
}

fn default_always_on_top() -> bool {
    true
}

fn default_tts_lang() -> String {
    "id".to_string()
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            assistant_name: "Laras".to_string(),
            user_callsign: "Mas".to_string(),
            personality_role: "partner".to_string(),
            stt_engine: "google".to_string(),
            groq_api_key: "".to_string(),
            llm_base_url: "http://localhost:20128/v1".to_string(),
            llm_model: "ag/gemini-3.7-flash-low".to_string(),
            llm_api_key: "".to_string(),
            voice_mode: "auto".to_string(),
            mic_sensitivity: "3".to_string(),
            vad_silence_limit: "2".to_string(),
            noise_reduction_level: 0,
            tts_engine: "google".to_string(),
            tts_lang: "id".to_string(),
            tts_speed: 1.15f32,
            tts_eleven_labs_key: "".to_string(),
            elevenlabs_voice_id: "".to_string(),
            avatar_type: "vrm".to_string(),
            always_on_top: true,
            enable_terminal_log: false,
        }
    }
}

pub fn get_app_config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            let p = PathBuf::from(appdata).join("ragasukma");
            let _ = fs::create_dir_all(&p);
            return p;
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

pub fn get_config_file_path() -> PathBuf {
    // 1. Cek di root project lokal ./config-ai.json
    let local = PathBuf::from("config-ai.json");
    if local.exists() {
        return local;
    }

    // 2. Cek di user config dir (misal ~/.config/ragasukma/config-ai.json atau %APPDATA%/ragasukma/config-ai.json)
    let p = get_app_config_dir().join("config-ai.json");
    if !p.exists() {
        // Tulis default config agar direktori tidak kosong
        let def = UserConfig::default();
        if let Ok(json) = serde_json::to_string_pretty(&def) {
            let _ = fs::write(&p, json);
        }
    }
    p
}

pub fn load_user_config() -> UserConfig {
    let path = get_config_file_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<UserConfig>(&content) {
                return cfg;
            }
        }
    }

    // Default configuration jika belum ada config-ai.json
    UserConfig::default()
}

pub fn save_user_config(cfg: &UserConfig) -> Result<(), String> {
    set_terminal_log_enabled(cfg.enable_terminal_log);
    let path = get_config_file_path();
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;

    // Atomic write menggunakan file temporary + rename agar aman dari race condition
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub stt_engine: String,
    pub groq_api_key: String,
    pub llm_base_url: String,
    pub llm_model: String,
    pub llm_api_key: String,
    pub voice_mode: String,
    pub vad_activation_threshold: f32,
    pub vad_consecutive_chunks: usize,
    pub vad_silence_limit_chunks: usize,
    pub noise_reduction_level: u8,
    pub tts_engine: String,
    pub tts_lang: String,
    pub tts_speed: f32,
    pub tts_eleven_labs_key: String,
    pub elevenlabs_voice_id: String,
    pub assistant_name: String,
    pub user_callsign: String,
    pub personality_role: String,
    pub avatar_type: String,
    pub always_on_top: bool,
    pub enable_terminal_log: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        let u = load_user_config();
        set_terminal_log_enabled(u.enable_terminal_log);

        let (vad_activation_threshold, required_consecutive) = match u.mic_sensitivity.trim().to_lowercase().as_str() {
            "0" => (0.045f32, 2), // 0 - Ekstra Kebal (Ruangan Sangat Ramai)
            "1" => (0.030f32, 2), // 1 - Sangat Rendah
            "2" => (0.018f32, 2), // 2 - Rendah
            "3" => (0.012f32, 2), // 3 - Normal (Standar)
            "4" => (0.007f32, 1), // 4 - Tinggi (Responsif)
            "5" => (0.004f32, 1), // 5 - Sangat Tinggi
            "6" => (0.002f32, 1), // 6 - Ultra Sensitif (Bisikan)
            other => (other.parse::<f32>().unwrap_or(0.012f32), 2),
        };

        let vad_silence_limit_chunks = match u.vad_silence_limit.trim().to_lowercase().as_str() {
            "0" => 12,
            "1" => 16,
            "2" => 20,
            "3" => 26,
            "4" => 35,
            other => other.parse::<usize>().unwrap_or(20),
        };

        Self {
            stt_engine: u.stt_engine,
            groq_api_key: u.groq_api_key,
            llm_base_url: u.llm_base_url,
            llm_model: u.llm_model,
            llm_api_key: u.llm_api_key,
            voice_mode: u.voice_mode,
            vad_activation_threshold,
            vad_consecutive_chunks: required_consecutive,
            vad_silence_limit_chunks,
            noise_reduction_level: u.noise_reduction_level,
            tts_engine: u.tts_engine,
            tts_lang: u.tts_lang,
            tts_speed: u.tts_speed,
            tts_eleven_labs_key: u.tts_eleven_labs_key,
            elevenlabs_voice_id: u.elevenlabs_voice_id,
            assistant_name: u.assistant_name,
            user_callsign: u.user_callsign,
            personality_role: u.personality_role,
            avatar_type: u.avatar_type,
            always_on_top: u.always_on_top,
            enable_terminal_log: u.enable_terminal_log,
        }
    }

    pub fn is_auto_mode(&self) -> bool {
        self.voice_mode.eq_ignore_ascii_case("auto")
    }
}
