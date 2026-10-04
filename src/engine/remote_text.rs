use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::mpsc::Sender;

pub static REMOTE_TEXT_INJECTOR: Mutex<Option<Sender<String>>> = Mutex::new(None);
static PENDING_TEXT_QUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

pub fn set_remote_text_sender(sender: Sender<String>) {
    if let Ok(mut lock) = REMOTE_TEXT_INJECTOR.lock() {
        // Flush any pending texts sent while the session was starting
        if let Ok(mut pending) = PENDING_TEXT_QUEUE.lock() {
            while let Some(msg) = pending.pop_front() {
                let _ = sender.send(msg);
            }
        }
        *lock = Some(sender);
    }
}

pub fn inject_remote_text(text: String) -> bool {
    if let Ok(lock) = REMOTE_TEXT_INJECTOR.lock() {
        if let Some(ref sender) = *lock {
            if sender.send(text.clone()).is_ok() {
                return true;
            }
        }
    }

    // If chat loop is not running yet, save to pending queue and trigger start_chat automatically
    if let Ok(mut pending) = PENDING_TEXT_QUEUE.lock() {
        if pending.len() < 10 {
            pending.push_back(text);
        }
    }
    crate::tray::TrayState::push_command("start_chat".to_string());
    true
}
