import { Component, Show } from 'solid-js';
import { ZoomIn, ZoomOut, Maximize, Minimize, RotateCcw, Glasses, Sparkles, Gauge } from 'lucide-solid';
import { FpsLimit, RenderQuality } from '../types';

interface AvatarCameraControlsProps {
  isFullscreen: boolean;
  isArSupported: boolean;
  quality: RenderQuality;
  fpsLimit: FpsLimit;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onToggleFullscreen: () => void;
  onResetCamera: () => void;
  onStartAr: () => void;
  onCycleQuality: () => void;
  onCycleFps: () => void;
}

export const AvatarCameraControls: Component<AvatarCameraControlsProps> = (props) => {
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
    <div class="absolute top-14 right-4 z-20 flex flex-col items-center gap-0.5 pointer-events-auto select-none">
      {/* AR Mode Button - Clean without box & border */}
      <Show when={props.isArSupported}>
        <button
          type="button"
          onClick={props.onStartAr}
          class="p-2 text-gi-primary hover:text-gi-primary-hover active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
          title="Open AR Mode (Real World)"
        >
          <Glasses size={22} class="stroke-[2.2]" />
        </button>
      </Show>

      {/* Graphic Quality Button (SD -> HD -> 4K) */}
      <button
        type="button"
        onClick={props.onCycleQuality}
        class={`p-2 font-mono text-[11px] font-bold tracking-wider active:scale-85 transition-transform drop-shadow-sm cursor-pointer ${qualityColor()}`}
        title={`Render Quality: ${qualityLabel()} (Tap to switch SD / HD / 4K)`}
      >
        <span>{qualityLabel()}</span>
      </button>

      {/* FPS Limit Toggle Button (30 FPS -> 60 FPS -> Max) */}
      <button
        type="button"
        onClick={props.onCycleFps}
        class={`p-2 font-mono text-[11px] font-bold tracking-wider active:scale-85 transition-transform drop-shadow-sm cursor-pointer ${
          props.fpsLimit === 30 ? 'text-gi-status-success hover:opacity-85' : props.fpsLimit === 60 ? 'text-gi-primary hover:text-gi-primary-hover' : 'text-gi-status-info hover:opacity-85'
        }`}
        title={`FPS Limit: ${props.fpsLimit === 0 ? 'Max' : `${props.fpsLimit} FPS`} (Tap to switch 30 / 60 / Max)`}
      >
        <span>{props.fpsLimit === 0 ? 'MAX' : `${props.fpsLimit}F`}</span>
      </button>

      <button
        type="button"
        onClick={props.onZoomIn}
        class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
        title="Zoom In"
      >
        <ZoomIn size={22} class="stroke-[2.2]" />
      </button>
      <button
        type="button"
        onClick={props.onZoomOut}
        class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
        title="Zoom Out"
      >
        <ZoomOut size={22} class="stroke-[2.2]" />
      </button>
      <button
        type="button"
        onClick={props.onToggleFullscreen}
        class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
        title={props.isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
      >
        <Show when={props.isFullscreen} fallback={<Maximize size={20} class="stroke-[2.2]" />}>
          <Minimize size={20} class="stroke-[2.2]" />
        </Show>
      </button>
      <button
        type="button"
        onClick={props.onResetCamera}
        class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
        title="Reset Camera Angle"
      >
        <RotateCcw size={20} class="stroke-[2.2]" />
      </button>
    </div>
  );
};
