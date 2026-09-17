# Analisis & Peningkatan Concurrency (Race Condition & Threading) — Ragasukma

Dokumen ini mendokumentasikan temuan potensi **Race Condition**, masalah sinkronisasi multi-threading, serta rekomendasi solusi untuk stabilitas sistem Ragasukma.

---

## 1. SSE Action Queue: Lost Event pada Multi-Client (Queue vs Pub-Sub Race)
* **Tingkat Risiko:** 🔴 **Tinggi**
* **Lokasi Kode:**
  * `src/app_state.rs` (`PENDING_CLIENT_ACTIONS`, `pop_client_actions()`)
  * `src/services/events_service.rs` (`handle_sse_state`)
* **Mekanisme Masalah:**
  Setiap koneksi klien ke SSE `/api/events/state` berjalan pada thread terpisah dan melakukan *polling* secara berkala:
  ```rust
  let pending_actions = crate::app_state::pop_client_actions();
  // Di app_state.rs: q.drain(..).collect()
  ```
  Jika ada lebih dari satu tab dashboard, browser extension yang terpasang, atau saat browser melakukan reconnect:
  1. Klien A memanggil `pop_client_actions()` dan menguras (*drain*) semua aksi.
  2. Klien B yang melakukan polling di waktu bersamaan mendapatkan antrean kosong (`[]`).
  3. Akibatnya, trigger aksi seperti `music:play`, `session:claimed`, dan aksi UI lainnya hanya diterima oleh satu klien dan hilang bagi klien lainnya.
* **Solusi Rekomendasi:**
  * Ganti mekanisme antrean global `VecDeque` + `drain()` dengan model **Broadcast Channel** (misalnya `tokio::sync::broadcast` jika menggunakan async runtime, atau ring buffer dengan tracking sequence ID per client connection).

---

## 2. Command Queue Overwrite (Lost Command Race Condition)
* **Tingkat Risiko:** 🟡 **Sedang**
* **Lokasi Kode:**
  * `src/services/tray_service.rs` (`TrayState::push_command`)
  * `src/services/menu_service.rs` (`handle_menu_cmd`, `handle_post_state`)
* **Mekanisme Masalah:**
  Implementasi `push_command` saat ini:
  ```rust
  pub fn push_command(cmd: String) {
      if cmd == "quit" {
          QUIT_REQUESTED.store(true, Ordering::SeqCst);
      }
      if let Ok(mut queue) = COMMAND_QUEUE.lock() {
          queue.clear(); // ⚠️ Menghapus seluruh perintah sebelumnya
          queue.push_back(cmd);
      }
  }
  ```
  Jika dua request HTTP masuk dalam waktu berdekatan (misalnya user mengubah status mute kemudian klik pause, atau saat batch state update):
  1. Thread A memasukkan `cmd_A`.
  2. Thread B masuk sebelum thread session sempat memanggil `pop_command()`.
  3. `queue.clear()` menghapus `cmd_A` dan menimpanya dengan `cmd_B`.
  4. Perintah `cmd_A` hilang (*lost command*).
* **Solusi Rekomendasi:**
  * Hapus `queue.clear()` di dalam `push_command()`. Biarkan perintah diproses berurutan secara FIFO (First-In, First-Out).

---

## 3. Provider Instance Overwrite di OpenAI v1 Service (State Loss & Rate Limit Collision)
* **Tingkat Risiko:** 🔴 **Tinggi**
* **Lokasi Kode:**
  * `src/services/openai_v1_service.rs` (`get_provider()`, `release_provider()`)
