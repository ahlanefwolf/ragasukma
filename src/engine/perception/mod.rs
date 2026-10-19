pub mod capture;
#[cfg(target_os = "macos")]
pub mod ocr_macos;
pub mod ocr_tesseract;
#[cfg(target_os = "windows")]
pub mod ocr_windows;
pub mod utils;

pub use capture::capture_screen_image;
pub use utils::{get_coordinate_scale_divisor, is_button_like_text, string_similarity};

use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualElement {
    pub id: usize,
    pub label: String,
    pub role: String,     // "button", "input", "text"
    pub bbox: [i32; 4],   // [x, y, w, h]
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
/// Prioritizes OS-native OCR (Windows Media OCR / Apple Vision) without loading extra RAM
/// with heavy models. Features fast fallback to Tesseract OCR (streaming via stdin/stdout pipe).
pub struct PerceptionEngine {}

static PERCEPTION_ENGINE: Mutex<Option<PerceptionEngine>> = Mutex::new(None);

impl PerceptionEngine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {})
    }

    /// Runs visual OCR scan on image
    pub fn analyze_image(&mut self, image: &DynamicImage) -> PerceptionResult {
        let mut visual_elements: Vec<VisualElement> = Vec::new();
        let mut texts: Vec<String> = Vec::new();
        let (orig_w, orig_h) = image.dimensions();
        if orig_w == 0 || orig_h == 0 {
            return PerceptionResult::default();
        }

        let mut element_id = 1;
        let scale_div = get_coordinate_scale_divisor(orig_w, orig_h);

        // 1. Eksekusi OCR with prioritas Native OS -> Fallback Tesseract OCR
        let ocr_boxes: Result<Vec<(String, [i32; 4], f32)>, String> = {
            #[cfg(target_os = "windows")]
            {
                // Coba native Windows Media OCR
                match ocr_windows::run_windows_native_ocr(image) {
                    Ok(detected) => Ok(detected),
                    Err(err) => {
                        eprintln!(
                            "[Perception] Windows Native OCR failed ({err}), attempting Tesseract OCR fallback..."
                        );
                        ocr_tesseract::run_tesseract_fallback(image)
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                // Coba native Apple Vision OCR
                match ocr_macos::run_macos_native_ocr(image) {
                    Ok(detected) => Ok(detected),
                    Err(err) => {
                        eprintln!(
                            "[Perception] Apple Vision OCR failed ({err}), attempting Tesseract OCR fallback..."
                        );
                        ocr_tesseract::run_tesseract_fallback(image)
                    }
                }
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            {
                // Linux: Use Tesseract OCR memory-streaming
                ocr_tesseract::run_tesseract_fallback(image)
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
                eprintln!("[Perception] Failed to process visual OCR: {err}");
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
            format!(
                "{} ... ({} texts detected)",
                texts[..30].join(" | "),
                texts.len()
            )
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum VisionIpcCmd {
    #[serde(rename = "scan")]
    Scan { save_snapshot: bool },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "quit")]
    Quit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VisionIpcResp {
    pub success: bool,
    pub result: Option<PerceptionResult>,
    pub error: Option<String>,
}

pub static VISION_SUPERVISOR: crate::services::ManagedWorkerProcess =
    crate::services::ManagedWorkerProcess::new(
        "--worker-vision",
        std::time::Duration::from_secs(60),
    );

static IS_SCANNING_ASYNC: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Dedicated Child Process Entry Point for Vision & OCR Worker.
/// Runs in an isolated OS process. If idle for 60 seconds, terminates itself to reclaim memory.
pub fn run_vision_worker_process() -> anyhow::Result<()> {
    use std::io::{BufRead, Write};
    let mut engine = PerceptionEngine::new().unwrap_or(PerceptionEngine {});
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    let (tx_line, rx_line) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("vision-ipc-stdin".to_string())
        .spawn(move || {
            let mut line = String::new();
            let mut handle = stdin.lock();
            while let Ok(n) = handle.read_line(&mut line) {
                if n == 0 {
                    break;
                }
                if tx_line.send(line.trim().to_string()).is_err() {
                    break;
                }
                line.clear();
            }
        })?;

    loop {
        // Idle timeout: terminate process if no scan request arrives within 60s
        let line = match rx_line.recv_timeout(std::time::Duration::from_secs(60)) {
            Ok(l) => l,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                eprintln!(
                    "[Vision Worker Process] 60s idle timeout reached without requests. Exiting process to release memory."
                );
                break;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break; // Parent process closed stdin
            }
        };

        if line.is_empty() {
            continue;
        }

        let cmd: VisionIpcCmd = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let err_resp = VisionIpcResp {
                    success: false,
                    result: None,
                    error: Some(format!("Invalid command JSON: {e}")),
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&err_resp)?);
                let _ = stdout.flush();
                continue;
            }
        };

        match cmd {
            VisionIpcCmd::Scan { save_snapshot } => {
                let res = match capture_screen_image() {
                    Ok(img) => engine.analyze_image(&img),
                    Err(e) => {
                        eprintln!("[Vision Worker Process] Failed to capture screen: {e}");
                        PerceptionResult::default()
                    }
                };

                if save_snapshot {
                    apply_perception_to_snapshot(&res);
                }

                let resp = VisionIpcResp {
                    success: true,
                    result: Some(res),
                    error: None,
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                let _ = stdout.flush();
            }
            VisionIpcCmd::Ping => {
                let resp = VisionIpcResp {
                    success: true,
                    result: None,
                    error: None,
                };
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp)?);
                let _ = stdout.flush();
            }
            VisionIpcCmd::Quit => {
                break;
            }
        }
    }

    Ok(())
}

