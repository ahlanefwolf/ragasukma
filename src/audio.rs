use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::config::AppConfig;
use crate::dsp::{apply_noise_reduction, normalize_volume, TARGET_SAMPLE_RATE};
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

        let err_fn = |err| eprintln!("Audio stream error: {err}");

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

    /// Unified Continuous Turn Engine untuk merekam suara interaktif
    pub fn record_turn(&self, stop_flag: Arc<AtomicBool>) -> Result<Vec<f32>> {
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

        while !stop_flag.load(Ordering::SeqCst) {
            std::thread::sleep(chunk_duration);

            let new_samples = {
                let mut lock = match self.buffer.lock() {
                    Ok(l) => l,
                    Err(_) => break,
                };
                let s = lock.clone();
                lock.clear();
                s
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
                // Jaga pre-roll buffer ~300ms agar kata awal tidak pernah terpotong
                pre_roll_buffer.extend_from_slice(&new_samples);
                let max_pre_roll = samples_per_chunk * 12;
                if pre_roll_buffer.len() > max_pre_roll {
                    let drain_len = pre_roll_buffer.len() - max_pre_roll;
                    pre_roll_buffer.drain(0..drain_len);
                }

                // 4 chunks pertama (~100ms) dipakai khusus kalibrasi noise floor ruangan riil
                if warmup_chunks < 4 {
                    warmup_chunks += 1;
                    noise_floor = noise_floor * 0.8 + voice_rms * 0.2;
                    continue;
                }

                noise_floor = noise_floor * 0.98 + voice_rms * 0.02;
                // Ambang aktivasi yang responsif untuk vokal manusia (tidak perlu teriak)
                let trigger_threshold = (noise_floor * 2.0).max(base_threshold * 0.65).max(0.015);

                if voice_rms > trigger_threshold {
                    consecutive_voice += 1;
                    // Cukup 2 chunks (~50ms) di atas threshold untuk mulai merekam
                    if consecutive_voice >= 2 {
                        voice_started = true;
                        peak_voice_rms = voice_rms;
                        active_voice_chunks = 2;
                        recorded_audio.extend_from_slice(&pre_roll_buffer);
                        pre_roll_buffer.clear();
                        consecutive_silence = 0;
                        TrayState::update(AssistantState::Listening, "Mendengarkan...");
                        eprint!("\r🔴 [MEREKAM...] Silakan bicara...\x1b[K\n");
                    }
                } else {
                    consecutive_voice = 0;
                }
            } else {
                // Sedang merekam suara pembicaraan
                recorded_audio.extend_from_slice(&new_samples);

                if voice_rms > peak_voice_rms {
                    peak_voice_rms = peak_voice_rms * 0.8 + voice_rms * 0.2;
                }

                // Threshold dinamis: 35% di bawah peak suara kamu saat bicara atau di atas noise floor
                let speech_threshold = (noise_floor * 1.6).max(base_threshold).max(peak_voice_rms * 0.35);

                if voice_rms > speech_threshold {
                    active_voice_chunks += 1;
                    consecutive_silence = 0;
                } else {
                    consecutive_silence += 1;
                }

                // Selesai jika jeda hening terpenuhi (maksimal 14-16 chunks = ~350 - 400ms cepat!)
                let target_silence_chunks = silence_limit_chunks.min(16);
                if consecutive_silence >= target_silence_chunks {
                    break;
                }

                // Batas maksimal 15 detik bicara tanpa henti
                if recorded_audio.len() > self.sample_rate as usize * self.channels * 15 {
                    break;
                }
            }
        }

        // Jika vokal aktifnya terlalu sedikit (< 4 chunks = < 100ms), buang langsung
        if recorded_audio.is_empty() || active_voice_chunks < 4 {
            if voice_started {
                TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
            }
            return Ok(Vec::new());
        }

        // 1. Convert multichannel ke mono
        let mono_samples = if self.channels > 1 {
            recorded_audio
                .chunks_exact(self.channels)
                .map(|frame| frame.iter().sum::<f32>() / self.channels as f32)
                .collect::<Vec<f32>>()
        } else {
            recorded_audio
        };

        // 2. Resample ke 16,000 Hz jika microphone belum 16kHz
        let mut resampled = if self.sample_rate != TARGET_SAMPLE_RATE {
            resample_to_16k(&mono_samples, self.sample_rate)?
        } else {
            mono_samples
        };

        // 3. DSP Noise Reduction Pipeline
        apply_noise_reduction(&mut resampled, TARGET_SAMPLE_RATE, self.config.noise_reduction_level);

        // 4. Normalisasi Volume Peak
        normalize_volume(&mut resampled, 0.85, 8.0);

        // 5. Pastikan panjang audio minimal 0.5s (~8000 sampel) agar kalkulasi mel spectrogram tidak gagal
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
