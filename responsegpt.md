Kalau gue lihat requirement lu, fondasinya sudah bagus. Tapi supaya **Ragasukma benar-benar stabil dan tidak bikin masalah saat release**, ada beberapa hal yang perlu dikondisikan lagi.

Gue urutkan berdasarkan prioritas.

## 🔴 1. Perbaiki aturan `winit` dan threading

Ini yang paling penting.

Requirement sekarang mewajibkan Linux/Windows memakai `Dedicated Primary Thread`, tetapi dokumentasi winit menyatakan event loop secara umum harus dibuat di **main thread** untuk kompatibilitas lintas platform. ([Rust Windowing][1])

### Sebaiknya ubah menjadi:

```text
EventLoop winit WAJIB dibuat dan dijalankan pada OS main thread
untuk seluruh platform yang membutuhkan kompatibilitas maksimum.

Thread tambahan digunakan untuk:
- Audio capture
- VAD/DSP
- STT/TTS
- LLM request
- Skill execution
- HTTP server
- yt-dlp
- Logging dan telemetry

Thread tambahan tidak boleh membuat atau menjalankan EventLoop winit.
Komunikasi dengan event loop menggunakan EventLoopProxy atau channel.
```

Jadi arsitekturnya:

```text
OS Main Thread
 └── Winit + Window + WGPU rendering

Worker Threads
 ├── Audio
 ├── AI Pipeline
 ├── TTS/STT
 ├── Skills
 ├── HTTP API
 └── Media Engine
```

Ini lebih aman daripada memaksa event loop masuk thread 64 MB.

---

## 🔴 2. Tambahkan aturan resource lifecycle

RAM lu bisa turun ke 54 MB, tetapi guideline belum menjelaskan kapan resource harus dibuang.

Tambahkan:

```text
Semua resource besar wajib memiliki lifecycle yang jelas.

- GPU texture dan buffer harus dilepas saat asset diganti.
- Audio buffer sementara harus di-drop setelah digunakan.
- TTS audio harus memiliki batas queue.
- HTTP response body tidak boleh ditahan tanpa kebutuhan.
- Task async harus dapat dibatalkan.
- JoinHandle worker wajib dikelola.
- Jangan membuat thread baru untuk setiap voice request.
- Gunakan bounded channel untuk audio dan event queue.
```

Khususnya:

```rust
const MAX_AUDIO_QUEUE: usize = 8;
```

Jangan membiarkan request suara menumpuk tanpa batas.

---

## 🔴 3. Tambahkan aturan memory leak testing

Jangan hanya memeriksa RAM awal dan akhir.

```text
Memory Stability Test:

- Startup
- Load avatar
- Replace avatar 100x
- TTS 100x
- STT 100x
- Start/stop microphone
- Open/close dashboard
- Play/stop music
- Execute skills repeatedly
- Reconnect API repeatedly

Catat:
- RSS / Working Set
- Private Memory
- Commit Size
- VRAM
- Thread count
- File descriptor / handle count
```

Kriteria yang lebih realistis:

```text
Memory usage tidak boleh terus meningkat secara
konsisten setelah aktivitas yang sama diulang.
```

Jangan menentukan angka RAM absolut sebelum melakukan benchmark di setiap OS.

---

## 🟠 4. WGPU perlu konfigurasi adapter yang lebih fleksibel

Jangan hanya bergantung pada:

```rust
wgpu::Backends::all()
```

Tambahkan aturan:

```text
- Log adapter name, vendor, backend, device type.
- Sediakan fallback backend.
- Tangani adapter/device initialization failure.
- Jangan mengasumsikan GPU mendukung fitur tertentu.
- Gunakan limits berdasarkan kebutuhan renderer.
- Uji integrated GPU dan software/virtual GPU.
- Tangani surface lost, timeout, dan device error.
```

Untuk `Limits`, gunakan baseline yang sesuai kebutuhan engine, bukan selalu memaksa limit WebGL2 yang rendah.

---

## 🟠 5. Perbaiki aturan subprocess

Jangan selalu menghapus semua environment variable.

Buat helper khusus:

```rust
fn configure_external_command(cmd: &mut Command) {
    if is_appimage_runtime() {
        cmd.env_remove("PYTHONHOME");
        cmd.env_remove("PYTHONPATH");
    }
}
```

Tambahkan juga:

```text
- Jangan menggunakan shell jika tidak diperlukan.
- Validasi argument sebelum menjalankan command.
- Gunakan timeout untuk subprocess.
- Tangani exit code.
- Batasi jumlah proses bersamaan.
- Kill child process saat aplikasi ditutup.
- Jangan memasukkan input user langsung ke shell.
```

