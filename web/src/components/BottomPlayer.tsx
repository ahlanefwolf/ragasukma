import { Component, Show } from 'solid-js';
import { Play, Pause, Square, Volume2, VolumeX, Music, ExternalLink } from 'lucide-solid';
import {
  globalStatus,
  currentTime,
  duration,
  volume,
  formatDuration,
  pauseMusic,
  resumeMusic,
  stopMusic,
  seekMusic,
  changeVolume,
} from '../playerStore';

interface BottomPlayerProps {
  onOpenMusicTab: () => void;
}

export const BottomPlayer: Component<BottomPlayerProps> = (props) => {
  const isVisible = () =>
    (globalStatus().is_playing || globalStatus().is_paused) && !!globalStatus().current_track;

  const currentTrack = () => globalStatus().current_track;

  return (
    <Show when={isVisible()}>
      <div class="fixed bottom-3 left-3 right-3 sm:left-6 sm:right-6 md:left-72 md:right-8 z-50 animate-in slide-in-from-bottom-4 fade-in duration-200 select-none">
        <div class="p-3 sm:p-3.5 rounded-2xl bg-theme-bg/95 backdrop-blur-xl border border-theme-cyan/40 shadow-2xl shadow-black/80 flex flex-col gap-2">
          {/* Main Controls Row */}
          <div class="flex items-center justify-between gap-3">
            {/* Left: Thumbnail & Info */}
            <div class="flex items-center gap-3 min-w-0 flex-1">
              <Show
                when={currentTrack()?.thumbnail}
                fallback={
                  <div class="w-10 h-10 rounded-xl bg-theme-card border border-theme-border flex items-center justify-center shrink-0">
                    <Music size={18} class="text-theme-cyan" />
                  </div>
                }
              >
                <img
                  src={currentTrack()!.thumbnail}
                  alt=""
                  class="w-10 h-10 rounded-xl object-cover shrink-0 border border-theme-cyan/30 shadow-md"
                />
              </Show>

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="w-2 h-2 rounded-full bg-theme-cyan animate-pulse" />
                  <p class="text-xs sm:text-sm font-bold text-theme-text truncate">
                    {currentTrack()?.title}
                  </p>
                </div>
                <p class="text-[11px] text-theme-text/60 truncate">
                  {currentTrack()?.uploader || 'Local Library'}
                </p>
              </div>
            </div>

            {/* Middle: Play / Pause / Stop Buttons */}
            <div class="flex items-center gap-1.5 shrink-0">
              <Show when={globalStatus().is_playing}>
                <button
                  onClick={pauseMusic}
                  class="p-2 sm:p-2.5 rounded-xl bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/40 transition-all shadow-sm"
                  title="Jeda"
                >
                  <Pause size={16} />
                </button>
              </Show>

              <Show when={globalStatus().is_paused}>
                <button
                  onClick={resumeMusic}
                  class="p-2 sm:p-2.5 rounded-xl bg-theme-cyan hover:bg-theme-cyan/90 text-theme-bg font-bold transition-all shadow-md shadow-[var(--cyan)]/20"
                  title="Lanjutkan"
                >
                  <Play size={16} />
                </button>
              </Show>

              <button
                onClick={stopMusic}
                class="p-2 sm:p-2.5 rounded-xl bg-theme-card hover:bg-theme-border text-theme-text/70 hover:text-theme-rose border border-theme-border transition-all"
                title="Stop"
              >
                <Square size={16} />
              </button>
            </div>

            {/* Right: Volume & Jump to Music Tab */}
            <div class="hidden sm:flex items-center gap-3 shrink-0">
              <div class="flex items-center gap-1.5">
                <button
                  onClick={() => changeVolume(volume() > 0 ? 0 : 1)}
                  class="text-theme-text/60 hover:text-theme-text p-1 transition-colors"
                >
                  <Show when={volume() > 0} fallback={<VolumeX size={15} />}>
                    <Volume2 size={15} />
                  </Show>
                </button>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={volume()}
                  onInput={(e) => changeVolume(parseFloat(e.currentTarget.value))}
                  class="w-16 h-1 bg-theme-card rounded-lg appearance-none cursor-pointer accent-theme-cyan"
                />
              </div>

              <button
                onClick={props.onOpenMusicTab}
                class="px-2.5 py-1.5 rounded-xl bg-theme-card hover:bg-theme-border text-theme-cyan text-xs font-semibold border border-theme-border hover:border-theme-cyan/40 transition-all flex items-center gap-1.5"
                title="Buka Tab Music"
              >
                <span class="text-[11px]">Library</span>
                <ExternalLink size={12} />
              </button>
            </div>
          </div>

          {/* Seekbar Timeline Progress */}
          <div class="flex items-center gap-2.5 pt-0.5">
            <span class="text-[10px] font-mono text-theme-text/50 shrink-0 w-8 text-right">
              {formatDuration(Math.floor(currentTime()))}
            </span>
            <input
              type="range"
              min="0"
              max={duration() || currentTrack()?.duration || 100}
              value={currentTime()}
              onInput={(e) => seekMusic(parseFloat(e.currentTarget.value))}
              class="w-full h-1 bg-theme-card rounded-lg appearance-none cursor-pointer accent-theme-cyan"
            />
            <span class="text-[10px] font-mono text-theme-text/50 shrink-0 w-8">
              {formatDuration(Math.floor(duration() || currentTrack()?.duration || 0))}
            </span>
          </div>
        </div>
      </div>
    </Show>
  );
};
