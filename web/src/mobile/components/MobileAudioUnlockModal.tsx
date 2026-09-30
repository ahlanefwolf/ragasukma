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
        <div class="max-w-xs w-full bg-theme-surface border border-theme-cyan/50 p-6 rounded-3xl shadow-2xl flex flex-col items-center gap-4">
          <div class="w-16 h-16 rounded-full bg-theme-cyan/20 border border-theme-cyan/40 flex items-center justify-center text-theme-cyan shadow-[0_0_20px_var(--cyan)]">
            <Volume2 size={32} />
          </div>

          <div class="space-y-1">
            <h2 class="text-base font-heading font-bold text-theme-text">
              Hubungkan Audio HP
            </h2>
            <p class="text-xs text-theme-text/60 leading-relaxed">
              Browser smartphone memerlukan 1x izin sentuhan agar suara asisten & musik otomatis dialihkan ke speaker smartphone ini.
            </p>
          </div>

          <button
            type="button"
            onClick={props.onUnlock}
            class="w-full py-3 px-4 bg-theme-cyan text-theme-card font-bold text-xs uppercase tracking-wider rounded-xl shadow-[0_0_15px_var(--cyan)] active:scale-95 transition-transform"
          >
            Aktifkan Audio HP
          </button>
        </div>
      </div>
    </Show>
  );
};
