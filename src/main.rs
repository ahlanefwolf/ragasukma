// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::collapsible_if)]

mod audio;
mod config;
pub mod engine;

mod llm;
pub mod native;
pub mod server;
mod tray;

use anyhow::Result;
use config::AppConfig;
use engine::AppSession;

#[cfg(unix)]
mod stack_fix {
    #[repr(C)]
    struct Rlimit {
        rlim_cur: u64,
        rlim_max: u64,
    }

    const RLIMIT_STACK: i32 = 3;

    unsafe extern "C" {
        fn getrlimit(resource: i32, rlim: *mut Rlimit) -> i32;
        fn setrlimit(resource: i32, rlim: *const Rlimit) -> i32;
    }

    pub fn ensure_adequate_stack_size() {
        unsafe {
            let mut rlim = Rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if getrlimit(RLIMIT_STACK, &mut rlim) == 0 {
                let target_stack = 32 * 1024 * 1024; // 32 MB
                if rlim.rlim_cur < target_stack {
                    rlim.rlim_cur = target_stack;
                    if rlim.rlim_max < target_stack && rlim.rlim_max != u64::MAX {
                        rlim.rlim_max = target_stack;
                    }
                    let _ = setrlimit(RLIMIT_STACK, &rlim);
                }
            }
        }
    }
}

fn main() -> Result<()> {
    #[cfg(unix)]
    stack_fix::ensure_adequate_stack_size();

    // Pastikan stack size default aman untuk lingkungan Linux AppImage / Embedded
    if std::env::var("RUST_MIN_STACK").is_err() {
        unsafe {
            std::env::set_var("RUST_MIN_STACK", "33554432"); // 32 MB
        }
    }

    // 🔒 Single Instance Check: Cegah aplikasi dibuka ganda
    let server_port = 42128;
    let single_instance_check = std::net::TcpListener::bind(("127.0.0.1", server_port));
    if single_instance_check.is_err() {
        println!("⚠️  [Single Instance] Ragasukma sudah berjalan di latar belakang!");
        println!("🌐 Membuka Dashboard yang sedang aktif di http://127.0.0.1:{} ...", server_port);
        server::open_browser(&format!("http://127.0.0.1:{}", server_port));
        std::process::exit(0);
    }
    // Drop listener pengecekan agar server utama di server.rs bisa bind dengan lancar
    drop(single_instance_check);

    let config = AppConfig::load();
    let config_clone = config.clone();

    crate::term_log!("====================================================");
    crate::term_log!("  Ragasukma - Asisten Pribadi: {} (Anime ID)", config.assistant_name);
    crate::term_log!("====================================================");

    // Perbesar stack size session thread agar aman di AppImage/portable Linux
    let session_thread = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024) // 8 MB stack
        .spawn(move || {
            if let Ok(mut session) = AppSession::new(config_clone) {
                let _ = session.run_interactive_menu();
            }
        })
        .expect("Gagal membuat session thread");

    // Jalankan 3D VRM Native Overlay di main thread (winit EventLoop memerlukan main thread)
    native::run_native_overlay();

    let _ = session_thread.join();
    std::process::exit(0);
}
