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
        <div class="max-w-xs w-full bg-theme-surface border border-[#E5B85C]/50 p-6 rounded-3xl shadow-2xl flex flex-col items-center gap-4">
          <div class="w-16 h-16 rounded-full bg-[#E5B85C]/20 border border-[#E5B85C]/40 flex items-center justify-center text-[#E5B85C] shadow-[0_0_20px_rgba(229,184,92,0.6)]">
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
            class="w-full py-3 px-4 bg-[#E5B85C] hover:bg-[#F3CE7A] text-[#111318] font-bold text-xs uppercase tracking-wider rounded-xl shadow-[0_0_15px_rgba(229,184,92,0.4)] active:scale-95 transition-transform"
          >
            Enable Phone Audio
          </button>
        </div>
      </div>
    </Show>
  );
};
