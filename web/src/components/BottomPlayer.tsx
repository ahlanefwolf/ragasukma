import { Component, Show } from 'solid-js';
import {
  Play,
  Pause,
  Square,
  Volume2,
  VolumeX,
  Music,
  ExternalLink,
  SkipBack,
  SkipForward,
  PictureInPicture2,
} from 'lucide-solid';
import { pipManager } from '../services/pipService';
import {
  globalStatus,
  currentTrack,
  isPlaying,
  isPaused,
  currentTime,
  duration,
  volume,
  isFullMobileActive,
  formatDuration,
  pauseMusic,
  resumeMusic,
  stopMusic,
  nextTrack,
  prevTrack,
  seekMusic,
  changeVolume,
} from '../playerStore';

interface BottomPlayerProps {
  onOpenMusicTab: () => void;
  activeTab?: string;
}

export const BottomPlayer: Component<BottomPlayerProps> = (props) => {
  const isVisible = () =>
    props.activeTab !== 'music' && !!currentTrack();

  const isPlayingTrack = () => isPlaying();

  const progressPercent = () => {
    const dur = duration() || currentTrack()?.duration || 100;
    if (dur <= 0) return 0;
    return Math.min(100, (currentTime() / dur) * 100);
  };

  return (
    <Show when={isVisible()}>
      <div class="w-full shrink-0 z-40 select-none border-t border-theme-border bg-theme-bg/95 backdrop-blur-xl">
        <div class="relative overflow-hidden">
          {/* Subtle Top Progress Line */}
          <div
            class={`w-full h-1 bg-theme-border/60 relative ${isFullMobileActive() ? 'cursor-not-allowed' : 'cursor-pointer'}`}
            onClick={(e) => {
              if (isFullMobileActive()) return;
              const rect = e.currentTarget.getBoundingClientRect();
              const pos = (e.clientX - rect.left) / rect.width;
              const dur = duration() || currentTrack()?.duration || 100;
              seekMusic(pos * dur);
            }}
          >
            <div
              class="h-full bg-gradient-to-r from-theme-cyan via-theme-rose to-theme-cyan transition-all duration-150 relative shadow-[0_0_8px_rgba(229,184,92,0.5)]"
              style={{ width: `${progressPercent()}%` }}
            >
              <span class="absolute right-0 top-1/2 -translate-y-1/2 translate-x-1/2 w-2 h-2 rounded-full bg-white shadow-[0_0_6px_rgba(255,255,255,0.8)]" />
            </div>
          </div>

          <div class="max-w-5xl mx-auto px-4 sm:px-6 py-2.5 sm:py-3 flex items-center justify-between gap-3 sm:gap-6">
            
            {/* Left: Album Art & Title */}
            <div
              class="flex items-center gap-3 min-w-0 flex-1 cursor-pointer group"
              onClick={props.onOpenMusicTab}
              title="Open Music Tab"
            >
              <div class="relative shrink-0 w-11 h-11 bg-theme-card border border-theme-border group-hover:border-theme-cyan/60 transition-all shadow-md overflow-hidden">
                <Show
                  when={currentTrack()?.thumbnail}
                  fallback={
                    <div class="w-full h-full flex items-center justify-center">
                      <Music size={20} class="text-theme-cyan/70" />
                    </div>
                  }
                >
                  <img
                    src={currentTrack()!.thumbnail}
                    alt=""
                    class="w-full h-full object-cover"
                  />
                </Show>
                
                {/* Live Animated Equalizer Overlay when playing */}
                <Show when={isPlaying()}>
                  <div class="absolute inset-0 bg-black/40 backdrop-blur-[1px] flex items-end justify-center gap-0.5 p-1">
                    <span class="w-1 bg-theme-cyan animate-[pulse_0.6s_ease-in-out_infinite] h-3" />
                    <span class="w-1 bg-theme-rose animate-[pulse_0.9s_ease-in-out_infinite] h-5" />
                    <span class="w-1 bg-theme-cyan animate-[pulse_0.7s_ease-in-out_infinite] h-2.5" />
                  </div>
                </Show>
              </div>

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class={`w-2 h-2 shrink-0 ${isPlaying() ? 'bg-theme-cyan animate-pulse shadow-[0_0_8px_var(--cyan)]' : 'bg-theme-rose'}`} />
                  <p class="text-sm font-bold text-theme-text truncate group-hover:text-theme-cyan transition-colors">
                    {currentTrack()?.title || 'Unknown Title'}
                  </p>
                </div>
                <p class="text-xs text-theme-text/60 truncate mt-0.5 font-medium">
                  {currentTrack()?.uploader || 'Local Library'}
                </p>
                <Show when={isFullMobileActive()}>
                  <span class="inline-block mt-0.5 px-1.5 py-0.2 text-[9.5px] font-mono font-bold bg-theme-rose/20 text-theme-rose border border-theme-rose/40">
                    AUDIO ON MOBILE (FULL HP)
                  </span>
                </Show>
              </div>
            </div>

            {/* Center: Playback Navigation Controls & Time */}
            <div class="flex flex-col items-center gap-1.5 shrink-0">
              <div class="flex items-center gap-2">
                {/* Prev Button */}
                <button
                  onClick={prevTrack}
                  class="p-2 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95"
                  title="Previous Track"
                  aria-label="Previous Track"
                >
                  <SkipBack size={16} />
                </button>

                {/* Play / Pause Toggle Button */}
                <Show when={isPlaying()}>
                  <button
                    onClick={pauseMusic}
                    class="w-9 h-9 bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/50 transition-all flex items-center justify-center active:scale-95 shadow-sm"
                    title="Pause"
                    aria-label="Pause"
                  >
                    <Pause size={17} />
                  </button>
                </Show>

                <Show when={!isPlaying()}>
                  <button
                    onClick={resumeMusic}
                    class="w-9 h-9 bg-theme-cyan hover:brightness-110 text-theme-bg font-bold transition-all flex items-center justify-center active:scale-95 shadow-md shadow-theme-cyan/20"
                    title="Resume"
                    aria-label="Play"
                  >
                    <Play size={17} class="ml-0.5 fill-current" />
                  </button>
                </Show>

                {/* Next Button */}
                <button
                  onClick={nextTrack}
                  class="p-2 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95"
                  title="Next Track"
                  aria-label="Next Track"
                >
                  <SkipForward size={16} />
                </button>

                {/* Stop Button */}
                <button
                  onClick={stopMusic}
                  class="p-2 bg-theme-card hover:bg-theme-rose/20 text-theme-text/70 hover:text-theme-rose border border-theme-border hover:border-theme-rose/40 transition-all active:scale-95"
                  title="Stop Music"
                  aria-label="Stop"
                >
                  <Square size={15} />
                </button>
              </div>

              <div class="flex items-center gap-1.5 text-xs font-mono text-theme-text/60">
                <span>{formatDuration(Math.floor(currentTime()))}</span>
                <span>/</span>
                <span>{formatDuration(Math.floor(duration() || currentTrack()?.duration || 0))}</span>
              </div>
            </div>

            {/* Right: Volume Slider & Tab Link */}
            <div class="hidden sm:flex items-center gap-3 shrink-0 flex-1 justify-end">
              <div class="flex items-center gap-2 px-3 py-1.5 bg-theme-card border border-theme-border">
                <button
                  onClick={() => changeVolume(volume() > 0 ? 0 : 1)}
                  class="text-theme-text/70 hover:text-theme-cyan p-0.5 transition-colors"
                  title={volume() > 0 ? 'Mute' : 'Unmute'}
                >
                  <Show when={volume() > 0} fallback={<VolumeX size={15} />}>
                    <Volume2 size={15} />
                  </Show>
                </button>
                {/* Volume slider */}
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={volume()}
                  disabled={isFullMobileActive()}
                  onInput={(e) => changeVolume(parseFloat(e.currentTarget.value))}
                  class={`w-20 h-1.5 bg-theme-border appearance-none cursor-pointer accent-theme-cyan ${
                    isFullMobileActive() ? 'opacity-50 cursor-not-allowed' : ''
                  }`}
                  title={isFullMobileActive() ? 'Mode Full HP aktif di HP' : `Volume: ${Math.round(volume() * 100)}%`}
                />
              </div>

              <button
                onClick={() => pipManager.togglePip()}
                class="px-3.5 py-1.5 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/50 transition-all flex items-center gap-2 shadow-sm active:scale-95 text-xs font-semibold"
                title="Picture-in-Picture (Floating Window)"
                aria-label="Picture-in-Picture"
              >
                <PictureInPicture2 size={16} class="text-theme-cyan shrink-0" />
                <span class="hidden sm:inline">PiP</span>
              </button>

              <button
                onClick={props.onOpenMusicTab}
                class="px-3.5 py-1.5 bg-theme-card hover:bg-theme-border text-theme-cyan text-xs font-semibold border border-theme-border hover:border-theme-cyan/50 transition-all flex items-center gap-1.5 shadow-sm active:scale-95"
                title="Open Music Tab"
              >
                <span>Music Tab</span>
                <ExternalLink size={14} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};
