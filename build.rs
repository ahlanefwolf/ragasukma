use std::fs;
use std::path::Path;

// CRC32 implementation for PNG chunks
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

// Adler32 checksum for zlib wrapper
fn adler32(data: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

fn create_rgba_png(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
    let row_len = 1 + (width as usize * 4);
    let mut raw_data = Vec::with_capacity(row_len * height as usize);
    for _ in 0..height {
        raw_data.push(0); // None filter
        for _ in 0..width {
            raw_data.push(r);
            raw_data.push(g);
            raw_data.push(b);
            raw_data.push(a);
        }
    }

    // Wrap raw_data in zlib stream with non-compressed DEFLATE blocks (max 65535 bytes per block)
    let mut zlib_data = Vec::new();
    zlib_data.push(0x78); // CMF: Deflate, 32K window
    zlib_data.push(0x01); // FLG: Check bits

    let chunks: Vec<&[u8]> = raw_data.chunks(65535).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        zlib_data.push(if is_last { 0x01 } else { 0x00 });
        let len = chunk.len() as u16;
        let nlen = !len;
        zlib_data.extend_from_slice(&len.to_le_bytes());
        zlib_data.extend_from_slice(&nlen.to_le_bytes());
        zlib_data.extend_from_slice(chunk);
    }

    let adler = adler32(&raw_data);
    zlib_data.extend_from_slice(&adler.to_be_bytes());

    // Assemble PNG
    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]); // PNG Header

    // IHDR
    let mut ihdr_body = Vec::new();
    ihdr_body.extend_from_slice(&width.to_be_bytes());
    ihdr_body.extend_from_slice(&height.to_be_bytes());
    ihdr_body.push(8); // Bit depth: 8
    ihdr_body.push(6); // Color type: RGBA (6)
    ihdr_body.push(0); // Compression method
    ihdr_body.push(0); // Filter method
    ihdr_body.push(0); // Interlace method

    append_chunk(&mut png, b"IHDR", &ihdr_body);
    append_chunk(&mut png, b"IDAT", &zlib_data);
    append_chunk(&mut png, b"IEND", &[]);

    png
}

fn append_chunk(png: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    let len = data.len() as u32;
    png.extend_from_slice(&len.to_be_bytes());
    png.extend_from_slice(chunk_type);
    png.extend_from_slice(data);

    let mut crc_input = Vec::with_capacity(4 + data.len());
    crc_input.extend_from_slice(chunk_type);
    crc_input.extend_from_slice(data);
    let crc_val = crc32(&crc_input);
    png.extend_from_slice(&crc_val.to_be_bytes());
}

fn ensure_icons() {
    let icons_dir = Path::new("icons");
    let _ = fs::create_dir_all(icons_dir);

    let png_32 = create_rgba_png(32, 32, 255, 148, 184, 255);
    let png_128 = create_rgba_png(128, 128, 255, 148, 184, 255);

    let _ = fs::write("icons/32x32.png", &png_32);
    let _ = fs::write("icons/128x128.png", &png_128);
    let _ = fs::write("icons/128x128@2x.png", &png_128);
    let _ = fs::write("icons/icon.png", &png_128);
}

fn main() {
    ensure_icons();
    tauri_build::build();
}
