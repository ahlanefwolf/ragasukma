use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static IS_PLAYING: AtomicBool = AtomicBool::new(false);
static IS_PAUSED: AtomicBool = AtomicBool::new(false);
static CURRENT_TITLE: Mutex<Option<String>> = Mutex::new(None);

/// Kirim perintah putar file audio (sinkronisasi state & delegasi stream ke Web Dashboard & PiP)
pub fn play_file(_path: impl AsRef<Path>, title: impl Into<String>) {
    let t = title.into();
    IS_PLAYING.store(true, Ordering::SeqCst);
    IS_PAUSED.store(false, Ordering::SeqCst);
    if let Ok(mut lock) = CURRENT_TITLE.lock() {
        *lock = Some(t.clone());
    }
    crate::term_log!("🎵 [Music Controller] Memutar trek '{}' via Web Dashboard & PiP stream", t);
}

/// Jeda pemutaran musik
pub fn pause() {
    IS_PLAYING.store(false, Ordering::SeqCst);
    IS_PAUSED.store(true, Ordering::SeqCst);
    crate::term_log!("⏸️ [Music Controller] Musik dijeda.");
}

/// Lanjutkan pemutaran musik
pub fn resume() {
    IS_PLAYING.store(true, Ordering::SeqCst);
    IS_PAUSED.store(false, Ordering::SeqCst);
    crate::term_log!("▶️ [Music Controller] Musik dilanjutkan.");
}

/// Hentikan pemutaran musik
pub fn stop() {
    IS_PLAYING.store(false, Ordering::SeqCst);
    IS_PAUSED.store(false, Ordering::SeqCst);
    if let Ok(mut lock) = CURRENT_TITLE.lock() {
        *lock = None;
    }
    crate::term_log!("⏹️ [Music Controller] Musik dihentikan.");
}

#[allow(dead_code)]
pub fn is_playing() -> bool {
    IS_PLAYING.load(Ordering::SeqCst)
}

#[allow(dead_code)]
pub fn is_paused() -> bool {
    IS_PAUSED.load(Ordering::SeqCst)
}

pub fn get_current_title() -> Option<String> {
    if let Ok(lock) = CURRENT_TITLE.lock() {
        lock.clone()
    } else {
        None
    }
}
