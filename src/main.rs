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

fn main() -> Result<()> {
    // Jalankan seluruh aplikasi di dalam Dedicated Primary Thread berkapasitas 64 MB Stack
    // untuk mengeliminasi sepenuhnya limitasi stack bawaan Linux AppImage / FUSE runtime.
    let main_thread = std::thread::Builder::new()
        .name("ragasukma-main".to_string())
        .stack_size(64 * 1024 * 1024) // 64 MB Heap-allocated stack
        .spawn(|| -> Result<()> {
            run_ragasukma_app()
        })
        .expect("Gagal membuat main application thread");

    match main_thread.join() {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ Ragasukma thread panicked: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn run_ragasukma_app() -> Result<()> {
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

    // Salin otomatis semua aset default ke folder config pengguna (~/.config/ragasukma/assets)
    server::ensure_default_user_assets();

    let config = AppConfig::load();
    let config_clone = config.clone();

    crate::term_log!("====================================================");
    crate::term_log!("  Ragasukma - Asisten Pribadi: {} (Anime ID)", config.assistant_name);
    crate::term_log!("====================================================");

    // Perbesar stack size session thread agar aman di AppImage/portable Linux
    let session_thread = std::thread::Builder::new()
        .name("ragasukma-session".to_string())
        .stack_size(16 * 1024 * 1024) // 16 MB stack
        .spawn(move || {
            if let Ok(mut session) = AppSession::new(config_clone) {
                let _ = session.run_interactive_menu();
            }
        })
        .expect("Gagal membuat session thread");

    // Jalankan 3D VRM Native Overlay
    native::run_native_overlay();

    let _ = session_thread.join();
    std::process::exit(0);
}
