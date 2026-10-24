import { Component, Show, JSX } from 'solid-js';
import { Lock, Unlock, X } from 'lucide-solid';
import { FpsLimit, RenderQuality } from '../types';

interface AvatarArControlsProps {
  isAvatarLocked: boolean;
  hasFoundSurface: boolean;
  quality: RenderQuality;
  fpsLimit: FpsLimit;
  onLockAvatar: () => void;
  onUnlockAvatar: () => void;
  onExitAr: () => void;
  onCycleQuality: () => void;
  onCycleFps: () => void;
  bottomSlot?: JSX.Element;
}

export const AvatarArControls: Component<AvatarArControlsProps> = (props) => {
  const qualityLabel = () => {
    switch (props.quality) {
      case 'sd':
        return 'SD';
      case 'ultra':
        return '4K';
      case 'hd':
      default:
        return 'HD';
    }
  };

  const qualityColor = () => {
    switch (props.quality) {
      case 'sd':
        return 'text-gi-status-warning hover:opacity-85';
      case 'ultra':
        return 'text-gi-primary hover:text-gi-primary-hover';
      case 'hd':
      default:
        return 'text-gi-status-info hover:opacity-85';
    }
  };

  return (
    <div class="absolute inset-0 pointer-events-none z-40 p-4 flex flex-col justify-between">
      {/* Floating Action Icons: X, Lock, Quality & FPS di sudut kanan atas */}
      <div class="absolute top-4 right-4 flex flex-col items-center gap-2 pointer-events-auto select-none">
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
                  ? 'text-gi-primary hover:text-gi-primary-hover scale-105'
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
            class="p-2 text-gi-status-warning hover:opacity-85 active:scale-80 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.85)] cursor-pointer"
            aria-label="Unlock Avatar"
          >
            <Unlock size={24} class="stroke-[2.2]" />
          </button>
        </Show>

        {/* AR Quality Switcher */}
        <button
          type="button"
          onClick={props.onCycleQuality}
          class={`p-2 font-mono text-xs font-bold tracking-wider active:scale-85 transition-transform drop-shadow-[0_2px_8px_rgba(0,0,0,0.85)] cursor-pointer ${qualityColor()}`}
          title={`AR Render Quality: ${qualityLabel()}`}
        >
          <span>{qualityLabel()}</span>
        </button>

        {/* AR FPS Switcher */}
        <button
          type="button"
          onClick={props.onCycleFps}
          class={`p-2 font-mono text-xs font-bold tracking-wider active:scale-85 transition-transform drop-shadow-[0_2px_8px_rgba(0,0,0,0.85)] cursor-pointer ${
            props.fpsLimit === 30 ? 'text-gi-status-success hover:opacity-85' : props.fpsLimit === 60 ? 'text-gi-primary hover:text-gi-primary-hover' : 'text-gi-status-info hover:opacity-85'
          }`}
          title={`AR FPS Limit: ${props.fpsLimit === 0 ? 'Max' : `${props.fpsLimit} FPS`}`}
        >
          <span>{props.fpsLimit === 0 ? 'MAX' : `${props.fpsLimit}F`}</span>
        </button>
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