/// Initialize Vision Worker in supervisor manager.
/// Checks idle state to drop process if idle > 60s.
pub fn start_vision_worker() {
    VISION_SUPERVISOR.drop_if_idle();
}

/// Helper sync for requesting scan from dedicated Vision worker process
pub fn request_vision_scan_sync() -> PerceptionResult {
    let cmd = VisionIpcCmd::Scan {
        save_snapshot: true,
    };
    let cmd_json = match serde_json::to_string(&cmd) {
        Ok(j) => j,
        Err(_) => return scan_screen_visual_elements(),
    };

    match VISION_SUPERVISOR.send_command(&cmd_json, std::time::Duration::from_secs(25)) {
        Ok(resp_str) => {
            if let Ok(resp) = serde_json::from_str::<VisionIpcResp>(&resp_str) {
                if resp.success {
                    if let Some(res) = resp.result {
                        // Notify assistant status update
                        let active_app = if let Ok(snap) =
                            crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file()
                        {
                            if !snap.window_title.is_empty() {
                                snap.window_title
                            } else if !snap.app_name.is_empty() {
                                snap.app_name
                            } else {
                                "desktop screen".to_string()
                            }
                        } else {
                            "desktop screen".to_string()
                        };

                        let summary_info = if !res.texts.is_empty() || !res.buttons.is_empty() {
                            format!(
                                "[SYSTEM STATUS: Screen scan for window '{}' complete. Data is ready. Inform the user briefly that the screen has been scanned and you're ready to help.]",
                                active_app
                            )
                        } else {
                            format!(
                                "[SYSTEM STATUS: Screen scan for window '{}' complete, but no text or buttons were detected. Inform the user briefly and casually.]",
                                active_app
                            )
                        };

                        crate::engine::remote_text::inject_remote_text(summary_info);
                        return res;
                    }
                }
            }
        }
        Err(e) => {
            crate::term_err!(
                "[Vision Supervisor] Child process error ({e}), falling back to direct scan..."
            );
        }
    }

    scan_screen_visual_elements()
}

/// Asynchronous fire-and-forget helper to trigger vision scan in background process
pub fn trigger_vision_scan_async() {
    if IS_SCANNING_ASYNC.swap(true, std::sync::atomic::Ordering::SeqCst) {
        crate::term_log!(
            "[Vision Worker] Screen scan already running in background. Skipping duplicate request."
        );
        return;
    }

    std::thread::Builder::new()
        .name("vision-async-trigger".to_string())
        .spawn(|| {
            let _ = request_vision_scan_sync();
            IS_SCANNING_ASYNC.store(false, std::sync::atomic::Ordering::SeqCst);
        })
        .ok();
}

/// Synchronizes perception results to `active_window_app.json`
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
    let mut lock = PERCEPTION_ENGINE.lock().unwrap_or_else(|e| e.into_inner());
    if lock.is_none() {
        match PerceptionEngine::new() {
            Ok(engine) => *lock = Some(engine),
            Err(e) => {
                eprintln!("[Perception] Failed to initialize Perception Engine: {e}");
                return PerceptionResult::default();
            }
        }
    }

    let img = match capture_screen_image() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("[Perception] Failed to capture screen: {e}");
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
    use super::*;

    #[test]
    fn test_perception_analyze_blank_image() {
        let mut engine = PerceptionEngine::new().unwrap();
        let img = DynamicImage::new_rgb8(100, 50);
        let res = engine.analyze_image(&img);
        assert!(res.elements.is_empty());
    }

    #[test]
    fn test_monitor_scale_factor() {
        if let Ok(monitors) = xcap::Monitor::all() {
            assert!(
                !monitors.is_empty(),
                "At least one monitor must be detected"
            );
        }
    }

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
