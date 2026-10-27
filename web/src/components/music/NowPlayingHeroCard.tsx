import { Component, Show } from 'solid-js';
import {
  Music,
  Play,
  Pause,
  Square,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
  PictureInPicture2,
} from 'lucide-solid';
import { pipManager } from '../../services/pipService';
import {
  globalStatus,
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
} from '../../playerStore';

export const NowPlayingHeroCard: Component = () => {
  const handleSeek = (e: Event) => {
    const val = parseFloat((e.target as HTMLInputElement).value);
    seekMusic(val);
  };

  const getTrackDuration = () => {
    return duration() || globalStatus().current_track?.duration || 0;
  };

  const progressPercent = () => {
    const dur = getTrackDuration();
    if (dur <= 0) return 0;
    return Math.min(100, (currentTime() / dur) * 100);
  };

  return (
    <Show
      when={
        (globalStatus().is_playing || globalStatus().is_paused) &&
        globalStatus().current_track
      }
    >
      <div class="relative overflow-hidden border border-theme-border bg-theme-card/95 backdrop-blur-xl shadow-2xl p-4 sm:p-6">
        <div class="relative flex flex-col md:flex-row items-center gap-5 sm:gap-6">
          {/* Album Art Display */}
          <div class="relative shrink-0 w-24 h-24 sm:w-28 sm:h-28 overflow-hidden bg-theme-card border border-theme-border hover:border-theme-cyan/60 transition-all shadow-md group">
            <Show
              when={globalStatus().current_track!.thumbnail}
              fallback={
                <div class="w-full h-full flex items-center justify-center">
                  <Music size={32} class="text-theme-cyan/70" />
                </div>
              }
            >
              <img
                src={globalStatus().current_track!.thumbnail}
                alt="thumbnail"
                class={`w-full h-full object-cover transition-transform duration-700 ${
                  globalStatus().is_playing ? 'scale-105' : ''
                }`}
              />
            </Show>

            {/* Live Audio Equalizer Waveform Overlay */}
            <Show when={globalStatus().is_playing}>
              <div class="absolute inset-0 bg-black/40 backdrop-blur-[1px] flex items-end justify-center gap-0.5 p-1.5">
                <span class="w-1 bg-theme-cyan animate-[pulse_0.6s_ease-in-out_infinite] h-4" />
                <span class="w-1 bg-theme-rose animate-[pulse_0.9s_ease-in-out_infinite] h-6" />
                <span class="w-1 bg-theme-cyan animate-[pulse_0.7s_ease-in-out_infinite] h-3" />
                <span class="w-1 bg-theme-rose animate-[pulse_0.5s_ease-in-out_infinite] h-5" />
              </div>
            </Show>
          </div>

          {/* Track Info & Interactive Player Controls */}
          <div class="flex-1 min-w-0 w-full space-y-3.5 text-center md:text-left">
            <div class="flex flex-col md:flex-row md:items-center justify-between gap-2">
              <div class="min-w-0">
                <div class="flex items-center gap-2 mb-1 flex-wrap">
                  <div class="inline-flex items-center gap-2 px-2.5 py-0.5 bg-theme-card border border-theme-border text-xs font-bold uppercase tracking-wider">
                    <span
                      class={`w-2 h-2 shrink-0 ${
                        globalStatus().is_playing
                          ? 'bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse'
                          : 'bg-theme-rose'
                      }`}
                    />
                    <span class="text-theme-text/80">
                      {globalStatus().is_paused
                        ? 'Paused'
                        : 'Now Playing'}
                    </span>
                  </div>
                  <Show when={isFullMobileActive()}>
                    <span class="px-2 py-0.5 text-xs font-mono font-bold bg-theme-rose/20 text-theme-rose border border-theme-rose/40">
                      MODE FULL HP (Controlled on Mobile)
                    </span>
                  </Show>
                </div>
                <h3 class="text-theme-text truncate hover:text-theme-cyan transition-colors">
                  {globalStatus().current_track!.title}
                </h3>
                <p class="text-xs text-theme-text/60 truncate mt-0.5 font-medium">
                  {globalStatus().current_track!.uploader || 'Local Library'}
                </p>
              </div>

              {/* Volume slider pill */}
              <div class="hidden sm:flex items-center justify-center md:justify-end gap-2 px-3 py-1.5 bg-theme-card border border-theme-border self-center md:self-start">
                <button
                  onClick={() => !isFullMobileActive() && changeVolume(volume() > 0 ? 0 : 1)}
                  disabled={isFullMobileActive()}
                  class={`text-theme-text/70 hover:text-theme-cyan p-0.5 transition-colors ${
                    isFullMobileActive() ? 'opacity-50 cursor-not-allowed' : ''
                  }`}
                  title={isFullMobileActive() ? 'Mode Full HP aktif' : volume() > 0 ? 'Mute' : 'Unmute'}
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
                  disabled={isFullMobileActive()}
                  onInput={(e) => changeVolume(parseFloat(e.currentTarget.value))}
                  class={`w-20 h-1.5 bg-theme-border appearance-none cursor-pointer accent-theme-cyan ${
                    isFullMobileActive() ? 'opacity-50 cursor-not-allowed' : ''
                  }`}
                  title={isFullMobileActive() ? 'Audio controlled from mobile device' : `Volume: ${Math.round(volume() * 100)}%`}
                />
                <span class="text-xs font-mono text-theme-text/60 w-8 text-right">
                  {Math.round(volume() * 100)}%
                </span>
              </div>
            </div>

            {/* Progress Seekbar */}
            <div class="space-y-1.5">
              <div
                class={`relative flex items-center group py-1.5 ${isFullMobileActive() ? 'cursor-not-allowed' : 'cursor-pointer'}`}
                onClick={(e) => {
                  if (isFullMobileActive()) return;
                  if (e.target instanceof HTMLInputElement) return;
                  const rect = e.currentTarget.getBoundingClientRect();
                  const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
                  const dur = getTrackDuration();
                  if (dur > 0) {
                    seekMusic(pos * dur);
                  }
                }}
              >
                <div class="w-full h-1.5 bg-theme-border/60 relative">
                  <div
                    class="h-full bg-gradient-to-r from-theme-cyan via-theme-rose to-theme-cyan transition-all duration-150 relative shadow-[0_0_8px_rgba(229,184,92,0.5)]"
                    style={{ width: `${progressPercent()}%` }}
                  >
                    <span class="absolute right-0 top-1/2 -translate-y-1/2 translate-x-1/2 w-2.5 h-2.5 bg-white shadow-[0_0_6px_rgba(255,255,255,0.8)] group-hover:scale-125 transition-transform" />
                  </div>
                </div>
                <input
                  type="range"
                  min="0"
                  max={getTrackDuration() || 100}
                  value={currentTime()}
                  disabled={isFullMobileActive()}
                  onInput={handleSeek}
                  class={`absolute inset-0 w-full h-full opacity-0 ${
                    isFullMobileActive() ? 'cursor-not-allowed pointer-events-none' : 'cursor-pointer'
                  }`}
                />
              </div>
              <div class="flex justify-between text-xs font-mono text-theme-text/60">
                <span>{formatDuration(Math.floor(currentTime()))}</span>
                <span>{formatDuration(Math.floor(getTrackDuration()))}</span>
              </div>
            </div>

            {/* Playback Action Buttons */}
            <div class="flex items-center justify-center md:justify-start gap-2 pt-1 flex-wrap">
              {/* Prev Button */}
              <button
                onClick={prevTrack}
                class="p-2 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95 shadow-sm"
                title="Previous Track"
                aria-label="Previous Track"
              >
                <SkipBack size={16} />
              </button>

              {/* Play / Pause Toggle Button */}
              <Show when={globalStatus().is_playing}>
                <button
                  onClick={pauseMusic}
                  class="h-9 px-4 bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/50 font-bold text-xs transition-all flex items-center gap-2 active:scale-95 shadow-sm"
                  title="Pause"
                  aria-label="Pause"
                >
                  <Pause size={16} />
                  <span>Pause</span>
                </button>
              </Show>

              <Show when={globalStatus().is_paused}>
                <button
                  onClick={resumeMusic}
                  class="h-9 px-4 bg-theme-cyan hover:brightness-110 text-theme-bg font-bold text-xs transition-all flex items-center gap-2 active:scale-95 shadow-md shadow-theme-cyan/20 border border-theme-cyan"
                  title="Resume"
                  aria-label="Play"
                >
                  <Play size={16} class="fill-current" />
                  <span>Resume</span>
                </button>
              </Show>

              {/* Next Button */}
              <button
                onClick={nextTrack}
                class="p-2 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95 shadow-sm"
                title="Next Track"
                aria-label="Next Track"
              >
                <SkipForward size={16} />
              </button>

              {/* Stop Button */}
              <button
                onClick={stopMusic}
                class="p-2 bg-theme-card hover:bg-theme-rose/20 text-theme-text/70 hover:text-theme-rose border border-theme-border hover:border-theme-rose/40 transition-all active:scale-95 shadow-sm"
                title="Stop Music"
                aria-label="Stop"
              >
                <Square size={15} />
              </button>

              {/* PiP Button */}
              <button
                onClick={() => pipManager.togglePip()}
                class="px-3.5 py-1.5 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/50 transition-all flex items-center gap-2 shadow-sm active:scale-95 text-xs font-semibold ml-auto"
                title="Picture-in-Picture (Floating Window)"
                aria-label="Picture-in-Picture"
              >
                <PictureInPicture2 size={16} class="text-theme-cyan shrink-0" />
                <span>PiP Mode</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};
