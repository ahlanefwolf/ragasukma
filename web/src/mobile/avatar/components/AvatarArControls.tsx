import { Component, Show, JSX } from 'solid-js';
import { Lock, Unlock, X } from 'lucide-solid';

interface AvatarArControlsProps {
  isAvatarLocked: boolean;
  hasFoundSurface: boolean;
  onLockAvatar: () => void;
  onUnlockAvatar: () => void;
  onExitAr: () => void;
  bottomSlot?: JSX.Element;
}

export const AvatarArControls: Component<AvatarArControlsProps> = (props) => {
  return (
    <div class="absolute inset-0 pointer-events-none z-40 p-4 flex flex-col justify-between">
      {/* Floating Action Icons: X dan Lock/Unlock di sudut kanan atas tanpa box/border/teks */}
      <div class="absolute top-4 right-4 flex flex-col items-center gap-3 pointer-events-auto select-none">
        {/* Tombol Keluar (X) */}
        <button
          type="button"
          onClick={props.onExitAr}
          class="p-2 text-white/85 hover:text-white active:scale-80 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.85)] cursor-pointer"
          aria-label="Exit AR"
        >
          <X size={26} class="stroke-[2.2]" />
        </button>

        {/* Tombol Lock / Unlock tepat di bawah X */}
        <Show
          when={props.isAvatarLocked}
          fallback={
            <button
              type="button"
              disabled={!props.hasFoundSurface}
              onClick={props.onLockAvatar}
              class={`p-2 transition-all duration-200 active:scale-80 drop-shadow-[0_2px_8px_rgba(0,0,0,0.85)] cursor-pointer ${
                props.hasFoundSurface
                  ? 'text-theme-cyan hover:text-cyan-300 scale-105'
                  : 'text-white/25 cursor-not-allowed'
              }`}
              aria-label="Lock Avatar"
            >
              <Lock size={24} class="stroke-[2.2]" />
            </button>
          }
        >
          <button
            type="button"
            onClick={props.onUnlockAvatar}
            class="p-2 text-amber-300 hover:text-amber-200 active:scale-80 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.85)] cursor-pointer"
            aria-label="Unlock Avatar"
          >
            <Unlock size={24} class="stroke-[2.2]" />
          </button>
        </Show>
      </div>

      {/* Spacer atas */}
      <div class="flex-1 pointer-events-none" />

      {/* Slot Bawah: Mic & Chat Dock di Mode AR */}
      <Show when={props.bottomSlot}>
        <div class="w-full pointer-events-auto pb-4">
          {props.bottomSlot}
        </div>
      </Show>
    </div>
  );
};
