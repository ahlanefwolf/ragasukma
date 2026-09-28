pub mod capture;
#[cfg(target_os = "macos")]
pub mod ocr_macos;
pub mod ocr_paddle;
#[cfg(target_os = "windows")]
pub mod ocr_windows;
pub mod utils;

pub use capture::{capture_screen_image, capture_window_image_by_title};
pub use utils::{get_coordinate_scale_divisor, is_button_like_text};

use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualElement {
    pub id: usize,
    pub label: String,
    pub role: String, // "button", "input", "text"
    pub bbox: [i32; 4], // [x, y, w, h]
    pub center: [i32; 2], // [cx, cy]
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerceptionResult {
    pub elements: Vec<VisualElement>,
    pub buttons: Vec<VisualElement>,
    pub inputs: Vec<VisualElement>,
    pub texts: Vec<String>,
    pub text_summary: String,
}

/// Lightweight PerceptionEngine
/// Mengutamakan OCR bawaan sistem operasi (Windows Media OCR) tanpa membebani RAM
/// dengan model ONNX yang berat. Memiliki fallback ke PaddleOCR via Python jika diperlukan.
pub struct PerceptionEngine {}

static PERCEPTION_ENGINE: Mutex<Option<PerceptionEngine>> = Mutex::new(None);

impl PerceptionEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {})
    }

    /// Menjalankan pemindaian OCR visual pada gambar
    pub fn analyze_image(&mut self, image: &DynamicImage) -> PerceptionResult {
        let mut visual_elements: Vec<VisualElement> = Vec::new();
        let mut texts: Vec<String> = Vec::new();
        let mut element_id = 1;
        let scale_div = get_coordinate_scale_divisor();

        let (orig_w, orig_h) = image.dimensions();
        if orig_w == 0 || orig_h == 0 {
            return PerceptionResult::default();
        }

        // 1. Eksekusi OCR dengan prioritas Native OS -> Fallback PaddleOCR
        let ocr_boxes: Result<Vec<(String, [i32; 4], f32)>, String> = {
            #[cfg(target_os = "windows")]
            {
                // Coba native Windows Media OCR
                match ocr_windows::run_windows_native_ocr(image) {
                    Ok(detected) => Ok(detected),
                    Err(err) => {
                        eprintln!("⚠️ [Perception] Windows Native OCR gagal ({err}), mencoba fallback PaddleOCR...");
                        ocr_paddle::run_paddleocr_fallback(image)
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                // Coba native Apple Vision OCR
                match ocr_macos::run_macos_native_ocr(image) {
                    Ok(detected) => Ok(detected),
                    Err(err) => {
                        eprintln!("⚠️ [Perception] Apple Vision OCR gagal ({err}), mencoba fallback PaddleOCR...");
                        ocr_paddle::run_paddleocr_fallback(image)
                    }
                }
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            {
                // Linux: Gunakan fallback PaddleOCR
                ocr_paddle::run_paddleocr_fallback(image)
            }
        };

        match ocr_boxes {
            Ok(boxes) => {
                for (text_label, bbox, conf) in boxes {
                    let trimmed = text_label.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    texts.push(trimmed.to_string());
                    let raw_cx = bbox[0] + bbox[2] / 2;
                    let raw_cy = bbox[1] + bbox[3] / 2;

                    let cx = (raw_cx as f32 / scale_div) as i32;
                    let cy = (raw_cy as f32 / scale_div) as i32;
                    let adjusted_bbox = [
                        (bbox[0] as f32 / scale_div) as i32,
                        (bbox[1] as f32 / scale_div) as i32,
                        (bbox[2] as f32 / scale_div) as i32,
                        (bbox[3] as f32 / scale_div) as i32,
                    ];

                    let role = if is_button_like_text(trimmed) {
                        "button".to_string()
                    } else {
                        "text".to_string()
                    };

                    visual_elements.push(VisualElement {
                        id: element_id,
                        label: trimmed.to_string(),
                        role,
                        bbox: adjusted_bbox,
                        center: [cx, cy],
                        confidence: conf,
                    });
                    element_id += 1;
                }
            }
            Err(err) => {
                eprintln!("❌ [Perception] Gagal memproses OCR visual: {err}");
            }
        }

        let mut buttons = Vec::new();
        let mut inputs = Vec::new();

        for el in &visual_elements {
            buttons.push(el.clone());
            if el.role == "input" {
                inputs.push(el.clone());
            }
        }

        let text_summary = if texts.len() > 30 {
            format!("{} ... ({} teks terdeteksi)", texts[..30].join(" | "), texts.len())
        } else {
            texts.join(" | ")
        };

        PerceptionResult {
            elements: visual_elements,
            buttons,
            inputs,
            texts,
            text_summary,
        }
    }
}

pub enum VisionRequest {
    ScanActiveScreen {
        save_snapshot: bool,
        respond_to: Option<std::sync::mpsc::Sender<PerceptionResult>>,
    },
    ScanTargetWindow {
        title: String,
        save_snapshot: bool,
        respond_to: Option<std::sync::mpsc::Sender<PerceptionResult>>,
    },
}

static VISION_SENDER: Mutex<Option<std::sync::mpsc::Sender<VisionRequest>>> = Mutex::new(None);

/// Inisialisasi Vision Worker di thread background terpisah.
/// Thread ini 100% idle (0% CPU, 0 MB beban model) saat tidak ada permintaan scan.
pub fn start_vision_worker() {
    let mut sender_lock = VISION_SENDER.lock().unwrap();
    if sender_lock.is_some() {
        return; // Sudah berjalan
    }

    let (tx, rx) = std::sync::mpsc::channel::<VisionRequest>();
    *sender_lock = Some(tx);

    std::thread::Builder::new()
        .name("ragasukma-vision-worker".to_string())
        .spawn(move || {
            let mut engine = PerceptionEngine::new().unwrap_or(PerceptionEngine {});

            while let Ok(request) = rx.recv() {
                match request {
                    VisionRequest::ScanActiveScreen { save_snapshot, respond_to } => {
                        let res = match capture_screen_image() {
                            Ok(img) => engine.analyze_image(&img),
                            Err(e) => {
                                eprintln!("❌ [Vision Worker] Gagal capture layar: {e}");
                                PerceptionResult::default()
                            }
                        };

                        if save_snapshot {
                            apply_perception_to_snapshot(&res);

                            // Dapatkan info jendela yang sedang aktif untuk laporan suara asisten
                            let active_app = if let Ok(snap) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
                                if !snap.window_title.is_empty() {
                                    snap.window_title
                                } else if !snap.app_name.is_empty() {
                                    snap.app_name
                                } else {
                                    "layar desktop".to_string()
                                }
                            } else {
                                "layar desktop".to_string()
                            };

                            let summary_info = if !res.texts.is_empty() || !res.buttons.is_empty() {
                                format!(
                                    "[STATUS SISTEM: Pemindaian layar untuk jendela {} sudah selesai, Mas! Datanya sudah masuk. Sampaikan dengan santai dan singkat bahwa layarnya sudah selesai dipindai dan siap jika ada yang mau ditanyakan/dibantu.]",
                                    active_app
                                )
                            } else {
                                format!(
                                    "[STATUS SISTEM: Pemindaian layar untuk jendela {} selesai, tapi tidak ada teks atau tombol yang terbaca. Sampaikan secara singkat dan santai.]",
                                    active_app
                                )
                            };

                            crate::engine::remote_text::inject_remote_text(summary_info);
                        }

                        if let Some(resp) = respond_to {
                            let _ = resp.send(res);
                        }
                    }
                    VisionRequest::ScanTargetWindow { title, save_snapshot, respond_to } => {
                        let res = match capture_window_image_by_title(&title) {
                            Ok(img) => engine.analyze_image(&img),
                            Err(e) => {
                                eprintln!("❌ [Vision Worker] Gagal capture window '{title}': {e}");
                                PerceptionResult::default()
                            }
                        };

                        if save_snapshot {
                            apply_perception_to_snapshot(&res);
                        }

                        if let Some(resp) = respond_to {
                            let _ = resp.send(res);
                        }
                    }
                }
            }
        })
        .expect("Gagal spawn vision worker thread");
}

/// Helper sinkron untuk meminta scan dari vision worker
pub fn request_vision_scan_sync() -> PerceptionResult {
    start_vision_worker();

    let (tx_resp, rx_resp) = std::sync::mpsc::channel();
    let req = VisionRequest::ScanActiveScreen {
        save_snapshot: true,
        respond_to: Some(tx_resp),
    };

    if let Ok(guard) = VISION_SENDER.lock() {
        if let Some(ref sender) = *guard {
            let _ = sender.send(req);
            return rx_resp
                .recv_timeout(std::time::Duration::from_secs(15))
                .unwrap_or_else(|_| scan_screen_visual_elements());
        }
    }

    scan_screen_visual_elements()
}

/// Helper asinkron fire-and-forget untuk trigger vision scan di background
pub fn trigger_vision_scan_async() {
    start_vision_worker();

    let req = VisionRequest::ScanActiveScreen {
        save_snapshot: true,
        respond_to: None,
    };

    if let Ok(guard) = VISION_SENDER.lock() {
        if let Some(ref sender) = *guard {
            let _ = sender.send(req);
        }
    }
}

/// Sinkronisasi hasil perception ke `active_window_app.json`
pub fn apply_perception_to_snapshot(res: &PerceptionResult) {
    if let Ok(mut snap) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
        snap.source = "vision_perception".to_string();
        snap.timestamp = crate::engine::window_scanner::current_epoch_ms();

        if !res.elements.is_empty() {
            snap.buttons = res
                .elements
                .iter()
                .map(|b| crate::engine::window_scanner::DesktopElement {
                    id: b.id,
                    label: b.label.clone(),
                    role: b.role.clone(),
                    center: Some(b.center),
                    bbox: Some(b.bbox),
                })
                .collect();
        } else if !res.buttons.is_empty() {
            snap.buttons = res
                .buttons
                .iter()
                .map(|b| crate::engine::window_scanner::DesktopElement {
                    id: b.id,
                    label: b.label.clone(),
                    role: b.role.clone(),
                    center: Some(b.center),
                    bbox: Some(b.bbox),
                })
                .collect();
        }
        if !res.inputs.is_empty() {
            snap.inputs = res
                .inputs
                .iter()
                .map(|inp| crate::engine::window_scanner::DesktopElement {
                    id: inp.id,
                    label: inp.label.clone(),
                    role: inp.role.clone(),
                    center: Some(inp.center),
                    bbox: Some(inp.bbox),
                })
                .collect();
        }
        if !res.texts.is_empty() {
            snap.texts = res.texts.clone();
            snap.text_summary = res.text_summary.clone();
        }
        let _ = snap.save_to_file();
    }
}

