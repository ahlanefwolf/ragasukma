# 🌐 Cross-Platform Engineering Guidelines (Linux, Windows, macOS)

Panduan standar ini wajib dibaca dan diikuti oleh developer maupun AI Assistant saat mengembangkan, me-refactor, atau menambahkan fitur baru pada project **Ragasukma (Rust Native Overlay)** agar aplikasi selalu 100% stabil, bebas race-condition, dan kompatibel di Linux (Fedora, Ubuntu, Arch), Windows (10/11), dan macOS (Apple Silicon & Intel).

---

## 📂 1. Penanganan Path & Direktori Konfigurasi

1. **Gunakan `PathBuf` & `Path::join()`**:
   - ❌ **DILARANG** hardcode path dengan string manual (`"assets/char.vrm"` atau `"assets\\char.vrm"`).
   - ✅ **GUNAKAN** `dir.join("assets").join("char.vrm")`.
2. **Standar Direktori Konfigurasi Sistem Operasi**:
   - **Linux**: `~/.config/ragasukma/` (`$XDG_CONFIG_HOME` atau `$HOME/.config`)
   - **Windows**: `%APPDATA%\ragasukma\` (`C:\Users\<User>\AppData\Roaming\ragasukma\`)
   - **macOS**: `~/Library/Application Support/ragasukma/`
3. **Case-Sensitivity & Penamaan File**:
   - Linux bersifat **Case-Sensitive** (`char.vrm` $\neq$ `Char.VRM`).
   - Selalu gunakan huruf kecil konsisten (*lowercase kebab-case / snake_case*) pada seluruh aset dan path.

---

## 🧠 2. Manajemen Memori, Threading & Stack Size

1. **Alokasi Heap vs Stack untuk Objek Besar**:
   - ❌ **DILARANG** meletakkan struct data besar (seperti 3D Mesh vertices, Audio PCM buffers, atau VRM model data) di local stack.
   - ✅ **GUNAKAN** `Box<T>`, `Vec<T>`, atau `Arc<T>` agar dialokasikan di Heap. Hal ini mencegah *Stack Overflow* (terutama di Linux AppImage/FUSE dengan batasan stack ketat).
2. **Dedicated Primary Thread vs macOS Main Thread**:
   - **Linux & Windows**: Jalankan loop aplikasi di dalam Dedicated Primary Thread dengan kapasitas stack minimal 64 MB (`ragasukma-main`) agar kebal terhadap batasan stack bawaan runtime OS / AppImage.
   - **macOS**: ❌ **DILARANG KERAS** memindahkan inisialisasi Cocoa/NSApplication (`EventLoop`) ke sub-thread baru. macOS **WAJIB** mengeksekusi `run_ragasukma_app()` langsung di OS Main Thread (Thread 0).
3. **Windowing & Event Loop Compatibility (`winit`)**:
   - Winit Event Loop / Windowing wajib dikonfigurasi per platform:
     - **Linux**: Menggunakan `EventLoopBuilderExtX11::with_any_thread(&mut b, true)` dengan fallback Wayland -> X11.
     - **Windows**: Menggunakan `EventLoopBuilderExtWindows::with_any_thread(&mut b, true)` agar EventLoop aman berjalan di dalam `ragasukma-main` thread tanpa memicu panic.
     - **macOS**: Dijalankan langsung di Main Thread (Thread 0) tanpa builder thread extension.

---

## 🎨 3. Standar Grafis & Shader (`wgpu` / WGSL)

1. **Kompatibilitas Metal, DirectX 12 & Vulkan**:
   - Gunakan `wgpu::Backends::all()` untuk pemilihan backend otomatis:
     - macOS: **Metal** (Apple Silicon M-series & Intel Mac).
     - Windows: **DirectX 12 / Vulkan**.
     - Linux: **Vulkan / Mesa**.
2. **Device Limits & Features**:
   - Gunakan `wgpu::Limits::downlevel_webgl2_defaults()` dan `wgpu::Features::empty()` sebagai baseline terendah agar kompatibel dengan GPU terintegrasi, virtual GPU runner CI/CD, maupun chip Apple Silicon yang ketat.
3. **Validasi Shader WGSL**:
   - Gunakan macro compile-time `wgpu::include_wgsl!()` agar sintaks shader divalidasi langsung saat `cargo build` dan tidak menimbulkan *Metal driver panic* pada macOS saat runtime.
4. **Dukungan Layar Retina & High-DPI**:
   - Selalu tangani event `WindowEvent::ScaleFactorChanged` untuk memicu `renderer.resize(physical_size)` agar tampilan 3D tidak buram/pecah pada layar Retina Mac maupun scaling display Windows (125%/150%).

---

## ⚡ 4. Eksekusi Subprocess, Shell Command & Runtime Isolation

1. **Abstraksi OS Command (`#[cfg(target_os = "...")]`)**:
   - **Membuka URL / Browser Default**:
     - Linux: `xdg-open <url>`
     - Windows: `cmd /C start <url>`
     - macOS: `open <url>`
   - **Eksekusi Script Terminal**:
     - Linux / macOS: `sh -c "<command>"`
     - Windows: `cmd /C "<command>"` atau `powershell`
