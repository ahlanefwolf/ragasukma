pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Pipeline Noise Reduction (Modular DSP: DC Offset + High-pass + Dynamic Noise Gate)
#[allow(dead_code)]
pub fn apply_noise_reduction(samples: &mut [f32], sample_rate: u32, nr_level: u8) {
    if samples.is_empty() {
        return;
    }

    if nr_level == 0 {
        remove_dc_offset(samples);
        return;
    }

    // 1. Hapus DC Offset
    remove_dc_offset(samples);

    // 2. High-Pass Filter Natural (~120Hz untuk level 1, ~150Hz untuk level 2+)
    let cutoff_hz = match nr_level {
        1 => 120.0f32,
        2 => 150.0f32,
        _ => 180.0f32,
    };
    apply_high_pass_filter(
        samples,
        sample_rate,
        cutoff_hz,
        if nr_level == 1 { 1 } else { 2 },
    );

    // 3. Dynamic Smooth Noise Gate
    apply_smooth_noise_gate(samples, sample_rate, nr_level);
}

/// Radix-2 FFT & IFFT (Fast Fourier Transform) murni
#[allow(dead_code)]
fn fft_radix2(re: &mut [f32], im: &mut [f32], inverse: bool) {
    let n = re.len();
    if n <= 1 {
        return;
    }

    let mut j = 0;
    for i in 0..n {
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
        let mut k = n >> 1;
        while k <= j && k > 0 {
            j -= k;
            k >>= 1;
        }
        j += k;
    }

    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = if inverse {
            2.0 * std::f32::consts::PI / len as f32
        } else {
            -2.0 * std::f32::consts::PI / len as f32
        };
        let w_step_re = angle.cos();
        let w_step_im = angle.sin();

        let mut i = 0;
        while i < n {
            let mut w_re = 1.0f32;
            let mut w_im = 0.0f32;
            for k in 0..half {
                let u_re = re[i + k];
                let u_im = im[i + k];
                let v_re = re[i + k + half] * w_re - im[i + k + half] * w_im;
                let v_im = re[i + k + half] * w_im + im[i + k + half] * w_re;

                re[i + k] = u_re + v_re;
                im[i + k] = u_im + v_im;
                re[i + k + half] = u_re - v_re;
                im[i + k + half] = u_im - v_im;

                let next_w_re = w_re * w_step_re - w_im * w_step_im;
                let next_w_im = w_re * w_step_im + w_im * w_step_re;
                w_re = next_w_re;
                w_im = next_w_im;
            }
            i += len;
        }
        len <<= 1;
    }

    if inverse {
        let inv_n = 1.0 / n as f32;
        for i in 0..n {
            re[i] *= inv_n;
            im[i] *= inv_n;
        }
    }
}

