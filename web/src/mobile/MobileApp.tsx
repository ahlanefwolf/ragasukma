import { Component, createSignal, onMount, onCleanup, createEffect, Show } from 'solid-js';
import { api, LogEntry } from '../api';
import { API_BASE } from '../services/base';
import { getChromeSttCode } from '../services/configService';
import { AvatarViewer } from './AvatarViewer';
import { globalAudioSync, MobileTrackState } from './AudioSyncManager';
import { MobileTopBar } from './components/MobileTopBar';
import { MobileAudioMode, MobileSettingsSheet } from './components/MobileSettingsSheet';
import { MobileFloatingMusic } from './components/MobileFloatingMusic';
import { MobileInteractionDock } from './components/MobileInteractionDock';
import { MobileLogsDrawer } from './components/MobileLogsDrawer';
import { MobileAudioUnlockModal } from './components/MobileAudioUnlockModal';
import { ModelLoadingOverlay } from '../components/ModelLoadingOverlay';
import { ModelDownloadStatus } from '../services/eventsService';

interface MobileAppProps {
  authKey: string;
  showToast: (msg: string) => void;
}

export const MobileApp: Component<MobileAppProps> = (props) => {
  const [inputText, setInputText] = createSignal('');
  const [isSending, setIsSending] = createSignal(false);
  const [lastSentText, setLastSentText] = createSignal('');
  const [showLogsDrawer, setShowLogsDrawer] = createSignal(false);
  const [showSettings, setShowSettings] = createSignal(false);
  const [showTextInput, setShowTextInput] = createSignal(false);
  const [viewportHeight, setViewportHeight] = createSignal<string>('100dvh');
  const [modelStatus, setModelStatus] = createSignal<ModelDownloadStatus | null>(null);

  const isModelLocked = () => {
    const s = modelStatus()?.status;
    return s === 'downloading' || s === 'extracting' || s === 'error';
  };

  const [isArActive, setIsArActive] = createSignal(false);

  // Status Karakter Realtime
  const [visualState, setVisualState] = createSignal<'idle' | 'listening' | 'speaking' | 'thinking'>('idle');
  const [emotion, setEmotion] = createSignal('neutral');

  let thinkingWatchdogTimer: number | null = null;
  let speakingWatchdogTimer: number | null = null;
  let listeningWatchdogTimer: number | null = null;
  const updateVisualStateSafe = (st: 'idle' | 'listening' | 'speaking' | 'thinking') => {
    setVisualState(st);
    if (thinkingWatchdogTimer) {
      window.clearTimeout(thinkingWatchdogTimer);
      thinkingWatchdogTimer = null;
    }
    if (speakingWatchdogTimer) {
      window.clearTimeout(speakingWatchdogTimer);
      speakingWatchdogTimer = null;
    }
    if (listeningWatchdogTimer) {
      window.clearTimeout(listeningWatchdogTimer);
      listeningWatchdogTimer = null;
    }
    if (st === 'thinking') {
      // Fail-safe watchdog: if thinking takes longer than 7s, sync with backend state
      thinkingWatchdogTimer = window.setTimeout(async () => {
        thinkingWatchdogTimer = null;
        try {
          const latest = await api.getState();
          if (latest && latest.state && latest.state !== 'thinking') {
            const s = latest.state.toLowerCase();
            if (['idle', 'listening', 'speaking'].includes(s)) {
              setVisualState(s as any);
            } else {
              setVisualState('idle');
            }
          } else {
            setVisualState('idle');
          }
        } catch {
          setVisualState('idle');
        }
      }, 7000);
    } else if (st === 'speaking') {
      // Fail-safe watchdog: speaking shouldn't get stuck on phone if audio already finished on PC
      speakingWatchdogTimer = window.setTimeout(async () => {
        speakingWatchdogTimer = null;
        if (!isHpAudioActive() || (!activeVoiceAudio && !isVoiceSpeakingOnMobile)) {
          try {
            const latest = await api.getState();
            if (latest && latest.state && latest.state !== 'speaking') {
              const s = latest.state.toLowerCase();
              if (['idle', 'listening', 'thinking'].includes(s)) {
                setVisualState(s as any);
              } else {
                setVisualState('idle');
              }
            } else {
              setVisualState('idle');
            }
          } catch {
            setVisualState('idle');
          }
        }
      }, 8000);
    } else if (st === 'listening') {
      // Fail-safe watchdog: listening shouldn't hang forever without speech
      listeningWatchdogTimer = window.setTimeout(async () => {
        listeningWatchdogTimer = null;
        if (!isHpMicListening()) {
          try {
            const latest = await api.getState();
            if (latest && latest.state && latest.state !== 'listening') {
              const s = latest.state.toLowerCase();
              if (['idle', 'speaking', 'thinking'].includes(s)) {
                setVisualState(s as any);
              } else {
                setVisualState('idle');
              }
            } else {
              setVisualState('idle');
            }
          } catch {
            setVisualState('idle');
          }
        }
      }, 10000);
    }
  };
  
  // Audio & Mic Mode Status on Mobile (Remote Desktop vs Full Mobile)
  const getInitialAudioMode = (): MobileAudioMode => {
    try {
      const saved = localStorage.getItem('ragasukma_audio_mode');
      if (saved === 'remote_desktop' || saved === 'full_mobile') {
        return saved;
      }
    } catch {}
    return 'remote_desktop';
  };

  const [audioMode, setAudioMode] = createSignal<MobileAudioMode>(getInitialAudioMode());
  const isHpAudioActive = () => audioMode() === 'full_mobile';

  const syncAudioModeFromState = (voiceMuted?: boolean, micMuted?: boolean) => {
    if (voiceMuted === undefined || micMuted === undefined) return;
    const saved = localStorage.getItem('ragasukma_audio_mode') as MobileAudioMode | null;
    if (saved && (saved === 'remote_desktop' || saved === 'full_mobile')) {
      if (audioMode() !== saved) {
        setAudioMode(saved);
      }
      return;
    }

    const targetMode: MobileAudioMode = (voiceMuted && micMuted) ? 'full_mobile' : 'remote_desktop';
    if (audioMode() !== targetMode) {
      setAudioMode(targetMode);
      try {
        localStorage.setItem('ragasukma_audio_mode', targetMode);
      } catch {}
      if (targetMode !== 'full_mobile') {
        audioSync.stopAll();
      }
    }
  };
  const [isHpMicListening, setIsHpMicListening] = createSignal(false);
  const [hasUnlockedAudio, setHasUnlockedAudio] = createSignal(false);
  const [mobileMusicState, setMobileMusicState] = createSignal<MobileTrackState>({
    track: null,
    isPlaying: false,
    isPaused: false,
    currentTime: 0,
    duration: 0,
  });

  // Live Console Logs
  const [logs, setLogs] = createSignal<LogEntry[]>([]);
  const [isRefreshingLogs, setIsRefreshingLogs] = createSignal(false);
  const [networkPing, setNetworkPing] = createSignal<number | null>(null);

  let logsContainerRef: HTMLDivElement | undefined;
  let isFetchingLogs = false;
  let speechRec: any = null;
  const audioSync = globalAudioSync;
  let lastActionId = 0;

  // Screen Wake Lock API to prevent phone screen sleep & keep SSE connection alive
  let wakeLockSentinel: any = null;
  const requestWakeLock = async () => {
    if (typeof navigator !== 'undefined' && 'wakeLock' in navigator && !wakeLockSentinel) {
      try {
        wakeLockSentinel = await (navigator as any).wakeLock.request('screen');
        wakeLockSentinel.addEventListener('release', () => {
          wakeLockSentinel = null;
        });
        console.log('📱 [Mobile] Screen Wake Lock active (prevents screen sleep & socket freeze)');
      } catch (err) {
        console.warn('⚠️ [Mobile] Wake Lock request error:', err);
      }
    }
  };

  const releaseWakeLock = () => {
    if (wakeLockSentinel) {
      try {
        wakeLockSentinel.release();
      } catch {}
      wakeLockSentinel = null;
    }
  };

  const fetchLogs = async () => {
    if (isFetchingLogs || document.hidden) return;
    isFetchingLogs = true;
    try {
      const data = await api.getLogs();
      if (data && Array.isArray(data)) {
        const trimmed = data.length > 60 ? data.slice(data.length - 60) : data;
        setLogs(trimmed);
      }
    } catch {
      // ignore
    } finally {
      isFetchingLogs = false;
    }
  };

  const notifyRemoteActive = () => {
    fetch(`${API_BASE}/api/remote/active`, { method: 'POST' }).catch(() => {});
  };

  let activeVoiceAudio: HTMLAudioElement | null = null;
  let isVoiceSpeakingOnMobile = false;
  let mobileSpeakSeq = 0;

  // Realtime Audio Chunk Queue for zero-latency progressive TTS playback
  interface ChunkQueueItem {
    chunkIndex: number;
    totalChunks: number;
    audioUrl: string;
    text: string;
    emotion: string;
    seq: number;
    prefetchedBlobUrl?: string;
  }
  let audioChunkQueue: Map<number, ChunkQueueItem> = new Map();
  let prefetchedBlobUrls: Map<number, string> = new Map();
  let nextExpectedChunkIndex = 0;
  let isPlayingChunk = false;
  let chunkTotalExpected = 1;
  let chunkWaitTimer: number | null = null;

  const clearPrefetchedBlobs = () => {
    prefetchedBlobUrls.forEach((blobUrl) => {
      try {
        URL.revokeObjectURL(blobUrl);
      } catch {}
    });
    prefetchedBlobUrls.clear();
  };

  const prefetchChunkAudio = async (item: ChunkQueueItem) => {
    if (prefetchedBlobUrls.has(item.chunkIndex) || item.seq !== mobileSpeakSeq) return;
    try {
      const fullUrl = item.audioUrl.startsWith('http') || item.audioUrl.startsWith('blob:')
        ? item.audioUrl
        : `${API_BASE}${item.audioUrl}`;
      const res = await fetch(fullUrl, { mode: 'cors' });
      if (!res.ok) return;
      const blob = await res.blob();
      if (item.seq !== mobileSpeakSeq) return;
      const blobUrl = URL.createObjectURL(blob);
      prefetchedBlobUrls.set(item.chunkIndex, blobUrl);
      const queueItem = audioChunkQueue.get(item.chunkIndex);
      if (queueItem && queueItem.seq === item.seq) {
        queueItem.prefetchedBlobUrl = blobUrl;
      }
    } catch {}
  };

  const notifySpeakingState = (speaking: boolean, em: string = 'neutral') => {
    fetch(`${API_BASE}/api/tts/state`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ speaking, state: speaking ? 'speaking' : 'idle', emotion: em }),
    }).catch(() => {});
  };

  const processAudioChunkQueue = async () => {
    if (isPlayingChunk) return;

    // Cari chunk yang persis sesuai urutan berikutnya (strictly sequential 0 -> 1 -> 2 -> ...)
    const item = audioChunkQueue.get(nextExpectedChunkIndex);
    if (!item) {
      // Jika chunk belum tiba tapi totalChunks belum tercapai, beri toleransi buffer
      if (nextExpectedChunkIndex < chunkTotalExpected) {
        if (!chunkWaitTimer) {
          chunkWaitTimer = window.setTimeout(() => {
            chunkWaitTimer = null;
            // Jika chunk hilang/gagal download setelah 3 detik, paksa lompat ke chunk berikutnya
            if (audioChunkQueue.size > 0) {
              const availableKeys = Array.from(audioChunkQueue.keys()).sort((a, b) => a - b);
              if (availableKeys.length > 0 && availableKeys[0] > nextExpectedChunkIndex) {
                nextExpectedChunkIndex = availableKeys[0];
                processAudioChunkQueue();
                return;
              }
            }
            if (isVoiceSpeakingOnMobile && audioChunkQueue.size === 0) {
              isVoiceSpeakingOnMobile = false;
              audioSync.unduckMusic();
              updateVisualStateSafe('idle');
              setEmotion('neutral');
              notifySpeakingState(false);
              clearPrefetchedBlobs();
            }
          }, 3000);
        }
      }
      return;
    }

    if (chunkWaitTimer) {
      window.clearTimeout(chunkWaitTimer);
      chunkWaitTimer = null;
    }

    if (item.seq !== mobileSpeakSeq) {
      audioChunkQueue.clear();
      clearPrefetchedBlobs();
      isPlayingChunk = false;
      return;
    }

    isPlayingChunk = true;
    audioChunkQueue.delete(nextExpectedChunkIndex);
    chunkTotalExpected = item.totalChunks;

    const playUrl = item.prefetchedBlobUrl || (item.audioUrl.startsWith('http') || item.audioUrl.startsWith('blob:')
      ? item.audioUrl
      : `${API_BASE}${item.audioUrl}`);

    isVoiceSpeakingOnMobile = true;
    updateVisualStateSafe('speaking');
    setEmotion(item.emotion);
    notifySpeakingState(true, item.emotion);

    const playNext = () => {
      // Clean up prefetched blob memory immediately after this chunk finishes playing
      if (item.prefetchedBlobUrl) {
        try {
          URL.revokeObjectURL(item.prefetchedBlobUrl);
        } catch {}
        prefetchedBlobUrls.delete(item.chunkIndex);
      }

      isPlayingChunk = false;
      nextExpectedChunkIndex++;

      if (nextExpectedChunkIndex < chunkTotalExpected || audioChunkQueue.size > 0) {
        processAudioChunkQueue();
      } else {
        // Semua chunk selesai diputar sempurna
        isVoiceSpeakingOnMobile = false;
        audioSync.unduckMusic();
        updateVisualStateSafe('idle');
        setEmotion('neutral');
        notifySpeakingState(false);
        clearPrefetchedBlobs();
      }
    };

    audioSync.playVoice(playUrl, {
      onPlaying: () => {
        isVoiceSpeakingOnMobile = true;
        updateVisualStateSafe('speaking');
        notifySpeakingState(true, item.emotion);
      },
      onEnded: playNext,
      onError: (err) => {
        console.warn('[Mobile Chunk Audio] Error playing chunk, trying WAV fallback:', err);
        if (!playUrl.includes('format=wav') && !playUrl.startsWith('blob:')) {
          const fallbackUrl = playUrl.includes('?') ? `${playUrl}&format=wav` : `${playUrl}?format=wav`;
          audioSync.playVoice(fallbackUrl, {
            onPlaying: () => {
              updateVisualStateSafe('speaking');
              notifySpeakingState(true, item.emotion);
            },
            onEnded: playNext,
            onError: () => {
              playNext();
            },
          });
        } else {
          playNext();
        }
      },
    });
  };

  const enqueueAudioChunk = (
    chunkIndex: number,
    totalChunks: number,
    audioUrl: string,
    text: string,
    emotion: string
  ) => {
    if (!isHpAudioActive()) return;

    if (chunkIndex === 0) {
      mobileSpeakSeq++;
      audioChunkQueue.clear();
      clearPrefetchedBlobs();
      nextExpectedChunkIndex = 0;
      chunkTotalExpected = totalChunks;
      isPlayingChunk = false;
      audioSync.stopVoice();
      isVoiceSpeakingOnMobile = true;
      updateVisualStateSafe('speaking');
      setEmotion(emotion);
      notifySpeakingState(true, emotion);
      audioSync.duckMusic(0.25);
    }

    const item: ChunkQueueItem = {
      chunkIndex,
      totalChunks,
      audioUrl,
      text,
      emotion,
      seq: mobileSpeakSeq,
    };
    audioChunkQueue.set(chunkIndex, item);

    // Pre-fetch in background as fast as network permits to eliminate any latency gaps
    prefetchChunkAudio(item);

    if (!isPlayingChunk) {
      processAudioChunkQueue();
    }
  };

  const speakOnMobile = (text: string, em: string, audioUrl?: string | null, lang?: string) => {
    if (!isHpAudioActive()) return;
    const mySeq = ++mobileSpeakSeq;

    // Stop previous TTS voice if any (do not stop music!)
    audioSync.stopVoice();
    if (activeVoiceAudio) {
      try {
        activeVoiceAudio.pause();
        activeVoiceAudio.currentTime = 0;
      } catch {}
      activeVoiceAudio = null;
    }

    // Lock speaking status immediately so mobile avatar responds without premature idle
    isVoiceSpeakingOnMobile = true;
    updateVisualStateSafe('speaking');
    setEmotion(em);

    // Audio Ducking: lower music volume so AI voice is clearly audible
    audioSync.duckMusic(0.25);

    let isFinished = false;
    const finishSpeaking = () => {
      if (isFinished || mySeq !== mobileSpeakSeq) return;
      isFinished = true;
      activeVoiceAudio = null;
      isVoiceSpeakingOnMobile = false;
      audioSync.unduckMusic();
      updateVisualStateSafe('idle');
      setEmotion('neutral');
      fetch(`${API_BASE}/api/tts/state`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
      }).catch(() => {});
    };

    // Absolute fail-safe maximum speaking timeout watchdog:
    // Guarantees speaking state never gets stuck on mobile even if onEnded is dropped
    const estDurationMs = Math.max(3000, Math.min(16000, (text.length || 20) * 110));
    const maxWatchdog = window.setTimeout(() => {
      finishSpeaking();
    }, estDurationMs + 2500);

    const playAudio = (url: string) => {
      const fullUrl = url.startsWith('http') || url.startsWith('blob:') ? url : `${API_BASE}${url}`;
      isVoiceSpeakingOnMobile = true;
      activeVoiceAudio = true as any;

      let hasReportedSpeaking = false;
      const notifySpeaking = () => {
        if (hasReportedSpeaking) return;
        hasReportedSpeaking = true;
        updateVisualStateSafe('speaking');
        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: true, emotion: em }),
        }).catch(() => {});
      };

      audioSync.playVoice(fullUrl, {
        onPlaying: notifySpeaking,
        onEnded: () => {
          window.clearTimeout(maxWatchdog);
          finishSpeaking();
        },
        onError: (err) => {
          console.warn('[Mobile Supertonic Audio] Playback error/blocked:', err);
          // Auto-fallback to WAV if Opus fails on specific device
          if (!fullUrl.includes('format=wav') && !fullUrl.startsWith('blob:')) {
            const fallbackUrl = fullUrl.includes('?') ? `${fullUrl}&format=wav` : `${fullUrl}?format=wav`;
            console.log('[Mobile Supertonic Audio] Retrying with raw WAV format:', fallbackUrl);
            audioSync.playVoice(fallbackUrl, {
              onPlaying: notifySpeaking,
              onEnded: () => {
                window.clearTimeout(maxWatchdog);
                finishSpeaking();
              },
              onError: (err2) => {
                console.warn('[Mobile Supertonic Audio] WAV fallback also failed:', err2);
                window.clearTimeout(maxWatchdog);
                const readingMs = Math.max(1500, Math.min(6000, text.length * 65));
                window.setTimeout(() => {
                  finishSpeaking();
                }, readingMs);
              },
            });
            return;
          }

          window.clearTimeout(maxWatchdog);
          // If audio playback failed due to gesture lock or network, simulate speaking for reading duration
          const readingMs = Math.max(1500, Math.min(6000, text.length * 65));
          window.setTimeout(() => {
            finishSpeaking();
          }, readingMs);
        },
      });
    };

    if (audioUrl) {
      playAudio(audioUrl);
      return;
    }

    if (!text.trim()) {
      isVoiceSpeakingOnMobile = false;
      setVisualState('idle');
      setEmotion('neutral');
      return;
    }

    fetch(`${API_BASE}/api/tts/synthesize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        text,
        voice: 'F4',
        lang: lang || mobileLang() || 'id',
        speed: 1.05,
      }),
    })
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.blob();
      })
      .then((blob) => {
        const url = URL.createObjectURL(blob);
        playAudio(url);
      })
      .catch((err) => {
        console.warn('[Mobile Supertonic Audio] Synthesize fetch error:', err);
        isVoiceSpeakingOnMobile = false;
        audioSync.unduckMusic();
        setVisualState('idle');
        setEmotion('neutral');
      });
  };

  // Handler Terpusat All Aksi (SSE + Polling Fallback) with Deduplikasi ID
  const handleIncomingAction = (act: any) => {
    if (!act || !act.action) return;

    if (act.id !== undefined) {
      if (act.id <= lastActionId) return;
      lastActionId = act.id;
    }

    console.log('[MobileApp] Action received:', act.action, act.payload);

    // 1. Voice respon Supertonic-3 Neural Audio (Opus / WAV stream)
    if (act.action === 'assistant:speak') {
      let textToSpeak = '';
      let em = 'neutral';
      let audioUrl: string | null = null;
      let lang: string | undefined;
      let isChunk = false;
      let chunkIndex = 0;
      let totalChunks = 1;
      try {
        const parsed = typeof act.payload === 'string' ? JSON.parse(act.payload) : act.payload;
        textToSpeak = parsed.text || '';
        em = parsed.emotion || 'neutral';
        audioUrl = parsed.audio_url || null;
        lang = parsed.lang;
        isChunk = !!parsed.is_chunk;
        chunkIndex = parsed.chunk_index ?? 0;
        totalChunks = parsed.total_chunks ?? 1;
      } catch {
        textToSpeak = typeof act.payload === 'string' ? act.payload : '';
      }
      if (em) {
        setEmotion(em);
      }

      if (isChunk && audioUrl) {
        console.log(`[MobileApp] assistant:speak (chunk ${chunkIndex}/${totalChunks}). audioUrl:`, audioUrl);
        if (isHpAudioActive()) {
          enqueueAudioChunk(chunkIndex, totalChunks, audioUrl, textToSpeak, em);
        }
        return;
      }

      // If non-chunk fallback (e.g., direct synthesize or single full audio)
      if (textToSpeak.trim() || audioUrl) {
        console.log('[MobileApp] assistant:speak received. Mode:', audioMode(), 'audioUrl:', audioUrl);
        if (isHpAudioActive()) {
          // If we already received streaming chunks for this turn or audio is still playing, ignore the duplicate full action
          if (isVoiceSpeakingOnMobile || audioChunkQueue.size > 0 || isPlayingChunk) {
            return;
          }
          speakOnMobile(textToSpeak.trim(), em, audioUrl, lang);
        } else {
          console.log('[MobileApp] Audio ignored on mobile because mode is:', audioMode());
        }
      }
    }

    // 2. Streaming music routed to phone or controlled on PC
    if (act.action === 'music:play') {
      const trackTitle = typeof act.payload === 'string' ? act.payload : 'Desktop Audio';
      if (isHpAudioActive()) {
        audioSync.playMusic(act.payload);
      } else {
        setMobileMusicState({
          track: { id: '', title: trackTitle, uploader: 'Desktop Audio', duration: 0, filename: '' },
          isPlaying: true,
          isPaused: false,
          currentTime: 0,
          duration: 0,
        });
      }
    } else if (act.action === 'music:pause') {
      if (isHpAudioActive()) {
        audioSync.pauseMusic();
      } else {
        setMobileMusicState((prev) => ({ ...prev, isPlaying: false, isPaused: true }));
      }
    } else if (act.action === 'music:resume') {
      if (isHpAudioActive()) {
        audioSync.resumeMusic();
      } else {
        setMobileMusicState((prev) => ({ ...prev, isPlaying: true, isPaused: false }));
      }
    } else if (act.action === 'music:seek') {
      if (act.payload) {
        const sec = parseFloat(act.payload);
        if (!isNaN(sec)) {
          if (isHpAudioActive()) {
            audioSync.seekMusic(sec);
          } else {
            setMobileMusicState((prev) => ({ ...prev, currentTime: sec }));
          }
        }
      }
    } else if (act.action === 'music:volume') {
      if (act.payload) {
        const v = parseFloat(act.payload);
        if (!isNaN(v)) {
          const clamped = Math.max(0, Math.min(1, v > 2 ? v / 100 : v));
          if (isHpAudioActive()) {
            audioSync.setVolume(clamped);
          }
        }
      }
    } else if (act.action === 'music:stop') {
      if (isHpAudioActive()) {
        audioSync.stopMusic();
      } else {
        setMobileMusicState({
          track: null,
          isPlaying: false,
          isPaused: false,
          currentTime: 0,
          duration: 0,
        });
      }
    }
  };

  const [mobileLang, setMobileLang] = createSignal('id');

  const setupSpeechRecognition = (langCode: string = 'id') => {
    if (typeof window === 'undefined') return;
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    if (speechRec) {
      try {
        speechRec.stop();
      } catch {}
    }

    speechRec = new SpeechRecognition();
    speechRec.lang = getChromeSttCode(langCode);
    speechRec.interimResults = false;
    speechRec.continuous = false;

    speechRec.onstart = () => {
      setIsHpMicListening(true);
      setVisualState('listening');
    };

    speechRec.onresult = (event: any) => {
      const transcript = event.results[0][0].transcript;
      if (transcript && transcript.trim()) {
        // Release STT mic immediately so Android exits Voice Telephony mode back to Media Hi-Fi
        try {
          speechRec.abort();
        } catch {}
        setIsHpMicListening(false);
        setInputText(transcript.trim());
        sendTextToHost(transcript.trim());
      }
    };

    speechRec.onerror = () => {
      setIsHpMicListening(false);
      setVisualState('idle');
    };

    speechRec.onend = () => {
      setIsHpMicListening(false);
      setVisualState('idle');
    };
  };

  const toggleHpMic = () => {
    if (!hasUnlockedAudio()) {
      unlockAudioContext(true);
    }
    if (!speechRec) {
      props.showToast('Use the microphone key on your smartphone keyboard.');
      return;
    }

    if (isHpMicListening()) {
      try {
        speechRec.stop();
      } catch {}
      setIsHpMicListening(false);
    } else {
      try {
        speechRec.start();
      } catch (e) {
        console.warn('Mic start error:', e);
      }
    }
  };

  const unlockAudioContext = async (silent = false) => {
    await audioSync.unlockAudio();
    setHasUnlockedAudio(true);
    if (!silent) {
      props.showToast('Phone audio active & synchronized');
    }
  };

  const switchAudioMode = async (mode: MobileAudioMode) => {
    if (audioMode() === mode) return;
    try {
      localStorage.setItem('ragasukma_audio_mode', mode);
    } catch {}
    setAudioMode(mode);

    if (mode === 'remote_desktop') {
      audioSync.stopAll();
      try {
        await api.sendCommand('mode_remote_desktop');
      } catch {}
      props.showToast('Switched to Remote Desktop Mode. Reloading session...');
      setShowSettings(false);
      setTimeout(() => {
        window.location.reload();
      }, 350);
    } else {
      try {
        await api.sendCommand('mode_full_mobile');
      } catch {}
      props.showToast('Switched to Full Mobile Mode. Reloading audio session...');
      setShowSettings(false);
      setTimeout(() => {
        window.location.reload();
      }, 350);
    }
  };

  const handleDisconnectRemote = async () => {
    try {
      audioSync.stopAll();
      localStorage.setItem('ragasukma_audio_mode', 'remote_desktop');
      setAudioMode('remote_desktop');
      await api.sendCommand('mode_reset_desktop');
      props.showToast('Remote session disconnected, audio & mic returned to desktop.');
      setShowSettings(false);
      setTimeout(() => {
        window.location.hash = '';
        window.location.reload();
      }, 400);
    } catch {
      props.showToast('Failed to disconnect remote session.');
    }
  };

  onMount(() => {
    // Auto unlock audio on any first user touch/click anywhere on mobile screen
    const handleGlobalUserInteraction = () => {
      if (!hasUnlockedAudio()) {
        unlockAudioContext(true);
      }
    };
    window.addEventListener('touchstart', handleGlobalUserInteraction, { passive: true, once: true });
    window.addEventListener('click', handleGlobalUserInteraction, { passive: true, once: true });

    onCleanup(() => {
      window.removeEventListener('touchstart', handleGlobalUserInteraction);
      window.removeEventListener('click', handleGlobalUserInteraction);
    });

    audioSync.setOnTrackStateChange((st) => {
      setMobileMusicState(st);
    });

    // Hydrate initial state immediately
    api.getState().then((st) => {
      if (st) {
        if (st.voice_muted !== undefined && st.mic_muted !== undefined) {
          syncAudioModeFromState(st.voice_muted, st.mic_muted);
        }
        if (st.state) {
          const s = st.state.toLowerCase();
          if (['idle', 'listening', 'speaking', 'thinking'].includes(s)) {
            setVisualState(s as any);
          }
          if (st.emotion) setEmotion(st.emotion);
        }
      }
    }).catch(() => {});

    // Check initial model status
    api.getModelStatus().then((st) => {
      setModelStatus(st);
    }).catch(() => {});

    // Sync active mode to backend (Full HP -> mode_full_mobile; Remote PC -> mode_remote_desktop)
    const currentMode = audioMode();
    if (currentMode === 'full_mobile') {
      api.sendCommand('mode_full_mobile').catch(() => {});
    } else {
      api.sendCommand('mode_remote_desktop').catch(() => {});
    }

    // Sync STT language dynamically from server config
    api.getConfig().then((cfg) => {
      if (cfg && cfg.lang) {
        setMobileLang(cfg.lang);
        setupSpeechRecognition(cfg.lang);
      } else {
        setupSpeechRecognition('id');
      }
    }).catch(() => {
      setupSpeechRecognition('id');
    });

    fetchLogs();

    // Monitor virtual keyboard on mobile via VisualViewport so layout is not obscured
    const updateViewport = () => {
      if (window.visualViewport) {
        const height = window.visualViewport.height;
        setViewportHeight(`${height}px`);
        window.scrollTo(0, 0);
      } else {
        setViewportHeight('100dvh');
      }
    };
    updateViewport();
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', updateViewport);
      window.visualViewport.addEventListener('scroll', updateViewport);
    } else {
      window.addEventListener('resize', updateViewport);
    }

    // Auto-unlock audio HP on sentuhan/interaksi pertama user
    const handleFirstInteraction = () => {
      if (!hasUnlockedAudio()) {
        unlockAudioContext(true);
      }
      requestWakeLock();
    };
    window.addEventListener('touchstart', handleFirstInteraction, { once: true, passive: true });
    window.addEventListener('click', handleFirstInteraction, { once: true });

    // Request wake lock so the mobile display stays awake
    requestWakeLock();
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        requestWakeLock();
      }
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);

    // ── Pure SSE Subscriptions (Zero Polling, Zero Battery Waste, Zero Latency) ──
    const unsubState = api.subscribeState((st) => {
      if (st) {
        if (st.voice_muted !== undefined && st.mic_muted !== undefined) {
          syncAudioModeFromState(st.voice_muted, st.mic_muted);
        }
        if (st.state) {
          const s = st.state.toLowerCase();
          // IF HP currently playing voice TTS, maintain speaking state
          if (activeVoiceAudio !== null || isVoiceSpeakingOnMobile) {
            updateVisualStateSafe('speaking');
          } else if (isHpMicListening()) {
            updateVisualStateSafe('listening');
          } else {
            if (['idle', 'listening', 'speaking', 'thinking'].includes(s)) {
              updateVisualStateSafe(s as any);
            }
            if (st.emotion) {
              setEmotion(st.emotion);
            } else if (s === 'idle') {
              setEmotion('neutral');
            }
          }
        }
      }
    });

    const unsubActions = api.subscribeActions((act) => {
      handleIncomingAction(act);
    });

    const unsubLogs = api.subscribeLogs((logEntry) => {
      setLogs((prev) => [...prev.slice(Math.max(0, prev.length - 59)), logEntry]);
    });

    const unsubModel = api.subscribeModelStatus((status) => {
      setModelStatus(status);
    });

    // ── Network Latency Ping Monitor (Check every 4s) ──
    let pingInterval: number | null = null;
    const measurePing = async () => {
      if (typeof document !== 'undefined' && document.hidden) return;
      const t0 = performance.now();
      try {
        await fetch(`${API_BASE}/api/state`, { method: 'GET', cache: 'no-store' });
        const roundtrip = Math.round(performance.now() - t0);
        setNetworkPing(roundtrip);
      } catch {
        setNetworkPing(null);
      }
    };

    measurePing();
    pingInterval = window.setInterval(measurePing, 4000);

    // Session recovery helper: restore full mic & voice to desktop on close
    const restoreDesktopSession = () => {
      try {
        const payload = JSON.stringify({ cmd: 'mode_reset_desktop' });
        if (typeof navigator !== 'undefined' && navigator.sendBeacon) {
          const blob = new Blob([payload], { type: 'application/json' });
          navigator.sendBeacon(`${API_BASE}/api/cmd`, blob);
        } else {
          fetch(`${API_BASE}/api/cmd`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: payload,
            keepalive: true,
          }).catch(() => {});
        }
      } catch {}
    };

    window.addEventListener('beforeunload', restoreDesktopSession);

    onCleanup(() => {
      document.title = 'Ragasukma - AI Companion & Dashboard';
      unsubState();
      unsubActions();
      unsubLogs();
      unsubModel();
      if (pingInterval) {
        clearInterval(pingInterval);
        pingInterval = null;
      }
      releaseWakeLock();
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('beforeunload', restoreDesktopSession);
      window.removeEventListener('touchstart', handleFirstInteraction);
      window.removeEventListener('click', handleFirstInteraction);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener('resize', updateViewport);
        window.visualViewport.removeEventListener('scroll', updateViewport);
      } else {
        window.removeEventListener('resize', updateViewport);
      }
      if (speechRec) {
        try {
          speechRec.onstart = null;
          speechRec.onresult = null;
          speechRec.onerror = null;
          speechRec.onend = null;
          speechRec.abort();
        } catch {}
        speechRec = null;
      }
      audioSync.stopAll();
      clearPrefetchedBlobs();
    });
  });

  createEffect(() => {
    logs();
    if (logsContainerRef) {
      logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
    }
  });

  // Synchronize browser tab title on mobile (Default: "Ragasukma", When music: "▶ Title - Ragasukma")
  createEffect(() => {
    const st = mobileMusicState();
    if (st.track && st.track.title) {
      if (st.isPlaying) {
        document.title = `▶ ${st.track.title} - Ragasukma`;
      } else if (st.isPaused) {
        document.title = `⏸ ${st.track.title} - Ragasukma`;
      } else {
        document.title = 'Ragasukma';
      }
    } else {
      document.title = 'Ragasukma';
    }
  });

  const sendTextToHost = async (customText?: string) => {
    if (!hasUnlockedAudio()) {
      unlockAudioContext(true);
    }
    const textToSend = (customText !== undefined ? customText : inputText()).trim();
    if (!textToSend || isSending()) return;

    // Ensure mobile STT mic is released so phone speaker is not trapped in telephony filter
    if (speechRec && isHpMicListening()) {
      try {
        speechRec.abort();
      } catch {}
      setIsHpMicListening(false);
    }

    // Interupsi response lama seketika: invalidasi seq, bersihkan antrean chunk & blob
    mobileSpeakSeq++;
    audioChunkQueue.clear();
    clearPrefetchedBlobs();
    isPlayingChunk = false;
    isVoiceSpeakingOnMobile = false;
    audioSync.unduckMusic();

    if (activeVoiceAudio) {
      try {
        activeVoiceAudio = null;
      } catch {}
    }
    // Stop assistant voice if speaking (user interruption), do not stop music!
    audioSync.stopVoice();
    notifySpeakingState(false);

    setIsSending(true);
    try {
      const res = await api.sendRemoteText(textToSend, props.authKey);
      if (res.success) {
        setLastSentText(textToSend);
        setInputText('');
        updateVisualStateSafe('thinking');
        props.showToast(`Sent: "${textToSend}"`);
      } else {
        props.showToast(res.error || 'Failed to send message');
      }
    } catch (err: any) {
      props.showToast(`Error: ${err.message || err}`);
    } finally {
      setIsSending(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendTextToHost();
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(text);
      }
      props.showToast('Logs copied to clipboard');
    } catch {
      props.showToast('Failed to copy to clipboard');
    }
  };

  const formatMessageStyle = (msg: string, level: string) => {
    if (level === 'error' || msg.includes('[ERROR]') || msg.includes('FAIL')) {
      return 'text-red-400 font-semibold';
    }
    if (msg.includes('[User]')) {
      return 'text-theme-cyan font-bold';
    }
    if (msg.includes('[LLM]') || msg.includes('responding')) {
      return 'text-theme-rose font-medium';
    }
    if (msg.includes('[RESUMED]') || msg.includes('[Music Controller]')) {
      return 'text-theme-cyan';
    }
    return 'text-theme-text/80';
  };

  return (
    <div
      class="relative w-full bg-theme-bg select-text max-w-lg mx-auto shadow-2xl overflow-hidden flex flex-col justify-between"
      style={{ height: viewportHeight() }}
    >
      {/* 1. Modal Unlock Audio HP (1x Interaksi) */}
      <MobileAudioUnlockModal
        show={isHpAudioActive() && !hasUnlockedAudio()}
        onUnlock={() => unlockAudioContext(false)}
      />

      {/* 2. Floating Top Controls (Ultra Clean: Left Floating Status Pill & Right Settings Button) */}
      {/* 1. Header Ringkas Mobile Modern (Status avatar, network ping & settings) */}
      <MobileTopBar
        visualState={visualState()}
        onOpenSettings={() => setShowSettings(true)}
        logsCount={logs().length}
        pingMs={networkPing()}
      />

      {/* Sheet Companion Settings */}
      <MobileSettingsSheet
        isOpen={showSettings()}
        onClose={() => setShowSettings(false)}
        audioMode={audioMode()}
        onSwitchMode={switchAudioMode}
        logsCount={logs().length}
        onOpenLogs={() => setShowLogsDrawer(true)}
        onUnlockAudio={() => unlockAudioContext(false)}
        onDisconnectRemote={handleDisconnectRemote}
      />

      {/* 3. Karakter 3D VRM 100% Fullscreen Full Without Terpotong Padding */}
      <div class="absolute inset-0 z-0 w-full h-full">
        <AvatarViewer
          state={visualState()}
          emotion={emotion()}
          isAudioStreaming={isHpAudioActive()}
          onArStateChange={setIsArActive}
          arBottomSlot={() => (
            <div class="w-full max-w-md mx-auto pointer-events-auto">
              <MobileInteractionDock
                inputText={inputText()}
                onInputText={setInputText}
                onSend={() => sendTextToHost()}
                isSending={isSending()}
                isListening={isHpMicListening()}
                onToggleMic={toggleHpMic}
                lastSentText={lastSentText()}
                showInput={showTextInput()}
                onToggleShowInput={() => setShowTextInput(!showTextInput())}
                onDismissLastSent={() => setLastSentText('')}
              />
            </div>
          )}
        />
      </div>

      {/* Flexible center spacer for floating bottom controls */}
      <div class="flex-1 min-h-0 pointer-events-none" />

      {/* 4. Bottom Controls: Floating Music + Circular Interaction Dock (Non-AR mode) */}
      <div
        class="relative z-20 w-full max-w-md mx-auto pointer-events-auto flex flex-col gap-1.5 transition-opacity duration-200"
        style={{ display: isArActive() ? 'none' : 'flex' }}
      >
        {/* Floating Music Player Mini Card (When Music Is Playing) */}
        <div class="px-3">
          <MobileFloatingMusic
            trackState={mobileMusicState()}
            isHpAudio={isHpAudioActive()}
            onTogglePlay={async () => {
              if (isHpAudioActive()) {
                if (mobileMusicState().isPlaying) {
                  audioSync.pauseMusic();
                  await api.controlMusic('pause').catch(() => {});
                } else {
                  audioSync.resumeMusic();
                  await api.controlMusic('resume').catch(() => {});
                }
              } else {
                const nextPlaying = !mobileMusicState().isPlaying;
                setMobileMusicState((prev) => ({ ...prev, isPlaying: nextPlaying, isPaused: !nextPlaying }));
                await api.controlMusic(nextPlaying ? 'resume' : 'pause');
              }
            }}
            onStop={async () => {
              if (isHpAudioActive()) {
                audioSync.stopMusic();
                await api.controlMusic('stop').catch(() => {});
              } else {
                setMobileMusicState({ track: null, isPlaying: false, isPaused: false, currentTime: 0, duration: 0 });
                await api.controlMusic('stop');
              }
            }}
          />
        </div>

        {/* Floating Circular Interaction Dock (Circular Mic PTT Button & Message Input Toggle) */}
        <MobileInteractionDock
          inputText={inputText()}
          onInputText={setInputText}
          onSend={() => sendTextToHost()}
          isSending={isSending()}
          isListening={isHpMicListening()}
          onToggleMic={toggleHpMic}
          lastSentText={lastSentText()}
          showInput={showTextInput()}
          onToggleShowInput={() => setShowTextInput(!showTextInput())}
          onDismissLastSent={() => setLastSentText('')}
        />
      </div>

      {/* 5. Slide-Up Bottom Sheet Console Logs */}
      <MobileLogsDrawer
        isOpen={showLogsDrawer()}
        onClose={() => setShowLogsDrawer(false)}
        logs={logs()}
        onRefresh={async () => {
          setIsRefreshingLogs(true);
          await fetchLogs();
          setTimeout(() => setIsRefreshingLogs(false), 300);
        }}
        isRefreshing={isRefreshingLogs()}
        onCopy={() => {
          const text = logs().map((l) => `[${l.timestamp}] ${l.message}`).join('\n');
          copyToClipboard(text);
        }}
        formatStyle={formatMessageStyle}
      />

      {/* Model Download & Extraction Fullscreen Overlay */}
      <Show when={isModelLocked()}>
        <ModelLoadingOverlay
          modelStatus={modelStatus()}
          onRetry={() => {
            api.triggerModelDownload().then((st) => {
              setModelStatus(st);
            }).catch(() => {});
          }}
        />
      </Show>
    </div>
  );
};
