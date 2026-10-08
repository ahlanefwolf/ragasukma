use ogg::PacketWriteEndInfo;
use ogg::writing::PacketWriter;
use opus::{Application, Channels, Encoder};
use std::io::Cursor;

/// Parses basic standard WAV (RIFF/WAVE 16-bit signed PCM) in-memory.
/// Returns (sample_rate, channels, pcm_samples: Vec<i16>).
fn parse_wav_pcm16(wav_bytes: &[u8]) -> Result<(u32, u16, Vec<i16>), String> {
    if wav_bytes.len() < 44 {
        return Err("WAV data too short for standard header".to_string());
    }

    if &wav_bytes[0..4] != b"RIFF" || &wav_bytes[8..12] != b"WAVE" {
        return Err("Invalid WAV RIFF/WAVE header".to_string());
    }

    let mut pos = 12;
    let mut channels = 1u16;
    let mut sample_rate = 24000u32;
    let mut bits_per_sample = 16u16;
    let mut audio_format = 1u16; // 1 = PCM
    let mut pcm_bytes_range: Option<(usize, usize)> = None;

    while pos + 8 <= wav_bytes.len() {
        let chunk_id = &wav_bytes[pos..pos + 4];
        let chunk_size = u32::from_le_bytes(
            wav_bytes[pos + 4..pos + 8]
                .try_into()
                .map_err(|_| "Failed to read chunk size")?,
        ) as usize;
        let chunk_data_start = pos + 8;
        let chunk_data_end = (chunk_data_start + chunk_size).min(wav_bytes.len());

        if chunk_id == b"fmt " {
            if chunk_size >= 16 && chunk_data_end <= wav_bytes.len() {
                audio_format = u16::from_le_bytes(
                    wav_bytes[chunk_data_start..chunk_data_start + 2]
                        .try_into()
                        .unwrap(),
                );
                channels = u16::from_le_bytes(
                    wav_bytes[chunk_data_start + 2..chunk_data_start + 4]
                        .try_into()
                        .unwrap(),
                );
                sample_rate = u32::from_le_bytes(
                    wav_bytes[chunk_data_start + 4..chunk_data_start + 8]
                        .try_into()
                        .unwrap(),
                );
                bits_per_sample = u16::from_le_bytes(
                    wav_bytes[chunk_data_start + 14..chunk_data_start + 16]
                        .try_into()
                        .unwrap(),
                );
            }
        } else if chunk_id == b"data" {
            pcm_bytes_range = Some((chunk_data_start, chunk_data_end));
            break;
        }

        pos = chunk_data_start + chunk_size;
        // Word alignment padding if chunk_size is odd
        if !chunk_size.is_multiple_of(2) {
            pos += 1;
        }
    }

    if audio_format != 1 {
        return Err(format!("Unsupported WAV audio format: {audio_format} (only PCM=1 supported)"));
    }
    if bits_per_sample != 16 {
        return Err(format!("Unsupported bits per sample: {bits_per_sample} (only 16-bit supported)"));
    }

    let (start, end) = pcm_bytes_range.ok_or_else(|| "WAV 'data' chunk not found".to_string())?;
    let data_slice = &wav_bytes[start..end];

    let mut samples = Vec::with_capacity(data_slice.len() / 2);
    let (chunks, _) = data_slice.as_chunks::<2>();
    for chunk in chunks {
        let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
        samples.push(sample);
    }

    Ok((sample_rate, channels, samples))
}

/// Creates RFC 7845 standard OpusHead header packet (19 bytes).
fn create_opus_head(sample_rate: u32, channels: u8, pre_skip: u16) -> Vec<u8> {
    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead"); // Magic signature
    head.push(1); // Version = 1
    head.push(channels); // Channel count
    head.extend_from_slice(&pre_skip.to_le_bytes()); // Pre-skip (samples at 48kHz)
    head.extend_from_slice(&sample_rate.to_le_bytes()); // Original input sample rate
    head.extend_from_slice(&0i16.to_le_bytes()); // Output gain (Q7.8 dB)
    head.push(0); // Mapping family (0 = mono or stereo)
    head
}

/// Creates RFC 7845 standard OpusTags comment packet.
fn create_opus_tags() -> Vec<u8> {
    let vendor = b"Ragasukma Native Opus";
    let mut tags = Vec::with_capacity(8 + 4 + vendor.len() + 4);
    tags.extend_from_slice(b"OpusTags"); // Magic signature
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor);
    tags.extend_from_slice(&0u32.to_le_bytes()); // User comment list length = 0
    tags
}

