use std::io::Cursor;
use std::path::Path;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_PLAYING: AtomicBool = AtomicBool::new(false);
static IS_PAUSED: AtomicBool = AtomicBool::new(false);
static CURRENT_TITLE: Mutex<Option<String>> = Mutex::new(None);

static AUDIO_HANDLE: OnceLock<Option<rodio::OutputStreamHandle>> = OnceLock::new();
static NATIVE_MUSIC_SINK: Mutex<Option<rodio::Sink>> = Mutex::new(None);
static NATIVE_VOICE_SINK: Mutex<Option<rodio::Sink>> = Mutex::new(None);

fn get_audio_handle() -> Option<rodio::OutputStreamHandle> {
    AUDIO_HANDLE
        .get_or_init(|| {
            rodio::OutputStream::try_default()
                .ok()
                .map(|(stream, handle)| {
                    // Biarkan stream tetap hidup sepanjang runtime aplikasi
                    Box::leak(Box::new(stream));
                    handle
                })
        })
        .clone()
}

/// Putar audio respon AI Supertonic (WAV bytes) langsung ke speaker native PC
pub fn play_native_voice(wav_bytes: Vec<u8>) {
    if crate::app_state::is_voice_muted() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(handle) = get_audio_handle() {
            if let Ok(decoder) = rodio::Decoder::new(Cursor::new(wav_bytes)) {
                if let Ok(sink) = rodio::Sink::try_new(&handle) {
                    // Duck musik native jika sedang berputar
                    if let Ok(music_lock) = NATIVE_MUSIC_SINK.lock() {
                        if let Some(ref m_sink) = *music_lock {
                            m_sink.set_volume(0.25);
                        }
                    }
                    sink.append(decoder);
                    if let Ok(mut lock) = NATIVE_VOICE_SINK.lock() {
                        if let Some(old) = lock.take() {
                            old.stop();
                        }
                        *lock = Some(sink);
                    }

                    // Pulihkan volume musik setelah suara AI selesai dan kembalikan status avatar ke Idle
                    std::thread::spawn(|| {
                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(150));
                            let is_done = if let Ok(lock) = NATIVE_VOICE_SINK.lock() {
                                lock.as_ref().map(|s| s.empty()).unwrap_or(true)
                            } else {
                                true
                            };
                            if is_done {
                                if let Ok(music_lock) = NATIVE_MUSIC_SINK.lock() {
                                    if let Some(ref m_sink) = *music_lock {
                                        m_sink.set_volume(1.0);
                                    }
                                }
                                crate::app_state::set_ai_speaking(false);
                                crate::tray::TrayState::update_with_emotion(
                                    crate::services::tray_service::AssistantState::Idle,
                                    "Standby",
                                    "neutral",
                                );
                                break;
                            }
                        }
                    });
                }
            }
        }
    }));
}

/// Kirim perintah putar file audio (sinkronisasi state & delegasi stream ke Web Dashboard & PiP)
pub fn play_file(path: impl AsRef<Path>, title: impl Into<String>) {
    let t = title.into();
    let p = path.as_ref();
    IS_PLAYING.store(true, Ordering::SeqCst);
    IS_PAUSED.store(false, Ordering::SeqCst);
    if let Ok(mut lock) = CURRENT_TITLE.lock() {
        *lock = Some(t.clone());
    }
    crate::term_log!(
        "🎵 [Music Controller] Playing track '{}' via Web Dashboard & PiP stream",
        t
    );

    // Putar ke speaker hardware PC di thread terpisah dengan isolasi catch_unwind
    // agar bug internal parser format rodio tidak pernah membunuh thread sesi percakapan utama
    let path_buf = p.to_path_buf();
    std::thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if !crate::app_state::is_voice_muted() && path_buf.exists() {
                if let Some(handle) = get_audio_handle() {
                    if let Ok(file) = std::fs::File::open(&path_buf) {
                        if let Ok(decoder) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                                sink.set_volume(1.0);
                                sink.append(decoder);
                                if let Ok(mut lock) = NATIVE_MUSIC_SINK.lock() {
                                    if let Some(old) = lock.take() {
                                        old.stop();
                                    }
                                    *lock = Some(sink);
                                }
                            }
                        }
                    }
                }
            }
        }));
    });
}

/// Jeda pemutaran musik
pub fn pause() {
    IS_PLAYING.store(false, Ordering::SeqCst);
    IS_PAUSED.store(true, Ordering::SeqCst);
    if let Ok(lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref sink) = *lock {
            sink.pause();
        }
    }
    crate::term_log!("⏸️ [Music Controller] Music paused.");
}

/// Lanjutkan pemutaran musik
pub fn resume() {
    IS_PLAYING.store(true, Ordering::SeqCst);
    IS_PAUSED.store(false, Ordering::SeqCst);
    if let Ok(lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref sink) = *lock {
            sink.play();
        }
    }
    crate::term_log!("▶️ [Music Controller] Music resumed.");
}

/// Hentikan pemutaran musik
pub fn stop() {
    IS_PLAYING.store(false, Ordering::SeqCst);
    IS_PAUSED.store(false, Ordering::SeqCst);
    if let Ok(mut lock) = CURRENT_TITLE.lock() {
        *lock = None;
    }
    if let Ok(mut lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(sink) = lock.take() {
            sink.stop();
        }
    }
    crate::term_log!("⏹️ [Music Controller] Music stopped.");
}

pub fn get_current_title() -> Option<String> {
    if let Ok(lock) = CURRENT_TITLE.lock() {
        lock.clone()
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn is_playing() -> bool {
    IS_PLAYING.load(Ordering::SeqCst)
}
