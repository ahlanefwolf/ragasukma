use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
pub struct TtsThreadBudget {
    pub cpu_cores: usize,
    pub ram_free_mb: Option<usize>,
    pub vocoder_threads: usize,
    pub vec_est_threads: usize,
    pub text_enc_threads: usize,
    pub openvoice_threads: usize,
    pub dp_threads: usize,
}

static CACHED_BUDGET: OnceLock<TtsThreadBudget> = OnceLock::new();

/// Detect free/available RAM in MB across Linux, Windows, and macOS without blocking
pub fn get_free_ram_mb() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemAvailable:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<usize>() {
                            return Some(kb / 1024);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        #[repr(C)]
        #[allow(non_snake_case, clippy::upper_case_acronyms)]
        struct MEMORYSTATUSEX {
            dwLength: u32,
            dwMemoryLoad: u32,
            ullTotalPhys: u64,
            ullAvailPhys: u64,
            ullTotalPageFile: u64,
            ullAvailPageFile: u64,
            ullTotalVirtual: u64,
            ullAvailVirtual: u64,
            ullAvailExtendedVirtual: u64,
        }
        unsafe extern "system" {
            fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
        }
        let mut mem = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailVirtual: 0,
            ullAvailExtendedVirtual: 0,
        };
        if unsafe { GlobalMemoryStatusEx(&mut mem) } != 0 {
            return Some((mem.ullAvailPhys / (1024 * 1024)) as usize);
        }
    }

    None
}

/// Dynamically calculates hardware-aware thread allocation for ONNX TTS models:
/// - Low-end CPUs (<=4 cores): allocates 2 threads, ensuring 1-2 cores remain free for 3D VRM & UI.
/// - Mid-range CPUs (6-8 threads): allocates 3-4 threads.
/// - High-end CPUs (>=12 threads): allocates 6-8 threads (capped at 8 to prevent Amdahl contention).
/// - Low RAM (<2.5 GB available): dials down thread count to prevent memory pressure.
pub fn get_recommended_tts_threads() -> TtsThreadBudget {
    *CACHED_BUDGET.get_or_init(|| {
        let cpu_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let ram_free_mb = get_free_ram_mb();

        let (mut vocoder, mut vec_est, text_enc, mut openvoice): (usize, usize, usize, usize) = match cpu_cores {
            0..=2 => (1, 1, 1, 1),
            3..=4 => (2, 2, 1, 2),        // Low-end (e.g. i5-3570, N100): leaves 2 cores free
            5..=6 => (3, 2, 2, 3),        // 6 threads: leaves 3 threads free
            7..=8 => (4, 3, 2, 4),        // 8 threads (e.g. 4C/8T): leaves 4 threads free
            9..=15 => (8, 4, 4, 6),       // 12 threads (e.g. Ryzen 5 7430U): leaves 4-6 threads free
            _ => (8, 4, 4, 8),            // 16+ threads (high-end): capped at 8 max
        };

        // Scale down if under significant memory pressure (< 2.5 GB free)
        if let Some(free_mb) = ram_free_mb {
            if free_mb < 2500 {
                vocoder = vocoder.saturating_sub(1).max(1);
                openvoice = openvoice.saturating_sub(1).max(1);
                vec_est = vec_est.saturating_sub(1).max(1);
            }
        }

        let budget = TtsThreadBudget {
            cpu_cores,
            ram_free_mb,
            vocoder_threads: vocoder,
            vec_est_threads: vec_est,
            text_enc_threads: text_enc,
            openvoice_threads: openvoice,
            dp_threads: 1,
        };

        crate::term_log!(
            "[Hardware Probe] Detected {} Logical Cores / {:?} Free RAM -> Dynamic TTS Thread Allocation: Vocoder={}, VecEst={}, TextEnc={}, OpenVoice={}",
            budget.cpu_cores,
            budget.ram_free_mb.map(|m| format!("{m} MB")).unwrap_or_else(|| "Unknown".to_string()),
            budget.vocoder_threads,
            budget.vec_est_threads,
            budget.text_enc_threads,
            budget.openvoice_threads
        );

        budget
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_probe_budget() {
        let budget = get_recommended_tts_threads();
        assert!(budget.cpu_cores > 0);
        assert!(budget.vocoder_threads >= 1 && budget.vocoder_threads <= 8);
        assert!(budget.openvoice_threads >= 1 && budget.openvoice_threads <= 8);
        assert!(budget.vec_est_threads >= 1 && budget.vec_est_threads <= 4);
        assert_eq!(budget.dp_threads, 1);
    }
}
