<div align="center">

  <img src="icons/128x128@2x.png" alt="Ragasukma Logo" width="120" height="120" />

  # Ragasukma
  **Native Desktop AI Voice Assistant & 3D Avatar Companion**

  [![Release](https://img.shields.io/github/v/release/ahlanefwolf/ragasukma?style=flat-square&color=7BDFF2)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![Platform](https://img.shields.io/badge/Platform-Linux_%7C_Windows_%7C_macOS-2F3B46?style=flat-square)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![License](https://img.shields.io/badge/License-Proprietary-F2B5D4?style=flat-square)](https://github.com/ahlanefwolf/ragasukma)

  <p align="center">
    Asisten suara AI desktop berbahasa Indonesia dengan avatar 3D/2D transparent overlay, automasi browser in-page, local music player, dan local OpenAI-compatible LLM router.
  </p>

</div>

---

## Fitur & Kemampuan Sistem

### 1. Interaksi Suara & Visual 3D
- **Transparent 3D VRM Overlay**: Engine rendering native WGPU dengan shader MToon 1:1, sinkronisasi ekspresi wajah (blendshapes/morph targets) multi-bahasa (*senang*, *malu*, *bingung*, *kaget*, *marah*, *semangat*), dan tuning pose realtime.
- **Pure Web Speech Architecture (Zero Backend Audio Overhead)**:
  - STT Native Chromium (`webkitSpeechRecognition`) berakurasi tinggi dengan dukungan multi-bahasa daerah (Bahasa Indonesia `id-ID`, Basa Jawa `jv-ID`, Basa Sunda `su-ID`, English, Japanese).
  - TTS Native Chromium (`window.speechSynthesis`) dengan kendali pitch, speed slider, pemilihan gender/vokal suara, dan pelestarian angka ribuan serta desimal.
  - Zero mic hardware locks: Backend bebas dari beban recorder, buffer VAD, dan model audio berat.
  - Visualizer Spectrum & VU Meter 60 FPS langsung via Web Audio API browser.
- **Barge-in Voice Interruption & Strict Single-Tab Ownership**:
  - Interupsi instan memotong TTS saat pengguna berbicara kembali.
  - Mekanisme eliminasi tab ganda yang otomatis menghentikan mic, membatalkan antrean TTS, dan menutup SSE stream tab lama untuk mencegah suara ganda.

### 2. Browser DOM Automation & Click Engine
- **Active Tab DOM Ground-of-Truth**: Analisis visual dan elemen interaktif tab aktif secara realtime untuk eksekusi perintah tanpa halusinasi.
- **In-Page Navigation**:
  - `browser:open` — Membuka atau berpindah ke tab website tertentu.
  - `browser:search` — Melakukan pencarian di situs target atau pencarian in-page.
  - `browser:type` & `browser:enter` — Mengetikkan teks ke form/input aktif dan mengeksekusi enter.
  - `browser:click_text` & `browser:click_button` — Menemukan dan mengklik teks, tautan, atau tombol interaktif.
  - `browser:switch` & `browser:close` — Manajemen tab aktif di browser.

### 3. Pemutar Musik & HiDPI PiP
- **Automated YouTube Download & Playback**: Unduh dan putar audio lokal otomatis di latar belakang via `yt-dlp` (`media:play`, `media:pause`, `media:resume`, `media:next`, `media:prev`, `media:stop`).
- **Retina Picture-in-Picture (PiP)**: Mini-player melayang widescreen (680x210 @ 2x HiDPI) dengan Dedicated Web Worker timer (30 FPS) yang tetap berjalan lancar saat browser diminimalkan.

### 4. Kontrol Sistem OS & Utilitas
- **Peluncur Aplikasi**: Otomatis mendeteksi dan membuka aplikasi desktop terinstal (`app:open`).
- **Kontrol Audio & OS**: Pengaturan volume sistem (`os:volume_up`, `os:volume_down`, `os:mute`), tangkapan layar (`os:screenshot`), kunci layar (`os:lock`), kalkulator, terminal, dan file manager.
- **Manajemen Catatan**: Pencatatan agenda dan to-do list terintegrasi (`todo:add`, `todo:done`).

### 5. Antigravity Pool & Local OpenAI Router
- **Multi-Account OAuth Pooling**: Manajemen akun Google OAuth ganda dengan failover kuota otomatis.
- **OpenAI-Compatible Local Endpoint (`http://127.0.0.1:42128/v1`)**: Kompatibel langsung dengan Cursor, VS Code, Cline, dan tools pengembang pihak ketiga.

---

## Persyaratan Sistem

| Platform | Kebutuhan Minimum |
| :--- | :--- |
| **Linux** | Kernel 5.4+, X11 / Wayland (Vulkan/OpenGL aktif), `libasound2` (ALSA/PulseAudio/PipeWire) |
| **Windows** | Windows 10 (64-bit) 1909+ / Windows 11, DirectX 12 / Vulkan support |
| **macOS** | macOS 12 Monterey ke atas, Apple Silicon (M1/M2/M3/M4) dengan Metal Engine |

---

## Panduan Penggunaan

### 1. Unduh & Jalankan
Unduh installer binary dari [GitHub Releases](https://github.com/ahlanefwolf/ragasukma/releases):
- **Linux**: `.AppImage`
- **Windows**: `.exe`
- **macOS**: `.dmg`

Setelah aplikasi dibuka, avatar 3D akan tampil melayang di layar dan dashboard kontrol web otomatis terbuka di `http://127.0.0.1:42128`.

### 2. Mulai Percakapan
1. Masuk ke tab **Main Menu** pada dashboard.
2. Klik **Mulai Percakapan (Aktifkan AI)**.
3. Berbicaralah langsung ke mikrofon untuk berinteraksi, memutar musik, atau menjalankan perintah sistem.

---

## Hak Cipta & Lisensi

Dibuat oleh **Ahlan Saputra** ([@ahlanefwolf](https://github.com/ahlanefwolf)).  
Hak Cipta © 2026 Ahlan Saputra. Seluruh hak cipta dilindungi undang-undang (*Proprietary*).
