pub mod input_listener;
pub mod llm_responder;
pub mod skill_runner;

use anyhow::Result;
use std::sync::Arc;

use crate::engine::session::{AppSession, check_and_consume_cmd};
use crate::tray::{AssistantState, TrayState};

use self::input_listener::{LoopAction, handle_paused_loop, wait_for_user_turn};
use self::llm_responder::process_llm_response;
use self::skill_runner::handle_skill_execution;

impl AppSession {
    /// Loop utama percakapan suara dan aksi asisten AI Ragasukma
    pub fn start_chat_loop(&mut self) -> Result<()> {
        crate::app_state::update_session_running(true);
        TrayState::update(AssistantState::Idle, "Standby");
        crate::term_log!("====================================================");
        crate::term_log!("✨ RAGASUKMA CHAT SESSION STARTED");
        crate::term_log!("   - Assistant: {}", self.config.assistant_name);
        crate::term_log!("   - Dashboard: http://127.0.0.1:42128");
        crate::term_log!("====================================================\n");

        let mut is_paused = false;

        let (text_tx, text_rx) = std::sync::mpsc::channel::<String>();
        crate::engine::remote_text::set_remote_text_sender(text_tx);

        loop {
            // 1. Cek perintah eksternal sebelum turn (GUI / Tray)
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
                    crate::app_state::update_session_running(false);
                    TrayState::update(AssistantState::Idle, "Standby");
                    crate::term_log!("\n[DONE] Chat session stopped.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                } else if cmd == "reset_session" {
                    self.llm.reset_history();
                    crate::term_log!("🔄 [Reset Session] AI conversation history has been reset!");
                }
            }

            self.reload_config_if_changed();

            // 2. Jika dalam kondisi dijeda (paused)
            if is_paused {
                let should_exit = handle_paused_loop(self);
                if should_exit {
                    crate::app_state::update_session_running(false);
                    TrayState::update(AssistantState::Idle, "Standby");
                    crate::term_log!("\n[DONE] Chat session stopped.");
                    return Ok(());
                }
                is_paused = false;
            }

            if !crate::app_state::is_ai_speaking() {
                TrayState::update(AssistantState::Idle, "Standby");
            }

            // 3. Tunggu giliran input teks dari Chrome Web Speech API / Mobile STT
            let raw_text = match wait_for_user_turn(self, &text_rx) {
                LoopAction::Exit => {
                    crate::app_state::update_session_running(false);
                    TrayState::update(AssistantState::Idle, "Standby");
                    crate::term_log!("\n[DONE] Chat session stopped.");
                    break;
                }
                LoopAction::Pause => {
                    is_paused = true;
                    continue;
                }
                LoopAction::TextInput(txt) => txt,
            };

            // Cek interupsi stop_chat/pause saat STT baru masuk
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
                    crate::app_state::update_session_running(false);
                    TrayState::update(AssistantState::Idle, "Main Menu");
                    crate::term_log!("\n[DONE] Chat session stopped.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                    continue;
                }
            }

            // 4. Validasi teks
            if raw_text.is_empty() {
                crate::term_log!("[INFO] (Voice unclear or too low)\n");
                if !crate::app_state::is_ai_speaking() {
                    TrayState::update(AssistantState::Idle, "Standby");
                }
                continue;
            }

            let user_transcript = raw_text.clone();
            let effective_input = user_transcript.clone();
            let asst_name = self.config.assistant_name.clone();

            crate::term_log!("\n[User] You [5.00ms]:");
            crate::term_log!("   \"{}\"", user_transcript);

            TrayState::emit_event(
                "assistant:transcript",
                serde_json::json!({
                    "role": "user",
                    "text": &user_transcript
                }),
            );

            // 5. Inferensi LLM dan respons suara Supertonic-3 Neural TTS
            let tts_clone = Arc::clone(&self.tts);
            let result =
                process_llm_response(&mut self.llm, &tts_clone, &asst_name, &effective_input);

            let was_interrupted = crate::app_state::is_interruption_requested();
            crate::app_state::clear_interruption();

            if was_interrupted {
                crate::term_log!("⚡ [Interrupt] Processing new speech turn...");
                continue;
            }

            // 6. Eksekusi Tools / Skills & Agentic Loop
            if let Ok(full_reply) = &result {
                handle_skill_execution(
                    &mut self.llm,
                    &tts_clone,
                    &asst_name,
                    full_reply,
                    &effective_input,
                );
            }

            crate::term_log!("----------------------------------------------------\n");

            if let Err(err) = result {
                crate::term_err!("[ERROR] Network error: {err}\n");
            }

            std::thread::sleep(std::time::Duration::from_millis(300));
            crate::term_log!("----------------------------------------------------\n");
        }

        Ok(())
    }
}
