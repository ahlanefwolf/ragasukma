import { Component } from 'solid-js';
import { Loader2 } from 'lucide-solid';

interface AvatarLoadingOverlayProps {
  progress: number;
}

export const AvatarLoadingOverlay: Component<AvatarLoadingOverlayProps> = (props) => {
  return (
    <div class="absolute inset-0 z-30 bg-theme-bg/85 backdrop-blur-md flex flex-col items-center justify-center gap-3 text-theme-cyan">
      <Loader2 size={28} class="animate-spin text-theme-cyan" />
      <span class="text-xs font-mono font-semibold tracking-wide">
        {props.progress > 0 ? `Loading 3D Character (${props.progress}%)` : 'Initializing 3D Engine...'}
      </span>
    </div>
  );
};
