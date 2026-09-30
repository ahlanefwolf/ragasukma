use anyhow::Result;
use std::sync::Arc;

use crate::audio::tts::TextToSpeech;
use crate::config::AppConfig;
use crate::llm::LlmClient;
use crate::tray::TrayState;

pub struct AppSession {
    pub(crate) config: AppConfig,
    pub(crate) llm: LlmClient,
    pub(crate) tts: Arc<TextToSpeech>,
    pub(crate) last_config_mtime: Option<std::time::SystemTime>,
}

pub(crate) fn check_and_consume_cmd() -> Option<String> {
    crate::tray::TrayState::pop_command()
}

impl AppSession {
    pub fn new(config: AppConfig) -> Result<Self> {
        let llm = LlmClient::new();
        crate::term_log!(
            "🎀 Character {} (Cute & Shy) ready to accompany {}.",
            config.assistant_name,
            config.user_callsign
        );

        let tts = Arc::new(TextToSpeech::new());
        crate::term_log!("🔊 TTS Engine: Supertonic-3 Neural ONNX ready.");

        let last_config_mtime = crate::config::get_config_modified_time();

        Ok(Self {
            config,
            llm,
            tts,
            last_config_mtime,
        })
    }

    pub fn reload_config_if_changed(&mut self) {
        let current_mtime = crate::config::get_config_modified_time();
        if self.last_config_mtime.is_some() && self.last_config_mtime == current_mtime {
            return;
        }
        self.last_config_mtime = current_mtime;

        let latest_config = AppConfig::load();

        // Cek apakah konfigurasi audio / bahasa / TTS berubah
        let lang_changed = latest_config.lang != self.config.lang;

        let gender_changed = latest_config.assistant_gender != self.config.assistant_gender;

        let tts_changed = lang_changed
            || (latest_config.tts_speed - self.config.tts_speed).abs() > 0.01
            || latest_config.tts_voice_name != self.config.tts_voice_name
            || gender_changed;

        let interrup_changed = latest_config.voice_interruption != self.config.voice_interruption;

        let llm_changed = latest_config.llm_provider != self.config.llm_provider
            || latest_config.llm_base_url != self.config.llm_base_url
            || latest_config.llm_model != self.config.llm_model
            || latest_config.llm_api_key != self.config.llm_api_key
            || latest_config.assistant_name != self.config.assistant_name
            || latest_config.user_callsign != self.config.user_callsign
            || latest_config.personality_role != self.config.personality_role
            || gender_changed;

        if interrup_changed {
            crate::term_log!("🔄 [Reactive Update] Voice interruption updated in realtime!");
        }

        if lang_changed {
            crate::term_log!(
                "🔄 [Reactive Update] Bahasa percakapan dialihkan ke: {}",
                latest_config.lang
            );
        }

        if tts_changed {
            self.tts = Arc::new(TextToSpeech::new());
            crate::term_log!(
                "🔄 [Reactive Update] Voice / Gender ({}) diperbarui.",
                latest_config.assistant_gender
            );
        }

        if llm_changed {
            self.llm = LlmClient::new();
            crate::term_log!(
                "🔄 [Reactive Update] LLM Provider / Model / Persona dialihkan ke: {} / {} ({})",
                latest_config.llm_provider,
                latest_config.llm_model,
                latest_config.assistant_name
            );
        }

        self.config = latest_config;
    }

    pub fn run_interactive_menu(&mut self) -> Result<()> {
        TrayState::update(crate::tray::AssistantState::Idle, "Standby");

        let mut last_config_check = std::time::Instant::now();

        loop {
            // Cek perubahan konfigurasi secara live saat standby
            if last_config_check.elapsed() >= std::time::Duration::from_millis(500) {
                self.reload_config_if_changed();
                last_config_check = std::time::Instant::now();
            }

            // Standby mode: menunggu komando dari Web Dashboard atau tombol Mulai
            if let Some(cmd) = check_and_consume_cmd() {
                match cmd.as_str() {
                    "start_chat" => {
                        TrayState::clear_commands();
                        let _ = self.start_chat_loop();
                    }
                    "reset_session" => {
                        self.llm.reset_history();
                        crate::term_log!(
                            "🔄 [Reset Session] Riwayat memori obrolan AI berhasil di-reset!"
                        );
                    }
                    "quit" => {
                        let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
                        TrayState::clear();
                        crate::term_log!("\n👋 Program finished. Goodbye!");
                        return Ok(());
                    }
                    _ => {}
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}
