import { createSignal } from 'solid-js';
import { api, YtTrackInfo } from './api';

const [currentTrack, setCurrentTrack] = createSignal<YtTrackInfo | null>(null);
const [isPlaying, setIsPlaying] = createSignal(false);
const [isPaused, setIsPaused] = createSignal(false);
const [currentTime, setCurrentTime] = createSignal(0);
const [duration, setDuration] = createSignal(0);
const [volume, setVolume] = createSignal(1);
const [isFullMobileActive, setIsFullMobileActive] = createSignal(false);

export { currentTrack, isPlaying, isPaused, currentTime, duration, volume, isFullMobileActive };

// Bridge status for komponen BottomPlayer, PiP & UI
export const globalStatus = () => ({
  current_track: currentTrack() || undefined,
  is_playing: isPlaying(),
  is_paused: isPaused(),
  volume: volume() * 100,
  is_full_mobile: isFullMobileActive(),
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

// Internal ticker for UI time update when desktop native player plays audio
let playbackTicker: any = null;

function startPlaybackTicker() {
  stopPlaybackTicker();
  playbackTicker = setInterval(() => {
    if (isPlaying() && !isPaused()) {
      const cur = currentTime();
      const dur = duration() || currentTrack()?.duration || 0;
      if (dur > 0 && cur >= dur) {
        setIsPlaying(false);
        setIsPaused(false);
        setCurrentTime(0);
        stopPlaybackTicker();
        nextTrack();
      } else {
        setCurrentTime(cur + 1);
      }
    }
  }, 1000);
}

function stopPlaybackTicker() {
  if (playbackTicker) {
    clearInterval(playbackTicker);
    playbackTicker = null;
  }
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

// Listen for mute status from server (if Full HP active, mute music playback on PC browser)
api.subscribeState((st) => {
  if (st && typeof st.voice_muted === 'boolean') {
    setIsFullMobileActive(st.voice_muted);
    if (st.voice_muted && audioEl && !audioEl.paused) {
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
// 1. LISTEN TO EVENTS FROM BACKEND SINGLE SOURCE OF TRUTH (SSE ACTION)
// ---------------------------------------------------------------------------
api.subscribeActions(async (act) => {
  // If this tab is a mobile remote client, let mobile audioSync handle playback
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

      // 1. Check Exact
      matched = list.find(
        (t) =>
          t.filename === raw ||
          t.title === raw ||
          t.id === raw ||
          t.title.toLowerCase() === query ||
          t.filename.toLowerCase() === query
      );

      // 2. Check Substring
      if (!matched) {
        matched = list.find((t) => {
          const tClean = clean(`${t.title} ${t.filename}`);
          return tClean.includes(qClean) || qClean.includes(clean(t.title));
        });
      }

      // 3. Check Multi-word Token
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
    stopPlaybackTicker();
  } else if (act.action === 'music:resume') {
    if (audioEl) audioEl.play().catch(() => {});
    setIsPlaying(true);
    setIsPaused(false);
    startPlaybackTicker();
  } else if (act.action === 'music:seek') {
    if (act.payload) {
      const sec = parseFloat(act.payload);
      if (!isNaN(sec)) {
        setCurrentTime(sec);
      }
    }
  } else if (act.action === 'music:volume') {
    if (act.payload) {
      const v = parseFloat(act.payload);
      if (!isNaN(v)) {
        const clamped = Math.max(0, Math.min(1, v > 2 ? v / 100 : v));
        setVolume(clamped);
        if (audioEl) audioEl.volume = clamped;
      }
    }
  } else if (act.action === 'music:stop') {
    if (audioEl) {
      audioEl.pause();
      audioEl.currentTime = 0;
    }
    setIsPlaying(false);
    setIsPaused(false);
    setCurrentTime(0);
    stopPlaybackTicker();
  }
});

function playLocalAudio(track: YtTrackInfo) {
  if (!track || !track.filename) return;
  // Always update track title and UI playback status
  setCurrentTrack(track);
  setCurrentTime(0);
  setDuration(track.duration || 0);
  setIsPlaying(true);
  setIsPaused(false);
  startPlaybackTicker();

  // Desktop browser must NOT play audio element if server native audio is playing on desktop speakers,
  // preventing double/echo sound. Audio is only emitted by browser if in Phone Only / explicit web streaming mode.
  if (isFullMobileActive()) {
    if (audioEl) {
      audioEl.pause();
      audioEl.currentTime = 0;
    }
    return;
  }

  // When on desktop web dashboard, native backend player handles desktop hardware speaker output.
  // We do not play audioEl in desktop browser tab to prevent double audio playback.
}

// ---------------------------------------------------------------------------
// 2. REMOTE CONTROL ACTIONS (WEB DASHBOARD & PIP)
// ---------------------------------------------------------------------------
export const playTrack = (track: YtTrackInfo) => {
  if (!track || !track.filename) return;
  if (isFullMobileActive()) return;
  playLocalAudio(track);
  api.controlMusic('play', track.title);
};

export const pauseMusic = () => {
  if (isFullMobileActive()) return;
  if (audioEl) audioEl.pause();
  setIsPlaying(false);
  setIsPaused(true);
  stopPlaybackTicker();
  api.controlMusic('pause');
};

export const resumeMusic = () => {
  if (isFullMobileActive()) return;
  setIsPlaying(true);
  setIsPaused(false);
  startPlaybackTicker();
  api.controlMusic('resume');
};

export const nextTrack = async () => {
  if (isFullMobileActive()) return;
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
  if (isFullMobileActive()) return;
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
  if (isFullMobileActive()) return;
  if (audioEl) {
    audioEl.pause();
    audioEl.currentTime = 0;
  }
  setIsPlaying(false);
  setIsPaused(false);
  setCurrentTime(0);
  stopPlaybackTicker();
  api.controlMusic('stop');
};

export const killGlobalAudio = () => {
  stopMusic();
  setCurrentTrack(null);
};

export const seekMusic = (time: number) => {
  if (isFullMobileActive()) return;
  setCurrentTime(time);
  if (audioEl) {
    audioEl.currentTime = time;
  }
  api.controlMusic('seek', time.toString());
};

let volumeDebounceTimer: any = null;

export const changeVolume = (val: number) => {
  if (isFullMobileActive()) return;
  const clamped = Math.max(0, Math.min(1, val));
  setVolume(clamped);
  if (audioEl) {
    audioEl.volume = clamped;
  }
  if (volumeDebounceTimer) {
    clearTimeout(volumeDebounceTimer);
  }
  volumeDebounceTimer = setTimeout(() => {
    api.controlMusic('volume', clamped.toFixed(2));
  }, 40);
};

// Initial sync with backend music player volume
api.listMusic().then((res) => {
  if (res && typeof res.volume === 'number' && !isNaN(res.volume)) {
    const v = Math.max(0, Math.min(1, res.volume));
    setVolume(v);
    if (audioEl) audioEl.volume = v;
  }
}).catch(() => {});
