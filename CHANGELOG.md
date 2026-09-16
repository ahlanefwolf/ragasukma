# 📋 Changelog Ragasukma

Semua pembaruan penting, fitur baru, dan perbaikan bug pada aplikasi **Ragasukma AI Voice Assistant** didokumentasikan di file ini.

Format pencatatan mengacu pada [Keep a Changelog](https://keepachangelog.com/id/1.0.0/) dan menganut [Semantic Versioning](https://semver.org/).

---

## [v0.1.4] - 2026-09-16

### ✨ Fitur Baru & Peningkatan
- **Native yt-dlp Music Engine & Offline Playback**:
  - Download otomatis audio berkualitas tinggi (format m4a/aac) via engine `yt-dlp` terintegrasi.
  - Caching & deteksi cerdas koleksi lagu lokal di folder `assets/music/` (tanpa download ulang jika lagu sudah tersedia).
  - Penghapusan ketergantungan browser companion script/ekstensi untuk pemutaran audio.
- **Embedded Web Music Player & Floating Bottom Bar**:
  - Tab baru **Music Player** di web dashboard dengan visualisasi daftar lagu, thumbnail, durasi, dan kontrol hapus file.
  - **Floating Bottom Mini-Player** yang persisten di seluruh tab dashboard (Play, Pause, Seek, Volume control, Loop mode).
- **Strict Single-Tab Session Takeover**:
  - Mekanisme pengambilalihan sesi secara instan saat tab baru dibuka.
  - Tab lama otomatis disconnect dan dinonaktifkan tanpa modal alert yang mengganggu.
- **Modern UI & Typographic Refinement**:
  - Penggunaan tipografi **Space Grotesk** dan palet tema CSS modern.
  - Pembersihan menyeluruh dependensi Cargo (`rodio` dioptimasi hanya untuk `symphonia-mp3`, penghapusan crate tak terpakai).

### 🐛 Perbaikan Bug & Optimasi
- **Zero Race Condition Audio Pipeline**: Koordinasi mikrofon VAD dan pemutaran audio/TTS dengan atomic flag murni dan pengurasan buffer instan.
- **Safe Teardown**: Semua interval, subscription, dan abort controller di frontend SolidJS dibersihkan via `onCleanup`.
- **Git Ignore Safeguard**: Direktori `assets/music/` dan file audio unduhan kini terisolasi sepenuhnya dari tracking Git repository.

---

## [v0.1.3] - 2026-09-16

### ✨ Fitur Baru & Peningkatan
- **Modular Architecture Overhaul**:
  - Refaktor arsitektur modular backend Rust: `src/server/` terpisah rapi (`assets`, `handlers`, `state`), dan `src/engine/` (`skills`, `system_actions`).
  - Pembersihan modul WhatsApp agar fokus penuh pada integrasi YouTube, YouTube Music, web navigasi, dan launcher aplikasi OS.
- **Gestur Kontrol Kamera Layar 3D Lebih Presisi**:
  - **Drag Kanan**: Orbit rotasi 3D.
  - **Ctrl + Drag Kanan**: Pan / geser posisi 3D.
  - **Scroll**: Zoom In / Zoom Out.
  - **Klik Tengah**: Reset kamera ke default.
- **Universal Browser Companion**:
  - 1-Click install script web companion untuk kontrol YouTube & YouTube Music langsung via suara AI.
- **Atomic Single-Client Session & Instant Revoke**:
  - Proteksi anti-double tab menggunakan Server-Sent Events (SSE) token broadcast (<50ms instant lock).
  - Mekanisme tombol *Ambil Alih* aman bebas race condition.
- **In-Flight Lock & Throttled Pose Tuner**:
  - Sinkronisasi real-time slider pose ditata ulang (max 20 req/dtk) dengan trailing-edge queue, mengeliminasi error Firefox `NS_BINDING_ABORT`.

### 🐛 Perbaikan Bug & Optimasi
- Mengatasi tabrakan (race condition) antara Live Pose Tuner dan aktivasi sesi percakapan obrolan (chat loop).
- Sinkronisasi dinamis konfigurasi audio pada `AudioRecorder` tanpa memutus/merusak stream thread mikrofon di background.
- Proteksi payload limit HTTP (100MB max) dan WebSocket (2MB max) untuk mencegah OOM/crash.
- Optimalisasi alokasi memori RAM dan penanganan buffer audio.

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
