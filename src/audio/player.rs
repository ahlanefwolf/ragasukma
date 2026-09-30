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

/// Ekstrak sampel PCM dan sample_rate dari format WAV
fn parse_wav_samples(bytes: &[u8]) -> Option<(Vec<f32>, u32)> {
    if bytes.len() < 44 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }

    let mut cursor = 12;
    let mut sample_rate = 24000u32;
    let mut channels = 1u16;
    let mut bits_per_sample = 16u16;
    let mut data_slice: Option<&[u8]> = None;

    while cursor + 8 <= bytes.len() {
        let chunk_id = &bytes[cursor..cursor + 4];
        let chunk_size =
            u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().ok()?) as usize;
        cursor += 8;

        if chunk_id == b"fmt " {
            if cursor + 14 <= bytes.len() {
                channels = u16::from_le_bytes(bytes[cursor + 2..cursor + 4].try_into().ok()?);
                sample_rate =
                    u32::from_le_bytes(bytes[cursor + 4..cursor + 8].try_into().ok()?);
                if cursor + 16 <= bytes.len() {
                    bits_per_sample =
                        u16::from_le_bytes(bytes[cursor + 14..cursor + 16].try_into().ok()?);
                }
            }
        } else if chunk_id == b"data" {
            let actual_len = chunk_size.min(bytes.len().saturating_sub(cursor));
            data_slice = Some(&bytes[cursor..cursor + actual_len]);
            break;
        }

        cursor = cursor.saturating_add(chunk_size);
    }

    let raw_data = data_slice?;
    if bits_per_sample != 16 || channels == 0 {
        return None;
    }

    let num_samples = raw_data.len() / (2 * channels as usize);
    let mut mono_samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let mut sum = 0.0f32;
        for c in 0..channels as usize {
            let byte_idx = (i * channels as usize + c) * 2;
            if byte_idx + 2 <= raw_data.len() {
                let sample_i16 =
                    i16::from_le_bytes([raw_data[byte_idx], raw_data[byte_idx + 1]]);
                sum += (sample_i16 as f32) / 32768.0;
            }
        }
        mono_samples.push(sum / channels as f32);
    }

    Some((mono_samples, sample_rate))
}

/// Menghitung bobot bibir AIUEO realtime berdasarkan frekuensi formant gelombang audio yang sedang diputar
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

    // Ambil window 512 sampel (~12-25 ms)
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

    // Noise floor: hening / jeda napas antar kalimat
    if rms < 0.012 {
        return LipSyncWeights::default();
    }

    let mouth_open = ((rms - 0.012) * 7.5).clamp(0.0, 1.0).powf(0.8) * 0.85;

    // 2. Goertzel Filter Power pada frekuensi formant vokal utama:
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

    let aa = (r_mid * 1.5).min(1.0) * mouth_open;
    let ih = (r_high * 1.8).min(1.0) * mouth_open * 0.75;
    let ou = ((r_low - r_mid * 0.3).max(0.0) * 1.7).clamp(0.0, 1.0) * mouth_open * 0.70;
    let ee = (r_midhigh * 1.7).min(1.0) * mouth_open * 0.80;
    let oh = ((r_low * 0.7 + r_mid * 0.4) * 1.4).min(1.0) * mouth_open * 0.85;

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

    // Ekstrak data sampel PCM dan sample_rate untuk sinkronisasi bibir AIUEO presisi
    if let Some((samples, sr)) = parse_wav_samples(&wav_bytes) {
        if let Ok(mut lock) = ACTIVE_VOICE_SPEECH.lock() {
            *lock = Some(ActiveVoiceSpeech {
                samples,
                sample_rate: sr,
                start_instant: std::time::Instant::now(),
            });
        }
    }

    let my_gen = VOICE_PLAYBACK_GEN.fetch_add(1, Ordering::SeqCst) + 1;

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
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
                    // Generation ID guard mencegah race condition jika suara baru dimulai saat suara lama belum selesai
                    std::thread::spawn(move || {
                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(150));
                            if VOICE_PLAYBACK_GEN.load(Ordering::SeqCst) != my_gen {
                                // Thread suara baru telah mengambil alih, hentikan pengawas lama ini
                                break;
                            }
                            let is_done = if let Ok(lock) = NATIVE_VOICE_SINK.lock() {
                                lock.as_ref().map(|s| s.empty()).unwrap_or(true)
                            } else {
                                true
                            };
                            if is_done {
                                if VOICE_PLAYBACK_GEN.load(Ordering::SeqCst) != my_gen {
                                    break;
                                }
                                if let Ok(music_lock) = NATIVE_MUSIC_SINK.lock() {
                                    if let Some(ref m_sink) = *music_lock {
                                        m_sink.set_volume(1.0);
                                    }
                                }
                                if let Ok(mut speech_lock) = ACTIVE_VOICE_SPEECH.lock() {
                                    *speech_lock = None;
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

/// Hentikan pemutaran suara AI native (misal saat interupsi suara / VAD barge-in)
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
            m_sink.set_volume(1.0);
        }
    }
}

/// Mengecek apakah audio native voice sedang berputar di speaker PC
pub fn is_native_voice_playing() -> bool {
    if let Ok(lock) = NATIVE_VOICE_SINK.lock() {
        lock.as_ref().map(|s| !s.empty()).unwrap_or(false)
    } else {
        false
    }
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
            // Jika mode Full HP aktif (voice_muted = true), musik hanya diputar di HP, jangan putar di PC
            if !crate::app_state::is_voice_muted() && path_buf.exists() {
                if let Some(handle) = get_audio_handle() {
                    // Gunakan Cursor in-memory buffer agar decoder symphonia tidak mengalami error seek saat inisialisasi
                    if let Ok(bytes) = std::fs::read(&path_buf) {
                        if let Ok(decoder) = rodio::Decoder::new(Cursor::new(bytes)) {
                            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                                // Jika AI sedang berbicara saat musik dimulai, pasang volume ducked langsung
                                let current_vol = if crate::app_state::is_ai_speaking() { 0.25 } else { 1.0 };
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
