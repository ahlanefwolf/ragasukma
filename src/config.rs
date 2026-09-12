use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

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
    pub tts_speed: f32,
    pub tts_eleven_labs_key: String,
    pub elevenlabs_voice_id: String,
    pub avatar_type: String,
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
            tts_speed: 1.15f32,
            tts_eleven_labs_key: "".to_string(),
            elevenlabs_voice_id: "".to_string(),
            avatar_type: "vrm".to_string(),
        }
    }
}

pub fn get_config_file_path() -> PathBuf {
    // 1. Cek di root project lokal ./config-ai.json
    let local = PathBuf::from("config-ai.json");
    if local.exists() {
        return local;
    }

    // 2. Cek di ~/.config/ai-voice/config-ai.json
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let p = PathBuf::from(home).join(".config/ai-voice/config-ai.json");
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        p
    } else {
        local
    }
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
    pub vad_silence_limit_chunks: usize,
    pub noise_reduction_level: u8,
    pub tts_engine: String,
    pub tts_speed: f32,
    pub tts_eleven_labs_key: String,
    pub elevenlabs_voice_id: String,
    pub assistant_name: String,
    pub user_callsign: String,
    pub personality_role: String,
    pub avatar_type: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let u = load_user_config();

        let vad_activation_threshold = match u.mic_sensitivity.trim().to_lowercase().as_str() {
            "0" => 0.055f32,
            "1" => 0.040f32,
            "2" => 0.028f32,
            "3" => 0.018f32,
            "4" => 0.010f32,
            other => other.parse::<f32>().unwrap_or(0.018f32),
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
            vad_silence_limit_chunks,
            noise_reduction_level: u.noise_reduction_level,
            tts_engine: u.tts_engine,
            tts_speed: u.tts_speed,
            tts_eleven_labs_key: u.tts_eleven_labs_key,
            elevenlabs_voice_id: u.elevenlabs_voice_id,
            assistant_name: u.assistant_name,
            user_callsign: u.user_callsign,
            personality_role: u.personality_role,
            avatar_type: u.avatar_type,
        }
    }

    pub fn is_auto_mode(&self) -> bool {
        self.voice_mode.eq_ignore_ascii_case("auto")
    }
}
