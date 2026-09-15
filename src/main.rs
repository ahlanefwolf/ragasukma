// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod config;
pub mod engine;
pub mod engine_browser;
mod llm;
pub mod native;
pub mod server;
mod tray;

use anyhow::Result;
use config::AppConfig;
use engine::AppSession;

fn main() -> Result<()> {
    // Pastikan stack size default aman untuk lingkungan Linux AppImage / Embedded
    if std::env::var("RUST_MIN_STACK").is_err() {
        unsafe {
            std::env::set_var("RUST_MIN_STACK", "16777216"); // 16 MB
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