2. **Isolasi Runtime & Pembersihan Environment Variable (AppImage Fix)**:
   - Saat memanggil binary eksternal OS dari aplikasi Linux (misal `yt-dlp`, `pip`, atau browser):
   - ✅ **WAJIB** hapus environment variable pembajak runtime:
     ```rust
     cmd.env_remove("PYTHONHOME");
     cmd.env_remove("PYTHONPATH");
     cmd.env_remove("LD_LIBRARY_PATH");
     cmd.env_remove("LD_PRELOAD");
     ```
   - *Alasan*: Mencegah bentrok pustaka dinamis (seperti OpenSSL `libcrypto.so.3`) antara wrapper AppImage dan paket sistem host.

---

## 📦 5. Asset Embedding & Fallback Resilien

1. **Embedded Default Assets (`RustEmbed`)**:
   - Seluruh aset vital default (`char.vrm`, `poses.json`, `skill-command.md`, avatar PNG, `icon.png`, `config-ai.json`) wajib di-embed ke dalam binary melalui `rust-embed`.
2. **Auto-Extract ke Config Directory**:
   - Saat aplikasi pertama kali dijalankan, sistem secara otomatis mengekstrak file default tersebut ke direktori config pengguna (`~/.config/ragasukma/assets/`).
   - Jika pengguna memodifikasi aset (misal mengedit `poses.json` atau mengganti `char.vrm`), aplikasi memprioritaskan aset di folder config pengguna tanpa mengubah binary.
3. **Desktop Shortcut Integration**:
   - Di Linux, aplikasi otomatis mendeteksi environment `$APPIMAGE` dan mendaftarkan shortcut `.desktop` ke `~/.local/share/applications/ragasukma.desktop`.

---

## 🔒 6. Universal Single-Instance & IPC Safety

1. **Cegah Multiple Instances dengan Port Binding**:
   - ❌ **DILARANG** mengandalkan file lock `/tmp/app.lock` (tidak portabel dan rawan stale lock).
   - ✅ **GUNAKAN** `TcpListener::bind(("127.0.0.1", PORT))` pada port server terisolasi (`42128`).
   - Jika port sudah dipakai, proses baru otomatis memicu `open_browser()` ke tab aktif lalu keluar secara aman (`std::process::exit(0)`).

---

## 🎛️ 7. Audio Backend Compatibility (`cpal` & `rodio`)

1. **Cross-Platform Audio Architecture**:
   - Linux: ALSA / PulseAudio / PipeWire (`libasound2-dev` pada CI build).
   - Windows: WASAPI native.
   - macOS: CoreAudio native.
2. **Sample Rate Resampling**:
   - Selalu gunakan DSP resampler (`rubato`) untuk mengonversi sample rate hardware mic (misal 44.1kHz, 48kHz, 96kHz) ke standar VAD/STT (16.000 Hz Mono).

---

## 🚀 8. Checklist Validasi CI/CD Release

Sebelum melakukan rilis tag baru (`v*`), pastikan:
- [ ] `cargo check` dan `cargo clippy --all-targets` bersih tanpa warning/error.
- [ ] Format target rilis:
  - **Linux**: Universal AppImage (`appimage`) via Ubuntu 22.04 runner.
  - **Windows**: NSIS Installer `.exe` (`nsis`) via Windows runner.
  - **macOS**: DMG bundle (`dmg`) via macOS runner.
- [ ] Assets `resources = ["assets"]` terdaftar pada metadata `Cargo.toml`.
