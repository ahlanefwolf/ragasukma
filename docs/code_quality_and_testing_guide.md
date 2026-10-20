# Panduan Audit & Validasi Kualitas Kode (Testing & Linting)

Dokumen ini memuat standar perintah pengujian kode untuk **Backend (Rust)** dan **Frontend (SolidJS + TypeScript)** pada proyek Ragasukma. Jalankan perintah-perintah ini secara berkala sebelum melakukan *commit* besar atau rilis produksi.

---

## 1. Backend (Rust / Cargo)

### A. Quick & Standard Checks
Untuk pengecekan cepat selama proses pengembangan:
```bash
# Cek kompilasi tanpa membuat binary
cargo check

# Linter standar Clippy untuk seluruh target (bin, test, build-script)
cargo clippy --all-targets --all-features
```

### B. Strict Code Audit (Gold Standard)
Perintah Clippy tingkat lanjut untuk mendeteksi **Dead Code, Memory Leak, Deadlock / Race Condition, dan Unhandled Panic**:

```bash
cargo clippy --bin ragasukma -- \
  -W dead_code \
  -W unused \
  -W clippy::redundant_clone \
  -W clippy::await_holding_lock \
  -W clippy::mutex_atomic \
  -W clippy::rc_buffer \
  -W clippy::panic \
  -W clippy::unwrap_used
```

**Penjelasan Lint:**
* `-W dead_code` & `-W unused`: Mendeteksi fungsi, struct, konstanta, atau import yang tidak pernah dipakai.
* `-W clippy::redundant_clone`: Menangkap alokasi RAM / kloning data yang sia-sia.
* `-W clippy::await_holding_lock`: Mencegah potensi **deadlock** akibat `MutexGuard` yang ditahan melewati titik `.await`.
* `-W clippy::mutex_atomic`: Menangkap penggunaan `Mutex` pada tipe primitif yang seharusnya memakai `Atomic`.
* `-W clippy::rc_buffer`: Mencegah alokasi heap ganda pada pointer pintar (misal `Arc<Vec<T>>` vs `Arc<[T]>`).
* `-W clippy::panic` & `-W clippy::unwrap_used`: Memaksa *graceful error handling* dan mencegah program crash mendadak di runtime akibat `.unwrap()`.

### C. Automated Unit & Integration Tests
```bash
cargo test
```

---

## 2. Frontend (`web/`)

Pindah ke folder web sebelum menjalankan perintah:
```bash
cd web
```

### A. Type Safety Check
Memvalidasi kebenaran tipe TypeScript tanpa meng-output file kompilasi:
```bash
npx tsc --noEmit
```

### B. Code Quality & Linting
Mendeteksi kesalahan logika, styling, dan best practice JavaScript / SolidJS:
```bash
npx eslint .
```

### C. Dead Code & Unused Dependencies Audit
Mendeteksi file yatim piatu (*unreferenced files*), export yang tidak terpakai, dan dependency di `package.json` yang mubazir:
```bash
npx knip
```

### D. Production Build Validation
Memvalidasi bahwa seluruh asset, Tailwind CSS, komponen JSX SolidJS, dan Three.js berhasil dibundel oleh Vite tanpa kesalahan bundler ke dalam `web/dist`:
```bash
npm run build
```

---

## 3. Checklist Sebelum Release / Commit Besar

- [ ] `cargo test` lulus semua test suite.
- [ ] `cargo clippy --all-targets --all-features` bersih dari warning.
- [ ] Strict clippy audit (`--bin ragasukma`) bebas dari deadlock dan unhandled panic.
- [ ] Di direktori `web/`: `npx tsc --noEmit` lolos 0 error.
- [ ] Di direktori `web/`: `npx eslint .` dan `npx knip` bersih.
- [ ] Di direktori `web/`: `npm run build` sukses meng-generate folder `web/dist`.
