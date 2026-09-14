use anyhow::Result;
use std::sync::Arc;

use crate::audio::recorder::AudioRecorder;
use crate::audio::stt::SttEngine;
use crate::audio::tts::TextToSpeech;
use crate::config::AppConfig;
use crate::llm::LlmClient;
use crate::tray::TrayState;

pub struct AppSession {
    pub(crate) config: AppConfig,
    pub(crate) recorder: Arc<AudioRecorder>,
    pub(crate) stt: SttEngine,
    pub(crate) llm: LlmClient,
    pub(crate) tts: Arc<TextToSpeech>,
}

pub(crate) fn check_and_consume_cmd() -> Option<String> {
    crate::tray::TrayState::pop_command()
}

impl AppSession {
    pub fn new(config: AppConfig) -> Result<Self> {
        let recorder = Arc::new(AudioRecorder::new(config.clone())?);
        crate::term_log!("🎤 Input Device : {}", recorder.device_name());

        let stt = SttEngine::new(&config.stt_engine)?;

        let llm = LlmClient::new();
        crate::term_log!("🎀 Karakter {} (Imut & Pemalu) siap menemani {}.", config.assistant_name, config.user_callsign);

        let tts = Arc::new(TextToSpeech::new());
        crate::term_log!(
            "🔊 Engine TTS: {} siap.",
            config.tts_engine.to_uppercase()
        );

        Ok(Self {
            config,
            recorder,
            stt,
            llm,
            tts,
        })
    }

    pub fn reload_config_if_changed(&mut self) {
        let latest_config = AppConfig::load();

        // Cek apakah konfigurasi audio / STT / TTS berubah
        let stt_changed = latest_config.stt_engine != self.config.stt_engine
            || latest_config.groq_api_key != self.config.groq_api_key;

        let tts_changed = latest_config.tts_engine != self.config.tts_engine
            || latest_config.tts_lang != self.config.tts_lang
            || (latest_config.tts_speed - self.config.tts_speed).abs() > 0.01
            || latest_config.tts_eleven_labs_key != self.config.tts_eleven_labs_key
            || latest_config.elevenlabs_voice_id != self.config.elevenlabs_voice_id;

        let vad_changed = (latest_config.vad_activation_threshold - self.config.vad_activation_threshold).abs() > 0.0001
            || latest_config.vad_silence_limit_chunks != self.config.vad_silence_limit_chunks
            || latest_config.noise_reduction_level != self.config.noise_reduction_level;

        let llm_changed = latest_config.llm_base_url != self.config.llm_base_url
            || latest_config.llm_model != self.config.llm_model
            || latest_config.llm_api_key != self.config.llm_api_key
            || latest_config.assistant_name != self.config.assistant_name
            || latest_config.user_callsign != self.config.user_callsign
            || latest_config.personality_role != self.config.personality_role;

        if vad_changed {
            if let Ok(new_recorder) = AudioRecorder::new(latest_config.clone()) {
                self.recorder = Arc::new(new_recorder);
                crate::term_log!("🔄 [Reactive Update] VAD Threshold & Recorder diupdate realtime!");
            }
        }

        if stt_changed {
            if let Ok(new_stt) = SttEngine::new(&latest_config.stt_engine) {
                self.stt = new_stt;
                crate::term_log!("🔄 [Reactive Update] STT Engine dialihkan ke: {}", latest_config.stt_engine.to_uppercase());
            }
        }

        if tts_changed {
            self.tts = Arc::new(TextToSpeech::new());
            crate::term_log!("🔄 [Reactive Update] TTS Engine / Bahasa dialihkan ke: {} (Lang: {})", latest_config.tts_engine.to_uppercase(), latest_config.tts_lang);
        }

        if llm_changed {
            self.llm = LlmClient::new();
            crate::term_log!("🔄 [Reactive Update] LLM Persona / Model dialihkan ke: {} ({})", latest_config.llm_model, latest_config.assistant_name);
        }

        self.config = latest_config;
    }

    pub fn run_interactive_menu(&mut self) -> Result<()> {
        TrayState::update(crate::tray::AssistantState::Idle, "Standby");

        loop {
            // Standby mode: menunggu komando dari Web Dashboard atau tombol Mulai
            if let Some(cmd) = check_and_consume_cmd() {
                match cmd.as_str() {
                    "start_chat" | "start" => {
                        TrayState::clear_commands();
                        self.start_chat_loop()?;
                    }
                    "quit" => {
                        TrayState::clear();
                        crate::term_log!("\n👋 Program selesai. Sampai jumpa!");
                        return Ok(());
                    }
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}
