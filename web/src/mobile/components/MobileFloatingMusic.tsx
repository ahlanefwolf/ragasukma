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
      <div class="px-3 py-2 bg-theme-surface border border-theme-border flex items-center justify-between gap-3">
        {/* Track Artwork & Titles */}
        <div class="flex items-center gap-2.5 min-w-0 flex-1">
          <div class="w-9 h-9 bg-theme-card border border-theme-border flex items-center justify-center shrink-0 text-theme-muted">
            <Music size={15} />
          </div>
          <div class="min-w-0 flex-1">
            <p class="text-xs font-semibold text-theme-text truncate leading-tight">
              {props.trackState.track?.title}
            </p>
            <p class="text-2xs text-theme-muted truncate mt-0.5">
              {props.trackState.track?.uploader || (props.isHpAudio ? 'Mobile Audio' : 'Desktop Audio')}
            </p>
          </div>
        </div>

        {/* Player Controls */}
        <div class="flex items-center gap-1 shrink-0">
          <button
            type="button"
            onClick={props.onTogglePlay}
            class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-gi-primary transition-colors active:scale-95"
            title={props.trackState.isPlaying ? 'Pause' : 'Play'}
            aria-label={props.trackState.isPlaying ? 'Pause' : 'Play'}
          >
            <Show when={props.trackState.isPlaying} fallback={<Play size={13} class="fill-current" />}>
              <Pause size={13} />
            </Show>
          </button>
          <button
            type="button"
            onClick={props.onStop}
            class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-muted hover:text-gi-status-danger transition-colors active:scale-95"
            title="Stop Music"
            aria-label="Stop Music"
          >
            <Square size={13} />
          </button>
        </div>
      </div>
    </Show>
  );
};