use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::Ordering;

static CURRENT_TITLE: Mutex<Option<String>> = Mutex::new(None);
static CURRENT_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static CURRENT_MUSIC_VOLUME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(100);
static CURRENT_VOICE_VOLUME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(160);

static AUDIO_HANDLE: OnceLock<Option<rodio::OutputStreamHandle>> = OnceLock::new();
static NATIVE_MUSIC_SINK: Mutex<Option<rodio::Sink>> = Mutex::new(None);
static NATIVE_VOICE_SINK: Mutex<Option<Arc<rodio::Sink>>> = Mutex::new(None);
static VOICE_PLAYBACK_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default)]
pub struct LipSyncWeights {
    pub aa: f32,
    pub ih: f32,
    pub ou: f32,
    pub ee: f32,
    pub oh: f32,
    pub mouth_open: f32,
}

struct ActiveVoiceSpeech {
    samples: Vec<f32>,
    sample_rate: u32,
    start_instant: std::time::Instant,
}

static ACTIVE_VOICE_SPEECH: Mutex<Option<ActiveVoiceSpeech>> = Mutex::new(None);



/// Calculates real-time AIUEO lip weights based on formant frequencies of the currently playing audio
pub fn get_current_lip_sync_weights() -> LipSyncWeights {
    let Ok(lock) = ACTIVE_VOICE_SPEECH.lock() else {
        return LipSyncWeights::default();
    };

    let Some(ref speech) = *lock else {
        return LipSyncWeights::default();
    };

    let elapsed_secs = speech.start_instant.elapsed().as_secs_f32();
    let cursor = (elapsed_secs * speech.sample_rate as f32) as usize;

    if cursor >= speech.samples.len() {
        return LipSyncWeights::default();
    }

    // Take 512-sample window (~12-25 ms)
    let window_size = 512.min(speech.samples.len() - cursor);
    if window_size < 64 {
        return LipSyncWeights::default();
    }

    let window = &speech.samples[cursor..cursor + window_size];

    // 1. RMS Energy
    let mut sum_sq = 0.0f32;
    for &s in window {
        sum_sq += s * s;
    }
    let rms = (sum_sq / window_size as f32).sqrt();

    // Noise floor: silence / breath pause between sentences
    if rms < 0.012 {
        return LipSyncWeights::default();
    }

    let mouth_open = ((rms - 0.012) * 6.5).clamp(0.0, 1.0).powf(0.85) * 0.65;

    // 2. Goertzel Filter Power on frekuensi formant vokal utama:
    // Low: ~450 Hz (U & O)
    // Mid: ~950 Hz (A)
    // Mid-High: ~1900 Hz (E)
    // High: ~2800 Hz (I)
    let goertzel_power = |target_freq: f32| -> f32 {
        let k = ((window_size as f32) * target_freq / speech.sample_rate as f32).round();
        let omega = (2.0 * std::f32::consts::PI * k) / window_size as f32;
        let coeff = 2.0 * omega.cos();

        let mut s0;
        let mut s1 = 0.0f32;
        let mut s2 = 0.0f32;

        for &x in window {
            s0 = x + coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }

        (s1 * s1 + s2 * s2 - coeff * s1 * s2).max(0.0)
    };

    let p_low = goertzel_power(450.0);
    let p_mid = goertzel_power(950.0);
    let p_midhigh = goertzel_power(1900.0);
    let p_high = goertzel_power(2800.0);

    let p_total = p_low + p_mid + p_midhigh + p_high + 1e-6;
    let r_low = p_low / p_total;
    let r_mid = p_mid / p_total;
    let r_midhigh = p_midhigh / p_total;
    let r_high = p_high / p_total;

    let aa = (r_mid * 1.3).min(1.0) * mouth_open;
    let ih = (r_high * 1.6).min(1.0) * mouth_open * 0.75;
    let ou = ((r_low - r_mid * 0.3).max(0.0) * 1.5).clamp(0.0, 1.0) * mouth_open * 0.70;
    let ee = (r_midhigh * 1.5).min(1.0) * mouth_open * 0.80;
    let oh = ((r_low * 0.7 + r_mid * 0.4) * 1.3).min(1.0) * mouth_open * 0.85;

    LipSyncWeights {
        aa,
        ih,
        ou,
        ee,
        oh,
        mouth_open,
    }
}

