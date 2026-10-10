# Rencana Desain Arsitektur: Dynamic Hardware Budgeting & Adaptive 3D Pipeline

Dokumen ini merinci rencana teknis implementasi sistem adaptif (*hardware-aware dynamic budgeting*) untuk inferensi Audio/TTS (Supertonic 3 + OpenVoice V2) dan modul Render 3D VRM di Ragasukma.

---

## 1. Dynamic Hardware & Memory Budgeting (Audio & ONNX Subsystem)

### 1.1. Latar Belakang & Masalah
- Model ONNX (Supertonic 3 Text Encoder, Duration Predictor, Vector Estimator, Vocoder, dan OpenVoice V2) berjalan di CPU menggunakan multi-threading.
- Menentukan angka thread statis (misal 8 thread) pada perangkat dengan RAM terbatas (misal 8 GB fisik dengan swap terpakai, bersamaan dengan OBS Studio dan Browser) berisiko memicu *memory pressure*, *OOM killer*, atau *frame drop* di aplikasi lain.
- Sebaliknya, membatasi terlalu rendah di PC high-end (misal Ryzen 9 / Core i9 16-32 Core) mengurangi throughput yang seharusnya bisa diperoleh.

### 1.2. Strategi Deteksi Saat Startup (`src/engine/hardware_probe.rs`)
Pemeriksaan dilakukan secara instan (*one-shot probe* $\sim 0.1$ ms) saat startup aplikasi:

1. **Deteksi Fisik & Logikal CPU**:
   - Total Core & Logical Threads via `std::thread::available_parallelism()`.
2. **Deteksi Kapasitas & Ketersediaan RAM**:
   - **Linux**: Membaca `MemAvailable` dari `/proc/meminfo` (atau `sysinfo`).
   - **Windows**: Membaca `GlobalMemoryStatusEx` (`ullAvailPhys`).
   - **macOS**: Membaca `host_statistics64` (`vm_statistics64_data_t`).

### 1.3. Matriks Alokasi Thread Otomatis

| Sisa RAM Bebas (Available) | Jumlah Core Logikal | Rekomendasi Intra-Threads ONNX | Mode Operasi |
|---|---|---|---|
| $< 2.5\text{ GB}$ (Ketat/OOM Risk) | Berapapun | **2 Threads** | *Ultra-Power Saver / Safe Mode* |
| $2.5 - 4.5\text{ GB}$ (Sedang / Spek 8GB Laptop) | $\le 6\text{ Core}$ | **3 - 4 Threads** | *Balanced Dynamic Mode* (Default) |
| $> 4.5\text{ GB}$ (Lega) | $\ge 8\text{ Core}$ | **6 - 8 Threads** | *Performance Mode* |
| $> 8.0\text{ GB}$ | $\ge 12\text{ Core}$ | **8 Threads (Capped Max)** | *Maximum Real-Time Throughput* |

> **Catatan Teknis**: Batas atas thread ONNX tetap dibatasi (*capped*) maksimal **8 thread**, karena hukum Amdahl menunjukkan overhead sinkronisasi antar-thread di atas 8 thread pada model tensor kecil justru mendegradasi latensi.

---

## 2. Adaptive 3D VRM Pipeline (Resolution, VRAM & Dynamic FPS)

### 2.1. Latar Belakang & Masalah
- Engine VRM/3D wgpu saat ini merender viewport 3D sesuai ukuran window desktop.
- Pengguna membutuhkan fleksibilitas untuk mengatur target frame rate (misal 30 FPS untuk hemat baterai/CPU saat bekerja, 60 FPS untuk motion mulus, atau Uncapped untuk high refresh rate monitor).
- Deteksi resolusi native layar dan informasi GPU/VRAM adapter penting agar internal render resolution scale bisa disesuaikan tanpa membuat GPU terintegrasi (iGPU Barcelo/Intel Iris Xe) overheat.

### 2.2. Komponen Deteksi & Adaptasi GPU (`src/ui/renderer/gpu_context.rs`)
1. **GPU Adapter Probing**:
   - Mengambil `wgpu::AdapterInfo` (`name`, `backend`, `device_type`, `driver_info`).
   - Klasifikasi adapter: `DiscreteGPU` (NVIDIA/AMD Radeon RX) vs `IntegratedGPU` (AMD Vega/Barcelo, Intel UHD/Iris).
2. **Monitor Resolution & DPI Scaling**:
   - Mendeteksi native resolution monitor via `winit::monitor::MonitorHandle::size()` dan DPI scale factor.
3. **Internal Render Scale**:
   - Mode `1.0x` (Native Display Buffer).
   - Mode `0.75x` / `0.5x` (Sub-sampled internal texture dengan linear upscaling) untuk iGPU hemat daya.

### 2.3. Pengaturan Baru di Dashboard UI (`web/src/components/settings/`)
Menambahkan sub-seksi **3D Performance & Display Settings** pada panel pengaturan:

1. **Target Frame Rate (FPS Limiter)**:
   - `30 FPS (Power Saver / Background Focus)`
   - `60 FPS (Smooth Standard - Recommended)`
   - `Uncapped / VSync (Match Monitor 120Hz/144Hz/240Hz)`
2. **Render Resolution Scale**:
   - `Auto (Hardware-Aware Profile)`
   - `High (100% Native Resolution)`
   - `Medium (75% Resolution - Recommended for iGPU)`
   - `Low (50% Resolution - Low VRAM)`
3. **Hardware Info Banner**:
   - Menampilkan ringkasan runtime: Nama GPU, Backend API (`Vulkan` / `DX12` / `Metal`), Status VRAM, dan Alokasi Thread Audio yang aktif.

---

## 3. Langkah Implementasi Bertahap

### Fase 1: Hardware Probe & Audio Engine Update
- [ ] Buat modul `src/engine/hardware_probe.rs` untuk deteksi non-blocking kapasitas RAM dan CPU core.
- [ ] Integrasikan `hardware_probe::get_recommended_onnx_threads()` ke `src/audio/supertonic.rs` dan `src/audio/openvoice.rs`.
- [ ] Log status alokasi hardware saat boot (`[Hardware Probe] Detected 7.14 GB Total / 2.8 GB Free RAM, 12 Cores -> Allocating 4 ONNX threads`).

### Fase 2: 3D Renderer FPS Limiter & Dynamic Viewport Scale
- [ ] Tambahkan konfigurasi `fps_limit` (30/60/0) dan `render_scale` di `src/config.rs` (`UserConfig`).
- [ ] Modifikasi game loop / frame ticker di `src/ui/app.rs` agar menghormati frame timing delta (`std::time::Instant` sleep target).
- [ ] Dukung rendering ke offscreen texture yang di-scale sesuai `render_scale` sebelum di-blit ke swapchain window.

### Fase 3: Frontend Integration (Dashboard Settings)
- [ ] Tambahkan card **Graphics & Performance** di `web/src/components/settings/`.
- [ ] Hubungkan endpoint GET/POST `/api/config` untuk parameter FPS dan Render Scale secara live tanpa perlu restart aplikasi.
