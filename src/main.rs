// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
mod dsp;
mod llm;
mod menu;
pub mod native;
pub mod skills;
mod stt;
mod tray;
mod tts;

use anyhow::Result;
use config::AppConfig;
use menu::AppSession;

fn main() -> Result<()> {
    let config = AppConfig::load();
    let config_clone = config.clone();

    crate::term_log!("====================================================");
    crate::term_log!("  Ragasukma - Asisten Pribadi: {} (Anime ID)", config.assistant_name);
    crate::term_log!("====================================================");

    // Jalankan interactive session & audio engine di background thread
    let session_thread = std::thread::spawn(move || {
        if let Ok(mut session) = AppSession::new(config_clone) {
            let _ = session.run_interactive_menu();
        }
    });

    // Jalankan 3D VRM Native Overlay di main thread (winit EventLoop memerlukan main thread)
    native::run_native_overlay();

    let _ = session_thread.join();
    std::process::exit(0);
}
