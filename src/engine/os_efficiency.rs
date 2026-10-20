use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

static LAST_ACTIVITY_TIME: AtomicU64 = AtomicU64::new(0);
static TIMER_INIT: OnceLock<()> = OnceLock::new();

/// Notify the efficiency subsystem that the audio/AI engine was active just now.
pub fn notify_active() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    LAST_ACTIVITY_TIME.store(now, Ordering::Relaxed);
    ensure_timer_started();
}

/// Start background efficiency watcher thread if not already running.
pub fn ensure_timer_started() {
    TIMER_INIT.get_or_init(|| {
        std::thread::Builder::new()
            .name("os_efficiency_worker".to_string())
            .stack_size(512 * 1024)
            .spawn(|| {
                loop {
                    std::thread::sleep(Duration::from_secs(5));
                    check_and_apply_efficiency();
                }
            })
            .expect("Failed to spawn OS efficiency worker thread");
    });
}

fn check_and_apply_efficiency() {
    let last = LAST_ACTIVITY_TIME.load(Ordering::Relaxed);
    if last == 0 {
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 2-tier cleanup:
    // Tier 1 (10s): Soft trim (mimalloc collect / malloc_trim)
    // Tier 2 (300s / 5 mins): Deep OS physical trim (EmptyWorkingSet on Windows)
    let idle_secs = now.saturating_sub(last);

    if idle_secs >= 300 {
        LAST_ACTIVITY_TIME.store(0, Ordering::Relaxed);
        deep_trim_process_memory();
    } else if (10..15).contains(&idle_secs) {
        trim_process_memory();
    }
}

/// Trims idle working set / physical memory across Windows, macOS, and Linux.
pub fn trim_process_memory() {
    #[cfg(target_os = "windows")]
    {
        // Tell mimalloc to forcefully purge freed heap pages back to Windows OS
        // without discarding resident ONNX model weights from physical RAM.
        unsafe extern "C" {
            fn mi_collect(force: bool);
        }
        unsafe {
            mi_collect(true);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Tell mimalloc to forcefully purge freed heap pages back to Linux OS (MADV_DONTNEED / munmap)
        unsafe extern "C" {
            fn mi_collect(force: bool);
        }
        unsafe {
            mi_collect(true);
        }

        // Also tell glibc ptmalloc to release any C-library allocated free memory
        unsafe extern "C" {
            fn malloc_trim(pad: usize) -> i32;
        }
        unsafe {
            malloc_trim(0);
        }
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, mimalloc will purge free pages back to the kernel.
        // Darwin also automatically compresses or pages out unreferenced physical pages.
    }
}

/// Deep trim: forcefully releases physical working set back to OS kernel (after ~5 mins true idle).
pub fn deep_trim_process_memory() {
    trim_process_memory();

    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
        use windows::Win32::System::Threading::GetCurrentProcess;

        unsafe {
            let _ = EmptyWorkingSet(GetCurrentProcess());
        }
    }
}
