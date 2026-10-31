//! OpenVoice V2 Tone Color Converter & Speaker Embedding Pipeline
//!
//! Adapted from OpenVoice by MyShell.ai:
//! - Repository: https://github.com/myshell-ai/openvoice
//! - Paper: OpenVoice: Versatile Instant Voice Cloning (Qin et al.)
//!
//! Implements high-performance DSP (Radix-2 FFT, STFT spectrogram)
//! and ONNX Tone Color Converter for zero-shot voice morphing.

use anyhow::{Result, anyhow};
use ort::ep::CPU;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub struct OpenVoiceEngine {
    ref_enc_sess: Mutex<Option<Session>>,
    tone_conv_sess: Mutex<Session>,
    ref_se_cache: Mutex<Option<(String, Vec<f32>)>>,
    base_se_cache: Mutex<Option<Vec<f32>>>,
    model_dir: PathBuf,
    intra_threads: usize,
}

static OPENVOICE_ENGINE: OnceLock<Option<OpenVoiceEngine>> = OnceLock::new();

fn find_openvoice_model_dir() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("models/openvoice-v2"),
        PathBuf::from("../models/openvoice-v2"),
        crate::config::get_models_dir().join("openvoice-v2"),
        crate::config::get_app_data_dir().join("models/openvoice-v2"),
        crate::config::get_app_config_dir().join("models/openvoice-v2"),
    ];

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("models/openvoice-v2"));
            candidates.push(parent.join("../models/openvoice-v2"));
        }
    }

    candidates
        .into_iter()
        .find(|c| c.join("openvoice_tone_converter.onnx").exists())
}

pub fn get_engine() -> Option<&'static OpenVoiceEngine> {
    OPENVOICE_ENGINE
        .get_or_init(|| match init_engine() {
            Ok(e) => Some(e),
            Err(e) => {
                crate::term_err!("[OpenVoice Engine] Failed to initialize: {e}");
                None
            }
        })
        .as_ref()
}

pub fn init_engine() -> Result<OpenVoiceEngine> {
    let model_dir = find_openvoice_model_dir().ok_or_else(|| {
        anyhow!("OpenVoice model directory not found (models/openvoice-v2/openvoice_tone_converter.onnx)")
    })?;

    let budget = crate::engine::hardware_probe::get_recommended_tts_threads();
    let intra_threads = budget.openvoice_threads;

    crate::term_log!(
        "[OpenVoice Engine] Loading Tone Color Converter from {:?} (threads: {})",
        model_dir,
        intra_threads
    );

    let tone_conv_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_execution_providers([CPU::default().with_arena_allocator(false).build()])
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(intra_threads)
        .map_err(|e| anyhow!("{e}"))?
        .with_inter_threads(1)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow!("{e}"))?
        .with_memory_pattern(false)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("openvoice_tone_converter.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    Ok(OpenVoiceEngine {
        ref_enc_sess: Mutex::new(None), // Loaded lazily on-demand only when .se.bin needs computation
        tone_conv_sess: Mutex::new(tone_conv_sess),
        ref_se_cache: Mutex::new(None),
        base_se_cache: Mutex::new(None),
        model_dir,
        intra_threads,
    })
}

