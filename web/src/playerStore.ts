import { createSignal } from 'solid-js';
import { api, YtTrackInfo } from './api';

export interface CurrentTrackState {
  track: YtTrackInfo | null;
  isPlaying: boolean;
  isPaused: boolean;
}

export const [currentTrack, setCurrentTrack] = createSignal<YtTrackInfo | null>(null);
export const [isPlaying, setIsPlaying] = createSignal(false);
export const [isPaused, setIsPaused] = createSignal(false);
export const [currentTime, setCurrentTime] = createSignal(0);
export const [duration, setDuration] = createSignal(0);
export const [volume, setVolume] = createSignal(1);

// Bridge status untuk komponen BottomPlayer & UI
export const globalStatus = () => ({
  current_track: currentTrack() || undefined,
  is_playing: isPlaying(),
  is_paused: isPaused(),
  volume: volume() * 100,
});

let audioEl: HTMLAudioElement | null = null;
let isInitialized = false;

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

  audioEl.addEventListener('play', () => {
    setIsPlaying(true);
    setIsPaused(false);
  });

  audioEl.addEventListener('pause', () => {
    setIsPlaying(false);
    setIsPaused(true);
  });

  audioEl.addEventListener('ended', () => {
    setIsPlaying(false);
    setIsPaused(false);
    nextTrack();
  });

  audioEl.addEventListener('error', (e) => {
    console.error('[AudioPlayer] Error:', e);
    setIsPlaying(false);
    setIsPaused(false);
  });
};

// Dengarkan perintah aksi musik secara global di modul
api.subscribeActions(async (act) => {
  console.log('[playerStore] Menerima action SSE:', act);
  if (act.action === 'music:play') {
    const query = act.payload?.toLowerCase().trim();
    const res = await api.listMusic();
    const list = Array.isArray(res) ? res : (res?.tracks || []);
    if (list.length === 0) return;

    if (query) {
      const raw = act.payload?.trim() || '';
      const clean = (s: string) => s.toLowerCase().replace(/[^a-z0-9\s]/g, ' ').replace(/\s+/g, ' ').trim();
      const qClean = clean(query);
      const qWords = qClean.split(' ').filter(Boolean);

      // 1. Cek Exact Filename / Title / ID
      let matched = list.find(
        (t) =>
          t.filename === raw ||
          t.title === raw ||
          t.id === raw ||
          t.title.toLowerCase() === query ||
          t.filename.toLowerCase() === query
      );

      // 2. Cek Substring Match
      if (!matched) {
        matched = list.find((t) => {
          const tClean = clean(`${t.title} ${t.filename}`);
          return tClean.includes(qClean) || qClean.includes(clean(t.title));
        });
      }

      // 3. Cek Multi-word Token Match
      if (!matched && qWords.length > 0) {
        matched = list.find((t) => {
          const tClean = clean(`${t.title} ${t.filename}`);
          return qWords.every((w) => tClean.includes(w));
        });
      }

      // 4. Putar hanya jika lagu ditemukan di lokal
      if (matched) {
        playTrack(matched);
      }
    } else {
      // Tanpa parameter query -> lanjutkan atau putar lagu pertama
      playTrack(list[0]);
    }
  } else if (act.action === 'music:pause') {
    pauseMusic();
  } else if (act.action === 'music:resume') {
    resumeMusic();
  } else if (act.action === 'music:next') {
    nextTrack();
  } else if (act.action === 'music:prev') {
    prevTrack();
  } else if (act.action === 'music:stop') {
    stopMusic();
  }
});

export const playTrack = (track: YtTrackInfo) => {
  if (!track || !track.filename) return;
  setCurrentTrack(track);

  if (audioEl) {
    audioEl.src = `/music/${encodeURIComponent(track.filename)}`;
    audioEl.play().catch((e) => console.log('[AudioPlayer] Autoplay catch:', e));
  }
};

export const pauseMusic = () => {
  if (audioEl) {
    audioEl.pause();
  }
};

export const resumeMusic = () => {
  if (audioEl) {
    if (!audioEl.src && currentTrack()?.filename) {
      audioEl.src = `/music/${encodeURIComponent(currentTrack()!.filename)}`;
    }
    audioEl.play().catch(() => {});
  }
};

export const nextTrack = async () => {
  const res = await api.listMusic();
  const list = Array.isArray(res) ? res : (res?.tracks || []);
  if (list.length === 0) return;

  const cur = currentTrack();
  if (!cur) {
    playTrack(list[0]);
    return;
  }

  const idx = list.findIndex((t) => t.id === cur.id || t.filename === cur.filename);
  const nextIdx = idx !== -1 ? (idx + 1) % list.length : 0;
  playTrack(list[nextIdx]);
};

export const prevTrack = async () => {
  const res = await api.listMusic();
  const list = Array.isArray(res) ? res : (res?.tracks || []);
  if (list.length === 0) return;

  const cur = currentTrack();
  if (!cur) {
    playTrack(list[0]);
    return;
  }

  const idx = list.findIndex((t) => t.id === cur.id || t.filename === cur.filename);
  const prevIdx = idx !== -1 ? (idx - 1 + list.length) % list.length : 0;
  playTrack(list[prevIdx]);
};

export const stopMusic = () => {
  if (audioEl) {
    audioEl.pause();
    audioEl.currentTime = 0;
  }
  setIsPlaying(false);
  setIsPaused(false);
};

export const killGlobalAudio = () => {
  if (audioEl) {
    audioEl.pause();
    audioEl.src = '';
  }
  setCurrentTrack(null);
  setIsPlaying(false);
  setIsPaused(false);
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