/// Encode standard PCM WAV bytes to standard RFC 7845 Ogg Opus format (`audio/ogg; codecs=opus`)
/// completely in-memory using native Rust Opus encoder and Ogg packager.
///
/// Fully cross-platform: No FFmpeg or external runtime CLI required!
pub fn encode_wav_to_opus(wav_bytes: &[u8]) -> Result<Vec<u8>, String> {
    if wav_bytes.is_empty() {
        return Err("Input WAV bytes are empty".to_string());
    }

    // 1. Parse PCM 16-bit WAV
    let (sample_rate, channels_count, pcm_samples) = parse_wav_pcm16(wav_bytes)?;
    let channels = match channels_count {
        1 => Channels::Mono,
        2 => Channels::Stereo,
        _ => return Err(format!("Unsupported channel count: {channels_count}")),
    };

    // Opus internally supports: 8000, 12000, 16000, 24000, 48000 Hz.
    let opus_sample_rate = match sample_rate {
        8000 => 8000,
        12000 => 12000,
        16000 => 16000,
        24000 => 24000,
        48000 => 48000,
        other => {
            // Default closest high-quality rate or 48000 if not standard
            if other <= 16000 {
                16000
            } else if other <= 24000 {
                24000
            } else {
                48000
            }
        }
    };

    // 2. Initialize native Opus Encoder (VoIP speech profile for fast high-clarity voice)
    let mut encoder = Encoder::new(opus_sample_rate, channels, Application::Voip)
        .map_err(|e| format!("Failed to create Opus encoder: {e:?}"))?;

    // Set 32 kbps bitrate for voice efficiency
    let _ = encoder.set_bitrate(opus::Bitrate::Bits(32_000));
    let _ = encoder.set_vbr(true);

    // Frame size: 20ms of audio
    // e.g. 24000 Hz * 0.02s = 480 samples per channel
    let frame_samples_per_channel = (opus_sample_rate as usize * 20) / 1000;
    let frame_total_samples = frame_samples_per_channel * (channels_count as usize);

    // 3. Initialize Ogg Stream Writer
    let serial_no = rand_serial_no();
    let mut out_buffer = Vec::with_capacity(wav_bytes.len() / 6);
    let mut packet_writer = PacketWriter::new(Cursor::new(&mut out_buffer));

    // Page 1: OpusHead (Beginning of Stream)
    let pre_skip = 312u16;
    let opus_head = create_opus_head(opus_sample_rate, channels_count as u8, pre_skip);
    packet_writer
        .write_packet(opus_head, serial_no, PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| format!("Failed to write OpusHead packet: {e}"))?;

    // Page 2: OpusTags
    let opus_tags = create_opus_tags();
    packet_writer
        .write_packet(opus_tags, serial_no, PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| format!("Failed to write OpusTags packet: {e}"))?;

    // Page 3..N: Encoded Audio Frames
    // In RFC 7845, granule position is ALWAYS counted at 48 kHz clock rate
    // 20ms frame = 48000 * 0.02 = 960 granule samples per frame
    let granule_increment_per_frame = (48_000 * 20) / 1000; // 960
    let mut current_granule: u64 = pre_skip as u64;

    let mut out_frame_buf = vec![0u8; 1024];
    let num_chunks = pcm_samples.len().div_ceil(frame_total_samples);

    for (i, chunk) in pcm_samples.chunks(frame_total_samples).enumerate() {
        let is_last = i + 1 == num_chunks;

        let encoded_len = if chunk.len() == frame_total_samples {
            encoder
                .encode(chunk, &mut out_frame_buf)
                .map_err(|e| format!("Opus frame encode error: {e:?}"))?
        } else {
            // Zero-pad the trailing partial frame to fit standard 20ms frame size
            let mut padded = vec![0i16; frame_total_samples];
            padded[..chunk.len()].copy_from_slice(chunk);
            encoder
                .encode(&padded, &mut out_frame_buf)
                .map_err(|e| format!("Opus trailing frame encode error: {e:?}"))?
        };

        current_granule += granule_increment_per_frame;

        let end_info = if is_last {
            PacketWriteEndInfo::EndStream
        } else {
            PacketWriteEndInfo::NormalPacket
        };

        packet_writer
            .write_packet(
                out_frame_buf[..encoded_len].to_vec(),
                serial_no,
                end_info,
                current_granule,
            )
            .map_err(|e| format!("Failed to write Opus data packet: {e}"))?;
    }

    Ok(out_buffer)
}

fn rand_serial_no() -> u32 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(123456);
    nanos | 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_wav_to_opus() {
        // Create 20ms of dummy 24000Hz 16-bit mono PCM WAV
        let sample_rate = 24000u32;
        let num_samples = 480usize; // 20ms
        let mut wav_bytes = Vec::new();

        // RIFF header
        wav_bytes.extend_from_slice(b"RIFF");
        let file_size = (36 + num_samples * 2) as u32;
        wav_bytes.extend_from_slice(&file_size.to_le_bytes());
        wav_bytes.extend_from_slice(b"WAVE");

        // fmt chunk
        wav_bytes.extend_from_slice(b"fmt ");
        wav_bytes.extend_from_slice(&16u32.to_le_bytes()); // chunk size
        wav_bytes.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav_bytes.extend_from_slice(&1u16.to_le_bytes()); // 1 channel
        wav_bytes.extend_from_slice(&sample_rate.to_le_bytes()); // 24000
        wav_bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
        wav_bytes.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav_bytes.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

        // data chunk
        wav_bytes.extend_from_slice(b"data");
        wav_bytes.extend_from_slice(&((num_samples * 2) as u32).to_le_bytes());
        for i in 0..num_samples {
            let s = ((i as f32 * 0.1).sin() * 10000.0) as i16;
            wav_bytes.extend_from_slice(&s.to_le_bytes());
        }

        let res = encode_wav_to_opus(&wav_bytes);
        assert!(res.is_ok(), "Encoding failed: {:?}", res.err());
        let opus_data = res.unwrap();
        assert!(opus_data.starts_with(b"OggS"), "Output must be standard Ogg stream");
        assert!(opus_data.len() < wav_bytes.len(), "Opus compressed output should be smaller than WAV");
    }
}
