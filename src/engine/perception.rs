use image::{DynamicImage, GenericImageView, RgbaImage};
use ndarray::Array4;
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Tensor,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use xcap::{Monitor, Window};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VisualElement {
    pub id: usize,
    pub label: String,
    pub role: String, // "button", "input", "icon", "text"
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

pub struct PerceptionEngine {
    det_session: Option<Session>,
    rec_session: Option<Session>,
    icon_session: Option<Session>,
    dict_chars: Vec<String>,
}

static PERCEPTION_ENGINE: Mutex<Option<PerceptionEngine>> = Mutex::new(None);

impl PerceptionEngine {
    pub fn new() -> Result<Self, String> {
        let models_dir = find_models_dir();

        let det_path = models_dir.join("ppocr_det.onnx");
        let rec_path = models_dir.join("ppocr_rec.onnx");
        let icon_path = models_dir.join("icon_detect.onnx");
        let dict_path = models_dir.join("ppocr_dict.txt");

        // Load dictionary characters
        let dict_chars = if dict_path.exists() {
            if let Ok(content) = fs::read_to_string(&dict_path) {
                let mut chars: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                chars.push(" ".to_string());
                chars
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let det_session = if det_path.exists() {
            Session::builder()
                .map_err(|e| format!("Failed to create det session builder: {e}"))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| format!("Failed to set opt level: {e}"))?
                .with_intra_threads(4)
                .map_err(|e| format!("Failed to set intra threads: {e}"))?
                .commit_from_file(&det_path)
                .ok()
        } else {
            None
        };

        let rec_session = if rec_path.exists() {
            Session::builder()
                .map_err(|e| format!("Failed to create rec session builder: {e}"))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| format!("Failed to set opt level: {e}"))?
                .with_intra_threads(4)
                .map_err(|e| format!("Failed to set intra threads: {e}"))?
                .commit_from_file(&rec_path)
                .ok()
        } else {
            None
        };

        let icon_session = if icon_path.exists() {
            Session::builder()
                .map_err(|e| format!("Failed to create icon session builder: {e}"))?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| format!("Failed to set opt level: {e}"))?
                .with_intra_threads(4)
                .map_err(|e| format!("Failed to set intra threads: {e}"))?
                .commit_from_file(&icon_path)
                .ok()
        } else {
            None
        };

        Ok(Self {
            det_session,
            rec_session,
            icon_session,
            dict_chars,
        })
    }

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

    /// Run full visual scan on an image
    pub fn analyze_image(&mut self, image: &DynamicImage) -> PerceptionResult {
        let mut visual_elements: Vec<VisualElement> = Vec::new();
        let mut texts: Vec<String> = Vec::new();
        let mut element_id = 1;
        let scale_div = Self::get_coordinate_scale_divisor();

        let (orig_w, orig_h) = image.dimensions();
        if orig_w == 0 || orig_h == 0 {
            return PerceptionResult::default();
        }

        // 1. Text Detection & Recognition (PP-OCR)
        if let Some(ref mut det_sess) = self.det_session {
            if let Ok(boxes) = run_detect_text_boxes(det_sess, image) {
                for bbox in boxes {
                    let mut text_label = String::new();
                    let mut conf = 0.85;

                    // Crop text region & recognize
                    if let Some(ref mut rec_sess) = self.rec_session {
                        let crop_x = bbox[0].max(0) as u32;
                        let crop_y = bbox[1].max(0) as u32;
                        let crop_w = (bbox[2].max(1) as u32).min(orig_w.saturating_sub(crop_x));
                        let crop_h = (bbox[3].max(1) as u32).min(orig_h.saturating_sub(crop_y));

                        if crop_w > 4 && crop_h > 4 {
                            let cropped = image.crop_imm(crop_x, crop_y, crop_w, crop_h);
                            if let Ok((rec_text, rec_conf)) = run_recognize_text(rec_sess, &self.dict_chars, &cropped) {
                                text_label = rec_text.trim().to_string();
                                conf = rec_conf;
                            }
                        }
                    }

                    if !text_label.is_empty() {
                        texts.push(text_label.clone());
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

                        let role = if is_button_like_text(&text_label) {
                            "button".to_string()
                        } else {
                            "text".to_string()
                        };

                        visual_elements.push(VisualElement {
                            id: element_id,
                            label: text_label,
                            role,
                            bbox: adjusted_bbox,
                            center: [cx, cy],
                            confidence: conf,
                        });
                        element_id += 1;
                    }
                }
            }
        }

        // 2. Icon Detection (OmniParser/YOLO)
        if let Some(ref mut icon_sess) = self.icon_session {
            if let Ok(icon_boxes) = run_detect_icons(icon_sess, image) {
                for (bbox, icon_conf) in icon_boxes {
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

                    let overlapping_text = visual_elements.iter_mut().find(|el| {
                        let [tx, ty, tw, th] = el.bbox;
                        let t_cx = tx + tw / 2;
                        let t_cy = ty + th / 2;
                        t_cx >= adjusted_bbox[0] && t_cx <= (adjusted_bbox[0] + adjusted_bbox[2]) && t_cy >= adjusted_bbox[1] && t_cy <= (adjusted_bbox[1] + adjusted_bbox[3])
                    });

                    if let Some(txt_el) = overlapping_text {
                        txt_el.role = "button".to_string();
                        txt_el.bbox = adjusted_bbox;
                        txt_el.center = [cx, cy];
                    } else {
                        visual_elements.push(VisualElement {
                            id: element_id,
                            label: format!("Icon_{element_id}"),
                            role: "button".to_string(),
                            bbox: adjusted_bbox,
                            center: [cx, cy],
                            confidence: icon_conf,
                        });
                        element_id += 1;
                    }
                }
            }
        }

        let mut buttons = Vec::new();
        let mut inputs = Vec::new();

        for el in &visual_elements {
            // Semua elemen visual (tombol, ikon, dan teks terdeteksi) dapat diinteraksikan/diklik
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

/// Preprocess & run PP-OCR Detection
fn run_detect_text_boxes(session: &mut Session, image: &DynamicImage) -> Result<Vec<[i32; 4]>, String> {
    let (orig_w, orig_h) = image.dimensions();
    let target_h = ((orig_h.min(960) / 32).max(1) * 32) as usize;
    let target_w = ((orig_w.min(960) / 32).max(1) * 32) as usize;

    let resized = image.resize_exact(target_w as u32, target_h as u32, image::imageops::FilterType::Triangle);
    let rgb_img = resized.to_rgb8();

    let mut input_arr = Array4::<f32>::zeros((1, 3, target_h, target_w));
    let mean = [0.485f32, 0.456, 0.406];
    let std = [0.229f32, 0.224, 0.225];

    for (x, y, pixel) in rgb_img.enumerate_pixels() {
        let r = (pixel[0] as f32 / 255.0 - mean[0]) / std[0];
        let g = (pixel[1] as f32 / 255.0 - mean[1]) / std[1];
        let b = (pixel[2] as f32 / 255.0 - mean[2]) / std[2];

        input_arr[[0, 0, y as usize, x as usize]] = r;
        input_arr[[0, 1, y as usize, x as usize]] = g;
        input_arr[[0, 2, y as usize, x as usize]] = b;
    }

    let input_tensor = Tensor::from_array(input_arr)
        .map_err(|e| format!("Tensor creation error: {e}"))?;

    let outputs = session.run(ort::inputs![input_tensor])
        .map_err(|e| format!("Inference error: {e}"))?;

    let (_shape, data) = outputs[0].try_extract_tensor::<f32>()
        .map_err(|e| format!("Extract tensor error: {e}"))?;

    let scale_x = orig_w as f32 / target_w as f32;
    let scale_y = orig_h as f32 / target_h as f32;

    let boxes = extract_boxes_from_prob_map(data, target_w, target_h, 0.3, scale_x, scale_y);
    Ok(boxes)
}

/// Preprocess & run PP-OCR Recognition on cropped line
fn run_recognize_text(session: &mut Session, dict_chars: &[String], cropped: &DynamicImage) -> Result<(String, f32), String> {
    let (crop_w, crop_h) = cropped.dimensions();
    if crop_w == 0 || crop_h == 0 {
        return Ok((String::new(), 0.0));
    }

    let target_h = 48usize;
    let target_w = (((crop_w as f32 / crop_h as f32) * target_h as f32).clamp(48.0, 640.0) as usize / 4) * 4;

    let resized = cropped.resize_exact(target_w as u32, target_h as u32, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();

    let mut input_arr = Array4::<f32>::zeros((1, 3, target_h, target_w));
    for (x, y, pixel) in rgb.enumerate_pixels() {
        input_arr[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 / 255.0 - 0.5) / 0.5;
        input_arr[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 / 255.0 - 0.5) / 0.5;
        input_arr[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 / 255.0 - 0.5) / 0.5;
    }

    let input_tensor = Tensor::from_array(input_arr)
        .map_err(|e| format!("Rec tensor error: {e}"))?;

    let outputs = session.run(ort::inputs![input_tensor])
        .map_err(|e| format!("Rec run error: {e}"))?;

    let (shape, data) = outputs[0].try_extract_tensor::<f32>()
        .map_err(|e| format!("Rec extract error: {e}"))?;

    if shape.len() < 3 {
        return Ok((String::new(), 0.0));
    }

    let seq_len = shape[1] as usize;
    let num_classes = shape[2] as usize;

    let mut text = String::new();
    let mut last_idx = 0usize;
    let mut conf_sum = 0.0f32;
    let mut char_count = 0;

    for t in 0..seq_len {
        let offset = t * num_classes;
        let slice = &data[offset..offset + num_classes];

        let mut max_idx = 0;
        let mut max_val = f32::NEG_INFINITY;
        for (idx, &val) in slice.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = idx;
            }
        }

        if max_idx != 0 && max_idx != last_idx {
            if let Some(ch) = dict_chars.get(max_idx - 1) {
                text.push_str(ch);
                conf_sum += (1.0 / (1.0 + (-max_val).exp())).min(1.0);
                char_count += 1;
            }
        }
        last_idx = max_idx;
    }

    let avg_conf = if char_count > 0 { conf_sum / char_count as f32 } else { 0.0 };
    Ok((text, avg_conf))
}

/// Preprocess & run Icon Detection (OmniParser YOLO)
fn run_detect_icons(session: &mut Session, image: &DynamicImage) -> Result<Vec<([i32; 4], f32)>, String> {
    let (orig_w, orig_h) = image.dimensions();
    let target_size = 640usize;

    let resized = image.resize_exact(target_size as u32, target_size as u32, image::imageops::FilterType::Triangle);
    let rgb = resized.to_rgb8();

    let mut input_arr = Array4::<f32>::zeros((1, 3, target_size, target_size));
    for (x, y, pixel) in rgb.enumerate_pixels() {
        input_arr[[0, 0, y as usize, x as usize]] = pixel[0] as f32 / 255.0;
        input_arr[[0, 1, y as usize, x as usize]] = pixel[1] as f32 / 255.0;
        input_arr[[0, 2, y as usize, x as usize]] = pixel[2] as f32 / 255.0;
    }

    let input_tensor = Tensor::from_array(input_arr)
        .map_err(|e| format!("Icon tensor error: {e}"))?;

    let outputs = session.run(ort::inputs![input_tensor])
        .map_err(|e| format!("Icon inference error: {e}"))?;

    let (shape, data) = outputs[0].try_extract_tensor::<f32>()
        .map_err(|e| format!("Icon extract error: {e}"))?;

    let mut detections: Vec<([i32; 4], f32)> = Vec::new();
    let scale_x = orig_w as f32 / target_size as f32;
    let scale_y = orig_h as f32 / target_size as f32;

    if shape.len() == 3 {
        let dim1 = shape[1] as usize;
        let dim2 = shape[2] as usize;

        if dim1 == 5 || dim1 == 6 {
            // Shape [1, 5/6, 8400]
            #[allow(clippy::erasing_op, clippy::identity_op)]
            for i in 0..dim2 {
                let cx = data[0 * dim2 + i] * scale_x;
                let cy = data[1 * dim2 + i] * scale_y;
                let w = data[2 * dim2 + i] * scale_x;
                let h = data[3 * dim2 + i] * scale_y;
                let conf = data[4 * dim2 + i];

                if conf > 0.35 && w > 8.0 && h > 8.0 && w < (orig_w as f32 * 0.9) {
                    let x = (cx - w / 2.0).max(0.0) as i32;
                    let y = (cy - h / 2.0).max(0.0) as i32;
                    detections.push(([x, y, w as i32, h as i32], conf));
                }
            }
        } else if dim2 == 5 || dim2 == 6 {
            // Shape [1, 8400, 5/6]
            for i in 0..dim1 {
                let offset = i * dim2;
                let cx = data[offset] * scale_x;
                let cy = data[offset + 1] * scale_y;
                let w = data[offset + 2] * scale_x;
                let h = data[offset + 3] * scale_y;
                let conf = data[offset + 4];

                if conf > 0.35 && w > 8.0 && h > 8.0 && w < (orig_w as f32 * 0.9) {
                    let x = (cx - w / 2.0).max(0.0) as i32;
                    let y = (cy - h / 2.0).max(0.0) as i32;
                    detections.push(([x, y, w as i32, h as i32], conf));
                }
            }
        }
    }

    detections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut final_boxes: Vec<([i32; 4], f32)> = Vec::new();

    for det in detections {
        let mut keep = true;
        for kept in &final_boxes {
            if iou_bbox(&det.0, &kept.0) > 0.45 {
                keep = false;
                break;
            }
        }
        if keep && final_boxes.len() < 50 {
            final_boxes.push(det);
        }
    }

    Ok(final_boxes)
}

/// Helper untuk mencari lokasi folder models yang valid (terpisah dari binary/assets)
fn find_models_dir() -> PathBuf {
    let mut candidates = vec![
        PathBuf::from("models"),
        PathBuf::from("../models"),
        PathBuf::from("assets/models"),
        PathBuf::from("../assets/models"),
    ];

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            candidates.push(exe_dir.join("models"));
            candidates.push(exe_dir.join("assets/models"));
            candidates.push(exe_dir.join("../Resources/models"));
        }
    }

    candidates.push(crate::config::get_app_config_dir().join("models"));
    candidates.push(crate::config::get_app_config_dir().join("assets").join("models"));

    for path in &candidates {
        if path.exists() {
            return path.clone();
        }
    }

    PathBuf::from("models")
}

