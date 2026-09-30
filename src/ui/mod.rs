pub mod app;
pub mod pose_system;
pub mod renderer;
pub mod spring_bone;
pub mod vrm_loader;

use winit::event::Event;
use winit::event_loop::EventLoopBuilder;

#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

pub fn run_native_overlay() {
    println!("✨ Starting Ragasukma 3D Native Overlay (Ultra Lightweight WGPU)...");

    let event_loop = {
        #[cfg(target_os = "linux")]
        {
            // Coba bangun event loop standar (Wayland jika tersedia), jika Wayland NoCompositor, fallback otomatis ke X11
            EventLoopBuilder::new()
                .build()
                .or_else(|_| {
                    println!("⚠️ [Wayland] Not detected or connection refused, falling back to X11 backend...");
                    let mut b = EventLoopBuilder::new();
                    b.with_x11().build()
                })
                .expect("Failed to create winit event loop (Wayland & X11 fallback)")
        }
        #[cfg(not(target_os = "linux"))]
        {
            EventLoopBuilder::new()
                .build()
                .expect("Failed to create winit event loop")
        }
    };

    let mut app = match pollster::block_on(app::App::new(&event_loop)) {
        Ok(a) => Box::new(a),
        Err(e) => {
            eprintln!("❌ Failed to initialize Native App: {:?}", e);
            return;
        }
    };

    let target_frame_duration = std::time::Duration::from_millis(16); // ~60 FPS
    let mut next_frame_time = std::time::Instant::now();

    let _ = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent { event, .. } => {
            app.handle_window_event(event, elwt);
        }
        Event::AboutToWait => {
            let now = std::time::Instant::now();
            if now >= next_frame_time {
                next_frame_time = now + target_frame_duration;
                app.window.request_redraw();
            }
            elwt.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(next_frame_time));
        }
        _ => {}
    });
}
