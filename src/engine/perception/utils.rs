#[cfg(target_os = "macos")]
use xcap::Monitor;

/// Mendapatkan pembagi skala koordinat agar hasil scan visual
/// sinkron dengan tool klik mouse di masing-masing sistem operasi.
pub fn get_coordinate_scale_divisor() -> f32 {
    #[cfg(target_os = "macos")]
    {
        // Di macOS Retina, capture gambar berada dalam Physical Pixels (misal 2880x1800),
        // sedangkan tool klik (cliclick & AppleScript) menerima input dalam Points (1440x900).
        if let Ok(monitors) = Monitor::all() {
            if let Some(p) = monitors.into_iter().find(|m| m.is_primary()) {
                let s = p.scale_factor();
                if s > 0.5 {
                    return s;
                }
            }
        }
        2.0 // Standar Retina display fallback
    }

    #[cfg(target_os = "windows")]
    {
        // Windows dalam mode Per-Monitor V2 DPI Aware bekerja murni dalam Physical Pixels 1:1
        1.0
    }

    #[cfg(target_os = "linux")]
    {
        // Linux X11/XWayland bekerja dalam pixel fisik 1:1 untuk xdotool
        1.0
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        1.0
    }
}

pub fn is_button_like_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    // 1. Tombol angka kalkulator & keypad numerik (0 s/d 9, 00, dll)
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() <= 4 {
        return true;
    }

    // 2. Tombol simbol matematika & tombol fungsi kalkulator
    let math_symbols = [
        "+", "-", "*", "/", "x", "×", "÷", "=", "%", "±", "+/-", ".", ",",
        "c", "ce", "del", "mc", "mr", "m+", "m-", "ms", "m", "sqrt", "√", "x²", "1/x",
        "exp", "mod", "log", "ln", "(", ")", "pi", "π", "e", "deg", "rad",
    ];
    let lower = trimmed.to_lowercase();
    for sym in &math_symbols {
        if lower == *sym {
            return true;
        }
    }

    // 3. Kata kunci aksi tombol UI umum (Indonesia & Inggris)
    let keywords = [
        "ok", "cancel", "save", "open", "close", "submit", "apply", "next", "back", "yes", "no",
        "kirim", "batal", "simpan", "buka", "tutup", "hapus", "cari", "search", "enter", "login",
        "masuk", "daftar", "edit", "play", "pause", "stop", "download", "install", "menu", "help",
        "home", "settings", "reload", "refresh", "copy", "cut", "paste", "undo", "redo",
    ];

    for kw in &keywords {
        if lower == *kw || (lower.len() < 25 && lower.contains(kw)) {
            return true;
        }
    }

    // 4. Tombol pendek (1-2 karakter alfanumerik) biasanya adalah tombol keypad / navigasi
    if trimmed.len() <= 2 && trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation()) {
        return true;
    }

    false
}
