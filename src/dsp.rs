pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Pipeline Noise Reduction (Modular DSP: DC Offset + High-pass + Dynamic Noise Gate)
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
    apply_high_pass_filter(samples, sample_rate, cutoff_hz, if nr_level == 1 { 1 } else { 2 });

    // 3. Dynamic Smooth Noise Gate
    apply_smooth_noise_gate(samples, sample_rate, nr_level);
}

/// Menghapus DC Offset pada sinyal audio
pub fn remove_dc_offset(samples: &mut [f32]) {
    let mean: f32 = samples.iter().sum::<f32>() / samples.len() as f32;
    for s in samples.iter_mut() {
        *s -= mean;
    }
}

/// High-Pass Filter IIR yang aman & natural
pub fn apply_high_pass_filter(samples: &mut [f32], sample_rate: u32, cutoff_hz: f32, passes: usize) {
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

/// Dynamic Smooth Noise Gate untuk menekan desis tanpa memotong huruf awal/akhir kata
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
    let noise_floor_idx = (sorted_energies.len() as f32 * 0.25) as usize;
    let noise_floor = sorted_energies[noise_floor_idx.min(sorted_energies.len() - 1)];

    let gate_threshold = match nr_level {
        1 => (noise_floor * 1.6).max(0.005),
        2 => (noise_floor * 2.0).max(0.008),
        _ => (noise_floor * 2.5).max(0.012),
    };

    let silence_floor_gain = match nr_level {
        1 => 0.35f32,
        2 => 0.20f32,
        _ => 0.08f32,
    };

    let mut gain = 1.0f32;
    let attack = 0.85f32;  // Cepat membuka saat vokal masuk
    let release = 0.15f32; // Halus saat menutup

    for (chunk_idx, chunk) in samples.chunks_mut(frame_size).enumerate() {
        let current_rms = frame_energies[chunk_idx];
        let target_gain = if current_rms < gate_threshold {
            silence_floor_gain
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
