use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::engine::session::{AppSession, check_and_consume_cmd};

pub enum LoopAction {
    Exit,
    Pause,
    TextInput(String),
}

/// Menunggu input teks transkrip dari Google Chrome Web Speech API / Mobile Web STT
/// sekaligus merespon kontrol GUI / Tray secara non-blocking dengan 0% CPU idle.
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
                crate::term_log!("🔄 [Reset Session] AI conversation history has been reset!");
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

/// Menangani loop saat asisten dalam kondisi dijeda (paused)
pub fn handle_paused_loop(session: &mut AppSession) -> bool {
    crate::tray::TrayState::update(crate::tray::AssistantState::Idle, "Dijeda");
    crate::term_log!(
        "\n[DIJEDA] {} berhenti mendengarkan sementara.",
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
                    "▶️  [RESUMED] {} ready to listen again!\n",
                    session.config.assistant_name
                );
                return false; // unpause
            } else if cmd == "stop_chat" || cmd == "quit" {
                return true; // exit
            } else if cmd == "reset_session" {
                session.llm.reset_history();
                crate::term_log!("🔄 [Reset Session] AI conversation history has been reset!");
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