/// Ekstrak bounding box dari prob map DBNet
fn extract_boxes_from_prob_map(
    prob_map: &[f32],
    w: usize,
    h: usize,
    threshold: f32,
    scale_x: f32,
    scale_y: f32,
) -> Vec<[i32; 4]> {
    let mut visited = vec![false; w * h];
    let mut boxes = Vec::new();

    for y in (0..h).step_by(2) {
        for x in (0..w).step_by(2) {
            let idx = y * w + x;
            if prob_map[idx] > threshold && !visited[idx] {
                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;
                let mut count = 0;

                let mut queue = vec![(x, y)];
                visited[idx] = true;

                while let Some((qx, qy)) = queue.pop() {
                    count += 1;
                    min_x = min_x.min(qx);
                    max_x = max_x.max(qx);
                    min_y = min_y.min(qy);
                    max_y = max_y.max(qy);

                    let neighbors = [
                        (qx.saturating_sub(1), qy),
                        ((qx + 1).min(w - 1), qy),
                        (qx, qy.saturating_sub(1)),
                        (qx, (qy + 1).min(h - 1)),
                    ];

                    for (nx, ny) in neighbors {
                        let n_idx = ny * w + nx;
                        if !visited[n_idx] && prob_map[n_idx] > threshold {
                            visited[n_idx] = true;
                            if queue.len() < 2000 {
                                queue.push((nx, ny));
                            }
                        }
                    }
                }

                let box_w = max_x.saturating_sub(min_x) + 1;
                let box_h = max_y.saturating_sub(min_y) + 1;

                if count > 15 && box_w > 4 && box_h > 4 {
                    let rx = (min_x as f32 * scale_x) as i32;
                    let ry = (min_y as f32 * scale_y) as i32;
                    let rw = (box_w as f32 * scale_x) as i32;
                    let rh = (box_h as f32 * scale_y) as i32;

                    boxes.push([rx, ry, rw, rh]);
                }
            }
        }
    }

    boxes
}

