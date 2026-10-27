import { Component, Show } from 'solid-js';
import { Volume2 } from 'lucide-solid';

interface MobileAudioUnlockModalProps {
  show: boolean;
  onUnlock: () => void;
}

export const MobileAudioUnlockModal: Component<MobileAudioUnlockModalProps> = (props) => {
  return (
    <Show when={props.show}>
      <div class="fixed inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-6 text-center select-none animate-in fade-in duration-200">
        <div class="max-w-xs w-full bg-theme-surface border border-gi-primary/50 p-6 shadow-2xl flex flex-col items-center gap-4">
          <div class="w-16 h-16 bg-gi-primary/20 border border-gi-primary/40 flex items-center justify-center text-gi-primary shadow-[0_0_20px_rgba(var(--gi-primary-rgb),0.6)]">
            <Volume2 size={32} />
          </div>

          <div class="space-y-1">
            <h2 class="font-heading text-theme-text">
              Connect Phone Audio
            </h2>
            <p class="text-xs text-theme-text/60 leading-relaxed">
              Smartphone browsers require 1x touch gesture to route assistant voice & music to phone speakers.
            </p>
          </div>

          <button
            type="button"
            onClick={props.onUnlock}
            class="w-full py-3 px-4 bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground font-bold text-xs uppercase tracking-wider shadow-[0_0_15px_rgba(var(--gi-primary-rgb),0.4)] active:scale-95 transition-transform"
          >
            Enable Phone Audio
          </button>
        </div>
      </div>
    </Show>
  );
};
