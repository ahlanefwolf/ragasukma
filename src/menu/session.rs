use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::sync::Arc;

use crate::audio::AudioRecorder;
use crate::config::AppConfig;
use crate::llm::LlmClient;
use crate::stt::SttEngine;
use crate::tray::{AssistantState, TrayState};
use crate::tts::TextToSpeech;

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
        println!("🎤 Input Device : {}", recorder.device_name());

        let stt = SttEngine::new(&config.stt_engine)?;

        let llm = LlmClient::new();
        println!("🎀 Karakter {} (Imut & Pemalu) siap menemani {}.", config.assistant_name, config.user_callsign);

        let tts = Arc::new(TextToSpeech::new());
        println!(
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
                println!("🔄 [Reactive Update] VAD Threshold & Recorder diupdate realtime!");
            }
        }

        if stt_changed {
            if let Ok(new_stt) = SttEngine::new(&latest_config.stt_engine) {
                self.stt = new_stt;
                println!("🔄 [Reactive Update] STT Engine dialihkan ke: {}", latest_config.stt_engine.to_uppercase());
            }
        }

        if tts_changed {
            self.tts = Arc::new(TextToSpeech::new());
            println!("🔄 [Reactive Update] TTS Engine dialihkan ke: {}", latest_config.tts_engine.to_uppercase());
        }

        if llm_changed {
            self.llm = LlmClient::new();
            println!("🔄 [Reactive Update] LLM Persona / Model dialihkan ke: {} ({})", latest_config.llm_model, latest_config.assistant_name);
        }

        self.config = latest_config;
    }

    pub fn run_interactive_menu(&mut self) -> Result<()> {
        // Langsung mulai mode ngobrol / active loop saat dijalankan
        self.start_chat_loop()?;

        // Jika keluar dari sesi chat tapi program belum quit, sediakan menu standby
        TrayState::clear_commands();
        TrayState::update(AssistantState::Idle, "Menu Utama");

        println!("----------------------------------------------------");
        println!("💡 MENU UTAMA:");
        println!("   - Tekan [SPASI] / [ENTER] untuk MULAI NGOBROL");
        println!("   - Tekan [T] untuk TES SUARA {}", self.config.assistant_name.to_uppercase());
        println!("   - Tekan [M] untuk TES MIC & FILTER SUARA");
        println!("   - Tekan [ESC] atau [Q] untuk KELUAR");
        println!("====================================================\n");

        loop {
            println!("👉 Pilih menu: [SPASI/ENTER] Mulai Ngobrol | [T] Tes {} | [M] Tes Mic | [Q] Keluar", self.config.assistant_name);
            enable_raw_mode()?;

            let action;
            loop {
                if let Some(cmd) = check_and_consume_cmd() {
                    match cmd.as_str() {
                        "start_chat" | "start" => {
                            action = 1;
                            break;
                        }
                        "test_voice" => {
                            action = 2;
                            break;
                        }
                        "test_mic" => {
                            action = 3;
                            break;
                        }
                        "quit" => {
                            action = 4;
                            break;
                        }
                        _ => {}
                    }
                }

                if event::poll(std::time::Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char(' ') | KeyCode::Enter => {
                                    action = 1;
                                    break;
                                }
                                KeyCode::Char('t') | KeyCode::Char('T') => {
                                    action = 2;
                                    break;
                                }
                                KeyCode::Char('m') | KeyCode::Char('M') => {
                                    action = 3;
                                    break;
                                }
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    action = 4;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            disable_raw_mode()?;

            match action {
                2 => {
                    self.test_assistant_voice()?;
                    continue;
                }
                3 => {
                    self.test_mic_dsp()?;
                    continue;
                }
                4 => {
                    TrayState::clear();
                    println!("\n👋 Program selesai. Sampai jumpa!");
                    return Ok(());
                }
                _ => {
                    TrayState::clear_commands();
                    self.start_chat_loop()?;
                    TrayState::clear_commands();
                    TrayState::update(AssistantState::Idle, "Menu Utama");
                }
            }
        }
    }
}
