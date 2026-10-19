use anyhow::{Result, anyhow};
use ort::ep::CPU;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub struct OpenVoiceEngine {
    ref_enc_sess: Mutex<Session>,
    tone_conv_sess: Mutex<Session>,
    ref_se_cache: Mutex<Option<(String, Vec<f32>)>>,
}

static OPENVOICE_ENGINE: OnceLock<Option<OpenVoiceEngine>> = OnceLock::new();

fn find_openvoice_model_dir() -> Option<PathBuf> {
    let mut candidates = vec![
        PathBuf::from("models/openvoice-v2"),
        PathBuf::from("../models/openvoice-v2"),
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

    let available_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let intra_threads = available_threads.min(4);

    crate::term_log!(
        "[OpenVoice Engine] Loading ONNX models from {:?} (threads: {})",
        model_dir,
        intra_threads
    );

    let ref_enc_sess = Session::builder()
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
        .commit_from_file(model_dir.join("openvoice_ref_encoder.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

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
        ref_enc_sess: Mutex::new(ref_enc_sess),
        tone_conv_sess: Mutex::new(tone_conv_sess),
        ref_se_cache: Mutex::new(None),
    })
}

/// Cari file audio referensi standar di folder assets (`assets/refvoice.*`)
pub fn find_ref_voice_path() -> Option<PathBuf> {
    let assets_dir = crate::assets::get_assets_dir();
    let supported_exts = ["wav", "mp3", "ogg", "flac", "m4a", "aac"];
    for ext in &supported_exts {
        let p = assets_dir.join(format!("refvoice.{ext}"));
        if p.exists() && p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Reset in-memory cache jika user mengupload audio referensi baru
pub fn invalidate_ref_cache() {
    if let Some(engine) = get_engine() {
        if let Ok(mut lock) = engine.ref_se_cache.lock() {
            *lock = None;
        }
    }
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

        // Compute magnitude for positive frequencies [0..513]
        for k in 0..num_bins {
            let r = real_buf[k];
            let im = imag_buf[k];
            let magnitude = (r * r + im * im).sqrt();
            spec_data[k * n_frames + frame_idx] = magnitude;
        }
    }

    (spec_data, n_frames)
}

/// Decode any supported audio file on disk to 22.05kHz mono f32 samples
pub fn load_audio_file_22k(file_path: &Path) -> Result<Vec<f32>> {
    let file = std::fs::File::open(file_path)?;
    let cursor = std::io::BufReader::new(file);
    let source = rodio::Decoder::new(cursor).map_err(|e| anyhow!("Failed to decode audio: {e}"))?;

    use rodio::Source;
    let channels = source.channels() as usize;
    let sample_rate = source.sample_rate();
    let samples_raw: Vec<f32> = source.convert_samples::<f32>().collect();

    // Convert to mono
    let mut mono = Vec::with_capacity(samples_raw.len() / channels);
    for chunk in samples_raw.chunks(channels) {
        let sum: f32 = chunk.iter().sum();
        mono.push(sum / channels as f32);
    }

    // Resample to 22050 Hz
    let resampled = resample_linear(&mono, sample_rate, 22050);
    Ok(resampled)
}

impl OpenVoiceEngine {
    /// Extract Speaker Embedding `g` [1, 256, 1] from 22.05kHz mono samples
    pub fn extract_se_from_samples(&self, samples_22k: &[f32]) -> Result<Vec<f32>> {
        let (spec, n_frames) = compute_linear_spectrogram(samples_22k, 1024, 256, 1024);

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
        let mut ref_sess = self.ref_enc_sess.lock().map_err(|e| anyhow!("{e}"))?;
        let outputs = ref_sess.run(inputs)?;

        let (out_shape, out_slice) = outputs["g"].try_extract_tensor::<f32>()?;

        if out_shape.as_ref() != [1, 256, 1] {
            return Err(anyhow!("Unexpected ref encoder output shape: {:?}", out_shape));
        }

        Ok(out_slice.to_vec())
    }

    /// Ambil target SE dari file referensi disk (dengan in-memory cache)
    pub fn get_target_se(&self, ref_path: &Path) -> Result<Vec<f32>> {
        let path_str = ref_path.to_string_lossy().to_string();
        if let Ok(lock) = self.ref_se_cache.lock() {
            if let Some((ref cached_path, ref se)) = *lock {
                if cached_path == &path_str {
                    return Ok(se.clone());
                }
            }
        }

        crate::term_log!("[OpenVoice Engine] Extracting speaker embedding from {:?}", ref_path);
        let samples = load_audio_file_22k(ref_path)?;
        let se = self.extract_se_from_samples(&samples)?;

        if let Ok(mut lock) = self.ref_se_cache.lock() {
            *lock = Some((path_str, se.clone()));
        }
        Ok(se)
    }

    /// Mengonversi timbre audio WAV (44.1k/24k) ke target voice reference dengan embedding target yang sudah diekstrak
    pub fn convert_voice_with_target_se(
        &self,
        src_samples: &[f32],
        src_sample_rate: u32,
        sid_tgt: &[f32],
        tau: f32,
    ) -> Result<Vec<f32>> {
        // 1. Resample source audio ke 22.05kHz
        let src_22k = resample_linear(src_samples, src_sample_rate, 22050);

        // 2. Extract Source SE
        let sid_src = self.extract_se_from_samples(&src_22k)?;

        // 3. Compute Linear Spectrogram untuk Tone Converter
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

    /// Mengonversi timbre audio WAV (44.1k/24k) ke target voice reference
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
        }
    }
}
