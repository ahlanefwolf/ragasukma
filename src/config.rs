use std::env;

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
    pub assistant_name: String,
    pub user_callsign: String,
    pub avatar_type: String,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let assistant_name = env::var("ASSISTANT_NAME")
            .unwrap_or_else(|_| "Niki".to_string());

        let user_callsign = env::var("USER_CALLSIGN")
            .or_else(|_| env::var("USER_NAME"))
            .unwrap_or_else(|_| "Mas".to_string());

        let stt_engine = env::var("STT_ENGINE").unwrap_or_else(|_| "google".to_string());
        let groq_api_key = env::var("GROQ_API_KEY").unwrap_or_default();

        let llm_base_url = env::var("LLM_BASE_URL")
            .or_else(|_| env::var("MISTRAL_BASE_URL"))
            .unwrap_or_else(|_| "http://localhost:20128/v1".to_string());
        let llm_model = env::var("LLM_MODEL")
            .or_else(|_| env::var("MISTRAL_MODEL"))
            .unwrap_or_else(|_| "ag/gemini-3.7-flash-low".to_string());
        let llm_api_key = env::var("LLM_API_KEY")
            .or_else(|_| env::var("MISTRAL_API_KEY"))
            .unwrap_or_else(|_| "local-key".to_string());

        let voice_mode = env::var("VOICE_MODE").unwrap_or_else(|_| "auto".to_string());
        let raw_vad_threshold = env::var("MIC_SENSITIVITY")
            .or_else(|_| env::var("VAD_ACTIVATION_THRESHOLD"))
            .unwrap_or_else(|_| "2".to_string());

        let vad_activation_threshold = match raw_vad_threshold.trim().to_lowercase().as_str() {
            "0" => 0.055f32, // Sangat Rendah (Ruangan Berisik / Suara Keras)
            "1" => 0.040f32, // Rendah
            "2" => 0.028f32, // Normal / Seimbang
            "3" => 0.018f32, // Tinggi (Responsif / Suara Santai)
            "4" => 0.010f32, // Ultra Sensitif (Bisikan / Jarak Jauh)
            other => other.parse::<f32>().unwrap_or(0.028f32),
        };

        let raw_vad_silence = env::var("VAD_SILENCE_LIMIT")
            .unwrap_or_else(|_| "2".to_string());

        let vad_silence_limit_chunks = match raw_vad_silence.trim().to_lowercase().as_str() {
            "0" => 12, // ~300ms (Sangat Cepat Putus)
            "1" => 16, // ~400ms (Cepat)
            "2" => 20, // ~500ms (Standar)
            "3" => 26, // ~650ms (Santai)
            "4" => 35, // ~875ms (Lambat)
            other => other.parse::<usize>().unwrap_or(20),
        };

        let noise_reduction_level = env::var("NOISE_REDUCTION_LEVEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let tts_engine = env::var("TTS_ENGINE").unwrap_or_else(|_| "google".to_string());
        let tts_speed = env::var("TTS_SPEED")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.15f32);

        let avatar_type = env::var("AVATAR_TYPE").unwrap_or_else(|_| "vrm".to_string());

        Self {
            stt_engine,
            groq_api_key,
            llm_base_url,
            llm_model,
            llm_api_key,
            voice_mode,
            vad_activation_threshold,
            vad_silence_limit_chunks,
            noise_reduction_level,
            tts_engine,
            tts_speed,
            assistant_name,
            user_callsign,
            avatar_type,
        }
    }

    pub fn is_auto_mode(&self) -> bool {
        self.voice_mode.eq_ignore_ascii_case("auto")
    }
}
