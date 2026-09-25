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
        if buf.len() >= 1000 {
            buf.pop_front();
        }
        buf.push_back(entry);
    }
}

pub fn get_all_logs() -> Vec<LogEntry> {
    if let Ok(buf) = LOG_BUFFER.lock() {
        buf.iter().cloned().collect()
    } else {
        Vec::new()
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default)]
    pub assistant_name: String,
    #[serde(default)]
    pub user_callsign: String,
    #[serde(default)]
    pub personality_role: String,
    #[serde(default)]
    pub assistant_gender: String,
    #[serde(default)]
    pub stt_engine: String,
    #[serde(default)]
    pub groq_api_key: String,
    #[serde(default)]
    pub llm_provider: String,
    #[serde(default)]
    pub llm_base_url: String,
    #[serde(default)]
    pub llm_model: String,
    #[serde(default)]
    pub llm_api_key: String,
    #[serde(default)]
    pub voice_mode: String,
    #[serde(default)]
    pub mic_sensitivity: String,
    #[serde(default)]
    pub vad_silence_limit: String,
    #[serde(default)]
    pub noise_reduction_level: u8,
    #[serde(default)]
    pub tts_engine: String,
    #[serde(default)]
    pub tts_lang: String,
    #[serde(default)]
    pub tts_speed: f32,
    #[serde(default)]
    pub tts_eleven_labs_key: String,
    #[serde(default)]
    pub elevenlabs_voice_id: String,
    #[serde(default)]
    pub avatar_type: String,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default)]
    pub voice_interruption: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            assistant_name: "".to_string(),
            user_callsign: "".to_string(),
            personality_role: "".to_string(),
            assistant_gender: "".to_string(),
            stt_engine: "".to_string(),
            groq_api_key: "".to_string(),
            llm_provider: "".to_string(),
            llm_base_url: "".to_string(),
            llm_model: "".to_string(),
            llm_api_key: "".to_string(),
            voice_mode: "".to_string(),
            mic_sensitivity: "".to_string(),
            vad_silence_limit: "".to_string(),
            noise_reduction_level: 0,
            tts_engine: "".to_string(),
            tts_lang: "".to_string(),
            tts_speed: 1.0f32,
            tts_eleven_labs_key: "".to_string(),
            elevenlabs_voice_id: "".to_string(),
            avatar_type: "".to_string(),
            always_on_top: false,
            voice_interruption: false,
        }
    }
}

pub fn get_music_dir() -> PathBuf {
    // 1. Jika ada folder lokal assets/music (mode development / portable), gunakan itu
    let local = PathBuf::from("assets").join("music");
    if local.exists() {
        return local;
    }

    let parent_local = PathBuf::from("../assets").join("music");
    if parent_local.exists() {
        return parent_local;
    }

    // 2. Jika tidak ada, gunakan folder user AppData / .config agar permanen dan tidak hilang saat update
    let user_music = get_app_config_dir().join("assets").join("music");
    let _ = fs::create_dir_all(&user_music);
    user_music
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
    // 1. Cek via kandidat standar (Lokal project assets/ -> beside exe -> Resources macOS -> AppData/Roaming)
    let candidate = crate::assets::find_file_in_candidates("config-ai.json");
    if candidate.exists() {
        return candidate;
    }

    // 2. Default target jika belum ada: selalu di dalam folder assets pengguna
    get_app_config_dir().join("assets").join("config-ai.json")
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
        // Coba baca sampai 3 kali dengan jeda singkat untuk mengatasi file lock sementara di Windows
        for _ in 0..3 {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<UserConfig>(&content) {
                    return cfg;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        crate::term_err!("⚠️ [Config Load] File config-ai.json sedang dikunci/tidak terbaca, menggunakan fallback memori.");
        return UserConfig::default();
    }

    // Default configuration HANYA jika file config-ai.json memang belum pernah ada di disk
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

    // Atomic write menggunakan file temporary + retry rename agar aman di Windows filesystem
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
        // Fallback: direct write jika rename terhalang file lock di Windows
        fs::write(&path, json).map_err(|e| e.to_string())?;
        let _ = fs::remove_file(&tmp_path);
    }

    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub stt_engine: String,
    pub groq_api_key: String,
    pub llm_provider: String,
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

        let (vad_activation_threshold, required_consecutive) =
            match u.mic_sensitivity.trim().to_lowercase().as_str() {
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
            llm_provider: u.llm_provider,
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
            assistant_gender: u.assistant_gender,
            user_callsign: u.user_callsign,
            personality_role: u.personality_role,
            avatar_type: u.avatar_type,
            always_on_top: u.always_on_top,
            voice_interruption: u.voice_interruption,
        }
    }
}
