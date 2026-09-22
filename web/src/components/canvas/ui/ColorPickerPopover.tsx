import { Component, Show } from 'solid-js';
import { ChevronDown } from 'lucide-solid';
import { parseColor, toColorString } from './colorUtils';

interface ColorPickerPopoverProps {
  label: string;
  color: string;
  isOpen: boolean;
  onToggle: () => void;
  onChangeColor: (colorStr: string) => void;
  onAfterChange?: () => void;
}

export const ColorPickerPopover: Component<ColorPickerPopoverProps> = (props) => {
  const parsed = () => parseColor(props.color);

  const handleHexInput = (hex: string) => {
    const curAlpha = parsed().alpha === 0 ? 100 : parsed().alpha;
    props.onChangeColor(toColorString(hex, curAlpha));
    props.onAfterChange?.();
  };

  const handleAlphaInput = (alpha: number) => {
    props.onChangeColor(toColorString(parsed().hex, alpha));
    props.onAfterChange?.();
  };

  return (
    <div class="relative">
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          props.onToggle();
        }}
        title={`${props.label} & Opacity`}
        class="flex items-center gap-1.5 px-2 py-1 bg-theme-surface hover:bg-theme-card border border-theme-border transition-all"
      >
        <span class="text-[10px] text-theme-text/50 font-bold uppercase">{props.label}:</span>
        <div
          class="w-4 h-4 border border-theme-border relative flex items-center justify-center overflow-hidden"
          style={{ 'background-color': props.color === 'transparent' ? 'transparent' : props.color }}
        >
          <Show when={props.color === 'transparent' || parsed().alpha === 0}>
            <span class="text-[10px] text-theme-rose font-bold">/</span>
          </Show>
        </div>
        <span class="text-[11px] text-theme-text/90 font-bold">{parsed().alpha}%</span>
        <ChevronDown size={11} class="text-theme-text/50" />
      </button>

      <Show when={props.isOpen}>
        <div class="absolute left-0 top-full mt-1.5 bg-theme-card border border-theme-border shadow-2xl p-2.5 z-[100] flex flex-col gap-2 min-w-[190px]">
          <span class="text-[10px] font-bold text-theme-text/60 uppercase">{props.label}</span>
          <div class="flex items-center gap-2">
            <input
              type="color"
              value={parsed().hex}
              onInput={(e) => handleHexInput(e.currentTarget.value)}
              class="w-8 h-8 p-0 border border-theme-border cursor-pointer bg-transparent"
            />
            <span class="text-xs font-mono font-bold text-theme-text/80 uppercase">
              {parsed().hex}
            </span>
          </div>
          <div class="flex flex-col gap-1 pt-1 border-t border-theme-border/60">
            <div class="flex justify-between text-[10px] text-theme-text/60 font-bold">
              <span>Transparansi</span>
              <span>{parsed().alpha}%</span>
            </div>
            <input
              type="range"
              min="0"
              max="100"
              value={parsed().alpha}
              onInput={(e) => handleAlphaInput(parseInt(e.currentTarget.value, 10))}
              class="w-full h-1.5 accent-theme-cyan bg-theme-bg cursor-pointer"
            />
          </div>
        </div>
      </Show>
    </div>
  );
};
