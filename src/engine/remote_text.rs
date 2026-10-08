use std::collections::VecDeque;
use std::sync::Mutex;

static REMOTE_TEXT_QUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

/// Push incoming text (from Web STT or Mobile remote) to the unified global queue
pub fn inject_remote_text(text: String) -> bool {
    let cfg = crate::assets::load_config();
    // Jika voice_interruption aktif (true), potong suara asisten seketika saat user berbicara
    if cfg.voice_interruption {
        crate::app_state::trigger_interruption();
        crate::audio::player::stop_native_voice();
        crate::app_state::set_ai_speaking(false);
    }

    if let Ok(mut queue) = REMOTE_TEXT_QUEUE.lock() {
        if queue.len() >= 20 {
            queue.pop_front();
        }
        queue.push_back(text);
        crate::term_log!("[Remote Text Queue] Push succeeded, queue len = {}", queue.len());
    } else {
        crate::term_err!("[Remote Text Queue] Failed to acquire lock on REMOTE_TEXT_QUEUE!");
    }
    true
}

/// Pop and concatenate all buffered speech chunks into one complete sentence
pub fn pop_remote_text() -> Option<String> {
    if let Ok(mut queue) = REMOTE_TEXT_QUEUE.lock() {
        if queue.is_empty() {
            return None;
        }

        let mut chunks = Vec::new();
        while let Some(msg) = queue.pop_front() {
            let trimmed = msg.trim();
            if !trimmed.is_empty() {
                chunks.push(trimmed.to_string());
            }
        }

        if chunks.is_empty() {
            return None;
        }

        // Gabungkan seluruh potongan teks di queue menjadi 1 kalimat percakapan utuh
        let combined = chunks.join(" ");
        let clean_combined = combined.trim();
        if !clean_combined.is_empty() {
            crate::term_log!("[Remote Text Queue] Merged {} speech chunks into single prompt: \"{}\"", chunks.len(), clean_combined);
            Some(clean_combined.to_string())
        } else {
            None
        }
    } else {
        None
    }
}
