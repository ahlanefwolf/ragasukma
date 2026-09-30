import { Component, Show } from 'solid-js';
import { Monitor, Smartphone, Terminal, X, Settings2 } from 'lucide-solid';
import { MobileAudioMode } from './MobileHeader';

interface MobileSettingsSheetProps {
  isOpen: boolean;
  onClose: () => void;
  audioMode: MobileAudioMode;
  onSwitchMode: (mode: MobileAudioMode) => void;
  logsCount: number;
  onOpenLogs: () => void;
  onUnlockAudio?: () => void;
}

export const MobileSettingsSheet: Component<MobileSettingsSheetProps> = (props) => {
  return (
    <Show when={props.isOpen}>
      {/* Backdrop Transparan Halus (Tidak Gelap Pekat) */}
      <div
        class="fixed inset-0 z-40 bg-black/25 backdrop-blur-[2px] animate-in fade-in duration-150"
        onClick={props.onClose}
      />

      {/* Floating Bottom Sheet Sesuai Tema Cerah & Bersih */}
      <div class="fixed bottom-0 left-0 right-0 z-50 max-w-lg mx-auto p-4 animate-in slide-in-from-bottom duration-200 select-none">
        <div class="bg-theme-surface border border-theme-border p-5 shadow-2xl space-y-4">
          {/* Header Tanpa Kotak-kotak (Icon Langsung Berwarna) */}
          <div class="flex items-center justify-between pb-3 border-b border-theme-border/60">
            <div class="flex items-center gap-2.5">
              <Settings2 size={22} class="text-theme-cyan stroke-[2.2]" />
              <div>
                <h3 class="text-sm font-bold text-theme-text font-heading leading-tight">
                  Pengaturan Companion
                </h3>
                <p class="text-[11px] text-theme-muted font-mono">
                  Ragasukma Mobile Control
                </p>
              </div>
            </div>
            <button
              type="button"
              onClick={props.onClose}
              class="p-1.5 text-theme-muted hover:text-theme-text active:scale-95 transition-colors"
              title="Tutup"
            >
              <X size={20} />
            </button>
          </div>

          {/* Section: Mode Audio & Input */}
          <div class="space-y-2">
            <label class="text-[11px] font-mono font-bold text-theme-muted uppercase tracking-wider block">
              Mode Audio & Percakapan
            </label>
            <div class="grid grid-cols-2 gap-2.5">
              {/* Tombol Remote PC */}
              <button
                type="button"
                onClick={() => props.onSwitchMode('remote_desktop')}
                class={`p-3 border text-left flex flex-col gap-1 transition-all active:scale-98 ${
                  props.audioMode === 'remote_desktop'
                    ? 'bg-theme-cyan text-theme-bg border-theme-cyan font-semibold shadow-sm'
                    : 'bg-theme-card border-theme-border text-theme-text hover:border-theme-cyan/60'
                }`}
              >
                <div class="flex items-center justify-between">
                  <Monitor size={17} class={props.audioMode === 'remote_desktop' ? 'text-theme-bg' : 'text-theme-cyan'} />
                  <Show when={props.audioMode === 'remote_desktop'}>
                    <span class="text-[10px] font-mono font-bold uppercase tracking-wider">AKTIF</span>
                  </Show>
                </div>
                <span class="text-xs font-bold mt-1">Remote PC</span>
                <span class={`text-[10.5px] leading-tight ${props.audioMode === 'remote_desktop' ? 'opacity-90' : 'text-theme-muted'}`}>
                  Suara & mic asisten di speaker PC
                </span>
              </button>

              {/* Tombol Full HP */}
              <button
                type="button"
                onClick={() => props.onSwitchMode('full_mobile')}
                class={`p-3 border text-left flex flex-col gap-1 transition-all active:scale-98 ${
                  props.audioMode === 'full_mobile'
                    ? 'bg-theme-cyan text-theme-bg border-theme-cyan font-semibold shadow-sm'
                    : 'bg-theme-card border-theme-border text-theme-text hover:border-theme-cyan/60'
                }`}
              >
                <div class="flex items-center justify-between">
                  <Smartphone size={17} class={props.audioMode === 'full_mobile' ? 'text-theme-bg' : 'text-theme-cyan'} />
                  <Show when={props.audioMode === 'full_mobile'}>
                    <span class="text-[10px] font-mono font-bold uppercase tracking-wider">AKTIF</span>
                  </Show>
                </div>
                <span class="text-xs font-bold mt-1">Full Mobile</span>
                <span class={`text-[10.5px] leading-tight ${props.audioMode === 'full_mobile' ? 'opacity-90' : 'text-theme-muted'}`}>
                  Suara & mic asisten di HP
                </span>
              </button>
            </div>
          </div>

          {/* Section: Live Console Logs */}
          <div class="pt-1">
            <button
              type="button"
              onClick={() => {
                props.onClose();
                props.onOpenLogs();
              }}
              class="w-full p-3 bg-theme-card hover:bg-theme-bg border border-theme-border hover:border-theme-cyan text-theme-text flex items-center justify-between transition-all active:scale-98"
            >
              <div class="flex items-center gap-2.5">
                <Terminal size={17} class="text-theme-cyan" />
                <div class="text-left">
                  <span class="text-xs font-bold block">Live System Logs</span>
                  <span class="text-[10.5px] text-theme-muted">Pantau console & transkrip AI</span>
                </div>
              </div>
              <span class="px-2 py-0.5 text-[10px] font-mono font-bold bg-theme-surface text-theme-cyan border border-theme-border">
                {props.logsCount} baris
              </span>
            </button>
          </div>

          {/* Section: Audio Sync Reset */}
          <Show when={props.onUnlockAudio}>
            <button
              type="button"
              onClick={() => {
                props.onUnlockAudio?.();
                props.onClose();
              }}
              class="w-full py-1.5 text-center text-[11px] font-mono text-theme-muted hover:text-theme-cyan transition-colors"
            >
              Sinkronkan Ulang Audio HP
            </button>
          </Show>
        </div>
      </div>
    </Show>
  );
};
