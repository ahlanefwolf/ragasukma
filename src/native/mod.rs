pub mod app;
pub mod pose_system;
pub mod renderer;
pub mod vrm_loader;

use winit::event::Event;
use winit::event_loop::EventLoopBuilder;

#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

pub fn run_native_overlay() {
    println!("✨ Memulai Ragasukma 3D Native Overlay (Ultra Lightweight WGPU)...");

    let event_loop = {
        #[cfg(target_os = "linux")]
        {
            // Coba bangun event loop standar (Wayland jika tersedia), jika Wayland NoCompositor, fallback otomatis ke X11
            EventLoopBuilderExtX11::with_any_thread(&mut EventLoopBuilder::new(), true)
                .build()
                .or_else(|_| {
                    println!("⚠️ [Wayland] Tidak terdeteksi atau koneksi ditolak, fallback ke X11 backend...");
                    let mut b = EventLoopBuilder::new();
                    EventLoopBuilderExtX11::with_any_thread(&mut b, true);
                    b.with_x11().build()
                })
                .expect("Gagal membuat winit event loop (Wayland & X11 fallback)")
        }
        #[cfg(not(target_os = "linux"))]
        {
            EventLoopBuilder::new()
                .build()
                .expect("Gagal membuat winit event loop")
        }
    };

    let mut app = match pollster::block_on(app::App::new(&event_loop)) {
        Ok(a) => Box::new(a),
        Err(e) => {
            eprintln!("❌ Gagal menginisialisasi Native App: {:?}", e);
            return;
        }
    };

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => {
                app.handle_window_event(event, elwt);
            }
            Event::AboutToWait => {
                app.window.request_redraw();
            }
            _ => {}
        }
    }).expect("Event loop error");
}
