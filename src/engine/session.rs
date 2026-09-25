use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

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
    pub(crate) last_config_mtime: Option<std::time::SystemTime>,
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
        crate::term_log!(
            "🎀 Karakter {} (Imut & Pemalu) siap menemani {}.",
            config.assistant_name,
            config.user_callsign
        );

        let tts = Arc::new(TextToSpeech::new());
        crate::term_log!("🔊 Engine TTS: {} siap.", config.tts_engine.to_uppercase());

        let last_config_mtime = crate::config::get_config_modified_time();

        Ok(Self {
            config,
            recorder,
            stt,
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

        // Cek apakah konfigurasi audio / STT / TTS berubah
        let stt_changed = latest_config.stt_engine != self.config.stt_engine
            || latest_config.groq_api_key != self.config.groq_api_key;

        let gender_changed = latest_config.assistant_gender != self.config.assistant_gender;

        let tts_changed = latest_config.tts_engine != self.config.tts_engine
            || latest_config.tts_lang != self.config.tts_lang
            || (latest_config.tts_speed - self.config.tts_speed).abs() > 0.01
            || latest_config.tts_eleven_labs_key != self.config.tts_eleven_labs_key
            || latest_config.elevenlabs_voice_id != self.config.elevenlabs_voice_id
            || gender_changed;

        let interrup_changed = latest_config.voice_interruption != self.config.voice_interruption;

        let vad_changed =
            (latest_config.vad_activation_threshold - self.config.vad_activation_threshold).abs()
                > 0.0001
                || latest_config.vad_silence_limit_chunks != self.config.vad_silence_limit_chunks
                || latest_config.noise_reduction_level != self.config.noise_reduction_level
                || interrup_changed;

        let llm_changed = latest_config.llm_provider != self.config.llm_provider
            || latest_config.llm_base_url != self.config.llm_base_url
            || latest_config.llm_model != self.config.llm_model
            || latest_config.llm_api_key != self.config.llm_api_key
            || latest_config.assistant_name != self.config.assistant_name
            || latest_config.user_callsign != self.config.user_callsign
            || latest_config.personality_role != self.config.personality_role
            || gender_changed;

        if vad_changed {
            self.recorder.update_config(latest_config.clone());
            crate::term_log!(
                "🔄 [Reactive Update] VAD Threshold & Audio Filter diupdate realtime!"
            );
        }

        if stt_changed {
            if let Ok(new_stt) = SttEngine::new(&latest_config.stt_engine) {
                self.stt = new_stt;
                crate::term_log!(
                    "🔄 [Reactive Update] STT Engine dialihkan ke: {}",
                    latest_config.stt_engine.to_uppercase()
                );
            }
        }

        if tts_changed {
            self.tts = Arc::new(TextToSpeech::new());
            crate::term_log!(
                "🔄 [Reactive Update] TTS Engine / Gender ({}) dialihkan ke: {}",
                latest_config.assistant_gender,
                latest_config.tts_engine.to_uppercase()
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

        // Jalankan background worker audio agar mic level meter di Web Dashboard
        // selalu mendeteksi audio secara realtime bahkan saat Standby
        let (standby_tx, _standby_rx) = std::sync::mpsc::channel::<Vec<f32>>();
        let stop_standby = Arc::new(AtomicBool::new(false));
        let pause_standby = Arc::new(AtomicBool::new(false));
        let stop_standby_clone = Arc::clone(&stop_standby);
        let pause_standby_clone = Arc::clone(&pause_standby);
        let recorder_ref = Arc::clone(&self.recorder);

        let mut standby_handle = Some(std::thread::spawn(move || {
            recorder_ref.start_continuous_worker(
                stop_standby_clone,
                pause_standby_clone,
                standby_tx,
            );
        }));

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
                    "start_chat" | "start" => {
                        // Jangan hentikan standby worker, cukup reset antrean komando dan jalankan chat loop
                        stop_standby.store(true, Ordering::SeqCst);
                        if let Some(handle) = standby_handle.take() {
                            let _ = handle.join();
                        }
                        TrayState::clear_commands();
                        let _ = self.start_chat_loop();

                        // Setelah keluar dari chat loop, hidupkan kembali standby worker
                        stop_standby.store(false, Ordering::SeqCst);
                        let stop_standby_clone2 = Arc::clone(&stop_standby);
                        let pause_standby_clone2 = Arc::clone(&pause_standby);
                        let recorder_ref2 = Arc::clone(&self.recorder);
                        let (new_tx, _new_rx) = std::sync::mpsc::channel::<Vec<f32>>();
                        standby_handle = Some(std::thread::spawn(move || {
                            recorder_ref2.start_continuous_worker(
                                stop_standby_clone2,
                                pause_standby_clone2,
                                new_tx,
                            );
                        }));
                    }
                    "quit" => {
                        let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
                        stop_standby.store(true, Ordering::SeqCst);
                        if let Some(handle) = standby_handle.take() {
                            let _ = handle.join();
                        }
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