Ini penting karena Ragasukma memiliki skill command dan terminal automation.

---

## 🟠 6. Single-instance perlu dibedakan dari API server

Port `42128` bisa digunakan untuk single-instance, tetapi tambahkan pengamanan:

```text
- Bind hanya ke 127.0.0.1.
- Verifikasi bahwa port tersebut benar-benar milik Ragasukma.
- Gunakan handshake/token lokal.
- Jangan langsung menganggap semua proses di port adalah instance Ragasukma.
- Tangani port yang digunakan aplikasi lain.
- Jangan menggunakan std::process::exit() sebelum resource penting ditutup.
```

Kalau port dipakai aplikasi lain, jangan otomatis mengirim perintah ke endpoint yang tidak dikenal.

---

## 🟠 7. Tambahkan keamanan dashboard dan skill

Ini wajib karena dashboard bisa menjalankan konfigurasi dan command.

```text
- API hanya listen pada localhost secara default.
- Endpoint sensitif harus memiliki authentication token lokal.
- Skill command memakai allowlist.
- Command berbahaya memerlukan konfirmasi pengguna.
- Jangan mengekspos arbitrary shell execution melalui HTTP.
- Log setiap eksekusi skill.
- Jangan menyimpan API key dalam frontend.
- Validasi seluruh input dashboard.
```

Terutama jika nanti dashboard bisa diakses melalui jaringan lokal.

---

## 🟡 8. Audio perlu aturan lifecycle dan device recovery

Tambahkan:

```text
- Tangani microphone disconnect.
- Tangani perubahan default audio device.
- Sediakan recovery jika stream berhenti.
- Jangan membuat stream baru setiap voice request.
- Gunakan bounded audio buffer.
- Tangani sample rate yang tidak didukung.
- Pisahkan capture, DSP, VAD, STT, dan playback.
- Jangan memblokir event loop dengan operasi audio atau network.
```

---

## 🟡 9. Tambahkan aturan shutdown

Ini belum terlihat di requirement lu.

```text
Graceful Shutdown:

- Hentikan microphone capture.
- Batalkan request AI yang sedang berjalan.
- Hentikan TTS/playback.
- Hentikan subprocess yt-dlp.
- Tutup HTTP server.
- Simpan konfigurasi jika diperlukan.
- Tunggu worker thread selesai dengan timeout.
- Lepaskan GPU dan audio resources.
```

Hindari `process::exit()` sebagai jalur normal karena destructor Rust bisa tidak berjalan sebagaimana mestinya.

---

## 🟡 10. CI/CD perlu matriks testing, bukan hanya build

Tambahkan pengujian:

```text
Linux:
- X11
- Wayland
- Fedora / Ubuntu
- Integrated GPU

Windows:
- Windows 10
- Windows 11
- Scaling 100%, 125%, 150%
- WASAPI

macOS:
- Apple Silicon
- Intel Mac jika masih didukung
- Retina display
- Metal
```

Dan validasi:

```text
- cargo check
- cargo fmt --check
- cargo clippy --all-targets --all-features
- cargo test
- startup smoke test
- asset extraction test
- single-instance test
- subprocess cleanup test
- audio initialization test
```

## Prioritas pengerjaan gue

| Prioritas | Yang perlu lu perbaiki                    |
| --------- | ----------------------------------------- |
| 🔴 P1     | Winit main thread dan worker architecture |
| 🔴 P1     | Resource lifecycle dan bounded queue      |
| 🔴 P1     | Security skill command dan localhost API  |
| 🔴 P1     | Graceful shutdown                         |
| 🟠 P2     | GPU adapter fallback dan error handling   |
| 🟠 P2     | Subprocess isolation dan timeout          |
| 🟠 P2     | Single-instance handshake                 |
| 🟡 P3     | Audio device recovery                     |
| 🟡 P3     | Memory benchmark otomatis                 |
| 🟡 P3     | CI runtime smoke test                     |

**Kesimpulan:** yang paling perlu lu kondisikan bukan lagi sekadar RAM atau path, tetapi **thread ownership, lifecycle resource, shutdown, security, dan recovery ketika perangkat/driver gagal**. Itu yang membedakan aplikasi yang sekadar berjalan dengan aplikasi native yang tahan dipakai berjam-jam.

[1]: https://rust-windowing.github.io/winit/winit/event_loop/struct.EventLoopBuilder.html?utm_source=chatgpt.com "EventLoopBuilder in winit::event_loop - Rust"
