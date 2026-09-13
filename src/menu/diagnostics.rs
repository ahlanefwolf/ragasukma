use anyhow::Result;
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::dsp::TARGET_SAMPLE_RATE;
use super::session::AppSession;

impl AppSession {
    pub fn test_assistant_voice(&self) -> Result<()> {
        let name = &self.config.assistant_name;
        crate::term_log!("\n[TES SUARA {}] (Tekan ESC / SPASI kapan saja untuk kembali ke menu)", name.to_uppercase());
        let callsign = &self.config.user_callsign;
        let test_phrase = format!("Halo {callsign}, suara {name} sudah siap menemani {callsign} hari ini!");
        crate::term_log!("{name}: \"{}\"", test_phrase);
        let test_interrupt = Arc::new(AtomicBool::new(false));
        self.tts.speak_interruptible(&test_phrase, test_interrupt)?;
        crate::term_log!("----------------------------------------------------\n");
        Ok(())
    }

    pub fn test_mic_dsp(&self) -> Result<()> {
        crate::term_log!("\n[TES MIC & FILTER SUARA]");
        crate::term_log!("   Silakan bicara selama 3 detik ke mic... (Tes noise reduction & normalisasi)");
        let is_recording_test = Arc::new(AtomicBool::new(true));
        let is_recording_test_clone = Arc::clone(&is_recording_test);
        let recorder_test = Arc::clone(&self.recorder);

        let record_handle = std::thread::spawn(move || {
            recorder_test.record_turn(is_recording_test_clone)
        });

        std::thread::sleep(std::time::Duration::from_millis(3200));
        is_recording_test.store(false, Ordering::SeqCst);

        let audio_samples = record_handle
            .join()
            .map_err(|_| anyhow::anyhow!("Test thread panicked"))??;

        if audio_samples.is_empty() {
            crate::term_log!("⚠️  Tidak ada sampel suara yang terekam.");
        } else {
            crate::term_log!("▶️  Memutar balik suara kamu setelah melalui DSP Noise Reduction Filter...");
            let (_stream, stream_handle) = OutputStream::try_default()?;
            let sink = Sink::try_new(&stream_handle)?;
            let source = SamplesBuffer::new(1, TARGET_SAMPLE_RATE, audio_samples);
            sink.append(source);
            sink.sleep_until_end();
            crate::term_log!("✅ Selesai memutar balik hasil filter suara.");
        }
        crate::term_log!("----------------------------------------------------\n");
        Ok(())
    }
}
