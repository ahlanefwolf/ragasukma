import { Component, Show } from 'solid-js';
import { Music, Play, Pause, Square } from 'lucide-solid';
import { MobileTrackState } from '../AudioSyncManager';

interface MobileFloatingMusicProps {
  trackState: MobileTrackState;
  isHpAudio: boolean;
  onTogglePlay: () => void;
  onStop: () => void;
}

export const MobileFloatingMusic: Component<MobileFloatingMusicProps> = (props) => {
  return (
    <Show when={props.trackState.track}>
      <div class="px-3.5 py-2.5 bg-theme-surface border border-theme-border flex items-center justify-between gap-3 shadow-lg animate-in fade-in slide-in-from-bottom-2 duration-150">
        {/* Track Artwork & Titles */}
        <div class="flex items-center gap-2.5 min-w-0 flex-1">
          <div class="w-9 h-9 bg-theme-card border border-theme-border flex items-center justify-center shrink-0">
            <Show when={props.trackState.isPlaying} fallback={<Music size={15} class="text-[#E5B85C]" />}>
              <div class="flex items-end justify-center gap-0.5 h-3.5">
                <span class="w-1 bg-[#E5B85C] animate-[pulse_0.6s_ease-in-out_infinite] h-2.5" />
                <span class="w-1 bg-theme-rose animate-[pulse_0.9s_ease-in-out_infinite] h-3.5" />
                <span class="w-1 bg-[#E5B85C] animate-[pulse_0.7s_ease-in-out_infinite] h-2" />
              </div>
            </Show>
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-1.5">
              <span class="text-2xs font-mono font-bold px-1.5 py-0.2 bg-[#E5B85C] text-[#111318]">
                {props.trackState.isPlaying ? 'PLAYING' : 'PAUSED'}
              </span>
              <p class="text-xs font-bold text-theme-text truncate leading-tight">
                {props.trackState.track?.title}
              </p>
            </div>
            <p class="text-2xs text-theme-muted truncate mt-0.5">
              {props.trackState.track?.uploader || (props.isHpAudio ? 'Mobile Audio' : 'Desktop Audio')}
            </p>
          </div>
        </div>

        {/* Player Controls */}
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={props.onTogglePlay}
            class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-[#E5B85C] hover:text-[#F3CE7A] transition-colors active:scale-95 shadow-sm"
            title={props.trackState.isPlaying ? 'Pause' : 'Play'}
          >
            <Show when={props.trackState.isPlaying} fallback={<Play size={13} class="fill-current" />}>
              <Pause size={13} />
            </Show>
          </button>
          <button
            type="button"
            onClick={props.onStop}
            class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-muted hover:text-gi-status-danger transition-colors active:scale-95 shadow-sm"
            title="Stop Music"
          >
            <Square size={13} />
          </button>
        </div>
      </div>
    </Show>
  );
};
