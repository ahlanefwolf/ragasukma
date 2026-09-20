use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::audio::dsp::{TARGET_SAMPLE_RATE, apply_noise_reduction, normalize_volume};
use crate::config::AppConfig;
use crate::tray::{AssistantState, TrayState};

pub struct AudioRecorder {
    _stream: Mutex<Option<cpal::Stream>>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<usize>>,
    stream_error: Arc<AtomicBool>,
    config: Arc<Mutex<AppConfig>>,
}

unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    fn build_stream(
        buffer: Arc<Mutex<Vec<f32>>>,
        stream_error: Arc<AtomicBool>,
        sample_rate_out: Arc<Mutex<u32>>,
        channels_out: Arc<Mutex<usize>>,
    ) -> Result<cpal::Stream> {
        let host = cpal::default_host();

        // 1. Dapatkan daftar seluruh input device yang tersedia
        let available_devices = host.input_devices().map(|iter| iter.collect::<Vec<_>>()).unwrap_or_default();
        crate::term_log!("🎙️  [Audio Init] Memeriksa {} perangkat mikrofon yang terdeteksi...", available_devices.len());

        let device = if let Some(def) = host.default_input_device() {
            def
        } else if let Some(first) = available_devices.into_iter().next() {
            first
        } else {
            anyhow::bail!("Tidak dapat menemukan perangkat input audio (microphone) yang aktif di sistem");
        };

        let dev_config = device
            .default_input_config()
            .context("Gagal mengambil konfigurasi default input device")?;

        let sample_rate = dev_config.sample_rate().0;
        let channels = dev_config.channels() as usize;
        let sample_format = dev_config.sample_format();

        if let Ok(name) = device.name() {
            crate::term_log!("🎙️  [Audio Device Aktif] \"{name}\" ({sample_rate} Hz, {channels} ch, format: {sample_format:?})");
        }

        if let Ok(mut sr) = sample_rate_out.lock() {
            *sr = sample_rate;
        }
        if let Ok(mut ch) = channels_out.lock() {
            *ch = channels;
        }

        let buffer_clone = Arc::clone(&buffer);
        let error_flag = Arc::clone(&stream_error);
        error_flag.store(false, Ordering::SeqCst);

        let err_fn = move |err| {
            crate::term_err!("⚠️ [Audio Device] Stream input terputus / error: {err}");
            error_flag.store(true, Ordering::SeqCst);
        };

        let stream_config: cpal::StreamConfig = dev_config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f32], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend_from_slice(data);
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::I16 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend(data.iter().map(|&s| s as f32 / 32768.0));
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::I32 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i32], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend(data.iter().map(|&s| s as f32 / 2147483648.0));
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::U16 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u16], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend(data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0));
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::U8 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[u8], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend(data.iter().map(|&s| (s as f32 - 128.0) / 128.0));
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::I8 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i8], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend(data.iter().map(|&s| s as f32 / 128.0));
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            cpal::SampleFormat::F64 => {
                let buf = Arc::clone(&buffer_clone);
                device.build_input_stream(
                    &stream_config,
                    move |data: &[f64], _: &_| {
                        if let Ok(mut lock) = buf.lock() {
                            lock.extend(data.iter().map(|&s| s as f32));
                            if lock.len() > sample_rate as usize * channels * 6 {
                                let drain_len = lock.len() - sample_rate as usize * channels * 3;
                                lock.drain(0..drain_len);
                            }
                        }
                    },
                    err_fn,
                    None,
                )?
            }
            _ => anyhow::bail!("Unsupported sample format: {:?}", sample_format),
        };

        stream
            .play()
            .context("Gagal memulai stream audio microphone berkelanjutan")?;
        Ok(stream)
    }

    pub fn new(config: AppConfig) -> Result<Self> {
        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let stream_error = Arc::new(AtomicBool::new(false));
        let sample_rate = Arc::new(Mutex::new(16000));
        let channels = Arc::new(Mutex::new(1));

        let stream_res = Self::build_stream(
            Arc::clone(&buffer),
            Arc::clone(&stream_error),
            Arc::clone(&sample_rate),
            Arc::clone(&channels),
        );

        let stream_opt = match stream_res {
            Ok(s) => Some(s),
            Err(e) => {
                crate::term_err!(
                    "⚠️ [Audio Init] Mic belum terhubung: {e}. Menunggu koneksi audio..."
                );
                stream_error.store(true, Ordering::SeqCst);
                None
            }
        };

        Ok(Self {
            _stream: Mutex::new(stream_opt),
            buffer,
            sample_rate,
            channels,
            stream_error,
            config: Arc::new(Mutex::new(config)),
        })
    }

    pub fn update_config(&self, new_config: AppConfig) {
        if let Ok(mut lock) = self.config.lock() {
            *lock = new_config;
        }
    }

    pub fn device_name(&self) -> String {
        "Default Microphone (Continuous Stream)".to_string()
    }

    pub fn clear_buffer(&self) {
        if let Ok(mut lock) = self.buffer.lock() {
            lock.clear();
        }
    }

    /// Background Streaming Worker: Terus menerus merekam & VAD di background,
    /// begitu satu giliran bicara selesai, audio siap saji langsung dikirim ke `turn_sender`.
    pub fn start_continuous_worker(
        &self,
        stop_worker: Arc<AtomicBool>,
        pause_worker: Arc<AtomicBool>,
        turn_sender: std::sync::mpsc::Sender<Vec<f32>>,
    ) {
        self.clear_buffer();

        let cur_sr = self.sample_rate.lock().map(|r| *r).unwrap_or(16000);
        let cur_ch = self.channels.lock().map(|c| *c).unwrap_or(1);

        let chunk_duration = std::time::Duration::from_millis(25);
        let mut samples_per_chunk = cur_sr as usize * cur_ch / 40; // ~25ms

        let (mut base_threshold, mut silence_limit_chunks, mut required_voice) = {
            if let Ok(cfg) = self.config.lock() {
                (
                    cfg.vad_activation_threshold,
                    cfg.vad_silence_limit_chunks,
                    cfg.vad_consecutive_chunks.max(1),
                )
            } else {
                (0.015, 20, 2)
            }
        };

        let cutoff = 160.0f32;
        let mut rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let mut dt = 1.0 / cur_sr as f32;
        let mut alpha = rc / (rc + dt);

        let mut prev_in = 0.0f32;
        let mut prev_out = 0.0f32;

        let mut noise_floor = 0.008f32;
        let mut peak_voice_rms = 0.020f32;
        let mut voice_started = false;
        let mut consecutive_voice = 0;
        let mut consecutive_silence = 0;
        let mut active_voice_chunks = 0;
        let mut warmup_chunks = 0;

        let mut pre_roll_buffer = Vec::<f32>::new();
        let mut recorded_audio = Vec::<f32>::new();
        let mut last_reconnect_attempt = std::time::Instant::now();
        let mut smoothed_mic_level: f32 = 0.0;

        while !stop_worker.load(Ordering::SeqCst) {
            std::thread::sleep(chunk_duration);

            // 🔄 Auto-Recovery: Cek jika stream terputus / mic dicabut
            if self.stream_error.load(Ordering::SeqCst) {
                if last_reconnect_attempt.elapsed() >= std::time::Duration::from_secs(2) {
                    last_reconnect_attempt = std::time::Instant::now();
                    if let Ok(new_stream) = Self::build_stream(
                        Arc::clone(&self.buffer),
                        Arc::clone(&self.stream_error),
                        Arc::clone(&self.sample_rate),
                        Arc::clone(&self.channels),
                    ) {
                        if let Ok(mut stream_guard) = self._stream.lock() {
                            *stream_guard = Some(new_stream);
                        }
                        self.stream_error.store(false, Ordering::SeqCst);
                        let updated_sr = self.sample_rate.lock().map(|r| *r).unwrap_or(16000);
                        let updated_ch = self.channels.lock().map(|c| *c).unwrap_or(1);
                        samples_per_chunk = updated_sr as usize * updated_ch / 40;
                        dt = 1.0 / updated_sr as f32;
                        rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
                        alpha = rc / (rc + dt);
                        crate::term_log!(
                            "🎙️  [Audio Reconnect] Microphone terhubung kembali & aktif!"
                        );
                    }
                }
            }

            let mut is_interruption_enabled = true;
            // Perbarui threshold secara live jika konfigurasi diubah di Web Dashboard
            if let Ok(cfg) = self.config.lock() {
                base_threshold = cfg.vad_activation_threshold;
                silence_limit_chunks = cfg.vad_silence_limit_chunks;
                required_voice = cfg.vad_consecutive_chunks.max(1);
                is_interruption_enabled = cfg.voice_interruption;
            }

            let session_active = crate::app_state::is_session_running();
            let tuning_active = crate::app_state::is_tuning_mode();
            let mic_muted = crate::app_state::is_mic_muted();
            let raw_paused = pause_worker.load(Ordering::SeqCst) || crate::app_state::is_paused();
            let ai_speaking = crate::app_state::is_ai_speaking();

            // Jika fitur interupsi suara aktif dan AI sedang berbicara, tetap aktifkan pantauan mic untuk barge-in
            let is_paused = if is_interruption_enabled && ai_speaking {
                false
            } else {
                raw_paused
            };

            let new_samples = {
                let mut lock = match self.buffer.lock() {
                    Ok(l) => l,
                    Err(poisoned) => poisoned.into_inner(),
                };
                if lock.is_empty() {
                    Vec::new()
                } else {
                    let s = lock.clone();
                    lock.clear();
                    s
                }
            };

            if new_samples.is_empty() {
                smoothed_mic_level *= 0.70;
                if smoothed_mic_level < 0.5 {
                    smoothed_mic_level = 0.0;
                }
                crate::app_state::update_mic_level(smoothed_mic_level.round() as u8);
                continue;
            }

            // 1. Hitung DC Offset (mean) untuk menghilangkan bias hardware / static hum
            let sample_count = new_samples.len() as f32;
            let mean: f32 = new_samples.iter().sum::<f32>() / sample_count;

            // 2. Evaluasi amplitudo AC murni (bebas DC offset) untuk Peak & RMS
            let mut ac_peak = 0.0f32;
            let mut ac_sum_sq = 0.0f32;
            let mut filtered_sum_sq = 0.0f32;

            for &sample in &new_samples {
                let cur_in = sample - mean;
                let abs_in = cur_in.abs();
                if abs_in > ac_peak {
                    ac_peak = abs_in;
                }
                ac_sum_sq += cur_in * cur_in;

                // High-pass filter (160 Hz) untuk isolasi pita suara manusia
                let cur_out = alpha * (prev_out + cur_in - prev_in);
                prev_in = cur_in;
                prev_out = cur_out;
                filtered_sum_sq += cur_out * cur_out;
            }

            let ac_rms = (ac_sum_sq / sample_count).sqrt();

            #[cfg(target_os = "linux")]
            let voice_rms = {
                // Di Linux, ALSA/PulseAudio tidak memiliki hardware APO/AEC driver seperti Windows/macOS.
                // Gunakan murni sinyal pita vokal manusia (High-Pass 160Hz) dan redam broadband static rumble.
                let filtered_rms = (filtered_sum_sq / sample_count).sqrt();
                if ac_rms > 0.0001 && (filtered_rms / ac_rms) < 0.45 {
                    filtered_rms * 0.5
                } else {
                    filtered_rms
                }
            };

            #[cfg(not(target_os = "linux"))]
            let voice_rms = (filtered_sum_sq / sample_count).sqrt().max(ac_rms * 0.7);

            // 3. Audio VU Meter Standar (dBFS):
            // Menggabungkan Peak (75%) dan RMS (25%) sinyal audio
            let signal_mag = (ac_peak * 0.75 + ac_rms * 0.25).max(0.00001);
            let dbfs = 20.0 * signal_mag.log10(); // Rentang -100 dBFS s/d 0 dBFS

            // Standar Rentang Dinamis: -60 dBFS (hening) s/d -10 dBFS (keras)
            let min_db = -60.0f32;
            let max_db = -10.0f32;
            let target_percent = if mic_muted || dbfs <= min_db {
                0.0
            } else {
                let scaled = ((dbfs - min_db) / (max_db - min_db)) * 100.0;
                scaled.clamp(0.0, 100.0)
            };

            smoothed_mic_level = if target_percent > smoothed_mic_level {
                // Fast attack: melonjak responsif (~25ms)
                smoothed_mic_level * 0.15 + target_percent * 0.85
            } else {
                // Smooth decay: meluruh halus saat jeda bicara
                smoothed_mic_level * 0.75 + target_percent * 0.25
            };

            if smoothed_mic_level < 0.5 {
                smoothed_mic_level = 0.0;
            }

            crate::app_state::update_mic_level(smoothed_mic_level.round() as u8);

            if !session_active || tuning_active || mic_muted || is_paused {
                voice_started = false;
                consecutive_voice = 0;
                consecutive_silence = 0;
                active_voice_chunks = 0;
                recorded_audio.clear();
                pre_roll_buffer.clear();
                continue;
            }

            if !voice_started {
                if crate::app_state::is_mic_muted() {
                    pre_roll_buffer.clear();
                    consecutive_voice = 0;
                    continue;
                }

                // Pre-roll buffer ~300ms
                pre_roll_buffer.extend_from_slice(&new_samples);
                let max_pre_roll = samples_per_chunk * 12;
                if pre_roll_buffer.len() > max_pre_roll {
                    let drain_len = pre_roll_buffer.len() - max_pre_roll;
                    pre_roll_buffer.drain(0..drain_len);
                }

                // Kalibrasi noise floor
                if warmup_chunks < 4 {
                    warmup_chunks += 1;
                    noise_floor = noise_floor * 0.8 + voice_rms * 0.2;
                    continue;
                }

                noise_floor = noise_floor * 0.95 + voice_rms * 0.05;

                if ai_speaking {
                    // Deteksi Barge-in / Interupsi saat AI berbicara (HANYA jika diaktifkan):
                    if is_interruption_enabled {
                        // Ambang batas dinaikkan agar pantulan speaker tidak memicu interupsi palsu
                        #[cfg(target_os = "linux")]
                        let interrupt_threshold = (noise_floor * 2.8).max(base_threshold * 2.0);
                        #[cfg(not(target_os = "linux"))]
                        let interrupt_threshold = (noise_floor * 2.2).max(base_threshold * 1.6);

                        if voice_rms > interrupt_threshold {
                            consecutive_voice += 1;
                            if consecutive_voice > required_voice {
                                voice_started = true;
                                peak_voice_rms = voice_rms;
                                active_voice_chunks = required_voice + 1;
                                recorded_audio.extend_from_slice(&pre_roll_buffer);
                                pre_roll_buffer.clear();
                                consecutive_silence = 0;
                                crate::app_state::trigger_interruption();
                                TrayState::update(AssistantState::Listening, "Mendengarkan (Interupsi)...");
                                crate::term_log!("\r⚡ [Barge-in / Interupsi] Respon AI dipotong, mendengarkan...");
                            }
                        } else {
                            consecutive_voice = 0;
                        }
                    } else {
                        consecutive_voice = 0;
                    }
                } else {
                    // Ambang batas deteksi VAD normal saat AI standby
                    #[cfg(target_os = "linux")]
                    let trigger_threshold = (noise_floor * 2.1).max(base_threshold * 1.15);
                    #[cfg(not(target_os = "linux"))]
                    let trigger_threshold = (noise_floor * 1.8).max(base_threshold);

                    if voice_rms > trigger_threshold {
                        consecutive_voice += 1;
                        if consecutive_voice >= required_voice {
                            voice_started = true;
                            peak_voice_rms = voice_rms;
                            active_voice_chunks = required_voice;
                            recorded_audio.extend_from_slice(&pre_roll_buffer);
                            pre_roll_buffer.clear();
                            consecutive_silence = 0;
                            TrayState::update(AssistantState::Listening, "Mendengarkan...");
                            crate::term_log!("\r[MEREKAM...] Silakan bicara...");
                        }
                    } else {
                        consecutive_voice = 0;
                    }
                }
            } else {
                recorded_audio.extend_from_slice(&new_samples);

                if voice_rms > peak_voice_rms {
                    peak_voice_rms = peak_voice_rms * 0.8 + voice_rms * 0.2;
                }

                // Voice continuation threshold yang lebih adaptif agar nafas/jeda pendek tidak langsung menutup rekaman
                let speech_threshold = (noise_floor * 1.35)
                    .max(base_threshold * 0.65)
                    .min(peak_voice_rms * 0.40);

                if voice_rms > speech_threshold {
                    active_voice_chunks += 1;
                    consecutive_silence = 0;
                } else {
                    consecutive_silence += 1;
                }

                // Minimum durasi hening sebelum dianggap selesai (sekitar ~450ms - 875ms)
                let target_silence_chunks = silence_limit_chunks.max(14);
                let max_length_exceeded = recorded_audio.len() > cur_sr as usize * cur_ch * 25;

                if consecutive_silence >= target_silence_chunks || max_length_exceeded {
                    // Turn selesai! Proses audio dan kirim ke worker channel
                    voice_started = false;
                    consecutive_voice = 0;
                    consecutive_silence = 0;

                    if active_voice_chunks >= 4 && !recorded_audio.is_empty() {
                        if let Ok(processed) = self.process_raw_audio(&recorded_audio) {
                            if !processed.is_empty() {
                                let _ = turn_sender.send(processed);
                            }
                        }
                    }

                    recorded_audio.clear();
                    pre_roll_buffer.clear();
                    active_voice_chunks = 0;
                }
            }
        }
    }

    fn process_raw_audio(&self, raw: &[f32]) -> Result<Vec<f32>> {
        let cur_sr = self.sample_rate.lock().map(|r| *r).unwrap_or(16000);
        let cur_ch = self.channels.lock().map(|c| *c).unwrap_or(1);

        let mono_samples = if cur_ch > 1 {
            raw.chunks_exact(cur_ch)
                .map(|frame| frame.iter().sum::<f32>() / cur_ch as f32)
                .collect::<Vec<f32>>()
        } else {
            raw.to_vec()
        };

        let mut resampled = if cur_sr != TARGET_SAMPLE_RATE {
            resample_to_16k(&mono_samples, cur_sr)?
        } else {
            mono_samples
        };

        let noise_reduction_level = self
            .config
            .lock()
            .map(|c| c.noise_reduction_level)
            .unwrap_or(0);

        #[cfg(target_os = "linux")]
        {
            // Windows APO Voice Processing Emulation for Linux:
            // 1. DC Offset & 2-Pass High-Pass 120Hz (menghilangkan dengung kipas/meja)
            crate::audio::dsp::remove_dc_offset(&mut resampled);
            crate::audio::dsp::apply_high_pass_filter(&mut resampled, TARGET_SAMPLE_RATE, 120.0, 2);

            // 2. Smooth Noise Gate (level 1 jika default/0, atau sesuai pilihan pengguna)
            let effective_nr = if noise_reduction_level == 0 { 1 } else { noise_reduction_level };
            crate::audio::dsp::apply_smooth_noise_gate(&mut resampled, TARGET_SAMPLE_RATE, effective_nr);

            // 3. Dynamic Speech AGC (menjaga dynamic range vokal stabil & jernih layaknya Windows WASAPI APO)
            crate::audio::dsp::apply_speech_agc(&mut resampled, 0.12);
            normalize_volume(&mut resampled, 0.85, 4.0);
        }

        #[cfg(not(target_os = "linux"))]
        {
            apply_noise_reduction(&mut resampled, TARGET_SAMPLE_RATE, noise_reduction_level);
            normalize_volume(&mut resampled, 0.85, 8.0);
        }

        if resampled.len() < TARGET_SAMPLE_RATE as usize / 2 {
            resampled.resize(TARGET_SAMPLE_RATE as usize / 2, 0.0);
        }

        Ok(resampled)
    }
}

fn resample_to_16k(input: &[f32], source_rate: u32) -> Result<Vec<f32>> {
    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        TARGET_SAMPLE_RATE as f64 / source_rate as f64,
        1.1,
        params,
        input.len(),
        1,
    )?;

    let waves_in = vec![input.to_vec()];
    let waves_out = resampler.process(&waves_in, None)?;

    Ok(waves_out.into_iter().next().unwrap_or_default())
}
