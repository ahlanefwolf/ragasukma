# 📋 Changelog Ragasukma

Semua pembaruan penting, fitur baru, dan perbaikan bug pada aplikasi **Ragasukma AI Voice Assistant** didokumentasikan di file ini.

Format pencatatan mengacu pada [Keep a Changelog](https://keepachangelog.com/id/1.0.0/) dan menganut [Semantic Versioning](https://semver.org/).

---

## [v0.2.0] - 2026-09-20

### 🚀 Fitur Baru & Peningkatan Utama
- **Pure Web Speech Engine (Zero Backend Audio Overhead)**:
  - Seluruh inferensi STT dan sintesis TTS dialihkan murni ke Web Speech API bawaan Chromium (`webkitSpeechRecognition` & `window.speechSynthesis`).
  - Menghapus total modul berat backend (`supertonic.rs`, `dsp.rs`, `recorder.rs`, `stt.rs`, dan buffer VAD berulang) sehingga backend 100% idle saat pengguna diam.
  - Dukungan aksen pengenalan suara multi-bahasa lokal (`id-ID`, `jv-ID`, `su-ID`, dll.) serta kontrol vokal fleksibel (gender, pitch, speed, voice name).
  - Web Audio API Spectrum & VU meter 60 FPS langsung di browser tanpa overhead jaringan.
- **Strict Single-Tab Ownership & Clean Teardown**:
  - Eliminasi total bug audio dobel: tab lama yang tereliminasi seketika memanggil `window.speechSynthesis.cancel()`, `recognition.abort()`, dan memutuskan SSE stream (`forceCloseSSE`).
  - Deduplikasi incremental ID pada SSE action buffer untuk memastikan kalimat ucapan tidak pernah dieksekusi berulang.
- **Ekspresi Wajah 3D & Lip-Sync Terpadu**:
  - Penyelarasan parser emosi multi-bahasa (`*senang*`, `*malu*`, `*bingung*`, `*kaget*`, `*marah*`, `*semangat*`, `*sedih*`) langsung ke morph targets / blendshapes VRM di renderer WGPU desktop.
  - Perbaikan aturan prompt LLM: melarang pengulangan pertanyaan basa-basi, membatasi obrolan santai tetap padat (1-2 kalimat lisan), namun tetap leluasa menjelaskan/merangkum konten web secara mendalam saat diminta.
- **Penyempurnaan Companion Extension**:
  - Ekstraktor DOM (`dom_extractor.js`) kini menyaring navigasi/header/footer dan fokus mengekstrak teks konten artikel / narasi aktual (`text_summary`).
  - Pembaruan paket distribusi `assets/ragasukma-companion.zip`.
- **Antigravity Pool & Local OpenAI-Compatible API Router (`:42128/v1`)**:
  - Dukungan multi-akun Google Multi-OAuth dengan pooling otomatis dan failover kuota cerdas saat mencapai batas rate limit.
  - Endpoint universal standar OpenAI-compatible (`http://127.0.0.1:42128/v1/chat/completions` & `/v1/models`) yang memungkinkan integrasi langsung dengan Cursor, VS Code, Cline, OpenWebUI, dan third-party tools lainnya.
  - Antigravity Quota Manager, model catalog viewer, dan token management instan 1-klik di dashboard.
- **In-Page Universal Browser DOM Automation & Precision Click Engine**:
  - *Ground of Truth* active tab snapshot DOM reader yang menyaring konten layar aktif browser tanpa halusinasi.
  - Engine klik cerdas (`click_engine.js`) dengan penelusuran hierarki DOM mendalam (shadow DOM traversal, closest `<a>` & `<button>` navigation, focus handling, dan fallback navigasi URL otomatis).
  - Canonical actions chaining untuk mengetik teks dan mengeksekusi enter/search di web (`[CMD:browser:type]`, `[CMD:browser:enter]`, `[CMD:browser:click_text]`, `[CMD:browser:click_button]`).
- **HiDPI Retina Picture-in-Picture (PiP) Widescreen Mini-Player**:
  - Mini-player melayang berdesain horizontal widescreen (680 × 210 px) yang dirender pada resolusi **2x HiDPI / Retina (1360 × 420 px)** untuk teks dan artwork yang super jernih.
  - Integrasi **Dedicated Web Worker Timer Loop (30 FPS)** yang kebal terhadap background tab throttling Chromium—membuat waveform equalizer, timeline progress, dan kontrol play/pause tetap tersinkronisasi 100% secara realtime saat browser diminimize atau dipindah ke background.
- **Overhaul Desain Studio Zero-Border-Radius & Tipografi Premium**:
  - Transformasi total UI dashboard web dengan pendekatan *Zero Radius (Crisp Edges)* dan hairline border halus `#2F3B46`.
  - Migrasi seluruh font sistem ke keluarga kelas dunia **IBM Plex Sans** (UI & Heading) dan **IBM Plex Mono** (Console log, durasi, status badge) untuk legibilitas maksimal dan estetika anti-AI slop.

### 🐛 Perbaikan Bug & Optimasi Performa
- **Pembersihan Modul & Standarisasi Aset**: Pembersihan seluruh referensi usang `char-avatar-talking` ke standar `char-avatar-speaking`, serta penataan struktur internal `chat_loop`.
- **Zero Race Condition Audio Pipeline**: Mengeliminasi ketergantungan audio decode ganda di backend dan mengandalkan single-source-of-truth streaming langsung ke frontend web & PiP.
- **Pembersihan Elemen Dekoratif & Focus Outline**: Menghilangkan outline 2px tebal dan box shadow berlebih, menggantinya dengan border highlight 1px yang bersih dan responsif.
- **Konsumsi Sumber Daya Minimal**: Render loop PiP dan Web Worker hanya mengonsumsi < 0.5% CPU dan ~3 MB RAM, dengan pembersihan memori otomatis (`terminate()`) saat ditutup.
- **Kebersihan Kode**: Diagnostik proyek 0 error dan 0 warning di seluruh backend Rust dan frontend TypeScript.

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