fn get_audio_handle() -> Option<rodio::OutputStreamHandle> {
    AUDIO_HANDLE
        .get_or_init(|| {
            rodio::OutputStream::try_default()
                .ok()
                .map(|(stream, handle)| {
                    // Keep stream alive throughout application runtime
                    Box::leak(Box::new(stream));
                    handle
                })
        })
        .clone()
}

pub struct VoiceChunk {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

pub struct VoiceStreamSender {
    sender: std::sync::mpsc::Sender<Option<VoiceChunk>>,
}

impl VoiceStreamSender {
    /// Pushes raw PCM samples directly to the audio player without WAV encoding/decoding overhead
    pub fn push_chunk_pcm(&self, samples: Vec<f32>, sample_rate: u32) {
        let _ = self.sender.send(Some(VoiceChunk {
            samples,
            sample_rate,
        }));
    }

    pub fn finish(&self) {
        let _ = self.sender.send(None);
    }
}

/// Starts an asynchronous, gapless streaming voice session.
/// Chunks of audio can be pushed sequentially while previous chunks are actively playing.
pub fn start_voice_stream(emotion: String, asst_name: String) -> VoiceStreamSender {
    let (tx, rx) = std::sync::mpsc::channel::<Option<VoiceChunk>>();

    if crate::app_state::is_voice_muted() {
        let _ = std::thread::Builder::new()
            .name("ragasukma-voice-muted-drain".to_string())
            .spawn(move || {
                while let Ok(Some(_)) = rx.recv() {}
            });
        return VoiceStreamSender { sender: tx };
    }

    let my_gen = VOICE_PLAYBACK_GEN.fetch_add(1, Ordering::SeqCst) + 1;

    let _ = std::thread::Builder::new()
        .name("ragasukma-voice-player".to_string())
        .stack_size(1024 * 1024)
        .spawn(move || {
            let mut sink_arc: Option<Arc<rodio::Sink>> = None;
        let mut is_speaking_triggered = false;

        while let Ok(msg) = rx.recv() {
            if VOICE_PLAYBACK_GEN.load(Ordering::SeqCst) != my_gen {
                break;
            }

            match msg {
                Some(chunk) => {
                    let pcm_samples = chunk.samples;
                    let pcm_rate = chunk.sample_rate;
                    let sr = pcm_rate;
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        if let Some(handle) = get_audio_handle() {
                            let source = rodio::buffer::SamplesBuffer::new(1, sr, pcm_samples.clone());
                            if sink_arc.is_none() {
                                if let Ok(new_sink) = rodio::Sink::try_new(&handle) {
                                    new_sink.set_volume(get_voice_volume());
                                    if let Ok(music_lock) = NATIVE_MUSIC_SINK.lock() {
                                        if let Some(ref m_sink) = *music_lock {
                                            m_sink.set_volume(get_music_volume() * 0.25);
                                        }
                                    }
                                    new_sink.append(source);
                                    let arc = Arc::new(new_sink);
                                    if let Ok(mut lock) = NATIVE_VOICE_SINK.lock() {
                                        if let Some(old) = lock.take() {
                                            old.stop();
                                        }
                                        *lock = Some(Arc::clone(&arc));
                                    }
                                    sink_arc = Some(arc);
                                }
                            } else if let Some(ref sink) = sink_arc {
                                sink.append(source);
                            }
                        }
                    }));

                    if !pcm_samples.is_empty() {
                        if let Ok(mut lock) = ACTIVE_VOICE_SPEECH.lock() {
                            if let Some(ref mut speech) = *lock {
                                let elapsed = speech.start_instant.elapsed().as_secs_f32();
                                let buffered_secs =
                                    speech.samples.len() as f32 / speech.sample_rate as f32;
                                if elapsed > buffered_secs {
                                    let gap_samples =
                                        ((elapsed - buffered_secs) * speech.sample_rate as f32) as usize;
                                    speech.samples.extend(std::iter::repeat_n(0.0f32, gap_samples));
                                }
                                speech.samples.extend(pcm_samples);
                            } else {
                                *lock = Some(ActiveVoiceSpeech {
                                    samples: pcm_samples,
                                    sample_rate: pcm_rate,
                                    start_instant: std::time::Instant::now(),
                                });
                            }
                        }
                    }

                    // Trigger Speaking state directly from worker player as soon as chunk 0 starts playing
                    if !is_speaking_triggered {
                        is_speaking_triggered = true;
                        crate::app_state::set_ai_speaking(true);
                        crate::app_state::touch_speech_activity();
                        crate::tray::TrayState::update_with_emotion(
                            crate::services::tray_service::AssistantState::Speaking,
                            &format!("{asst_name} Speaking"),
                            &emotion,
                        );
                    }
                }
                None => {
                    // End of stream marker from producer
                    break;
                }
            }
        }

