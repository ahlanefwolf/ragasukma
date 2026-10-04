use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::engine::session::{AppSession, check_and_consume_cmd};

pub enum LoopAction {
    Exit,
    Pause,
    TextInput(String),
}

/// Waits for transcript text input from Web STT / Mobile STT
/// while simultaneously responding to GUI / Tray controls non-blockingly with 0% idle CPU.
pub fn wait_for_user_turn(session: &mut AppSession, text_rx: &Receiver<String>) -> LoopAction {
    let mut last_audio_check = Instant::now();
    loop {
        if last_audio_check.elapsed() >= std::time::Duration::from_millis(500) {
            session.reload_config_if_changed();
            last_audio_check = Instant::now();
        }

        if let Some(cmd) = check_and_consume_cmd() {
            if cmd == "toggle_pause" {
                return LoopAction::Pause;
            } else if cmd == "stop_chat" || cmd == "quit" {
                return LoopAction::Exit;
            } else if cmd == "reset_session" {
                session.llm.reset_history();
                crate::term_log!("[Reset Session] AI conversation history has been reset!");
            }
        }

        if let Ok(incoming_text) = text_rx.try_recv() {
            let trimmed = incoming_text.trim();
            if !trimmed.is_empty() {
                return LoopAction::TextInput(trimmed.to_string());
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(30));
    }
}

/// Handles loop when assistant is in paused state
pub fn handle_paused_loop(session: &mut AppSession) -> bool {
    crate::tray::TrayState::update(crate::tray::AssistantState::Idle, "Paused");
    crate::term_log!(
        "\n[PAUSED] {} paused listening.",
        session.config.assistant_name
    );

    let mut last_pause_check = Instant::now();
    loop {
        if last_pause_check.elapsed() >= std::time::Duration::from_millis(500) {
            session.reload_config_if_changed();
            last_pause_check = Instant::now();
        }

        if let Some(cmd) = check_and_consume_cmd() {
            if cmd == "toggle_pause" || cmd == "start_chat" {
                crate::term_log!(
                    "▶  [RESUMED] {} ready to listen again!\n",
                    session.config.assistant_name
                );
                return false; // unpause
            } else if cmd == "stop_chat" || cmd == "quit" {
                return true; // exit
            } else if cmd == "reset_session" {
                session.llm.reset_history();
                crate::term_log!("[Reset Session] AI conversation history has been reset!");
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
