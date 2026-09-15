pub mod app;
pub mod pose_system;
pub mod renderer;
pub mod server;
pub mod vrm_loader;

use winit::{
    event::Event,
    event_loop::EventLoop,
};

pub fn run_native_overlay() {
    println!("✨ Memulai Ragasukma 3D Native Overlay (Ultra Lightweight WGPU)...");

    let event_loop = EventLoop::new().expect("Gagal membuat winit event loop");
    let mut app = Box::new(pollster::block_on(app::App::new(&event_loop)).expect("Gagal menginisialisasi Native App"));

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
