import { Component, Show } from 'solid-js';
import { RotateCcw } from 'lucide-solid';
import { AvatarState } from '../types';
import { AVATAR_IMAGES, getStatusBadge } from '../constants';

interface Avatar2DFallbackProps {
  state: AvatarState;
  hasError: boolean;
  onRetry: () => void;
}

export const Avatar2DFallback: Component<Avatar2DFallbackProps> = (props) => {
  const badge = () => getStatusBadge(props.state);

  return (
    <div class="relative w-full h-full flex flex-col items-center justify-center p-6">
      <img
        src={AVATAR_IMAGES[props.state] || AVATAR_IMAGES.idle}
        alt="Avatar"
        class={`max-h-[70vh] max-w-full object-contain transition-all duration-300 ${badge().glow}`}
      />
      <Show when={props.hasError}>
        <button
          type="button"
          onClick={props.onRetry}
          class="mt-3 px-4 py-2 bg-gi-primary/20 border border-gi-primary/40 text-gi-primary hover:bg-gi-primary/30 text-xs font-mono flex items-center gap-1.5 shadow-md active:scale-95 transition-all"
        >
          <RotateCcw size={12} />
          <span>Retry 3D Load</span>
        </button>
      </Show>
    </div>
  );
};
