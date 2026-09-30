import { createSignal } from 'solid-js';
import { api, YtTrackInfo } from './api';

const [currentTrack, setCurrentTrack] = createSignal<YtTrackInfo | null>(null);
const [isPlaying, setIsPlaying] = createSignal(false);
const [isPaused, setIsPaused] = createSignal(false);
const [currentTime, setCurrentTime] = createSignal(0);
const [duration, setDuration] = createSignal(0);
const [volume, setVolume] = createSignal(1);

export { currentTrack, isPlaying, isPaused, currentTime, duration, volume };

// Bridge status untuk komponen BottomPlayer, PiP & UI
export const globalStatus = () => ({
  current_track: currentTrack() || undefined,
  is_playing: isPlaying(),
  is_paused: isPaused(),
  volume: volume() * 100,
});

let audioEl: HTMLAudioElement | null = null;

export const formatDuration = (secs: number) => {
  if (!secs || isNaN(secs)) return '0:00';
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
};

const playStateListeners: Array<(playing: boolean) => void> = [];

export const onPlayStateChange = (cb: (playing: boolean) => void) => {
  playStateListeners.push(cb);
};

function notifyPlayState(playing: boolean) {
  for (const cb of playStateListeners) {
    try { cb(playing); } catch (_) {}
  }
}

function bindAudioListeners(el: HTMLAudioElement) {
  const updateDuration = () => {
    if (el.duration && !isNaN(el.duration) && isFinite(el.duration)) {
      setDuration(el.duration);
    } else {
      const cur = currentTrack();
      if (cur?.duration) {
        setDuration(cur.duration);
      }
    }
  };

  el.addEventListener('timeupdate', () => {
    setCurrentTime(el.currentTime);
    updateDuration();
  });

  el.addEventListener('loadedmetadata', updateDuration);
  el.addEventListener('durationchange', updateDuration);
  el.addEventListener('canplay', updateDuration);

  el.addEventListener('play', () => {
    setIsPlaying(true);
    setIsPaused(false);
    notifyPlayState(true);
  });

  el.addEventListener('pause', () => {
    setIsPlaying(false);
    setIsPaused(true);
    notifyPlayState(false);
  });

  el.addEventListener('ended', () => {
    setIsPlaying(false);
    setIsPaused(false);
    notifyPlayState(false);
    nextTrack();
  });

  el.addEventListener('error', (e) => {
    console.warn('⚠️ [AudioPlayer] Error saat memutar audio stream:', e);
  });
}

// Inisialisasi otomatis audio element singleton di browser
if (typeof window !== 'undefined' && typeof Audio !== 'undefined') {
  audioEl = new Audio();
  audioEl.volume = volume();
  bindAudioListeners(audioEl);
}

export const initGlobalAudio = (el?: HTMLAudioElement) => {
  if (el && el !== audioEl) {
    if (audioEl) {
      audioEl.pause();
    }
    audioEl = el;
    audioEl.volume = volume();
    bindAudioListeners(audioEl);
  }
};

let isDucked = false;
let preDuckVolume = 1.0;
let isDesktopVoiceMuted = false;

// Dengarkan status mute dari server (jika Full HP aktif, matikan pemutaran musik di browser PC)
api.subscribeState((st) => {
  if (st && typeof st.voice_muted === 'boolean') {
    isDesktopVoiceMuted = st.voice_muted;
    if (isDesktopVoiceMuted && audioEl && !audioEl.paused) {
      audioEl.pause();
      setIsPlaying(false);
      setIsPaused(false);
    }
  }
});

export const duckMusic = (duckLevel = 0.25) => {
  if (audioEl && !audioEl.paused && !isDucked) {
    isDucked = true;
    preDuckVolume = audioEl.volume;
    audioEl.volume = Math.max(0, Math.min(1, duckLevel));
  }
};

export const unduckMusic = () => {
  if (audioEl && isDucked) {
    isDucked = false;
    audioEl.volume = preDuckVolume;
  }
};

