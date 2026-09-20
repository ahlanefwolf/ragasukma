import { Component } from 'solid-js';
import { Monitor, Info } from 'lucide-solid';

interface SystemOverlaySectionProps {
  alwaysOnTop: boolean;
  setAlwaysOnTop: (val: boolean) => void;
}

export const SystemOverlaySection: Component<SystemOverlaySectionProps> = (props) => {
  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-cyan flex items-center gap-2">
          <Monitor size={18} />
          DESKTOP OVERLAY & WINDOW PREFERENCES
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="flex items-center justify-between p-4 bg-theme-card border border-theme-border">
            <div>
              <span class="block text-sm font-bold text-theme-text">Always On Top (Selalu di Depan)</span>
              <span class="text-xs text-theme-text/60">Overlay 3D avatar tetap berada di atas jendela aplikasi lain</span>
            </div>
            <button
              type="button"
              onClick={() => props.setAlwaysOnTop(!props.alwaysOnTop)}
              class={`px-3 py-1.5 text-xs font-bold uppercase font-mono transition-all ${
                props.alwaysOnTop
                  ? 'bg-theme-cyan text-theme-card shadow anime-shadow-green'
                  : 'bg-theme-surface text-theme-text/60 border border-theme-border'
              }`}
            >
              {props.alwaysOnTop ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>

        <div class="p-3.5 bg-theme-card border border-theme-border flex items-start gap-3 text-xs text-theme-text/70 leading-relaxed">
          <Info size={16} class="text-theme-cyan shrink-0 mt-0.5" />
          <div>
            <span class="font-bold text-theme-text block mb-0.5">Catatan Performa GPU & Rendering</span>
            Aplikasi menggunakan backend grafis native (DirectX 12 / Vulkan / Metal) dengan konsumsi VRAM minimal. Pengaturan jendela otomatis disinkronkan ke config lokal.
          </div>
        </div>
      </div>
    </div>
  );
};