        // Monitor playback until sink finishes playing all enqueued chunks
        if let Some(sink) = sink_arc {
            while !sink.empty() {
                std::thread::sleep(std::time::Duration::from_millis(40));
                if VOICE_PLAYBACK_GEN.load(Ordering::SeqCst) != my_gen {
                    if let Ok(mut speech_lock) = ACTIVE_VOICE_SPEECH.lock() {
                        *speech_lock = None;
                    }
                    return;
                }
            }
        }

        if VOICE_PLAYBACK_GEN.load(Ordering::SeqCst) != my_gen {
            if let Ok(mut speech_lock) = ACTIVE_VOICE_SPEECH.lock() {
                *speech_lock = None;
            }
            return;
        }

        if let Ok(music_lock) = NATIVE_MUSIC_SINK.lock() {
            if let Some(ref m_sink) = *music_lock {
                m_sink.set_volume(get_music_volume());
            }
        }
        if let Ok(mut speech_lock) = ACTIVE_VOICE_SPEECH.lock() {
            *speech_lock = None;
        }
        if let Ok(mut voice_lock) = NATIVE_VOICE_SINK.lock() {
            if let Some(s) = voice_lock.take() {
                s.stop();
            }
        }
        crate::app_state::set_ai_speaking(false);
        crate::tray::TrayState::update_with_emotion(
            crate::services::tray_service::AssistantState::Idle,
            "Standby",
            "neutral",
        );
    });

    VoiceStreamSender { sender: tx }
}

/// Stops native AI voice playback (e.g. during voice interruption or VAD barge-in)
pub fn stop_native_voice() {
    VOICE_PLAYBACK_GEN.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut lock) = NATIVE_VOICE_SINK.lock() {
        if let Some(sink) = lock.take() {
            sink.stop();
        }
    }
    if let Ok(mut speech_lock) = ACTIVE_VOICE_SPEECH.lock() {
        *speech_lock = None;
    }
    if let Ok(music_lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref m_sink) = *music_lock {
            m_sink.set_volume(get_music_volume());
        }
    }
}

/// Check whether native audio voice is currently playing on PC speakers
pub fn is_native_voice_playing() -> bool {
    if let Ok(lock) = NATIVE_VOICE_SINK.lock() {
        lock.as_ref().map(|s| !s.empty()).unwrap_or(false)
    } else {
        false
    }
}

