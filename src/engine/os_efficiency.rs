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

    // If 10 seconds of idle have passed since last activity, trigger OS memory cleanup & efficiency trim
    if now.saturating_sub(last) >= 10 {
        // Reset last activity to 0 so we only trim once per idle window
        LAST_ACTIVITY_TIME.store(0, Ordering::Relaxed);
        trim_process_memory();
    }
}

/// Trims idle working set / physical memory across Windows, macOS, and Linux.
pub fn trim_process_memory() {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
            use windows::Win32::System::Threading::GetCurrentProcess;

            let handle = GetCurrentProcess();
            let _ = EmptyWorkingSet(handle);
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Tell allocator / libc to release free memory back to OS
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
