use image::DynamicImage;
use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct MacVisionResponse {
    boxes: Option<Vec<(String, [i32; 4], f32)>>,
    error: Option<String>,
}

/// Native OCR bawaan macOS menggunakan Apple Vision Framework (`VNRecognizeTextRequest`).
/// Memanfaatkan Apple Neural Engine / CoreML bawaan sistem macOS (0 MB model file, 0 MB alokasi RAM).
#[cfg(target_os = "macos")]
pub fn run_macos_native_ocr(image: &DynamicImage) -> Result<Vec<(String, [i32; 4], f32)>, String> {
    let temp_dir = std::env::temp_dir();
    let file_id = format!(
        "ragasukma_vision_{}_{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let temp_img_path = temp_dir.join(file_id);

    image
        .save(&temp_img_path)
        .map_err(|e| format!("Failed to save temporary screenshot for macOS Vision: {e}"))?;

    let swift_script = r#"
import Foundation
import Vision
import Cocoa

guard CommandLine.arguments.count > 1 else {
    print("{\"error\":\"No image path argument\"}")
    exit(1)
}

let imagePath = CommandLine.arguments[1]
guard let image = NSImage(contentsOfFile: imagePath),
      let cgImage = image.cgImage(forProposedRect: nil, context: nil, hints: nil) else {
    print("{\"error\":\"Failed to open cgImage\"}")
    exit(1)
}

let imgW = CGFloat(cgImage.width)
let imgH = CGFloat(cgImage.height)

let request = VNRecognizeTextRequest { request, error in
    if let err = error {
        print("{\"error\":\"\(err.localizedDescription)\"}")
        return
    }

    guard let observations = request.results as? [VNRecognizedTextObservation] else {
        print("{\"boxes\":[]}")
        return
    }

    var output: [[Any]] = []

    for obs in observations {
        guard let candidate = obs.topCandidates(1).first else { continue }
        let text = candidate.string.trimmingCharacters(in: .whitespacesAndNewlines)
        if text.isEmpty { continue }

        // Apple Vision menggunakan sistem koordinat Normalized Bottom-Left (0.0 - 1.0)
        // Kita konversikan ke sistem koordinat Top-Left Pixel standar
        let box = obs.boundingBox
        let bx = Int(round(box.origin.x * imgW))
        let bw = Int(round(box.size.width * imgW))
        let bh = Int(round(box.size.height * imgH))
        let by = Int(round((1.0 - box.origin.y - box.size.height) * imgH))
        let conf = Float(candidate.confidence)

        output.append([text, [bx, by, bw, bh], conf])
    }

    if let jsonData = try? JSONSerialization.data(withJSONObject: ["boxes": output]),
       let jsonString = String(data: jsonData, encoding: .utf8) {
        print(jsonString)
    } else {
        print("{\"boxes\":[]}")
    }
}

request.recognitionLevel = .accurate
request.usesLanguageCorrection = true

let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])
do {
    try handler.perform([request])
} catch {
    print("{\"error\":\"\(error.localizedDescription)\"}")
}
"#;

    let path_str = temp_img_path.to_string_lossy().to_string();

    let output = Command::new("swift")
        .args(["-e", swift_script, &path_str])
        .output();

    let _ = fs::remove_file(&temp_img_path);

    let res = output.map_err(|e| format!("Failed to call swift Apple Vision: {e}"))?;
    let stdout_str = String::from_utf8_lossy(&res.stdout).trim().to_string();

    if let Ok(resp) = serde_json::from_str::<MacVisionResponse>(&stdout_str) {
        if let Some(err) = resp.error {
            return Err(format!("Apple Vision error: {err}"));
        }
        if let Some(boxes) = resp.boxes {
            return Ok(boxes);
        }
    }

    Err("Output Apple Vision tidak sesuai format JSON yang diharapkan".to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn run_macos_native_ocr(_image: &DynamicImage) -> Result<Vec<(String, [i32; 4], f32)>, String> {
    Err("Apple Vision OCR hanya tersedia pada sistem operasi macOS.".to_string())
}
