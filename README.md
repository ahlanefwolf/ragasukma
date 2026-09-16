<div align="center">

  <img src="icons/128x128@2x.png" alt="Ragasukma Logo" width="130" height="130" />

  # 🔮 Ragasukma
  **Asisten Suara AI Interaktif Bahasa Indonesia dengan Avatar 3D / 2D Transparent Overlay**

  [![Release](https://img.shields.io/github/v/release/ahlanefwolf/ragasukma?style=for-the-badge&color=7BDFF2)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![Platform](https://img.shields.io/badge/Platform-Linux_|_Windows_|_macOS-lightgrey?style=for-the-badge)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![License](https://img.shields.io/badge/License-Proprietary-red?style=for-the-badge)](https://github.com/ahlanefwolf/ragasukma)

  <p align="center">
    <b>Ragasukma</b> adalah asisten suara AI pintar berbahasa Indonesia yang hadir langsung di layar desktop kamu. Menampilkan karakter avatar 3D/2D yang hidup, responsif mendengarkan suara secara otomatis, dan dapat membantu berbagai aktivitas harian serta otomasi perintah komputer.
  </p>

</div>

---

## ✨ Fitur Utama

- 🎭 **Avatar 3D & 2D di Layar Desktop**:
  - Avatar transparan melayang di layar dengan animasi hidup (bernapas, berkedip, bergerak alami, dan ekspresi emosi wajah yang menyesuaikan percakapan).
  - Mendukung model 3D kustom (**VRoid / VRM / GLB**) serta gambar 2D avatar pilihanmu sendiri.

- 🎙️ **Deteksi Suara Otomatis (VAD & DSP)**:
  - Cukup bicara tanpa perlu menekan tombol (*hands-free*). Sistem cerdas menyaring suara desis/kebisingan ruangan dan fokus menangkap suara vokalmu.

- 🧠 **Kecerdasan Buatan Multi-Engine**:
  - **Speech-to-Text**: Mendengar dengan akurat menggunakan Google STT atau Whisper.
  - **Otak AI (LLM)**: Terhubung ke berbagai model AI pilihan (Gemini, Groq, OpenAI, Ollama, DeepSeek, dll) dengan persona teman bicara yang hangat.
  - **Suara Natural (TTS)**: Suara ucapan yang jernih dan ekspresif (Google TTS & ElevenLabs).

- ⚡ **Skill & Otomasi Perintah Cerdas**:
  - **Kontrol Sistem Komputer**: Atur volume audio (+/- 10%), mute/unmute, ambil tangkapan layar (screenshot), hingga kunci layar (*lock workstation*) langsung via suara.
  - **Peluncur Aplikasi Otomatis**: Cukup ucapkan *"Buka Spotify"*, *"Buka Steam"*, *"Buka VS Code"*, dsb. Ragasukma otomatis mendeteksi dan menjalankan aplikasi yang terpasang di OS.
  - **Pencarian Web & Navigasi**: Buka website atau lakukan pencarian Google dinamis (*"Cari resep soto di Google"*, *"Buka GitHub"*).
  - **Pemutar Musik & Unduhan Lokal (yt-dlp Engine)**:
    - Cari dan putar lagu favorit langsung dari perintah suara (*"Putar musik Denny Caknan Sugeng Dalu"*).
    - Otomatis mengunduh audio jernih ke koleksi lokal dan memutarnya langsung via Web Music Player & Floating Bottom Bar tanpa perlu browser extension.
    - Manajemen koleksi musik offline di tab **Music Player** dashboard.
  - **Custom Script Automation**: Tetap mendukung penambahan script bash / terminal buatanmu sendiri di dashboard.

- 🎛️ **Dashboard Pengaturan Modern**:
  - Begitu aplikasi dijalankan, browser otomatis membuka panel kendali interaktif untuk mengatur sensitivitas suara, pose karakter, gaya bicara, dan melihat status sistem.

---

## 💻 Persyaratan Sistem (System Requirements)

Untuk menjalankan Ragasukma dengan lancar menggunakan akselerasi grafis WGPU 3D dan audio realtime:

- 🐧 **Linux**:
  - **Distro**: Ubuntu 20.04+, Debian 11+, Fedora 36+, Arch Linux, CentOS / RHEL, atau distro modern lainnya.
  - **Kernel**: Linux Kernel **5.4** ke atas.
  - **Display Server**: X11 atau Wayland (dengan layer Vulkan / OpenGL aktif).
  - **Paket Tambahan**: `libasound2` (ALSA/PulseAudio/PipeWire).
- 🪟 **Windows**:
  - **Sistem Operasi**: Windows 10 (64-bit) versi 1909+ atau **Windows 11**.
  - **Grafis**: Dukungan DirectX 12 / Vulkan.
- 🍏 **macOS**:
  - **Sistem Operasi**: macOS 12 Monterey ke atas (macOS 13/14/15+ disarankan).
  - **Arsitektur**: Apple Silicon (**M1 / M2 / M3 / M4**) dengan Metal Graphics Engine.

---

## 🚀 Panduan Penggunaan Cepat

### 1. Download & Pasang
Unduh file installer resmi sesuai sistem operasi kamu di halaman [**GitHub Releases**](https://github.com/ahlanefwolf/ragasukma/releases):
- **Linux**: File paket `.deb` (Debian/Ubuntu) atau `.rpm` (Fedora/RHEL)
- **Windows**: File installer `.exe`
- **macOS**: File `.dmg`

### 2. Jalankan Ragasukma
1. Buka aplikasi **Ragasukma**.
2. Karakter avatar akan langsung muncul di sudut layar desktop kamu.
3. Halaman web dashboard kendali akan otomatis terbuka di browser (`http://127.0.0.1:42128`).

### 3. Mulai Mengobrol
- Buka tab **Menu Utama** di dashboard lalu klik tombol **Mulai Percakapan (Aktifkan AI)**.
- Bicara ke mikrofon seperti biasa—Ragasukma akan mendengarkan, berpikir, dan menjawab langsung dengan suara serta animasi gerak!

---

## 🎨 Kustomisasi Karakter & Suara

Melalui dashboard kendali, kamu bisa bebas menyesuaikan:
- **Karakter**: Unggah model 3D `.vrm` buatan sendiri (dari VRoid Studio) atau foto 2D favoritmu.
- **Nama & Karakteristik**: Ganti nama asisten, panggilanmu (*Mas/Kak/Boss*), dan gaya bahasanya.
- **Pose 3D**: Atur sendiri posisi dan gaya gerak tubuh karakter di tab *Pose Tuner*.
- **Pembaruan**: Cukup klik tombol **Cek Pembaruan** di dashboard untuk melihat fitur-fitur baru.

---

## 📜 Hak Cipta & Pengembang

Dibuat dengan ❤️ oleh **Ahlan Saputra** ([@ahlanefwolf](https://github.com/ahlanefwolf)).  
Hak Cipta © 2026 **Ahlan Saputra**. Seluruh hak cipta dilindungi undang-undang (*All Rights Reserved / Proprietary*).
