# 📋 Changelog Ragasukma

Semua pembaruan penting, fitur baru, dan perbaikan bug pada aplikasi **Ragasukma AI Voice Assistant** didokumentasikan di file ini.

Format pencatatan mengacu pada [Keep a Changelog](https://keepachangelog.com/id/1.0.0/) dan menganut [Semantic Versioning](https://semver.org/).

---

## [v0.1.2] - 2026-09-14

### ✨ Fitur Baru & Peningkatan
- **Auto-Launch Default Browser**: Aplikasi otomatis membuka web dashboard di browser default sistem saat binary pertama kali dijalankan.
- **Port Dashboard Terisolasi (`42128`)**: Migrasi port server lokal ke `42128` untuk mencegah konflik port dengan service lokal lain.
- **Sistem Cek Pembaruan Terintegrasi**: Tombol *Periksa Pembaruan* kini terhubung langsung ke GitHub Releases API untuk mengecek versi dan menampilkan changelog / rilis terbaru.
- **Transisi & Kecepatan Pose Alami**: Sinkronisasi interpolasi tulang WGPU dan siklus pose A ⇄ B mengikuti parameter `speed` dari `poses.json`.
- **Ekspresi Emosi Dinamis**: Sinkronisasi emosi wajah avatar (blendshapes/morph targets) langsung dari respon tag LLM (`[EMO:emotion]`).
- **Live Mic Audio Monitor**: Fitur pemantauan suara mikrofon langsung dari browser dengan DSP noise gate filter.

### 🐛 Perbaikan Bug & Optimasi
- Mengatasi glitch tombol kontrol status di dashboard dengan pemisahan status visual WGPU dan status loop recorder.
- Memperbaiki kompilasi `MeshPrimitiveData` dan `VrmModelData` pada loader VRM.
- Efisiensi alokasi memori RAM (konsumsi stabil di ~200 MB).

---

## [v0.1.1] - 2026-09-10

### ✨ Fitur Awal
- Engine Native WGPU 3D Cel-shaded VRM 0.x / 1.0 Overlay.
- Continuous Microphone VAD (Voice Activity Detection) dengan Noise Reduction DSP & High-Pass Filter.
- STT Terintegrasi (Google Speech Recognition & Groq Cloud Whisper).
- LLM Streaming Client (OpenAI-compatible & Groq).
- TTS Terintegrasi (Google Translate TTS & ElevenLabs Multilingual v2).
- Sistem Skill Command Registry (`assets/skill-command.md`).
- Multiplatform Packaging (Linux `.deb` & `.AppImage`, Windows `.exe`, macOS `.dmg`).
