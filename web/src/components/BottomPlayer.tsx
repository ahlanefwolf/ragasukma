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
  currentTime,
  duration,
  volume,
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
    props.activeTab !== 'music' &&
    (globalStatus().is_playing || globalStatus().is_paused) &&
    !!globalStatus().current_track;

  const currentTrack = () => globalStatus().current_track;
  const isPlaying = () => globalStatus().is_playing;

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
          <div class="w-full h-1 bg-theme-border/60 relative cursor-pointer" onClick={(e) => {
            const rect = e.currentTarget.getBoundingClientRect();
            const pos = (e.clientX - rect.left) / rect.width;
            const dur = duration() || currentTrack()?.duration || 100;
            seekMusic(pos * dur);
          }}>
            <div
              class="h-full bg-gradient-to-r from-theme-cyan via-theme-rose to-theme-cyan transition-all duration-150 relative shadow-[0_0_8px_rgba(123,223,242,0.8)]"
              style={{ width: `${progressPercent()}%` }}
            >
              <span class="absolute right-0 top-1/2 -translate-y-1/2 translate-x-1/2 w-2 h-2 rounded-full bg-white shadow-[0_0_6px_#fff]" />
            </div>
          </div>

          <div class="max-w-5xl mx-auto px-4 sm:px-6 py-2.5 sm:py-3 flex items-center justify-between gap-3 sm:gap-6">
            
            {/* Left: Album Art & Title */}
            <div
              class="flex items-center gap-3 min-w-0 flex-1 cursor-pointer group"
              onClick={props.onOpenMusicTab}
              title="Buka Tab Musik"
            >
              <div class="relative shrink-0 w-10 h-10 sm:w-11 sm:h-11 rounded-xl overflow-hidden bg-theme-card border border-theme-border group-hover:border-theme-cyan/60 transition-all shadow-md">
                <Show
                  when={currentTrack()?.thumbnail}
                  fallback={
                    <div class="w-full h-full flex items-center justify-center">
                      <Music size={18} class="text-theme-cyan/70" />
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
                    <span class="w-1 bg-theme-cyan rounded-t animate-[pulse_0.6s_ease-in-out_infinite] h-3" />
                    <span class="w-1 bg-theme-rose rounded-t animate-[pulse_0.9s_ease-in-out_infinite] h-4.5" />
                    <span class="w-1 bg-theme-cyan rounded-t animate-[pulse_0.7s_ease-in-out_infinite] h-2" />
                  </div>
                </Show>
              </div>

              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-1.5">
                  <span class={`w-2 h-2 rounded-full shrink-0 ${isPlaying() ? 'bg-theme-cyan animate-pulse shadow-[0_0_6px_#7BDFF2]' : 'bg-theme-rose'}`} />
                  <p class="text-xs sm:text-sm font-bold text-theme-text truncate group-hover:text-theme-cyan transition-colors">
                    {currentTrack()?.title || 'Unknown Title'}
                  </p>
                </div>
                <p class="text-[11px] text-theme-text/60 truncate mt-0.5">
                  {currentTrack()?.uploader || 'Local Library'}
                </p>
              </div>
            </div>

            {/* Center: Playback Navigation Controls & Time */}
            <div class="flex flex-col items-center gap-1 shrink-0">
              <div class="flex items-center gap-1.5 sm:gap-2">
                {/* Prev Button */}
                <button
                  onClick={prevTrack}
                  class="p-1.5 sm:p-2 rounded-xl bg-theme-card/80 hover:bg-theme-card text-theme-text/80 hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95"
                  title="Lagu Sebelumnya"
                  aria-label="Previous Track"
                >
                  <SkipBack size={15} />
                </button>

                {/* Play / Pause Toggle Button */}
                <Show when={isPlaying()}>
                  <button
                    onClick={pauseMusic}
                    class="w-8 h-8 sm:w-9 sm:h-9 rounded-xl bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/50 transition-all flex items-center justify-center active:scale-95 shadow-sm"
                    title="Jeda"
                    aria-label="Pause"
                  >
                    <Pause size={16} />
                  </button>
                </Show>

                <Show when={!isPlaying()}>
                  <button
                    onClick={resumeMusic}
                    class="w-8 h-8 sm:w-9 sm:h-9 rounded-xl bg-gradient-to-tr from-theme-cyan to-[#9BE7F6] hover:brightness-110 text-theme-bg font-bold transition-all flex items-center justify-center active:scale-95 shadow-md shadow-theme-cyan/20"
                    title="Lanjutkan"
                    aria-label="Play"
                  >
                    <Play size={16} class="ml-0.5 fill-current" />
                  </button>
                </Show>

                {/* Next Button */}
                <button
                  onClick={nextTrack}
                  class="p-1.5 sm:p-2 rounded-xl bg-theme-card/80 hover:bg-theme-card text-theme-text/80 hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95"
                  title="Lagu Berikutnya"
                  aria-label="Next Track"
                >
                  <SkipForward size={15} />
                </button>

                {/* Stop Button */}
                <button
                  onClick={stopMusic}
                  class="p-1.5 sm:p-2 rounded-xl bg-theme-card/80 hover:bg-theme-rose/20 text-theme-text/60 hover:text-theme-rose border border-theme-border hover:border-theme-rose/40 transition-all active:scale-95"
                  title="Hentikan Musik"
                  aria-label="Stop"
                >
                  <Square size={14} />
                </button>
              </div>

              <div class="flex items-center gap-1 text-[10px] font-mono text-theme-text/50">
                <span>{formatDuration(Math.floor(currentTime()))}</span>
                <span>/</span>
                <span>{formatDuration(Math.floor(duration() || currentTrack()?.duration || 0))}</span>
              </div>
            </div>

            {/* Right: Volume Slider & Tab Link */}
            <div class="hidden sm:flex items-center gap-3 shrink-0 flex-1 justify-end">
              <div class="flex items-center gap-2 px-2.5 py-1 rounded-xl bg-theme-card/60 border border-theme-border">
                <button
                  onClick={() => changeVolume(volume() > 0 ? 0 : 1)}
                  class="text-theme-text/60 hover:text-theme-cyan p-0.5 transition-colors"
                  title={volume() > 0 ? 'Mute' : 'Unmute'}
                >
                  <Show when={volume() > 0} fallback={<VolumeX size={14} />}>
                    <Volume2 size={14} />
                  </Show>
                </button>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={volume()}
                  onInput={(e) => changeVolume(parseFloat(e.currentTarget.value))}
                  class="w-16 h-1 bg-theme-border rounded-lg appearance-none cursor-pointer accent-theme-cyan"
                  title={`Volume: ${Math.round(volume() * 100)}%`}
                />
              </div>

              <button
                onClick={() => pipManager.togglePip()}
                class="px-3 py-1.5 rounded-xl bg-theme-card hover:bg-theme-border text-theme-text/90 hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/50 transition-all flex items-center gap-1.5 shadow-sm active:scale-95 text-xs font-semibold"
                title="Picture-in-Picture (Layar Melayang)"
                aria-label="Picture-in-Picture"
              >
                <PictureInPicture2 size={16} class="text-theme-cyan shrink-0" />
                <span class="hidden sm:inline">PiP</span>
              </button>

              <button
                onClick={props.onOpenMusicTab}
                class="px-3 py-1.5 rounded-xl bg-theme-card hover:bg-theme-border text-theme-cyan text-xs font-semibold border border-theme-border hover:border-theme-cyan/50 transition-all flex items-center gap-1.5 shadow-sm active:scale-95"
                title="Buka Tampilan Penuh Musik"
              >
                <span class="text-xs">Tab Musik</span>
                <ExternalLink size={14} />
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};
