# Struktur Folder & Dokumentasi File Modul Canvas

Dokumen referensi arsitektur direktori modul Canvas (`web/src/components/canvas/`) beserta fungsi dan tanggung jawab masing-masing file.

---

## 📁 Peta Struktur Direktori

```text
web/src/components/canvas/
├── index.ts                      # Barrel public export modul canvas
├── types.ts                      # Kontrak tipe data TypeScript & interface domain
├── canvasStore.ts                # Reactive state & signal Solid.js
├── fabricService.ts              # Singleton Facade: UI <-> Fabric Engine
├── README.md                     # Panduan arsitektur & navigasi AI
│
├── ui/                           # Lapisan Komponen Tampilan (Solid.js)
│   ├── index.ts                  # Barrel export seluruh komponen UI
│   ├── CanvasTab.tsx             # Kontainer utama tab kanvas & mount Fabric.js
│   ├── CanvasToolbar.tsx         # Floating toolbar atas (pemilihan tools)
│   ├── InspectorToolbar.tsx      # Floating toolbar kontekstual (warna, ukuran, transparansi)
│   ├── LayersPanel.tsx           # Panel sidebar hirarki layer objek
│   ├── AiDrawingPanel.tsx        # Panel simulasi AI & MCP drawing executor
│   ├── ExportModal.tsx           # Dialog modal ekspor resolusi tinggi (PNG/JPEG/SVG)
│   └── ShortcutsModal.tsx        # Modal contekan pintasan keyboard (hotkeys)
│
└── engine/                       # Lapisan Logika & Manipulasi Canvas (Fabric.js v7 Murni)
    ├── index.ts                  # Barrel export modul engine
    ├── shapeBuilder.ts           # Logika boolean poligon (irisan, union, partisi pecahan)
    ├── connector.ts              # Logika rute konektor siku/lengkung & port snapping
    ├── portOverlay.ts            # Rendering overlay visual titik snap port
    ├── layers.ts                 # Logika z-index layer, urutan tumpukan, grouping & locking
    ├── export.ts                 # Logika ekspor resolusi tinggi & waypoint-aware bounding box
    ├── mcp.ts                    # Runner eksekusi langkah AI drawing berbasis tool-calling
    │
    ├── objects/                  # Factory & Creator Masing-Masing Objek Kanvas
    │   ├── index.ts              # Registry shapeRegistry & getShapeCreator()
    │   ├── types.ts              # Interface ShapeCreator & FinalizeContext
    │   ├── baseProps.ts          # Normalisasi properti border/stroke/fill dari store
    │   ├── rect.ts               # Factory & resize kalkulator Kotak (Rect)
    │   ├── circle.ts             # Factory & resize kalkulator Lingkaran (Circle)
    │   ├── line.ts               # Factory garis bebas & poligon loop converter
    │   ├── connector.ts          # Factory konektor/panah relasi dengan obstacle avoidance
    │   └── text.ts               # Factory elemen teks & label
    │
    ├── core/                     # Fondasi Lifecycle Fabric.js
    │   ├── index.ts              # Kelas FabricCore (manajer inisialisasi & lifecycle kanvas)
    │   ├── viewport.ts           # Logika zoom mouse wheel & pan kanvas (space + drag)
    │   ├── history.ts            # Undo & redo berbasis snapshot state pointer
    │   ├── clipboard.ts          # Operasi copy, cut, paste, dan duplikasi objek
    │   └── events/               # Event listener kanvas
    │       ├── index.ts          # Penggabung listener (mouse:down, move, up, selection)
    │       ├── creationHandlers.ts # Handler interaktif saat user menggambar bentuk baru
    │       ├── objectTransform.ts  # Handler saat objek digeser, di-scale, atau dirotasi
    │       └── selectionSync.ts    # Sinkronisasi objek terpilih ke sinyal UI & deteksi irisan
    │
    └── drawing/                  # Rendering Geometri & Kustomisasi Objek
        ├── index.ts              # Barrel export modul drawing
        ├── arrow.ts              # Custom render mata panah & serializer SVG
        ├── controls.ts           # Custom handles titik ujung untuk manipulasi garis
        ├── endpointControls.ts   # Utilitas posisi titik p1 & p2 pada konektor
        ├── strokeAlign.ts        # Override render perataan stroke (center, inner, outer)
        └── helpers.ts            # Utilitas kalkulasi grid snap & scene coordinate pointer
```