/// On-demand visual perception scan
pub fn scan_screen_visual_elements() -> PerceptionResult {
    let mut lock = PERCEPTION_ENGINE.lock().unwrap();
    if lock.is_none() {
        match PerceptionEngine::new() {
            Ok(engine) => *lock = Some(engine),
            Err(e) => {
                eprintln!("[Perception] Gagal inisialisasi Perception Engine: {e}");
                return PerceptionResult::default();
            }
        }
    }

    let img = match capture_screen_image() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[Perception] Gagal capture layar: {e}");
            return PerceptionResult::default();
        }
    };

    if let Some(engine) = lock.as_mut() {
        engine.analyze_image(&img)
    } else {
        PerceptionResult::default()
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_perception_scan_and_snapshot() {
        let res = scan_screen_visual_elements();
        println!("Perception elements count: {}", res.elements.len());
        println!("Perception texts count: {}", res.texts.len());
        if !res.texts.is_empty() {
            println!("Sample text: {:?}", &res.texts[..res.texts.len().min(5)]);
        }
        apply_perception_to_snapshot(&res);
        if let Ok(snap) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
            println!("Snapshot source: {}", snap.source);
            println!("Snapshot buttons: {}", snap.buttons.len());
            println!("Snapshot texts: {}", snap.texts.len());
        }
    }
}
