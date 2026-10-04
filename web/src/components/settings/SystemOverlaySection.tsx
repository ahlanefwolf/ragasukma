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
        <h3 class="text-base sm:text-lg font-bold text-theme-cyan flex items-center gap-2">
          <Monitor size={18} />
          Desktop Overlay & Window Preferences
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="flex items-center justify-between p-4 bg-theme-card border border-theme-border">
            <div>
              <span class="block text-sm font-bold text-theme-text">Always On Top</span>
              <span class="text-xs text-theme-text/60">Keep 3D avatar overlay above all other application windows</span>
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
            <span class="font-bold text-theme-text block mb-0.5">GPU & Rendering Performance Note</span>
            Application runs on native graphics backends (DirectX 12 / Vulkan / Metal) with minimal VRAM footprint. Window settings automatically sync to local config.
          </div>
        </div>
      </div>
    </div>
  );
};
