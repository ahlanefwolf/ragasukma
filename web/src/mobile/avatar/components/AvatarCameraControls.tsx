import { Component, Show } from 'solid-js';
import { Maximize, Minimize, ZoomIn, ZoomOut, RotateCcw } from 'lucide-solid';
import { FpsLimit, RenderQuality } from '../types';

interface AvatarCameraControlsProps {
  isFullscreen: boolean;
  quality?: RenderQuality;
  fpsLimit?: FpsLimit;
  onZoomIn?: () => void;
  onZoomOut?: () => void;
  onToggleFullscreen: () => void;
  onResetCamera?: () => void;
  onCycleQuality?: () => void;
  onCycleFps?: () => void;
}

export const AvatarCameraControls: Component<AvatarCameraControlsProps> = (props) => {
  const formatQualityLabel = (q?: RenderQuality) => {
    switch (q) {
      case 'sd':
        return 'SD';
      case 'ultra':
        return '4K';
      case 'hd':
      default:
        return 'HD';
    }
  };

  const formatFpsLabel = (fps?: FpsLimit) => {
    if (fps === 0) return 'MAX';
    if (fps === 30) return '30F';
    return '60F';
  };

  return (
    <div class="absolute top-16 right-3.5 z-20 flex flex-col items-center gap-2 pointer-events-auto select-none">
      {/* 1. Quality Toggle (SD / HD / 4K) */}
      <Show when={props.onCycleQuality}>
        <button
          type="button"
          onClick={props.onCycleQuality}
          class="p-2 text-2xs font-mono font-bold tracking-wider text-white/80 hover:text-gi-primary-hover active:scale-75 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.9)] cursor-pointer"
          title={`Render Quality: ${formatQualityLabel(props.quality)} (Tap to switch)`}
          aria-label={`Render Quality: ${formatQualityLabel(props.quality)}`}
        >
          {formatQualityLabel(props.quality)}
        </button>
      </Show>

      {/* 2. FPS Limit Toggle (30F / 60F / MAX) */}
      <Show when={props.onCycleFps}>
        <button
          type="button"
          onClick={props.onCycleFps}
          class="p-2 text-2xs font-mono font-bold tracking-wider text-white/80 hover:text-gi-primary-hover active:scale-75 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.9)] cursor-pointer"
          title={`FPS Limit: ${formatFpsLabel(props.fpsLimit)} (Tap to switch)`}
          aria-label={`FPS Limit: ${formatFpsLabel(props.fpsLimit)}`}
        >
          {formatFpsLabel(props.fpsLimit)}
        </button>
      </Show>

      {/* 3. Zoom In */}
      <Show when={props.onZoomIn}>
        <button
          type="button"
          onClick={props.onZoomIn}
          class="p-2 text-white/80 hover:text-white active:scale-75 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.9)] cursor-pointer"
          title="Zoom In"
          aria-label="Zoom In"
        >
          <ZoomIn size={20} class="stroke-[2.2]" />
        </button>
      </Show>

      {/* 4. Zoom Out */}
      <Show when={props.onZoomOut}>
        <button
          type="button"
          onClick={props.onZoomOut}
          class="p-2 text-white/80 hover:text-white active:scale-75 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.9)] cursor-pointer"
          title="Zoom Out"
          aria-label="Zoom Out"
        >
          <ZoomOut size={20} class="stroke-[2.2]" />
        </button>
      </Show>

      {/* 5. Reset Camera */}
      <Show when={props.onResetCamera}>
        <button
          type="button"
          onClick={props.onResetCamera}
          class="p-2 text-white/80 hover:text-white active:scale-75 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.9)] cursor-pointer"
          title="Reset Camera View"
          aria-label="Reset Camera View"
        >
          <RotateCcw size={18} class="stroke-[2.2]" />
        </button>
      </Show>

      {/* 6. Fullscreen Toggle */}
      <button
        type="button"
        onClick={props.onToggleFullscreen}
        class="p-2 text-white/80 hover:text-white active:scale-75 transition-all drop-shadow-[0_2px_8px_rgba(0,0,0,0.9)] cursor-pointer"
        title={props.isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
        aria-label={props.isFullscreen ? 'Exit Fullscreen' : 'Fullscreen'}
      >
        <Show when={props.isFullscreen} fallback={<Maximize size={20} class="stroke-[2.2]" />}>
          <Minimize size={20} class="stroke-[2.2]" />
        </Show>
      </button>
    </div>
  );
};
