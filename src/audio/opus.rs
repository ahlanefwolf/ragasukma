use std::io::Write;
use std::process::{Command, Stdio};

/// Encode standard PCM WAV bytes to standard Ogg Opus format (RFC 7845) using system FFmpeg.
/// Uses 32 kbps VBR VoIP profile, reducing file size by ~95% while keeping clear speech.
pub fn encode_wav_to_opus(wav_bytes: &[u8]) -> Result<Vec<u8>, String> {
    if wav_bytes.is_empty() {
        return Err("Input WAV bytes are empty".to_string());
    }

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel", "error",
            "-i", "pipe:0",
            "-c:a", "libopus",
            "-b:a", "32k",
            "-vbr", "on",
            "-application", "voip",
            "-f", "ogg",
            "pipe:1",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(wav_bytes)
            .map_err(|e| format!("Failed to write to ffmpeg stdin: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("FFmpeg failed while encoding: {e}"))?;

    if output.status.success() && !output.stdout.is_empty() {
        Ok(output.stdout)
    } else {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("FFmpeg Opus encoding failed: {err_msg}"))
    }
}