/// Calculate Intersection over Union (IoU) between two bounding boxes
fn iou_bbox(box_a: &[i32; 4], box_b: &[i32; 4]) -> f32 {
    let x_a = box_a[0].max(box_b[0]);
    let y_a = box_a[1].max(box_b[1]);
    let x_b = (box_a[0] + box_a[2]).min(box_b[0] + box_b[2]);
    let y_b = (box_a[1] + box_a[3]).min(box_b[1] + box_b[3]);

    let inter_w = (x_b - x_a).max(0);
    let inter_h = (y_b - y_a).max(0);
    let inter_area = (inter_w * inter_h) as f32;

    let area_a = (box_a[2] * box_a[3]) as f32;
    let area_b = (box_b[2] * box_b[3]) as f32;

    let union_area = area_a + area_b - inter_area;
    if union_area <= 0.0 {
        0.0
    } else {
        inter_area / union_area
    }
}

fn is_button_like_text(text: &str) -> bool {
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
/// Thread ini 100% idle (0% CPU) saat tidak ada permintaan scan.
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
            println!("👁️  [Vision Worker] Standby di background thread (0% CPU idle)...");
            let mut engine_opt: Option<PerceptionEngine> = None;

            while let Ok(request) = rx.recv() {
                // Inisialisasi engine secara lazy hanya saat pertama kali dibutuhkan
                if engine_opt.is_none() {
                    match PerceptionEngine::new() {
                        Ok(e) => {
                            println!("👁️  [Vision Worker] Model ONNX (PP-OCR + OmniParser) dimuat ke memory.");
                            engine_opt = Some(e);
                        }
                        Err(err) => {
                            eprintln!("❌ [Vision Worker] Gagal load ONNX engine: {err}");
                            match request {
                                VisionRequest::ScanActiveScreen { respond_to, .. } => {
                                    if let Some(resp) = respond_to {
                                        let _ = resp.send(PerceptionResult::default());
                                    }
                                }
                                VisionRequest::ScanTargetWindow { respond_to, .. } => {
                                    if let Some(resp) = respond_to {
                                        let _ = resp.send(PerceptionResult::default());
                                    }
                                }
                            }
                            continue;
                        }
                    }
                }

                let engine = engine_opt.as_mut().unwrap();

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
            return rx_resp.recv_timeout(std::time::Duration::from_secs(15))
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
            snap.buttons = res.elements.iter().map(|b| crate::engine::window_scanner::DesktopElement {
                id: b.id,
                label: b.label.clone(),
                role: b.role.clone(),
                center: Some(b.center),
                bbox: Some(b.bbox),
            }).collect();
        } else if !res.buttons.is_empty() {
            snap.buttons = res.buttons.iter().map(|b| crate::engine::window_scanner::DesktopElement {
                id: b.id,
                label: b.label.clone(),
                role: b.role.clone(),
                center: Some(b.center),
                bbox: Some(b.bbox),
            }).collect();
        }
        if !res.inputs.is_empty() {
            snap.inputs = res.inputs.iter().map(|inp| crate::engine::window_scanner::DesktopElement {
                id: inp.id,
                label: inp.label.clone(),
                role: inp.role.clone(),
                center: Some(inp.center),
                bbox: Some(inp.bbox),
            }).collect();
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
                eprintln!("[Perception] Gagal inisialisasi ONNX Perception Engine: {e}");
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
