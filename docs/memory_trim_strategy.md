# Spesifikasi Strategi Pembersihan Memori & Malloc Trim (Linux RAM Optimization)

Dokumen ini merangkum strategi pembersihan memori dinamis menggunakan `malloc_trim(0)` pada arsitektur Ragasukma untuk mengatasi retensi heap memori glibc di sistem operasi Linux.

---

## 1. Latar Belakang Masalah
Di Linux, memori yang sudah di-*free/drop* oleh Rust tidak langsung dikembalikan ke kernel OS karena mekanisme *heap retention* (arena caching) dari glibc `malloc`. Akibatnya, pemakaian Resident Set Size (RSS) di Task Manager terlihat membengkak (~1 GB+) meskipun struktur datanya sudah tidak terpakai.

---

## 2. Titik Pemicu Eksekusi `malloc_trim(0)`

| No | Modul / Titik Eksekusi | Alokasi Memori yang Dibebaskan | Dampak Pengurangan RAM |
| :---: | :--- | :--- | :---: |
| **1** | **Ganti Avatar / Model VRM 3D & 2D** (`src/ui/`) | Mesh buffer, vertex array, tekstur RGBA GLTF/VRM lama. | **~50 – 150 MB** |
| **2** | **Selesai Inferensi TTS & Audio Playback** (`src/audio/`) | ONNX Runtime workspace arena, buffer raw WAV, Opus encoder chunks. | **~100 – 300 MB** |
| **3** | **Selesai LLM Stream Turn & Pruning History** (`src/llm/`) | Buffer parsing SSE JSON, token ring-buffer, temporary string builder. | **~20 – 60 MB** |
| **4** | **Selesai Background Heavy Task** (`src/engine/`) | Download buffer yt-dlp, image bitmap OCR Vision, app scanner list. | **~40 – 120 MB** |
| **5** | **Client Remote / Browser Disconnect** (`src/services/`) | DOM snapshot caching, frame buffer WebSocket, ring buffer action client. | **~10 – 30 MB** |

---

## 3. Implementasi Helper di Rust

```rust
/// Memaksa glibc allocator mengembalikan unused heap pages ke kernel Linux
#[inline]
pub fn trim_memory_if_linux() {
    #[cfg(target_os = "linux")]
    unsafe {
        libc::malloc_trim(0);
    }
}
```

---

## 4. Karakteristik & Efisiensi
- **Zero Idle Overhead**: Dipanggil secara deterministik hanya pada saat transisi/selesai proses (*event-driven*), bukan dengan polling interval konstan.
- **System Stability**: Aman 100% dan tidak merusak pointer yang masih aktif digunakan oleh thread lain.
- **Optimal RAM Footprint**: Menjaga aplikasi tetap ramping mendekati estimasi teoritis saat idle.
