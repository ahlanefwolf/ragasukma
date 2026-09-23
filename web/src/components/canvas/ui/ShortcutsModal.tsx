import { Component, Show, For } from 'solid-js';
import { Keyboard, X, Copy, Scissors, Undo2, Redo2, ZoomIn, Maximize, MousePointer } from 'lucide-solid';

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ShortcutsModal: Component<ShortcutsModalProps> = (props) => {
  const shortcutGroups = [
    {
      title: 'Editing & Manipulasi Objek',
      items: [
        { label: 'Gabung Jadi Group', keys: ['Ctrl', 'G'] },
        { label: 'Pisahkan Group (Ungroup)', keys: ['Ctrl', 'Shift', 'G'] },
        { label: 'Pilih Multi Objek (Toggle)', keys: ['Ctrl', 'Klik'] },
        { label: 'Pilih Rentang Objek', keys: ['Shift', 'Klik'] },
        { label: 'Salin Objek (Copy)', keys: ['Ctrl', 'C'] },
        { label: 'Tempel Objek / Gambar (Paste)', keys: ['Ctrl', 'V'] },
        { label: 'Duplikasi Objek Instan', keys: ['Ctrl', 'D'] },
        { label: 'Hapus Objek Terpilih', keys: ['Delete', 'Backspace'] },
        { label: 'Undo (Batalkan Aksi)', keys: ['Ctrl', 'Z'] },
        { label: 'Redo (Ulangi Aksi)', keys: ['Ctrl', 'Y / Shift+Z'] },
      ],
    },
    {
      title: 'Alat Gambar & Bentuk (Tools)',
      items: [
        { label: 'Pilih / Transform (Select)', keys: ['V', 'S'] },
        { label: 'Kuas Bebas (Brush)', keys: ['B', 'P'] },
        { label: 'Kotak / Rectangle', keys: ['R'] },
        { label: 'Lingkaran / Circle', keys: ['C'] },
        { label: 'Panah Relasi (Smart Arrow)', keys: ['A'] },
        { label: 'Garis Lurus / Polygon (Line)', keys: ['L'] },
        { label: 'Teks Label (Text)', keys: ['T'] },
        { label: 'Hapus Langsung (Eraser)', keys: ['E'] },
      ],
    },
    {
      title: 'Navigasi, Panel & Tampilan',
      items: [
        { label: 'Pan / Geser Kanvas', keys: ['Space', 'Drag'] },
        { label: 'Zoom In / Out Kanvas', keys: ['Ctrl', '+ / -'] },
        { label: 'Reset Zoom (100%)', keys: ['Ctrl', '0'] },
        { label: 'Toggle Fullscreen Mode', keys: ['F'] },
        { label: 'Tutup Modal / Keluar Fullscreen', keys: ['Escape'] },
        { label: 'Toggle Snap to Dot Grid (24px)', keys: ['G'] },
        { label: 'Buka Ekspor Gambar', keys: ['Ctrl', 'E'] },
        { label: 'Buka AI MCP Studio', keys: ['Ctrl', 'K'] },
      ],
    },
  ];

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-[150] flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
        <div class="bg-theme-card border border-theme-cyan/50 w-full max-w-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
          {/* Modal Header */}
          <div class="bg-gradient-to-r from-theme-surface to-theme-bg p-4 border-b border-theme-border flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <div class="p-2 bg-theme-cyan/20 border border-theme-cyan/40 text-theme-cyan">
                <Keyboard size={18} />
              </div>
              <div>
                <h3 class="text-base font-bold text-theme-text uppercase tracking-wide">
                  PANDUAN SHORTCUT KEYBOARD
                </h3>
                <p class="text-xs text-theme-text/60">
                  Daftar tombol pintas untuk mempercepat pembuatan diagram dan whiteboard.
                </p>
              </div>
            </div>
            <button
              type="button"
              onClick={props.onClose}
              class="text-theme-text/60 hover:text-theme-rose p-1.5 transition-colors"
            >
              <X size={18} />
            </button>
          </div>

          {/* Modal Body */}
          <div class="p-4 sm:p-5 space-y-5 overflow-y-auto">
            <For each={shortcutGroups}>
              {(group) => (
                <div class="space-y-2.5">
                  <span class="text-xs font-mono font-bold text-theme-cyan uppercase tracking-wider flex items-center gap-1.5">
                    <span class="w-1.5 h-1.5 bg-theme-cyan inline-block" />
                    {group.title}
                  </span>
                  <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                    <For each={group.items}>
                      {(item) => (
                        <div class="flex items-center justify-between p-2.5 bg-theme-surface border border-theme-border hover:border-theme-border/80 transition-colors">
                          <span class="text-xs text-theme-text/80">{item.label}</span>
                          <div class="flex items-center gap-1">
                            <For each={item.keys}>
                              {(k) => (
                                <kbd class="px-2 py-0.5 bg-theme-bg text-theme-cyan font-mono font-bold border border-theme-border text-[11px] shadow-sm">
                                  {k}
                                </kbd>
                              )}
                            </For>
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>

          {/* Modal Footer */}
          <div class="bg-theme-surface p-3.5 border-t border-theme-border flex items-center justify-between">
            <span class="text-xs font-mono text-theme-text/60 hidden sm:inline">
              Tips: Tahan tombol <kbd class="px-1.5 py-0.5 bg-theme-bg text-theme-cyan font-mono border border-theme-border">Space</kbd> untuk menggeser kanvas dengan mouse.
            </span>
            <button
              type="button"
              onClick={props.onClose}
              class="px-5 py-2 text-xs font-bold bg-theme-cyan hover:bg-[#4a5a3a] text-theme-card border border-theme-cyan shadow-sm anime-shadow-green ml-auto transition-all"
            >
              Tutup
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default ShortcutsModal;
