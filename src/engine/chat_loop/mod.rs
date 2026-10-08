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
    /// Loop utama conversation voice and aksi assistant AI Ragasukma
    pub fn start_chat_loop(&mut self) -> Result<()> {
        crate::app_state::update_session_running(true);
        TrayState::update(AssistantState::Idle, "Ready");
        crate::term_log!("====================================================");
        crate::term_log!(" RAGASUKMA CHAT SESSION ACTIVE");
        crate::term_log!("   - Assistant: {}", self.config.assistant_name);
        crate::term_log!("   - Dashboard: http://127.0.0.1:42128");
        crate::term_log!("====================================================\n");

        let mut is_paused = false;

        loop {
            // 1. Check external commands before turn (GUI / Tray)
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "quit" {
                    crate::app_state::update_session_running(false);
                    let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
                    TrayState::clear();
                    crate::term_log!("\n[DONE] Program finished. Goodbye!");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                } else if cmd == "reset_session" {
                    self.llm.reset_history();
                    crate::term_log!("[Reset Session] AI conversation history has been reset!");
                }
            }

            // 2. If in paused state
            if is_paused {
                let should_exit = handle_paused_loop(self);
                if should_exit {
                    crate::app_state::update_session_running(false);
                    let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
                    TrayState::clear();
                    crate::term_log!("\n[DONE] Program finished. Goodbye!");
                    return Ok(());
                }
                is_paused = false;
            }

            if !crate::app_state::is_ai_speaking() && !crate::audio::player::is_native_voice_playing() {
                TrayState::update(AssistantState::Idle, "Ready");
            }

            // 3. Wait for text input turn from Chrome Web Speech API / Mobile STT
            let raw_text = match wait_for_user_turn(self) {
                LoopAction::Exit => {
                    crate::app_state::update_session_running(false);
                    let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
                    TrayState::clear();
                    crate::term_log!("\n[DONE] Program finished. Goodbye!");
                    break;
                }
                LoopAction::Pause => {
                    is_paused = true;
                    continue;
                }
                LoopAction::TextInput(txt) => txt,
            };

            // Check interupsi pause / quit
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "quit" {
                    crate::app_state::update_session_running(false);
                    let _ = crate::engine::cloudflared::stop_cloudflared_tunnel();
                    TrayState::clear();
                    crate::term_log!("\n[DONE] Program finished. Goodbye!");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                    continue;
                }
            }

            // 4. Validate text
            if raw_text.is_empty() {
                if !crate::app_state::is_ai_speaking() && !crate::audio::player::is_native_voice_playing() {
                    TrayState::update(AssistantState::Idle, "Ready");
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

            // 5. Inferensi LLM and respons voice Supertonic-3 Neural TTS
            let tts_clone = Arc::clone(&self.tts);
            let result =
                process_llm_response(&mut self.llm, &tts_clone, &asst_name, &effective_input);

            let was_interrupted = crate::app_state::is_interruption_requested();
            crate::app_state::clear_interruption();

            if was_interrupted {
                crate::term_log!("[Interrupt] Processing new speech turn...");
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

            std::thread::sleep(std::time::Duration::from_millis(300));
            crate::term_log!("----------------------------------------------------\n");
        }

        Ok(())
    }
}
