import { createSignal } from 'solid-js';
import { api, YtTrackInfo, MusicStatus } from './api';

export const [globalStatus, setGlobalStatus] = createSignal<MusicStatus>({
  is_playing: false,
  is_paused: false,
  volume: 100,
});

export const [currentTime, setCurrentTime] = createSignal(0);
export const [duration, setDuration] = createSignal(0);
export const [volume, setVolume] = createSignal(1);

let audioEl: HTMLAudioElement | null = null;
let pollTimer: number | null = null;

export const formatDuration = (secs: number) => {
  if (!secs || isNaN(secs)) return '0:00';
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
};

export const initGlobalAudio = (el: HTMLAudioElement) => {
  audioEl = el;
  audioEl.volume = volume();

  audioEl.addEventListener('timeupdate', () => {
    if (audioEl) {
      setCurrentTime(audioEl.currentTime);
      if (audioEl.duration && !isNaN(audioEl.duration)) {
        setDuration(audioEl.duration);
      }
    }
  });

  audioEl.addEventListener('loadedmetadata', () => {
    if (audioEl && audioEl.duration && !isNaN(audioEl.duration)) {
      setDuration(audioEl.duration);
    }
  });

  audioEl.addEventListener('ended', () => {
    stopMusic();
  });

  audioEl.addEventListener('error', (e) => {
    console.error('[GlobalAudio] Error stream:', e);
  });

  // Start polling status from backend
  if (!pollTimer) {
    pollTimer = window.setInterval(async () => {
      const s = await api.getMusicStatus();
      setGlobalStatus(s);

      // Sinkronisasi jika musik diputar via AI / Skill
      if (s.is_playing && s.current_track?.filename) {
        const streamUrl = api.getMusicStreamUrl(s.current_track.filename);
        if (audioEl) {
          const currentSrc = decodeURIComponent(audioEl.src);
          if (!currentSrc.includes(s.current_track.filename)) {
            audioEl.src = streamUrl;
            audioEl.play().catch((e) => console.log('[GlobalAudio] Autoplay catch:', e));
          } else if (audioEl.paused) {
            audioEl.play().catch(() => {});
          }
        }
      } else if (s.is_paused && audioEl && !audioEl.paused) {
        audioEl.pause();
      } else if (!s.is_playing && !s.is_paused && audioEl && !audioEl.paused) {
        audioEl.pause();
        audioEl.currentTime = 0;
      }
    }, 1500);
  }
};

export const playTrack = async (track: YtTrackInfo) => {
  setGlobalStatus({
    current_track: track,
    is_playing: true,
    is_paused: false,
    volume: 100,
  });

  if (audioEl) {
    audioEl.src = api.getMusicStreamUrl(track.filename);
    audioEl.play().catch((e) => console.log('[GlobalAudio] Play catch:', e));
  }

  await api.playMusic(track.filename);
};

export const pauseMusic = async () => {
  if (audioEl) audioEl.pause();
  setGlobalStatus((prev) => ({ ...prev, is_playing: false, is_paused: true }));
  await api.pauseMusic();
};

export const resumeMusic = async () => {
  if (audioEl) {
    if (!audioEl.src && globalStatus().current_track?.filename) {
      audioEl.src = api.getMusicStreamUrl(globalStatus().current_track!.filename);
    }
    audioEl.play().catch(() => {});
  }
  setGlobalStatus((prev) => ({ ...prev, is_playing: true, is_paused: false }));
  await api.resumeMusic();
};

export const stopMusic = async () => {
  if (audioEl) {
    audioEl.pause();
    audioEl.currentTime = 0;
  }
  setGlobalStatus({ is_playing: false, is_paused: false, volume: 100 });
  await api.stopMusic();
};

export const killGlobalAudio = () => {
  if (pollTimer) {
    window.clearInterval(pollTimer);
    pollTimer = null;
  }
  if (audioEl) {
    audioEl.pause();
    audioEl.src = '';
  }
  setGlobalStatus({ is_playing: false, is_paused: false, volume: 100 });
};

export const seekMusic = (time: number) => {
  setCurrentTime(time);
  if (audioEl) {
    audioEl.currentTime = time;
  }
};

export const changeVolume = (val: number) => {
  setVolume(val);
  if (audioEl) {
    audioEl.volume = val;
  }
};