---

## 📄 Kegunaan Masing-Masing File

### 1. Root Modul (`components/canvas/`)

| File | Kegunaan |
|---|---|
| **`index.ts`** | Gerbang utama ekspor modul. Mengizinkan file luar (`App.tsx`) mengimpor komponen seperti `import { CanvasTab } from './components/canvas'` secara rapi. |
| **`types.ts`** | Menyimpan seluruh definisi tipe data domain seperti `CanvasTool`, `ShapeProps`, `CanvasLayerItem`, opsi routing, jenis panah, hingga tipe rencana AI (`McpDrawingPlan`). |
| **`canvasStore.ts`** | Menyimpan seluruh sinyal reaktif Solid.js (tool aktif, warna garis & transparansi, warna isi & transparansi, daftar layer, tingkat zoom, riwayat undo/redo). |
| **`fabricService.ts`** | **Facade Singleton**. Jembatan tunggal antara komponen UI Solid.js dengan engine Fabric.js agar UI tidak langsung mengotori instance Fabric mentah. |

---

### 2. Lapisan UI (`components/canvas/ui/`)

| File | Kegunaan |
|---|---|
| **`CanvasTab.tsx`** | Komponen utama tab kanvas yang mengontrol elemen `<canvas>`, menangani resize otomatis melalui ResizeObserver, drag-and-drop file gambar, dan shortcut keyboard. |
| **`CanvasToolbar.tsx`** | Toolbar floating di bagian atas untuk memilih alat: Select, Rectangle, Circle, Line, Arrow, Pen/Brush, Text, Eraser, zoom, dan toggle panel layer. |
| **`InspectorToolbar.tsx`** | Toolbar floating kontekstual yang muncul saat objek dipilih. Mengatur warna hex, slider transparansi (0–100%), tebal garis, gaya garis, sudut lengkung (radius), ukuran font, dan perataan stroke. |
| **`LayersPanel.tsx`** | Panel samping untuk mengelola urutan layer: drag & drop reorder z-index, sembunyikan/tampilkan mata objek, kunci (lock/unlock) objek, ganti nama layer, dan grouping. |
| **`AiDrawingPanel.tsx`** | Panel simulator instruksi AI/MCP. Menjalankan animasi demonstrasi langkah demi langkah penggambaran diagram oleh AI. |
| **`ExportModal.tsx`** | Dialog ekspor hasil karya ke format PNG, JPEG, atau SVG dengan opsi background (transparan/putih/gelap), scope (pilihan vs seluruh kanvas), dan skala resolusi hingga 4K. |
| **`ShortcutsModal.tsx`** | Modal petunjuk daftar tombol pintas keyboard kanvas (Undo, Redo, Copy, Paste, Duplicate, Delete, Group, Space+Drag). |

---

### 3. Lapisan Engine (`components/canvas/engine/`)

| File | Kegunaan |
|---|---|
| **`shapeBuilder.ts`** | Mesin operasi geometri Boolean (menggunakan `polygon-clipping`): mendeteksi irisan objek (hatch preview), memecah objek bertumpuk menjadi fragmen mandiri, dan menggabungkan bentuk (Union). |
| **`connector.ts`** | Logika pembuatan garis penghubung cerdas: menentukan titik anchor/port pada objek, routing garis siku ortogonal (elbow) yang rapi, dan sinkronisasi rute garis saat objek asal/tujuan dipindahkan. |
| **`portOverlay.ts`** | Menggambar titik-titik lingkaran snap port pada tepi objek menggunakan hook canvas `after:render` tanpa mencemari riwayat undo/redo. |
| **`layers.ts`** | Logika manipulasi urutan z-index, penguncian pergerakan objek, visibilitas objek, pembungkusan ke dalam `fabric.Group`, dan pembongkaran group (*ungroup*). |
| **`export.ts`** | Ekstraksi gambar kanvas ke DataURL atau SVG string. Dilengkapi kalkulasi *waypoint-aware bounding box* agar garis rute yang melengkung keluar objek tidak terpotong saat diekspor. |
| **`mcp.ts`** | Eksekutor perintah aksi gambar dari AI (Model Context Protocol), memetakan aksi `draw_rect`, `draw_circle`, `draw_arrow`, dll. menjadi objek nyata di kanvas secara teranimasi. |