/// Find standard reference voice audio in the assets folder (`assets/refvoice.*` / candidates)
pub fn find_ref_voice_path() -> Option<PathBuf> {
    let supported_exts = ["wav", "mp3", "ogg", "flac", "m4a", "aac"];
    for ext in &supported_exts {
        let filename = format!("refvoice.{ext}");
        let candidate = crate::assets::find_file_in_candidates(&filename);
        if candidate.exists() && candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Reset in-memory cache and delete disk cache (.se.bin) when user uploads a new reference audio.
/// Avoids calling get_engine() so ONNX runtime initialization is not triggered in main process.
pub fn invalidate_ref_cache() {
    if let Some(Some(engine)) = OPENVOICE_ENGINE.get() {
        if let Ok(mut lock) = engine.ref_se_cache.lock() {
            *lock = None;
        }
    }

    if let Some(ref_path) = find_ref_voice_path() {
        let _ = std::fs::remove_file(ref_path.with_extension("se.bin"));
    }
    let fallback_cache = crate::config::get_app_cache_dir().join("refvoice.se.bin");
    let _ = std::fs::remove_file(fallback_cache);
}

fn load_se_from_disk(cache_path: &Path, audio_mtime: Option<std::time::SystemTime>) -> Option<Vec<f32>> {
    let cache_meta = std::fs::metadata(cache_path).ok()?;
    if let (Some(audio_mt), Ok(cache_mt)) = (audio_mtime, cache_meta.modified()) {
        if cache_mt < audio_mt {
            return None; // reference audio is newer than cache
        }
    }
    let bytes = std::fs::read(cache_path).ok()?;
    if bytes.len() != 256 * 4 {
        return None;
    }
    let (chunks, _) = bytes.as_chunks::<4>();
    let mut se = Vec::with_capacity(256);
    for &arr in chunks {
        se.push(f32::from_le_bytes(arr));
    }
    Some(se)
}

fn save_se_to_disk(cache_path: &Path, se: &[f32]) {
    if let Some(parent) = cache_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut bytes = Vec::with_capacity(se.len() * 4);
    for &val in se {
        bytes.extend_from_slice(&val.to_le_bytes());
    }
    let _ = std::fs::write(cache_path, bytes);
}

// ---------------------------------------------------------------------------
// High-Performance DSP: Fast Radix-2 FFT (N=1024), STFT, Resampling
// ---------------------------------------------------------------------------

struct FastFft1024 {
    bit_rev: [usize; 1024],
    twiddles: [(f32, f32); 512],
    hann_window: [f32; 1024],
}

impl FastFft1024 {
    fn new() -> Self {
        let mut bit_rev = [0usize; 1024];
        for (i, item) in bit_rev.iter_mut().enumerate() {
            let mut rev = 0;
            let mut val = i;
            for _ in 0..10 {
                rev = (rev << 1) | (val & 1);
                val >>= 1;
            }
            *item = rev;
        }

        let mut twiddles = [(0.0f32, 0.0f32); 512];
        for (k, item) in twiddles.iter_mut().enumerate() {
            let angle = -2.0 * PI * k as f32 / 1024.0;
            *item = (angle.cos(), angle.sin());
        }

        let mut hann_window = [0.0f32; 1024];
        for (i, item) in hann_window.iter_mut().enumerate() {
            *item = 0.5 * (1.0 - (2.0 * PI * i as f32 / 1023.0).cos());
        }

        Self {
            bit_rev,
            twiddles,
            hann_window,
        }
    }

    #[inline(always)]
    fn forward(&self, real: &mut [f32; 1024], imag: &mut [f32; 1024]) {
        for i in 0..1024 {
            let j = self.bit_rev[i];
            if i < j {
                real.swap(i, j);
                imag.swap(i, j);
            }
        }

        let mut len = 2;
        while len <= 1024 {
            let half = len / 2;
            let step = 1024 / len;
            for i in (0..1024).step_by(len) {
                for k in 0..half {
                    let (cos_val, sin_val) = self.twiddles[k * step];
                    let u_r = real[i + k];
                    let u_i = imag[i + k];
                    let v_r = real[i + k + half];
                    let v_i = imag[i + k + half];

                    let t_r = v_r * cos_val - v_i * sin_val;
                    let t_i = v_r * sin_val + v_i * cos_val;

                    real[i + k] = u_r + t_r;
                    imag[i + k] = u_i + t_i;
                    real[i + k + half] = u_r - t_r;
                    imag[i + k + half] = u_i - t_i;
                }
            }
            len <<= 1;
        }
    }
}

static FAST_FFT_1024: OnceLock<FastFft1024> = OnceLock::new();

fn get_fast_fft() -> &'static FastFft1024 {
    FAST_FFT_1024.get_or_init(FastFft1024::new)
}

/// Resample single-channel f32 audio using linear interpolation
pub fn resample_linear(input: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || input.is_empty() {
        return input.to_vec();
    }
    let ratio = to_rate as f64 / from_rate as f64;
    let target_len = (input.len() as f64 * ratio).round() as usize;
    let mut out = Vec::with_capacity(target_len);

    for i in 0..target_len {
        let src_idx = i as f64 / ratio;
        let idx0 = src_idx.floor() as usize;
        let idx1 = (idx0 + 1).min(input.len() - 1);
        let frac = (src_idx - idx0 as f64) as f32;

        let s0 = input[idx0];
        let s1 = input[idx1];
        out.push(s0 + frac * (s1 - s0));
    }
    out
}

/// PyTorch / OpenVoice compatible STFT Spectrogram using Ultra-Fast Radix-2 FFT:
/// n_fft = 1024, hop_length = 256, win_length = 1024, center = false
/// Returns: (spec_data [513 * n_frames], n_frames) where shape is [1, 513, n_frames]
pub fn compute_linear_spectrogram(
    samples: &[f32],
    filter_length: usize,
    hop_length: usize,
    win_length: usize,
) -> (Vec<f32>, usize) {
    let num_bins = filter_length / 2 + 1; // 513
    if samples.len() < win_length {
        return (vec![0.0; num_bins], 1);
    }

    let n_frames = (samples.len() - win_length) / hop_length + 1;
    let fft = get_fast_fft();
    let mut spec_data = vec![0.0f32; num_bins * n_frames];

    let mut real_buf = [0.0f32; 1024];
    let mut imag_buf = [0.0f32; 1024];

    for frame_idx in 0..n_frames {
        let start = frame_idx * hop_length;
        let frame_samples = &samples[start..start + win_length];

        // Apply precomputed Hann window
        for n in 0..1024 {
            real_buf[n] = frame_samples[n] * fft.hann_window[n];
            imag_buf[n] = 0.0;
        }

        // Run O(N log N) in-place FFT in ~1 microsecond
        fft.forward(&mut real_buf, &mut imag_buf);

        // Compute magnitude for positive frequencies [0..513] directly into continuous slice
        let out_offset = frame_idx;
        for k in 0..num_bins {
            let r = real_buf[k];
            let im = imag_buf[k];
            spec_data[k * n_frames + out_offset] = (r * r + im * im).sqrt();
        }
    }

    (spec_data, n_frames)
}

/// Limits reference audio duration to the first 8.0 seconds.
/// OpenVoice V2 requires only 3-5 seconds of clean audio to extract speaker embedding [1, 256, 1].
pub const MAX_REF_AUDIO_DURATION_SECS: f32 = 8.0;

/// Decode any supported audio file on disk to 22.05kHz mono f32 samples,
/// capped to the first 5-10 seconds to conserve memory and STFT computation time.
pub fn load_audio_file_22k(file_path: &Path) -> Result<Vec<f32>> {
    let file = std::fs::File::open(file_path)?;
    let cursor = std::io::BufReader::new(file);
    let source = rodio::Decoder::new(cursor).map_err(|e| anyhow!("Failed to decode audio: {e}"))?;

    use rodio::Source;
    let channels = source.channels() as usize;
    let sample_rate = source.sample_rate();

    // Bound sample reading from decoder to prevent memory bloat
    let max_raw_samples = (sample_rate as f32 * channels as f32 * MAX_REF_AUDIO_DURATION_SECS).ceil() as usize;
    let samples_raw: Vec<f32> = source.convert_samples::<f32>().take(max_raw_samples).collect();

    // Convert to mono
    let mut mono = Vec::with_capacity(samples_raw.len() / channels);
    for chunk in samples_raw.chunks(channels) {
        let sum: f32 = chunk.iter().sum();
        mono.push(sum / channels as f32);
    }

    // Skip leading silence (threshold amplitude 0.01) so encoder captures pure vocal timbre
    let speech_start = mono.iter().position(|&s| s.abs() > 0.01).unwrap_or(0);
    let active_mono = if speech_start < mono.len() {
        &mono[speech_start..]
    } else {
        &mono[..]
    };

    // Resample to 22050 Hz
    let mut resampled = resample_linear(active_mono, sample_rate, 22050);

    // Hard limit resampled samples to 22050 * MAX_REF_AUDIO_DURATION_SECS
    let max_22k_samples = (22050.0 * MAX_REF_AUDIO_DURATION_SECS).round() as usize;
    if resampled.len() > max_22k_samples {
        resampled.truncate(max_22k_samples);
    }

    Ok(resampled)
}

impl OpenVoiceEngine {
    /// Load `openvoice_ref_encoder.onnx` on demand if .se.bin does not exist yet and needs computation
    fn get_or_load_ref_encoder(&self) -> Result<std::sync::MutexGuard<'_, Option<Session>>> {
        let mut guard = self.ref_enc_sess.lock().map_err(|e| anyhow!("{e}"))?;
        if guard.is_none() {
            crate::term_log!(
                "[OpenVoice Engine] Loading openvoice_ref_encoder.onnx on-demand (threads: {})...",
                self.intra_threads
            );
            let sess = Session::builder()
                .map_err(|e| anyhow!("{e}"))?
                .with_execution_providers([CPU::default().with_arena_allocator(false).build()])
                .map_err(|e| anyhow!("{e}"))?
                .with_intra_threads(self.intra_threads)
                .map_err(|e| anyhow!("{e}"))?
                .with_inter_threads(1)
                .map_err(|e| anyhow!("{e}"))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| anyhow!("{e}"))?
                .with_memory_pattern(false)
                .map_err(|e| anyhow!("{e}"))?
                .commit_from_file(self.model_dir.join("openvoice_ref_encoder.onnx"))
                .map_err(|e| anyhow!("{e}"))?;
            *guard = Some(sess);
        }
        Ok(guard)
    }

    /// Immediately unloads `openvoice_ref_encoder.onnx` from RAM once embedding extraction is completed
    pub fn unload_ref_encoder(&self) {
        if let Ok(mut guard) = self.ref_enc_sess.lock() {
            if guard.is_some() {
                crate::term_log!("[OpenVoice Engine] Unloading openvoice_ref_encoder.onnx from RAM");
                *guard = None;
            }
        }
    }

    /// Extract Speaker Embedding `g` [1, 256, 1] from 22.05kHz mono samples
    pub fn extract_se_from_samples(&self, samples_22k: &[f32]) -> Result<Vec<f32>> {
        // Capping tambahan maksimal 10 detik audio (220500 samples)
        let max_samples = 22050 * 10;
        let samples_slice = if samples_22k.len() > max_samples {
            &samples_22k[..max_samples]
        } else {
            samples_22k
        };

        let (spec, n_frames) = compute_linear_spectrogram(samples_slice, 1024, 256, 1024);

        // Transpose [1, 513, n_frames] to [1, n_frames, 513]
        let mut spec_transposed = vec![0.0f32; n_frames * 513];
        for f in 0..n_frames {
            for k in 0..513 {
                spec_transposed[f * 513 + k] = spec[k * n_frames + f];
            }
        }

        let input_tensor = Tensor::from_array(([1usize, n_frames, 513usize], spec_transposed))
            .map_err(|e| anyhow!("{e}"))?;

        let inputs = ort::inputs!["spec_transposed" => input_tensor];
        let mut ref_sess_guard = self.get_or_load_ref_encoder()?;
        let ref_sess = ref_sess_guard.as_mut().ok_or_else(|| anyhow!("Ref encoder session not loaded"))?;
        let outputs = ref_sess.run(inputs)?;

        let (out_shape, out_slice) = outputs["g"].try_extract_tensor::<f32>()?;

        if out_shape.as_ref() != [1, 256, 1] {
            return Err(anyhow!("Unexpected ref encoder output shape: {:?}", out_shape));
        }

        Ok(out_slice.to_vec())
    }

    /// Retrieve target SE from reference file on disk (with in-memory & persistent disk cache).
    /// If disk cache .se.bin (1 KB) exists, the binary file is read directly without decoding audio
    /// or loading openvoice_ref_encoder.onnx.
    pub fn get_target_se(&self, ref_path: &Path) -> Result<Vec<f32>> {
        let path_str = ref_path.to_string_lossy().to_string();
        if let Ok(lock) = self.ref_se_cache.lock() {
            if let Some((ref cached_path, ref se)) = *lock {
                if cached_path == &path_str {
                    return Ok(se.clone());
                }
            }
        }

        let audio_mtime = std::fs::metadata(ref_path).and_then(|m| m.modified()).ok();
        let cache_file = ref_path.with_extension("se.bin");
        let fallback_cache = crate::config::get_app_cache_dir().join("refvoice.se.bin");

        // 1. Try loading from disk cache (.se.bin) for instant retrieval without inference or encoder load
        if let Some(se) = load_se_from_disk(&fallback_cache, audio_mtime)
            .or_else(|| load_se_from_disk(&cache_file, audio_mtime))
        {
            crate::term_log!("[OpenVoice Engine] Loaded cached speaker embedding from cache (1 KB): {:?}", fallback_cache);
            if let Ok(mut lock) = self.ref_se_cache.lock() {
                *lock = Some((path_str, se.clone()));
            }
            return Ok(se);
        }

        // 2. If disk cache does not exist, extract speaker embedding and save to .se.bin
        crate::term_log!("[OpenVoice Engine] Extracting speaker embedding from {:?} (capped to first {}s)", ref_path, MAX_REF_AUDIO_DURATION_SECS);
        let samples = load_audio_file_22k(ref_path)?;
        let se = self.extract_se_from_samples(&samples)?;

        // Save to app cache directory (~/.cache/ragasukma/refvoice.se.bin)
        save_se_to_disk(&fallback_cache, &se);
        if PathBuf::from("Cargo.toml").exists() {
            save_se_to_disk(&cache_file, &se);
        }
        crate::term_log!("[OpenVoice Engine] Saved speaker embedding (.se.bin) to cache: {:?}", fallback_cache);

        // Immediately unload openvoice_ref_encoder.onnx from RAM
        self.unload_ref_encoder();

        if let Ok(mut lock) = self.ref_se_cache.lock() {
            *lock = Some((path_str, se.clone()));
        }
        Ok(se)
    }

    /// Retrieve base speaker embedding (Supertonic source voice).
    /// Cached in memory and disk (1 KB) so tone conversion avoids repeated extraction.
    fn get_or_extract_base_se(&self, src_22k: &[f32]) -> Result<Vec<f32>> {
        if let Ok(lock) = self.base_se_cache.lock() {
            if let Some(ref se) = *lock {
                return Ok(se.clone());
            }
        }

        let base_cache_file = crate::config::get_app_cache_dir().join("base_speaker.se.bin");
        let model_base_cache = self.model_dir.join("base_speaker.se.bin");

        if let Some(se) = load_se_from_disk(&base_cache_file, None)
            .or_else(|| load_se_from_disk(&model_base_cache, None))
        {
            if let Ok(mut lock) = self.base_se_cache.lock() {
                *lock = Some(se.clone());
            }
            return Ok(se);
        }

        // OpenVoice ref encoder requires at least 2.5-3 seconds of audio for stable timbre extraction.
        // If the initial word chunk is too short (e.g. "Hello" < 1s), repeat up to ~3 seconds for a clean voice profile.
        let min_samples = 22050 * 3;
        let mut padded = Vec::new();
        let samples_to_encode = if src_22k.len() < min_samples && !src_22k.is_empty() {
            while padded.len() < min_samples {
                padded.extend_from_slice(src_22k);
            }
            &padded[..min_samples]
        } else {
            src_22k
        };

        let se = self.extract_se_from_samples(samples_to_encode)?;
        save_se_to_disk(&base_cache_file, &se);
        self.unload_ref_encoder();

        if let Ok(mut lock) = self.base_se_cache.lock() {
            *lock = Some(se.clone());
        }
        Ok(se)
    }

    /// Converts audio timbre to target voice reference using pre-extracted target embedding
    pub fn convert_voice_with_target_se(
        &self,
        src_samples: &[f32],
        src_sample_rate: u32,
        sid_tgt: &[f32],
        tau: f32,
    ) -> Result<Vec<f32>> {
        // 1. Resample source audio to 22.05kHz
        let src_22k = resample_linear(src_samples, src_sample_rate, 22050);

        // 2. Base speaker embedding (cached in RAM & 1 KB disk, avoids repeated per-chunk extraction)
        let sid_src = self.get_or_extract_base_se(&src_22k)?;

        // 3. Compute Linear Spectrogram for Tone Converter
        let (spec, n_frames) = compute_linear_spectrogram(&src_22k, 1024, 256, 1024);

        let spec_tensor = Tensor::from_array(([1usize, 513usize, n_frames], spec))
            .map_err(|e| anyhow!("{e}"))?;
        let lengths_tensor = Tensor::from_array(([1usize], vec![n_frames as i64]))
            .map_err(|e| anyhow!("{e}"))?;
        let src_tensor = Tensor::from_array(([1usize, 256usize, 1usize], sid_src))
            .map_err(|e| anyhow!("{e}"))?;
        let tgt_tensor = Tensor::from_array(([1usize, 256usize, 1usize], sid_tgt.to_vec()))
            .map_err(|e| anyhow!("{e}"))?;
        let tau_tensor = Tensor::from_array(([1usize], vec![tau]))
            .map_err(|e| anyhow!("{e}"))?;

        let inputs = ort::inputs![
            "spec" => spec_tensor,
            "spec_lengths" => lengths_tensor,
            "sid_src" => src_tensor,
            "sid_tgt" => tgt_tensor,
            "tau" => tau_tensor
        ];

        let mut tone_sess = self.tone_conv_sess.lock().map_err(|e| anyhow!("{e}"))?;
        let outputs = tone_sess.run(inputs)?;

        let (_out_shape, out_slice) = outputs["audio"].try_extract_tensor::<f32>()?;

        Ok(out_slice.to_vec())
    }

    /// Converts audio timbre to target voice reference
    #[allow(dead_code)]
    pub fn convert_voice(
        &self,
        src_samples: &[f32],
        src_sample_rate: u32,
        ref_path: &Path,
        tau: f32,
    ) -> Result<Vec<f32>> {
        let sid_tgt = self.get_target_se(ref_path)?;
        self.convert_voice_with_target_se(src_samples, src_sample_rate, &sid_tgt, tau)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openvoice_init() {
        if find_openvoice_model_dir().is_some() {
            let engine = init_engine();
            assert!(engine.is_ok(), "OpenVoice ONNX engine failed to load: {:?}", engine.err());
            let engine = engine.unwrap();

            // Verify ref encoder is lazy (not loaded immediately upon init)
            assert!(engine.ref_enc_sess.lock().unwrap().is_none(), "Ref encoder should not be loaded on init");

            let dummy_samples = vec![0.0f32; 22050 * 3]; // 3s
            let se = engine.extract_se_from_samples(&dummy_samples);
            assert!(se.is_ok(), "SE extraction failed: {:?}", se.err());
            let se = se.unwrap();
            assert_eq!(se.len(), 256, "Speaker embedding must have 256 dimensions");

            let target_se = se.clone();
            let res = engine.convert_voice_with_target_se(&dummy_samples, 22050, &target_se, 0.3);
            assert!(res.is_ok(), "convert failed: {:?}", res.err());
            let converted = res.unwrap();
            assert!(!converted.is_empty());
        }
    }

    #[test]
    fn test_se_bin_caching_roundtrip() {
        let temp_dir = std::env::temp_dir().join("ragasukma_test_se_bin");
        let temp_file = temp_dir.join("test.se.bin");
        let fake_se = (0..256).map(|i| (i as f32) * 0.01).collect::<Vec<f32>>();

        save_se_to_disk(&temp_file, &fake_se);
        let loaded = load_se_from_disk(&temp_file, None);
        assert!(loaded.is_some());
        let loaded_se = loaded.unwrap();
        assert_eq!(loaded_se.len(), 256);
        assert_eq!(loaded_se, fake_se);
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_ref_audio_duration_capping() {
        let max_samples = (22050.0 * MAX_REF_AUDIO_DURATION_SECS).round() as usize;
        let long_samples = vec![0.1f32; 22050 * 30]; // 30 seconds
        let capped = if long_samples.len() > max_samples {
            &long_samples[..max_samples]
        } else {
            &long_samples[..]
        };
        assert_eq!(capped.len(), max_samples);
    }
}
