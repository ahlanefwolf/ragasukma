mod audio;
mod config;
mod dsp;
mod llm;
mod menu;
pub mod webview;
mod overlay;
pub mod skills;
mod stt;
mod tray;
mod tts;

use anyhow::Result;
use config::AppConfig;
use menu::AppSession;
use overlay::NativeOverlay;

fn main() -> Result<()> {
    let config = AppConfig::load();
    let config_clone = config.clone();

    println!("====================================================");
    println!("  Ragasukma - Asisten Pribadi: {} (Anime ID)", config.assistant_name);
    println!("====================================================");

    // Jalankan interactive session & audio engine di background thread
    let session_thread = std::thread::spawn(move || {
        if let Ok(mut session) = AppSession::new(config_clone) {
            let _ = session.run_interactive_menu();
            // Jika user keluar dari menu terminal, sinyalkan tauri untuk keluar
            crate::tray::TrayState::request_quit();
        }
    });

    // Jalankan GUI Overlay Tauri di main thread
    NativeOverlay::run();

    let _ = session_thread.join();
    std::process::exit(0);
}
