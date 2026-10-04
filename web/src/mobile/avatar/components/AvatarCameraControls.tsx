import { Component, Show } from 'solid-js';
import { ZoomIn, ZoomOut, Maximize, Minimize, RotateCcw } from 'lucide-solid';

interface AvatarCameraControlsProps {
  isFullscreen: boolean;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onToggleFullscreen: () => void;
  onResetCamera: () => void;
}

export const AvatarCameraControls: Component<AvatarCameraControlsProps> = (props) => {
  return (
    <div class="absolute top-14 right-4 z-20 flex flex-col items-center gap-0.5 pointer-events-auto select-none">
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
