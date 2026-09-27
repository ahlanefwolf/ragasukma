use image::DynamicImage;

#[cfg(target_os = "windows")]
pub fn run_windows_native_ocr(image: &DynamicImage) -> Result<Vec<(String, [i32; 4], f32)>, String> {
    use windows::{
        Graphics::Imaging::BitmapDecoder,
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    };

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image.write_to(&mut cursor, image::ImageOutputFormat::Png)
        .map_err(|e| format!("PNG encode error: {e}"))?;

    let stream = InMemoryRandomAccessStream::new()
        .map_err(|e| format!("InMemoryRandomAccessStream error: {e}"))?;
    let writer = DataWriter::CreateDataWriter(&stream)
        .map_err(|e| format!("DataWriter error: {e}"))?;
    writer.WriteBytes(&buffer)
        .map_err(|e| format!("WriteBytes error: {e}"))?;
    writer.StoreAsync()
        .map_err(|e| format!("StoreAsync error: {e}"))?
        .get()
        .map_err(|e| format!("StoreAsync get error: {e}"))?;
    writer.DetachStream()
        .map_err(|e| format!("DetachStream error: {e}"))?;
    stream.Seek(0)
        .map_err(|e| format!("Seek error: {e}"))?;

    let decoder = BitmapDecoder::CreateAsync(&stream)
        .map_err(|e| format!("BitmapDecoder error: {e}"))?
        .get()
        .map_err(|e| format!("BitmapDecoder get error: {e}"))?;

    let bitmap = decoder.GetSoftwareBitmapAsync()
        .map_err(|e| format!("GetSoftwareBitmap error: {e}"))?
        .get()
        .map_err(|e| format!("GetSoftwareBitmap get error: {e}"))?;

    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|e| format!("OcrEngine TryCreateFromUserProfileLanguages error: {e}"))?;

    let ocr_result = engine.RecognizeAsync(&bitmap)
        .map_err(|e| format!("RecognizeAsync error: {e}"))?
        .get()
        .map_err(|e| format!("RecognizeAsync get error: {e}"))?;

    let lines = ocr_result.Lines()
        .map_err(|e| format!("Lines error: {e}"))?;

    let mut detected = Vec::new();
    for line in lines {
        let words = match line.Words() {
            Ok(w) => w,
            Err(_) => continue,
        };
        for word in words {
            let text = word.Text().map(|h| h.to_string()).unwrap_or_default().trim().to_string();
            if text.is_empty() {
                continue;
            }
            if let Ok(rect) = word.BoundingRect() {
                let x = rect.X.round() as i32;
                let y = rect.Y.round() as i32;
                let w = rect.Width.round() as i32;
                let h = rect.Height.round() as i32;
                detected.push((text, [x, y, w, h], 1.0f32));
            }
        }
    }

    Ok(detected)
}

#[cfg(not(target_os = "windows"))]
pub fn run_windows_native_ocr(_image: &DynamicImage) -> Result<Vec<(String, [i32; 4], f32)>, String> {
    Err("Windows Native OCR hanya tersedia pada sistem operasi Windows 10/11.".to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_native_ocr_engine_available() {
        use windows::Media::Ocr::OcrEngine;
        let engine = OcrEngine::TryCreateFromUserProfileLanguages();
        assert!(engine.is_ok(), "Windows Native OcrEngine harus dapat diinisialisasi");
    }
}