* **Mekanisme Masalah:**
  ```rust
  fn get_provider() -> AntigravityProvider {
      let mut lock = GLOBAL_ANTIGRAVITY.lock().unwrap();
      if lock.is_none() {
          *lock = Some(AntigravityProvider::new());
      }
      let mut prov = lock.take().unwrap(); // ⚠️ State lock menjadi None
      let _ = prov.load_accounts();
      prov
  }

  fn release_provider(prov: AntigravityProvider) {
      let mut lock = GLOBAL_ANTIGRAVITY.lock().unwrap();
      *lock = Some(prov); // ⚠️ Menimpa provider yang ada
  }
  ```
  Ketika ada request simultan ke `/v1/chat/completions`:
  1. Request 1 memanggil `get_provider()` -> memindahkan (*take*) instance sehingga lock menjadi `None`.
  2. Request 2 masuk saat Request 1 masih menunggu streaming respons LLM. Karena `lock.is_none()`, Request 2 menginisialisasi provider baru secara terpisah.
  3. Request 1 selesai dan memanggil `release_provider(prov1)`.
  4. Request 2 selesai dan memanggil `release_provider(prov2)`, menimpa seluruh state `prov1` (termasuk status token refresh, rate-limit cooldown, dan error counters).
* **Solusi Rekomendasi:**
  * Gunakan `Arc<Mutex<AntigravityProvider>>` atau `Arc<RwLock<AntigravityProvider>>` bersama tanpa pola `take()` dan `replace()`.

---

## 4. Race Condition Baca-Tulis File Akun Antigravity (Lost Updates)
* **Tingkat Risiko:** 🟡 **Sedang**
* **Lokasi Kode:**
  * `src/services/llm_providers/antigravity_service.rs`
  * `src/llm/providers/antigravity/provider.rs` (`save_accounts`)
* **Mekanisme Masalah:**
  Setiap endpoint API Antigravity (tambah akun, hapus akun, refresh kuota, OAuth callback) membuat instance `AntigravityProvider::new()` independen:
  * Jika ada 2 operasi bersamaan (misal OAuth login selesai bersamaan dengan background refresh akun), masing-masing thread membaca `antigravity_accounts.json`, mengubah data di memori masing-masing, lalu menimpa file disk via `fs::write()`. Salah satu perubahan akun akan tertimpa dan hilang.
  * `LlmClient` di session thread juga memegang instance `AntigravityProvider` miliknya sendiri tanpa sinkronisasi otomatis saat file di disk diperbarui lewat Web Dashboard.
* **Solusi Rekomendasi:**
  * Jadikan state akun Antigravity tersentralisasi di dalam `ServerSharedState` (misalnya `Arc<Mutex<AntigravityManager>>`) agar semua thread mengakses dan memodifikasi referensi memori yang sama secara terproteksi lock.

---

## 5. TOCTOU (Time-of-Check to Time-of-Use) pada Single Instance Check
* **Tingkat Risiko:** 🟢 **Rendah**
* **Lokasi Kode:**
  * `src/main.rs` (baris 24–62) & `src/server.rs` (`start_http_server`)
* **Mekanisme Masalah:**
  Pada inisialisasi aplikasi:
  ```rust
  let single_instance_check = std::net::TcpListener::bind(("127.0.0.1", server_port));
  // ... verifikasi handshake ...
  drop(single_instance_check); // ⚠️ Port dilepas
  // ... beberapa langkah inisialisasi ...
  // Nanti di server.rs baru di-bind kembali
  ```
  Terdapat jeda waktu antara `drop(single_instance_check)` dan pemanggilan `TcpListener::bind` di `server.rs`. Jika user membuka aplikasi dua kali secara sangat cepat (misal double click agresif), kedua proses dapat lolos cek awal secara bersamaan sebelum salah satunya crash saat bind di server thread.
* **Solusi Rekomendasi:**
  * Gunakan mekanisme OS Single Instance Lock (seperti named mutex OS di Windows `CreateMutexW` / lockfile atomic dengan `flock` di Unix) atau teruskan listener TCP yang sudah di-bind dari `main.rs` ke `server.rs` tanpa melakukan `drop`.

---

## Ringkasan Prioritas Perbaikan

| No | Masalah Concurrency | Prioritas | Estimasi Kompleksitas |
|---|---|---|---|
| 1 | SSE Action Queue (Multi-Client) | P0 (Kritis) | Menengah |
| 2 | Provider Instance Overwrite (OpenAI v1) | P0 (Kritis) | Mudah |
| 3 | Command Queue Clear (Lost Command) | P1 (Tinggi) | Sangat Mudah |
| 4 | Antigravity File Sync & Shared State | P1 (Tinggi) | Menengah |
| 5 | Single Instance TOCTOU | P2 (Rendah) | Mudah |