---

### 4. Engine Core (`components/canvas/engine/core/`)

| File | Kegunaan |
|---|---|
| **`index.ts`** | Mengelola instansiasi `fabric.Canvas`, siklus hidup canvas (init, resize, destroy), dan mendelegasikan aksi ke sub-manajer (viewport, history, clipboard, events). |
| **`viewport.ts`** | Menangani navigasi kanvas: zoom in/out proporsional berbasis posisi mouse wheel, serta dragging kanvas (pan) saat tombol Space ditekan. |
| **`history.ts`** | Mesin Undo & Redo berbasis pointer snapshot serialization JSON untuk memastikan status canvas simetris dan tidak pernah kehilangan state. |
| **`clipboard.ts`** | Menyimpan buffer objek saat tombol Copy ditekan dan membuat klon objek baru dengan offset posisi saat tombol Paste ditekan. |

---

### 5. Engine Events (`components/canvas/engine/core/events/`)

| File | Kegunaan |
|---|---|
| **`creationHandlers.ts`** | Menangani event mouse saat user mulai mengklik, menarik, dan melepas kursor untuk membuat bentuk (Rect, Circle, Line, Arrow, Polygon) secara interaktif. |
| **`objectTransform.ts`** | Menangani event pergeseran (`object:moving`), perubahan skala (`object:scaling`), dan perputaran (`object:rotating`), termasuk snap-to-grid dan pembaharuan otomatis garis konektor yang terhubung. |
| **`selectionSync.ts`** | Menjembatani event seleksi Fabric ke sinyal UI. Membaca properti objek aktif untuk ditampilkan di inspector dan memicu preview irisan jika beberapa objek tertutup 2D saling bertumpuk. |

---

### 6. Engine Drawing & Geometry (`components/canvas/engine/drawing/`)

| File | Kegunaan |
|---|---|
| **`arrow.ts`** | Kustomisasi render mata panah pada garis `Line` Fabric.js di kanvas 2D serta pembuatan elemen SVG `<polygon>` saat diekspor. |
| **`controls.ts`** | Custom interactive control handles pada garis agar titik ujung garis (`p1` dan `p2`) dapat digeser secara bebas dan melakukan snap ke port objek lain. |
| **`endpointControls.ts`** | Menghitung transformasi matriks posisi scene global untuk titik ujung garis konektor. |
| **`strokeAlign.ts`** | Menyesuaikan kalkulasi perataan garis tepi (stroke alignment) untuk opsi Center, Inner (dalam), dan Outer (luar) pada objek poligon. |
| **`helpers.ts`** | Utilitas pendukung kalkulasi koordinat, konversi pointer canvas, pembulatan grid snap (`GRID_SIZE`), dan sinkronisasi clip-path bentuk. |

---

### 7. Aturan Tegas: Perbedaan `Line` vs `Connector`

| Aspek | `Line` (Garis Dekoratif / Bebas) | `Connector` (Garis Relasi & Flowchart) |
|---|---|---|
| **Alat / Tool** | `line` | `arrow` atau garis dengan port snapping / routing |
| **Bodi Objek** | **Bebas menembus / menabrak** interior bodi objek apa pun (dekoratif, coretan, underline). | **Dilarang menembus / masuk ke dalam bodi objek**. Ujung berlabuh di port perimeter luar (`top`, `right`, `bottom`, `left`). |
| **Magnetic Snap** | Tidak mencari port snap objek. | Snap magnetik otomatis ke port tepi objek terdekat (radius 24px). |
| **Routing / Siku** | Straight line 2-titik murni `(x1, y1) -> (x2, y2)`. | Smart orthogonal / curved routing dengan kalkulasi **Obstacle Avoidance** memutar mengitari luar bounding box objek. |
| **Endpoint Handles** | Handle titik bebas di scene kanvas. | Handle titik p1/p2 dapat ditautkan ke port objek lain dan memperbarui rute penghubung secara otomatis. |
| **Ekspor** | Bounding box standar garis 2-titik. | Waypoint-aware bounding box mencakup seluruh belokan siku. |
