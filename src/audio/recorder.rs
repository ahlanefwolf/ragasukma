use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::config::AppConfig;
use crate::audio::dsp::{apply_noise_reduction, normalize_volume, TARGET_SAMPLE_RATE};
use crate::tray::{AssistantState, TrayState};

pub struct AudioRecorder {
    _stream: Arc<cpal::Stream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: usize,
    config: AppConfig,
}

unsafe impl Send for AudioRecorder {}
unsafe impl Sync for AudioRecorder {}

impl AudioRecorder {
    pub fn new(config: AppConfig) -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("Tidak dapat menemukan perangkat input audio (microphone)")?;

        let dev_config = device
            .default_input_config()
            .context("Gagal mengambil konfigurasi default input device")?;

        let sample_rate = dev_config.sample_rate().0;
        let channels = dev_config.channels() as usize;
        let sample_format = dev_config.sample_format();

        let buffer = Arc::new(Mutex::new(Vec::<f32>::new()));
        let buffer_clone = Arc::clone(&buffer);

        let err_fn = |err| crate::term_err!("Audio stream error: {err}");

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &dev_config.into(),
                move |data: &[f32], _: &_| {
                    if let Ok(mut lock) = buffer_clone.lock() {
                        lock.extend_from_slice(data);
                        if lock.len() > sample_rate as usize * channels * 6 {
                            let drain_len = lock.len() - sample_rate as usize * channels * 3;
                            lock.drain(0..drain_len);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &dev_config.into(),
                move |data: &[i16], _: &_| {
                    if let Ok(mut lock) = buffer_clone.lock() {
                        lock.extend(data.iter().map(|&s| s as f32 / 32768.0));
                        if lock.len() > sample_rate as usize * channels * 6 {
                            let drain_len = lock.len() - sample_rate as usize * channels * 3;
                            lock.drain(0..drain_len);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_input_stream(
                &dev_config.into(),
                move |data: &[u16], _: &_| {
                    if let Ok(mut lock) = buffer_clone.lock() {
                        lock.extend(data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0));
                        if lock.len() > sample_rate as usize * channels * 6 {
                            let drain_len = lock.len() - sample_rate as usize * channels * 3;
                            lock.drain(0..drain_len);
                        }
                    }
                },
                err_fn,
                None,
            )?,
            _ => anyhow::bail!("Unsupported sample format: {:?}", sample_format),
        };

        stream.play().context("Gagal memulai stream audio microphone berkelanjutan")?;

        Ok(Self {
            _stream: Arc::new(stream),
            buffer,
            sample_rate,
            channels,
            config,
        })
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

        let chunk_duration = std::time::Duration::from_millis(25);
        let samples_per_chunk = self.sample_rate as usize * self.channels / 40; // ~25ms

        let base_threshold = self.config.vad_activation_threshold;
        let silence_limit_chunks = self.config.vad_silence_limit_chunks;

        let cutoff = 160.0f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff);
        let dt = 1.0 / self.sample_rate as f32;
        let alpha = rc / (rc + dt);

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

        while !stop_worker.load(Ordering::SeqCst) {
            std::thread::sleep(chunk_duration);

            let session_active = crate::native::server::is_session_running();
            let tuning_active = crate::native::server::is_tuning_mode();
            let mic_muted = crate::native::server::is_mic_muted();
            let is_paused = pause_worker.load(Ordering::SeqCst) || crate::native::server::is_paused();

            if !session_active || tuning_active || mic_muted || is_paused {
                voice_started = false;
                consecutive_voice = 0;
                consecutive_silence = 0;
                active_voice_chunks = 0;
                recorded_audio.clear();
                pre_roll_buffer.clear();
                self.clear_buffer();
                continue;
            }

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
                continue;
            }

            // Hitung RMS vokal manusia
            let mean: f32 = new_samples.iter().sum::<f32>() / new_samples.len() as f32;
            let mut filtered_sum_sq = 0.0f32;
            for &sample in &new_samples {
                let cur_in = sample - mean;
                let cur_out = alpha * (prev_out + cur_in - prev_in);
                prev_in = cur_in;
                prev_out = cur_out;
                filtered_sum_sq += cur_out * cur_out;
            }
            let voice_rms = (filtered_sum_sq / new_samples.len() as f32).sqrt();

            if !voice_started {
                if crate::native::server::is_mic_muted() {
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
                // Threshold dinamis: 1.8x dari noise lantai ruangan riil, dibatasi minimum base_threshold
                let trigger_threshold = (noise_floor * 1.8).max(base_threshold);

                if voice_rms > trigger_threshold {
                    consecutive_voice += 1;
                    let required_voice = self.config.vad_consecutive_chunks.max(1);
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
            } else {
                recorded_audio.extend_from_slice(&new_samples);

                if voice_rms > peak_voice_rms {
                    peak_voice_rms = peak_voice_rms * 0.8 + voice_rms * 0.2;
                }

                // Voice continuation threshold yang lebih adaptif agar nafas/jeda pendek tidak langsung menutup rekaman
                let speech_threshold = (noise_floor * 1.35).max(base_threshold * 0.65).min(peak_voice_rms * 0.40);

                if voice_rms > speech_threshold {
                    active_voice_chunks += 1;
                    consecutive_silence = 0;
                } else {
                    consecutive_silence += 1;
                }

                // Minimum durasi hening sebelum dianggap selesai (sekitar ~450ms - 875ms)
                let target_silence_chunks = silence_limit_chunks.max(14);
                let max_length_exceeded = recorded_audio.len() > self.sample_rate as usize * self.channels * 25;

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
        let mono_samples = if self.channels > 1 {
            raw.chunks_exact(self.channels)
                .map(|frame| frame.iter().sum::<f32>() / self.channels as f32)
                .collect::<Vec<f32>>()
        } else {
            raw.to_vec()
        };

        let mut resampled = if self.sample_rate != TARGET_SAMPLE_RATE {
            resample_to_16k(&mono_samples, self.sample_rate)?
        } else {
            mono_samples
        };

        apply_noise_reduction(&mut resampled, TARGET_SAMPLE_RATE, self.config.noise_reduction_level);
        normalize_volume(&mut resampled, 0.85, 8.0);

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
