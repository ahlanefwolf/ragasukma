# 📋 Changelog Ragasukma

Semua pembaruan penting, fitur baru, dan perbaikan bug pada aplikasi **Ragasukma AI Voice Assistant** didokumentasikan di file ini.

Format pencatatan mengacu pada [Keep a Changelog](https://keepachangelog.com/id/1.0.0/) dan menganut [Semantic Versioning](https://semver.org/).

---

## [v0.2.0] - 2026-09-20

### 🚀 Fitur Baru & Peningkatan Utama
- **Supertonic-3 Native Rust TTS Runtime (Pure ONNX, Zero Python)**:
  - Mengintegrasikan engine sintesis suara Supertonic-3 secara native di Rust menggunakan `ort v2.0.0-rc.13` (ONNX Runtime) tanpa runtime/subproses Python.
  - Memuat 10 gaya suara studio alami (5 Pria: M1..M5, 5 Wanita: F1..F5) dengan preloading memori untuk latensi instan.
  - Menyediakan endpoint streaming audio WAV 24kHz (`/api/tts/audio` dan `/api/tts/synthesize`) yang disalurkan langsung ke frontend web dashboard dan smartphone client.
  - Pembersihan total (100%) Google Chrome TTS (`window.speechSynthesis` & `SpeechSynthesisUtterance`), mengeliminasi bug PipeWire audio deadlock di Linux dan issue garbage collection V8 Chromium.
- **Chromium Native Web STT & Sinkronisasi Bahasa Terpadu**:
  - Mempertahankan akurasi tinggi Google Chrome Web Speech API (`webkitSpeechRecognition`) di sisi client dengan opsi bahasa tersinkronisasi via properti tunggal `lang` (`id-ID`, `en-US`, `ja-JP`, dll).
  - Mekanisme VAD Barge-in instan yang memutus pemutaran audio WAV saat pengguna mulai berbicara kembali, serta filter anti-echolalia terpadu.
- **Three-VRM Spring Bone Physics & Ekspresi Wajah 3D**:
  - Implementasi penuh simulasi fisika pegas rambut dan pakaian (Verlet Spring Bone) di Rust (`src/ui/spring_bone.rs`) yang 1:1 kompatibel dengan spesifikasi Three-VRM.
  - Evaluasi tabrakan dynamic sphere collider, gravitasi, stiffness, dan redaman (damping) langsung di pipeline rendering WGPU.
  - Penyelarasan parser emosi kanonikal 8 emosi (`*senang*`, `*malu*`, `*semangat*`, `*kaget*`, `*mikir*`, `*sedih*`, `*marah*`, `*netral*`) dengan dynamic auto-blink dan lip-sync vokal AIUEO.
- **Native OS OCR & Modular Perception Engine**:
  - Menghapus total modul Python PaddleOCR (`ocr_paddle.rs`).
  - Mengadopsi OCR native sistem operasi: Apple Vision OCR di macOS, Windows Media OCR di Windows, dan streaming pipa in-memory Tesseract TSV di Linux.
- **Antigravity Pool & Local OpenAI-Compatible API Router (`:42128/v1`)**:
  - Dukungan multi-akun Google Multi-OAuth dengan pooling otomatis, caching thought signature, dan failover kuota cerdas saat mencapai batas rate limit.
  - Endpoint universal standar OpenAI-compatible (`http://127.0.0.1:42128/v1/chat/completions` & `/v1/models`) yang kompatibel langsung dengan Cursor, VS Code, Cline, OpenWebUI, dan third-party tools.
- **Multiplatform Desktop Precision Click & Fallback Visual OCR Engine**:
  - Deteksi elemen antarmuka desktop berbasis accessibility tree (UI Automation di Windows, AT-SPI D-Bus di Linux, System Events di macOS) kini mengekstrak elemen tombol dan teks secara terpadu.
  - Fallback otomatis ke Visual OCR perception scan (`request_vision_scan_sync`) jika accessibility tree tidak menyertakan koordinat absolut atau target berada pada UI non-native (canvas, game, electron tanpa bridge accessibility).
  - Eksekusi aksi `[CMD:desktop:click_text|...]` dan `[CMD:desktop:click_button|...]` berakurasi tinggi dengan koordinat aktual pixel monitor dan toleransi typo berbasis fuzzy string similarity.
- **Defensive LLM Tool Call Guard & Multi-Language System Prompt**:
  - Pencegahan halusinasi format XML tool call (`<dots_function_call>`, `<invoke>`, `<parameter>`) dari model keluarga Claude/Anthropic di `prompt.rs`.
  - Filter defensif `RE_XML_TOOL_CALL` di `cleaner.rs` untuk membersihkan tag XML tak terduga agar audio TTS bebas dari artefak kode.
  - Standardisasi 100% system prompt internal ke Bahasa Inggris universal yang kompatibel dan adaptif terhadap 30 bahasa dunia di sisi percakapan pengguna.
  - Sinkronisasi format `assets/todo.md` ke Bahasa Inggris (`## ⏳ In Progress / To-Do` & `## ✅ Done`) selaras 1:1 dengan parser engine Rust.
- **HiDPI Retina Picture-in-Picture (PiP) Widescreen Mini-Player**:
  - Mini-player melayang berdesain horizontal widescreen (680 × 210 px) yang dirender pada resolusi **2x HiDPI / Retina (1360 × 420 px)** untuk teks dan artwork yang super jernih.
  - Integrasi **Dedicated Web Worker Timer Loop (30 FPS)** yang kebal terhadap background tab throttling Chromium.
- **Overhaul Desain Studio Zero-Border-Radius & Tipografi Premium**:
  - Transformasi total UI dashboard web dengan pendekatan *Zero Radius (Crisp Edges)* dan hairline border halus `#2F3B46`.
  - Migrasi seluruh font sistem ke keluarga **IBM Plex Sans** dan **IBM Plex Mono**.
- **Standar Kualitas & Verifikasi Kode Terpadu (Docs & Linters 100% Bersih)**:
  - Dokumentasi standar verifikasi kode di [`docs/quality.md`](docs/quality.md).
  - Konfigurasi flat modern ESLint 9+ (`web/eslint.config.js`) dan deteksi dead code (`web/knip.json`).
  - Lolos 100% pemeriksaan: `cargo clippy && cargo fmt --check && (cd web && npx eslint . && npx tsc --noEmit && npx knip && npx jscpd .)`.

### 🐛 Perbaikan Bug & Optimasi Performa
- **Pembersihan Modul Audio Usang**: Menghapus seluruh dependensi subprocess Python audio, speech synthesis worker browser lama, dan file duplikat.
- **Zero Race Condition Audio Pipeline**: Mengeliminasi ketergantungan audio decode ganda di backend dan mengandalkan streaming WAV Supertonic single-source-of-truth langsung ke frontend web & mobile client.
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