/// Spectral Subtraction Stationary Noise Suppression (Windows Voice Clarity / DMO style)
/// Mengurangi desis ruangan, suara kipas PC, dan kebisingan konstan di setiap frekuensi bahkan saat pengguna sedang berbicara.
#[allow(dead_code)]
pub fn apply_spectral_noise_suppression(samples: &mut [f32], nr_strength: f32) {
    let n_fft = 512;
    let hop = 256;
    if samples.len() < n_fft {
        return;
    }

    // 1. Hann window
    let mut window = Vec::with_capacity(n_fft);
    for i in 0..n_fft {
        let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / n_fft as f32).cos());
        window.push(w);
    }

    // 2. Kumpulkan seluruh frame untuk estimasi noise profile
    let mut frame_start = 0;
    let mut frames_mag = Vec::new();
    let mut frames_energy = Vec::new();

    while frame_start + n_fft <= samples.len() {
        let mut re = vec![0.0f32; n_fft];
        let mut im = vec![0.0f32; n_fft];

        for i in 0..n_fft {
            re[i] = samples[frame_start + i] * window[i];
        }

        fft_radix2(&mut re, &mut im, false);

        let mut mag = Vec::with_capacity(n_fft / 2 + 1);
        let mut energy = 0.0f32;
        for i in 0..=(n_fft / 2) {
            let m = (re[i] * re[i] + im[i] * im[i]).sqrt();
            mag.push(m);
            energy += m * m;
        }

        frames_mag.push((re, im, mag));
        frames_energy.push(energy);
        frame_start += hop;
    }

    if frames_mag.is_empty() {
        return;
    }

    // 3. Estimasi noise spectrum dari 25% frame paling senyap
    let mut sorted_indices: Vec<usize> = (0..frames_energy.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        frames_energy[a]
            .partial_cmp(&frames_energy[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let noise_frame_count = (frames_energy.len() / 4).max(1);
    let mut noise_spectrum = vec![0.0f32; n_fft / 2 + 1];

    for &idx in &sorted_indices[..noise_frame_count] {
        let (_, _, ref mag) = frames_mag[idx];
        for (bin, &m) in mag.iter().enumerate() {
            noise_spectrum[bin] += m;
        }
    }
    for val in noise_spectrum.iter_mut() {
        *val /= noise_frame_count as f32;
    }

    // 4. Spectral Subtraction & Overlap-Add Reconstruct
    let mut output = vec![0.0f32; samples.len()];
    let mut weight = vec![0.0f32; samples.len()];

    let alpha = (2.2 * nr_strength).clamp(1.2, 4.0); // Over-subtraction factor
    let beta = 0.02f32; // Spectral floor (-34 dB)

    for (f_idx, (mut re, mut im, mag)) in frames_mag.into_iter().enumerate() {
        let start = f_idx * hop;

        for bin in 0..=(n_fft / 2) {
            let orig_m = mag[bin];
            let noise_m = noise_spectrum[bin];
            let sub_m = (orig_m - alpha * noise_m).max(beta * orig_m);
            let gain = if orig_m > 1e-6 {
                (sub_m / orig_m).clamp(beta, 1.0)
            } else {
                beta
            };

            re[bin] *= gain;
            im[bin] *= gain;

            if bin > 0 && bin < n_fft / 2 {
                let sym_bin = n_fft - bin;
                re[sym_bin] *= gain;
                im[sym_bin] *= gain;
            }
        }

        fft_radix2(&mut re, &mut im, true);

        for i in 0..n_fft {
            output[start + i] += re[i] * window[i];
            weight[start + i] += window[i] * window[i];
        }
    }

    for (i, s) in samples.iter_mut().enumerate() {
        if weight[i] > 1e-4 {
            *s = output[i] / weight[i];
        }
    }
}

/// Menghapus DC Offset pada sinyal audio
pub fn remove_dc_offset(samples: &mut [f32]) {
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    for s in samples.iter_mut() {
        *s -= mean;
    }
}

/// High-Pass Filter IIR yang aman & natural
pub fn apply_high_pass_filter(
    samples: &mut [f32],
    sample_rate: u32,
    cutoff_hz: f32,
    passes: usize,
) {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let dt = 1.0 / sample_rate as f32;
    let alpha = rc / (rc + dt);

    for _ in 0..passes {
        let mut prev_in = samples[0];
        let mut prev_out = 0.0f32;
        for s in samples.iter_mut() {
            let cur_in = *s;
            let cur_out = alpha * (prev_out + cur_in - prev_in);
            prev_in = cur_in;
            prev_out = cur_out;
            *s = cur_out;
        }
    }
}

/// Low-Pass Filter IIR untuk meredam desis frekuensi tinggi (thermal noise DAC/preamp)
#[allow(dead_code)]
pub fn apply_low_pass_filter(
    samples: &mut [f32],
    sample_rate: u32,
    cutoff_hz: f32,
    passes: usize,
) {
    if samples.is_empty() {
        return;
    }
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);

    for _ in 0..passes {
        let mut prev_out = samples[0];
        for s in samples.iter_mut() {
            let cur_out = prev_out + alpha * (*s - prev_out);
            prev_out = cur_out;
            *s = cur_out;
        }
    }
}

/// Dynamic Downward Expander Noise Gate untuk menekan desis tanpa memotong huruf awal/akhir kata
pub fn apply_smooth_noise_gate(samples: &mut [f32], sample_rate: u32, nr_level: u8) {
    let frame_size = (sample_rate as usize / 100).max(160); // 10ms frame
    let mut frame_energies = Vec::new();

    for chunk in samples.chunks(frame_size) {
        let rms = (chunk.iter().map(|&x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
        frame_energies.push(rms);
    }

    if frame_energies.is_empty() {
        return;
    }

    let mut sorted_energies = frame_energies.clone();
    sorted_energies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let noise_floor_idx = (sorted_energies.len() as f32 * 0.20) as usize;
    let noise_floor = sorted_energies[noise_floor_idx.min(sorted_energies.len() - 1)];

    let gate_threshold = match nr_level {
        0 => (noise_floor * 1.5).max(0.004),
        1 => (noise_floor * 1.9).max(0.006),
        2 => (noise_floor * 2.3).max(0.009),
        _ => (noise_floor * 2.8).max(0.013),
    };

    let silence_floor_gain = match nr_level {
        0 => 0.15f32,
        1 => 0.05f32,
        2 => 0.015f32,
        _ => 0.005f32,
    };

    let mut gain = 1.0f32;
    let attack = 0.88f32; // Cepat membuka saat vokal masuk
    let release = 0.12f32; // Halus saat menutup

    for (chunk_idx, chunk) in samples.chunks_mut(frame_size).enumerate() {
        let current_rms = frame_energies[chunk_idx];
        let target_gain = if current_rms < gate_threshold {
            let ratio = (current_rms / gate_threshold).clamp(0.0, 1.0);
            silence_floor_gain + (1.0 - silence_floor_gain) * ratio.powf(2.0)
        } else {
            1.0f32
        };

        if target_gain > gain {
            gain = gain * (1.0 - attack) + target_gain * attack;
        } else {
            gain = gain * (1.0 - release) + target_gain * release;
        }

        for s in chunk.iter_mut() {
            *s *= gain;
        }
    }
}

/// Normalisasi volume puncak (Peak Normalization)
pub fn normalize_volume(samples: &mut [f32], target_amp: f32, max_gain: f32) {
    let max_amp = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0f32, |a, b| a.max(b));

    if max_amp > 0.001 {
        let gain = (target_amp / max_amp).min(max_gain);
        for s in samples.iter_mut() {
            *s *= gain;
        }
    }
}

/// AGC / Dynamic Speech Compressor yang meniru karakteristik Windows APO Voice Processing:
/// Meratakan vokal yang pelan/jauh tanpa menimbulkan clipping saat berbicara keras/dekat.
#[allow(dead_code)]
pub fn apply_speech_agc(samples: &mut [f32], target_rms: f32) {
    if samples.is_empty() {
        return;
    }

    // Evaluasi RMS frame aktif (hanya mengukur energi saat ada vokal, bukan saat hening)
    let chunk_size = 160;
    let mut speech_sum_sq = 0.0f32;
    let mut speech_samples_count = 0usize;

    for chunk in samples.chunks(chunk_size) {
        let chunk_rms = (chunk.iter().map(|&x| x * x).sum::<f32>() / chunk.len() as f32).sqrt();
        if chunk_rms > 0.008 {
            speech_sum_sq += chunk.iter().map(|&x| x * x).sum::<f32>();
            speech_samples_count += chunk.len();
        }
    }

    let speech_rms = if speech_samples_count > 0 {
        (speech_sum_sq / speech_samples_count as f32).sqrt()
    } else {
        (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
    };

    if speech_rms < 0.005 {
        return; // Jangan angkat hening / noise murni
    }

    let target_gain = (target_rms / speech_rms).clamp(0.7, 2.2);
    for s in samples.iter_mut() {
        let val = *s * target_gain;
        // Soft saturation untuk menjaga suara tetap natural dan bebas distorsi
        *s = if val > 0.95 {
            0.95 + (val - 0.95) * 0.1
        } else if val < -0.95 {
            -0.95 + (val + 0.95) * 0.1
        } else {
            val
        };
    }
}