// ---------------------------------------------------------------------------
// 1. DENGARKAN EVENT DARI BACKEND SINGLE-SOURCE-OF-TRUTH (SSE ACTION)
// ---------------------------------------------------------------------------
api.subscribeActions(async (act) => {
  // Jika tab saat ini adalah remote client di smartphone, biarkan audioSync di mobile yang memutar audio
  try {
    const hash = window.location.hash.toLowerCase();
    const origin = window.location.origin.toLowerCase();
    const search = window.location.search;
    const isMobileClient =
      (search.includes('key=') || hash.includes('key=')) &&
      (origin.includes('trycloudflare.com') || hash.includes('mobile'));
    if (isMobileClient || hash === '#mobile' || hash === '#mobile-client') {
      return;
    }
  } catch (_) {}



  console.log('🎵 [playerStore] Menerima action SSE:', act);

  if (act.action === 'music:play') {
    const query = act.payload?.toLowerCase().trim();
    const res = await api.listMusic();
    const list = Array.isArray(res) ? res : res?.tracks || [];
    if (list.length === 0) return;

    let matched: YtTrackInfo | undefined;

    if (query) {
      const raw = act.payload?.trim() || '';
      const clean = (s: string) =>
        s.toLowerCase().replace(/[^a-z0-9\s]/g, ' ').replace(/\s+/g, ' ').trim();
      const qClean = clean(query);
      const qWords = qClean.split(' ').filter(Boolean);

      // 1. Cek Exact
      matched = list.find(
        (t) =>
          t.filename === raw ||
          t.title === raw ||
          t.id === raw ||
          t.title.toLowerCase() === query ||
          t.filename.toLowerCase() === query
      );

      // 2. Cek Substring
      if (!matched) {
        matched = list.find((t) => {
          const tClean = clean(`${t.title} ${t.filename}`);
          return tClean.includes(qClean) || qClean.includes(clean(t.title));
        });
      }

      // 3. Cek Multi-word Token
      if (!matched && qWords.length > 0) {
        matched = list.find((t) => {
          const tClean = clean(`${t.title} ${t.filename}`);
          return qWords.every((w) => tClean.includes(w));
        });
      }
    }

    const trackToPlay = matched || list[0];
    if (trackToPlay) {
      playLocalAudio(trackToPlay);
    }
  } else if (act.action === 'music:pause') {
    if (audioEl) audioEl.pause();
    setIsPlaying(false);
    setIsPaused(true);
  } else if (act.action === 'music:resume') {
    if (audioEl) audioEl.play().catch(() => {});
    setIsPlaying(true);
    setIsPaused(false);
  } else if (act.action === 'music:stop') {
    if (audioEl) {
      audioEl.pause();
      audioEl.currentTime = 0;
    }
    setIsPlaying(false);
    setIsPaused(false);
    setCurrentTime(0);
  }
});

function playLocalAudio(track: YtTrackInfo) {
  if (!track || !track.filename) return;
  // Jika server sedang mode Full HP (isDesktopVoiceMuted = true), browser PC tidak boleh bersuara (PC hening)
  if (isDesktopVoiceMuted) return;
  setCurrentTrack(track);
  setCurrentTime(0);
  setDuration(track.duration || 0);
  setIsPlaying(true);
  setIsPaused(false);

  if (audioEl) {
    audioEl.src = api.getMusicUrl(track.filename);
    audioEl.currentTime = 0;
    audioEl.play().catch((err) => {
      console.warn('⚠️ [AudioPlayer] Autoplay tertahan:', err);
    });
  }
}

// ---------------------------------------------------------------------------
// 2. REMOTE CONTROL ACTIONS (WEB DASHBOARD & PIP)
// ---------------------------------------------------------------------------
export const playTrack = (track: YtTrackInfo) => {
  if (!track || !track.filename) return;
  playLocalAudio(track);
  api.controlMusic('play', track.title);
};

export const pauseMusic = () => {
  if (audioEl) audioEl.pause();
  setIsPlaying(false);
  setIsPaused(true);
  api.controlMusic('pause');
};

export const resumeMusic = () => {
  if (audioEl) {
    if (!audioEl.src && currentTrack()?.filename) {
      audioEl.src = api.getMusicUrl(currentTrack()!.filename);
    }
    audioEl.play().catch(() => {});
  }
  setIsPlaying(true);
  setIsPaused(false);
  api.controlMusic('resume');
};

export const nextTrack = async () => {
  const res = await api.listMusic();
  const list = Array.isArray(res) ? res : res?.tracks || [];
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
  const list = Array.isArray(res) ? res : res?.tracks || [];
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
  setCurrentTime(0);
  api.controlMusic('stop');
};

export const killGlobalAudio = () => {
  stopMusic();
  setCurrentTrack(null);
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
