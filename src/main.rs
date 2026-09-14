// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod engine;
mod llm;
pub mod native;
pub mod skills;
mod tray;

use anyhow::Result;
use config::AppConfig;
use engine::AppSession;

fn main() -> Result<()> {
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