/// Sends audio play command (state synchronization and stream delegation to Web Dashboard & PiP)
pub fn play_file(path: impl AsRef<Path>, title: impl Into<String>) {
    let t = title.into();
    let p = path.as_ref();
    let path_buf = p.to_path_buf();
    if let Ok(mut lock) = CURRENT_TITLE.lock() {
        *lock = Some(t.clone());
    }
    if let Ok(mut lock) = CURRENT_PATH.lock() {
        *lock = Some(path_buf.clone());
    }
    crate::term_log!(
        "[Music Controller] Playing track '{}' via Web Dashboard & PiP stream",
        t
    );

    // Play to PC hardware speakers in a separate thread isolated with catch_unwind
    // so internal rodio format parser bugs never crash the main conversation session thread
    let _ = std::thread::Builder::new()
        .name("ragasukma-music-player".to_string())
        .stack_size(1024 * 1024)
        .spawn(move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // If Phone Only mode is active (voice_muted = true), music only plays on phone, not on PC
            if !crate::app_state::is_voice_muted() && path_buf.exists() {
                if let Some(handle) = get_audio_handle() {
                    // Open file stream directly with BufReader for optimal seek & decoder compatibility
                    if let Ok(file) = std::fs::File::open(&path_buf) {
                        if let Ok(decoder) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                                // If AI is currently speaking when music starts, apply ducked volume directly
                                let user_vol = get_music_volume();
                                let current_vol = if crate::app_state::is_ai_speaking() { user_vol * 0.25 } else { user_vol };
                                sink.set_volume(current_vol);
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

/// Pause music playback
pub fn pause() {
    if let Ok(lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref sink) = *lock {
            sink.pause();
        }
    }
    crate::term_log!("[Music Controller] Music paused.");
}

/// Resume music playback
pub fn resume() {
    if let Ok(lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref sink) = *lock {
            sink.play();
        }
    }
    crate::term_log!("▶ [Music Controller] Music resumed.");
}

/// Stops music playback
pub fn stop() {
    if let Ok(mut lock) = CURRENT_TITLE.lock() {
        *lock = None;
    }
    if let Ok(mut lock) = CURRENT_PATH.lock() {
        *lock = None;
    }
    if let Ok(mut lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(sink) = lock.take() {
            sink.stop();
        }
    }
    crate::term_log!("[Music Controller] Music stopped.");
}

/// Seek playback time in seconds
pub fn seek(seconds: f32) {
    if seconds < 0.0 {
        return;
    }
    let dur = std::time::Duration::from_secs_f32(seconds);

    // 1. First try native Sink try_seek
    let mut seek_success = false;
    if let Ok(lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref sink) = *lock {
            if sink.try_seek(dur).is_ok() {
                seek_success = true;
                crate::term_log!("[Music Controller] Seeked to {:.1}s via native sink", seconds);
            }
        }
    }

    // 2. If try_seek is not supported on current decoder, re-open audio file and seek stream
    if !seek_success {
        let was_paused = NATIVE_MUSIC_SINK
            .lock()
            .ok()
            .and_then(|lock| lock.as_ref().map(|s| s.is_paused()))
            .unwrap_or(false);

        let path_opt = CURRENT_PATH.lock().ok().and_then(|p| p.clone());
        if let Some(path_buf) = path_opt {
            if path_buf.exists() {
                if let Some(handle) = get_audio_handle() {
                    if let Ok(file) = std::fs::File::open(&path_buf) {
                        if let Ok(decoder) = rodio::Decoder::new(std::io::BufReader::new(file)) {
                            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                                let user_vol = get_music_volume();
                                let current_vol = if crate::app_state::is_ai_speaking() {
                                    user_vol * 0.25
                                } else {
                                    user_vol
                                };
                                sink.set_volume(current_vol);
                                sink.append(decoder);
                                let _ = sink.try_seek(dur);
                                if was_paused {
                                    sink.pause();
                                }
                                if let Ok(mut lock) = NATIVE_MUSIC_SINK.lock() {
                                    if let Some(old) = lock.take() {
                                        old.stop();
                                    }
                                    *lock = Some(sink);
                                    crate::term_log!(
                                        "[Music Controller] Seeked to {:.1}s via stream reload",
                                        seconds
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn get_current_title() -> Option<String> {
    if let Ok(lock) = CURRENT_TITLE.lock() {
        lock.clone()
    } else {
        None
    }
}

/// Set internal voice volume (0.0 to 3.0)
pub fn set_voice_volume(vol: f32) {
    let clamped = vol.clamp(0.0, 3.0);
    let int_val = (clamped * 100.0).round() as u32;
    CURRENT_VOICE_VOLUME.store(int_val, Ordering::SeqCst);

    if let Ok(lock) = NATIVE_VOICE_SINK.lock() {
        if let Some(ref sink) = *lock {
            sink.set_volume(clamped);
        }
    }
}

/// Get internal voice volume as float (default 1.6)
pub fn get_voice_volume() -> f32 {
    let int_val = CURRENT_VOICE_VOLUME.load(Ordering::SeqCst);
    (int_val as f32) / 100.0
}

/// Set internal music volume (0.0 to 1.0 or higher)
pub fn set_music_volume(vol: f32) {
    let clamped = vol.clamp(0.0, 2.0);
    let int_val = (clamped * 100.0).round() as u32;
    CURRENT_MUSIC_VOLUME.store(int_val, Ordering::SeqCst);

    if let Ok(lock) = NATIVE_MUSIC_SINK.lock() {
        if let Some(ref sink) = *lock {
            let actual = if crate::app_state::is_ai_speaking() {
                clamped * 0.25
            } else {
                clamped
            };
            sink.set_volume(actual);
        }
    }
}

/// Get internal music volume as float (0.0 to 1.0)
pub fn get_music_volume() -> f32 {
    let int_val = CURRENT_MUSIC_VOLUME.load(Ordering::SeqCst);
    (int_val as f32) / 100.0
}
