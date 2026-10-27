import { Component, Show, JSX } from 'solid-js';
import { Target, Move, X } from 'lucide-solid';

export interface AvatarArControlsProps {
  isAvatarLocked: boolean;
  hasFoundSurface: boolean;
  onLockAvatar: () => void;
  onUnlockAvatar: () => void;
  onExitAr: () => void;
  bottomSlot?: JSX.Element;
}

export const AvatarArControls: Component<AvatarArControlsProps> = (props) => {
  return (
    <div class="absolute inset-0 pointer-events-none z-40 p-4 pb-6 flex flex-col justify-between select-none">
      {/* 1. Top Bar: Minimal Floating Status Dot on Left & Glassmorphic Exit Button on Right */}
      <div class="w-full flex items-center justify-between pointer-events-none pt-safe">
        {/* Minimal Floating Status Indicator (Pulsing Gold = Searching surface, Emerald = Anchored) */}
        <div class="flex items-center gap-2 px-2.5 py-1 rounded-full bg-gi-canvas/60 backdrop-blur-md border border-gi-border-subtle/40 shadow-sm pointer-events-none">
          <Show
            when={props.isAvatarLocked}
            fallback={
              <div class="flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-gi-primary animate-ping" />
                <span class="text-2xs font-mono text-gi-text-secondary">
                  {props.hasFoundSurface ? 'Surface Found' : 'Scan Surface...'}
                </span>
              </div>
            }
          >
            <div class="flex items-center gap-1.5">
              <span class="w-2 h-2 rounded-full bg-gi-status-success shadow-[0_0_8px_rgba(var(--gi-status-success-rgb),0.9)]" />
              <span class="text-2xs font-mono text-gi-text-secondary">Anchored</span>
            </div>
          </Show>
        </div>

        {/* Clean Circular Glass Exit Button */}
        <button
          type="button"
          onClick={props.onExitAr}
          class="pointer-events-auto w-10 h-10 rounded-full bg-gi-canvas/60 backdrop-blur-md border border-gi-border-subtle/50 text-gi-text-primary hover:text-gi-primary active:scale-90 transition-all flex items-center justify-center shadow-md cursor-pointer"
          aria-label="Exit AR Mode"
          title="Exit AR Mode"
        >
          <X size={20} class="stroke-[2.2]" />
        </button>
      </div>

      {/* 2. Middle Spacer */}
      <div class="flex-1 pointer-events-none" />

      {/* 3. Bottom Controls: Placement / Reposition Action + Interaction Dock */}
      <div class="w-full flex flex-col items-center gap-4 pointer-events-none pb-safe">
        {/* Floating Action Button: Anchor or Reposition */}
        <Show
          when={props.isAvatarLocked}
          fallback={
            <button
              type="button"
              onClick={props.onLockAvatar}
              disabled={!props.hasFoundSurface}
              class={`pointer-events-auto w-14 h-14 rounded-full flex items-center justify-center transition-all cursor-pointer ${
                props.hasFoundSurface
                  ? 'bg-gi-primary text-gi-primary-foreground shadow-[0_0_24px_rgba(var(--gi-primary-rgb),0.65)] active:scale-90 animate-pulse'
                  : 'bg-gi-canvas/60 backdrop-blur-md border border-gi-border-subtle/50 text-gi-text-muted/40 cursor-not-allowed opacity-40'
              }`}
              aria-label="Anchor Avatar"
              title={props.hasFoundSurface ? 'Anchor Avatar to Surface' : 'Scanning surface...'}
            >
              <Target size={28} class="stroke-[2.4]" />
            </button>
          }
        >
          <button
            type="button"
            onClick={props.onUnlockAvatar}
            class="pointer-events-auto w-12 h-12 rounded-full bg-gi-canvas/70 backdrop-blur-md border border-gi-border-gold/60 text-gi-primary hover:text-gi-primary-hover active:scale-90 transition-all flex items-center justify-center shadow-lg cursor-pointer"
            aria-label="Reposition Avatar"
            title="Reposition Avatar"
          >
            <Move size={22} class="stroke-[2.2]" />
          </button>
        </Show>

        {/* Bottom Interaction Slot (Mic & Chat Dock) */}
        <Show when={props.bottomSlot}>
          <div class="w-full pointer-events-auto">
            {props.bottomSlot}
          </div>
        </Show>
      </div>
    </div>
  );
};
