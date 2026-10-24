import { Component, Show } from 'solid-js';
import { Smartphone, Mic, Terminal, X, Settings2, Power } from 'lucide-solid';
export type MobileAudioMode = 'remote_desktop' | 'full_mobile';

interface MobileSettingsSheetProps {
  isOpen: boolean;
  onClose: () => void;
  audioMode: MobileAudioMode;
  onSwitchMode: (mode: MobileAudioMode) => void;
  logsCount: number;
  onOpenLogs: () => void;
  onUnlockAudio?: () => void;
  onDisconnectRemote?: () => void;
}

export const MobileSettingsSheet: Component<MobileSettingsSheetProps> = (props) => {
  return (
    <Show when={props.isOpen}>
      {/* Backdrop Transparan Smooth (Not Gelap Pekat) */}
      <div
        class="fixed inset-0 z-40 bg-black/25 backdrop-blur-[2px] animate-in fade-in duration-150"
        onClick={props.onClose}
      />

      {/* Floating Bottom Sheet Sesuai Tema Cerah & Clean */}
      <div class="fixed bottom-0 left-0 right-0 z-50 max-w-lg mx-auto p-4 animate-in slide-in-from-bottom duration-200 select-none">
        <div class="bg-theme-surface border border-theme-border p-5 shadow-2xl space-y-4">
          {/* Header Without Kotak-kotak (Icon Direct Berwarna) */}
          <div class="flex items-center justify-between pb-3 border-b border-theme-border/60">
            <div class="flex items-center gap-2.5">
              <Settings2 size={22} class="text-theme-cyan stroke-[2.2]" />
              <div>
                <h3 class="text-theme-text font-heading leading-tight">
                  Companion Settings
                </h3>
                <p class="text-xs text-theme-muted font-mono">
                  Ragasukma Mobile Control
                </p>
              </div>
            </div>
            <button
              type="button"
              onClick={props.onClose}
              class="p-1.5 text-theme-muted hover:text-theme-text active:scale-95 transition-colors"
              title="Close"
            >
              <X size={20} />
            </button>
          </div>

          {/* Section: Mode Audio & Input */}
          <div class="space-y-2">
            <label class="text-xs font-mono font-bold text-theme-muted uppercase tracking-wider block">
              Audio & Conversation Mode
            </label>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              {/* Remote Desktop Button */}
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
                  <Mic size={17} class={props.audioMode === 'remote_desktop' ? 'text-theme-bg' : 'text-theme-cyan'} />
                  <Show when={props.audioMode === 'remote_desktop'}>
                    <span class="text-2xs font-mono font-bold uppercase tracking-wider">ACTIVE</span>
                  </Show>
                </div>
                <span class="text-xs font-bold mt-1">Remote PC</span>
                <span class={`text-2xs leading-tight ${props.audioMode === 'remote_desktop' ? 'opacity-90' : 'text-theme-muted'}`}>
                  Speaker PC, Mic HP
                </span>
              </button>

              {/* Phone Only Button */}
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
                    <span class="text-2xs font-mono font-bold uppercase tracking-wider">ACTIVE</span>
                  </Show>
                </div>
                <span class="text-xs font-bold mt-1">Phone Only</span>
                <span class={`text-2xs leading-tight ${props.audioMode === 'full_mobile' ? 'opacity-90' : 'text-theme-muted'}`}>
                  Speaker & Mic di HP
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
                  <span class="text-2xs text-theme-muted">Monitor console & AI transcripts</span>
                </div>
              </div>
              <span class="px-2 py-0.5 text-2xs font-mono font-bold bg-theme-surface text-theme-cyan border border-theme-border">
                {props.logsCount} lines
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
              class="w-full py-1.5 text-center text-xs font-mono text-theme-muted hover:text-theme-cyan transition-colors"
            >
              Re-sync Phone Audio
            </button>
          </Show>

          {/* Section: Putuskan Session Remote */}
          <Show when={props.onDisconnectRemote}>
            <div class="pt-2 border-t border-theme-border/60">
              <button
                type="button"
                onClick={() => {
                  props.onClose();
                  props.onDisconnectRemote?.();
                }}
                class="w-full p-2.5 bg-theme-rose/10 hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/30 flex items-center justify-center gap-2 text-xs font-bold transition-all active:scale-98"
              >
                <Power size={15} />
                <span>Disconnect Remote Session (Return to PC)</span>
              </button>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
