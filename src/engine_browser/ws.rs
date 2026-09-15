use std::io::{Read, Write};
use std::net::TcpStream;
use super::manager::get_browser_manager;
use super::types::BrowserTabInfo;

/// Helper untuk WebSocket Handshake (RFC 6455)
pub fn handle_websocket_upgrade(mut stream: TcpStream, header_str: &str) {
    let mut sec_key = String::new();
    for line in header_str.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("sec-websocket-key:") {
            if let Some(val) = line.split(':').nth(1) {
                sec_key = val.trim().to_string();
            }
        }
    }

    if sec_key.is_empty() {
        return;
    }

    // RFC 6455 Sec-WebSocket-Accept generation
    let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let combined = format!("{}{}", sec_key, magic);
    let mut hasher = Sha1::new();
    hasher.update(combined.as_bytes());
    let hash_result = hasher.finalize();
    let accept_key = base64_encode(&hash_result);

    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_key
    );

    if stream.write_all(response.as_bytes()).is_err() {
        return;
    }

    let stream_clone = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };

    // Daftarkan koneksi ke manager
    {
        let mgr = get_browser_manager();
        let mut lock = mgr.lock().unwrap();
        lock.active_connections.push(stream_clone);
    }

    println!("🌐 [Browser Companion] Tab browser berhasil terhubung ke AI Assistant!");

    // Loop baca frame WebSocket dari browser
    let mut reader_stream = stream;
    loop {
        match read_websocket_frame(&mut reader_stream) {
            Ok(Some(msg)) => {
                if let Ok(tab_info) = serde_json::from_str::<BrowserTabInfo>(&msg) {
                    let mgr = get_browser_manager();
                    let mut lock = mgr.lock().unwrap();
                    println!("📄 [Tab Aktif]: {} ({})", tab_info.title, tab_info.domain);
                    lock.active_tabs.insert(tab_info.domain.clone(), tab_info);
                }
            }
            Ok(None) => continue,
            Err(_) => {
                println!("🔌 [Browser Companion] Koneksi tab browser terputus.");
                break;
            }
        }
    }
}

/// Minimal SHA1 implementation untuk WebSocket handshake
struct Sha1 {
    h: [u32; 5],
    len: u64,
    buf: [u8; 64],
    buf_len: usize,
}

impl Sha1 {
    fn new() -> Self {
        Self {
            h: [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0],
            len: 0,
            buf: [0; 64],
            buf_len: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.len += data.len() as u64;
        while !data.is_empty() {
            let need = 64 - self.buf_len;
            if data.len() >= need {
                self.buf[self.buf_len..64].copy_from_slice(&data[..need]);
                self.process_block();
                data = &data[need..];
                self.buf_len = 0;
            } else {
                self.buf[self.buf_len..self.buf_len + data.len()].copy_from_slice(data);
                self.buf_len += data.len();
                break;
            }
        }
    }

    fn process_block(&mut self) {
        let mut w = [0u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                self.buf[i * 4],
                self.buf[i * 4 + 1],
                self.buf[i * 4 + 2],
                self.buf[i * 4 + 3],
            ]);
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }

        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };
            let temp = a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
    }

    fn finalize(mut self) -> [u8; 20] {
        let bit_len = self.len * 8;
        self.buf[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            self.buf[self.buf_len..64].fill(0);
            self.process_block();
            self.buf_len = 0;
        }

        self.buf[self.buf_len..56].fill(0);
        self.buf[56..64].copy_from_slice(&bit_len.to_be_bytes());
        self.process_block();

        let mut out = [0u8; 20];
        for (i, &v) in self.h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes());
        }
        out
    }
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

pub fn make_websocket_text_frame(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut frame = Vec::new();
    frame.push(0x81); // FIN + Text Frame opcode

    let len = bytes.len();
    if len <= 125 {
        frame.push(len as u8);
    } else if len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }

    frame.extend_from_slice(bytes);
    frame
}

pub fn read_websocket_frame(stream: &mut TcpStream) -> std::io::Result<Option<String>> {
    let mut header = [0u8; 2];
    stream.read_exact(&mut header)?;

    let is_masked = (header[1] & 0x80) != 0;
    let mut payload_len = (header[1] & 0x7F) as usize;

    if payload_len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext)?;
        payload_len = u16::from_be_bytes(ext) as usize;
    } else if payload_len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext)?;
        payload_len = u64::from_be_bytes(ext) as usize;
    }

    // Safety limit: 2MB max frame to prevent RAM exhaustion / OOM
    if payload_len > 2 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "WebSocket payload frame too large",
        ));
    }

    let mask = if is_masked {
        let mut m = [0u8; 4];
        stream.read_exact(&mut m)?;
        Some(m)
    } else {
        None
    };

    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload)?;

    if let Some(mask) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    Ok(String::from_utf8(payload).ok())
}
