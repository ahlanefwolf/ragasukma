use image::{DynamicImage, RgbaImage};
use xcap::{Monitor, Window};

#[cfg(target_os = "linux")]
fn capture_screen_image_linux_fallback() -> Result<DynamicImage, String> {
    // 1. Coba grim (standar emas Wayland: Sway, Hyprland, wlroots)
    if let Ok(output) = std::process::Command::new("grim").arg("-").output() {
        if output.status.success() && !output.stdout.is_empty() {
            if let Ok(img) = image::load_from_memory(&output.stdout) {
                return Ok(img);
            }
        }
    }

    // 2. Coba wayshot (Wayland alternatif)
    if let Ok(output) = std::process::Command::new("wayshot").args(["--stdout"]).output() {
        if output.status.success() && !output.stdout.is_empty() {
            if let Ok(img) = image::load_from_memory(&output.stdout) {
                return Ok(img);
            }
        }
    }

    // 3. Coba maim (X11 modern)
    if let Ok(output) = std::process::Command::new("maim").output() {
        if output.status.success() && !output.stdout.is_empty() {
            if let Ok(img) = image::load_from_memory(&output.stdout) {
                return Ok(img);
            }
        }
    }

    // 4. Coba scrot (X11 standar)
    if let Ok(output) = std::process::Command::new("scrot").args(["-z", "-o", "-"]).output() {
        if output.status.success() && !output.stdout.is_empty() {
            if let Ok(img) = image::load_from_memory(&output.stdout) {
                return Ok(img);
            }
        }
    }

    // 5. Coba ImageMagick import
    if let Ok(output) = std::process::Command::new("import").args(["-window", "root", "png:-"]).output() {
        if output.status.success() && !output.stdout.is_empty() {
            if let Ok(img) = image::load_from_memory(&output.stdout) {
                return Ok(img);
            }
        }
    }

    Err("Tidak dapat mengambil screenshot di Linux: xcap gagal dan tool fallback (grim/wayshot/maim/scrot/import) tidak berhasil.".to_string())
}

/// Capture screenshot of the primary display or target window
pub fn capture_screen_image() -> Result<DynamicImage, String> {
    let xcap_res = (|| -> Result<DynamicImage, String> {
        let monitors = Monitor::all().map_err(|e| format!("Failed to list monitors: {e}"))?;
        if monitors.is_empty() {
            return Err("No monitor found".to_string());
        }

        let primary = monitors.into_iter().find(|m| m.is_primary()).unwrap_or_else(|| {
            Monitor::all().unwrap().remove(0)
        });

        let frame = primary.capture_image().map_err(|e| format!("Monitor capture error: {e}"))?;
        let img_buf = RgbaImage::from_raw(frame.width(), frame.height(), frame.into_raw())
            .ok_or_else(|| "Failed to construct ImageBuffer from frame".to_string())?;

        Ok(DynamicImage::ImageRgba8(img_buf))
    })();

    match xcap_res {
        Ok(img) => Ok(img),
        Err(err) => {
            #[cfg(target_os = "linux")]
            {
                crate::term_log!("⚠️ [Perception] xcap gagal ({}). Menggunakan Linux capture fallback...", err);
                capture_screen_image_linux_fallback()
            }
            #[cfg(not(target_os = "linux"))]
            {
                Err(err)
            }
        }
    }
}

/// Capture specific active window image
pub fn capture_window_image_by_title(target_title: &str) -> Result<DynamicImage, String> {
    let windows = Window::all().map_err(|e| format!("Failed to list windows: {e}"))?;
    let lower = target_title.to_lowercase();

    for win in windows {
        if win.title().to_lowercase().contains(&lower) {
            let frame = win.capture_image().map_err(|e| format!("Window capture error: {e}"))?;
            let img_buf = RgbaImage::from_raw(frame.width(), frame.height(), frame.into_raw())
                .ok_or_else(|| "Failed to construct ImageBuffer from window frame".to_string())?;
            return Ok(DynamicImage::ImageRgba8(img_buf));
        }
    }

    capture_screen_image()
}
