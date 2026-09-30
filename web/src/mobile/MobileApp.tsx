import { Component, createSignal, onMount, onCleanup, createEffect, For, Show } from 'solid-js';
import { 
  Terminal, 
  RefreshCw, 
  Send, 
  Trash2, 
  CheckCircle2, 
  Mic, 
  MicOff, 
  Volume2, 
  VolumeX, 
  Copy,
  Monitor,
  Smartphone,
  Music,
  Play,
  Pause,
  Square
} from 'lucide-solid';
import { api, LogEntry } from '../api';
import { API_BASE } from '../services/base';
import { AvatarViewer } from './AvatarViewer';
import { MobileAudioSyncManager, MobileTrackState } from './AudioSyncManager';

type MobileAudioMode = 'remote_desktop' | 'full_mobile';

interface MobileAppProps {
  authKey: string;
  showToast: (msg: string) => void;
}

export const MobileApp: Component<MobileAppProps> = (props) => {
  const [inputText, setInputText] = createSignal('');
  const [isSending, setIsSending] = createSignal(false);
  const [lastSentText, setLastSentText] = createSignal('');

  // Status Karakter Realtime
  const [visualState, setVisualState] = createSignal<'idle' | 'listening' | 'speaking' | 'thinking'>('idle');
  const [emotion, setEmotion] = createSignal('neutral');
  
  // Status Mode Audio & Mic di HP (Remote Desktop vs Full Mobile)
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

  let logsContainerRef: HTMLDivElement | undefined;
  let isFetchingLogs = false;
  let isPollingActions = false;
  let speechRec: any = null;
  const audioSync = new MobileAudioSyncManager();
  let lastActionId = 0;

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

  // Heartbeat Polling State Fallback (Menembus buffering tunnel Cloudflare / proxy seluler)
  const pollServerState = async () => {
    if (document.hidden) return;
    try {
      const res = await fetch(`${API_BASE}/api/state`);
      if (res.ok) {
        const data = await res.json();
        if (data && data.state) {
          const s = data.state.toLowerCase();
          if (['idle', 'listening', 'speaking', 'thinking'].includes(s)) {
            setVisualState(s as any);
          }
          if (data.emotion) {
            setEmotion(data.emotion);
          }
        }
      }
    } catch (_) {}
  };

  // Polling Actions Resilient: Mengambil event assistant:speak & music:play meskipun SSE dibuffer Cloudflare
  const pollServerActions = async () => {
    if (isPollingActions || document.hidden) return;
    isPollingActions = true;
    try {
      const res = await fetch(`${API_BASE}/api/actions/since?last_id=${lastActionId}`);
      if (res.ok) {
        const data = await res.json();
        if (data && Array.isArray(data.actions)) {
          for (const act of data.actions) {
            handleIncomingAction(act);
          }
          if (data.last_id && data.last_id > lastActionId) {
            lastActionId = data.last_id;
          }
        }
      }
    } catch (_) {} finally {
      isPollingActions = false;
    }
  };

  const notifyRemoteActive = () => {
    fetch(`${API_BASE}/api/remote/active`, { method: 'POST' }).catch(() => {});
  };

  const speakOnMobile = (text: string, em: string, audioUrl?: string | null) => {
    if (!isHpAudioActive()) return;

    audioSync.stopAll();
    setEmotion(em);

    const playAudio = (url: string, revokeOnEnd = false) => {
      const fullUrl = url.startsWith('http') || url.startsWith('blob:') ? url : `${API_BASE}${url}`;
      const audio = new Audio(fullUrl);

      let hasReportedSpeaking = false;
      const notifySpeaking = () => {
        if (hasReportedSpeaking) return;
        hasReportedSpeaking = true;
        setVisualState('speaking');
        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: true, emotion: em }),
        }).catch(() => {});
      };

      audio.onplaying = notifySpeaking;
      audio.onplay = notifySpeaking;

      audio.onended = () => {
        setVisualState('idle');
        if (revokeOnEnd) {
          try { URL.revokeObjectURL(url); } catch {}
        }
        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
        }).catch(() => {});
      };

      audio.onerror = (e) => {
        console.warn('[Mobile Supertonic Audio] Playback error:', e);
        setVisualState('idle');
      };

      audio.play().catch((err) => {
        console.warn('[Mobile Supertonic Audio] Play blocked/interrupted:', err);
        setVisualState('idle');
      });
    };

    if (audioUrl) {
      playAudio(audioUrl, false);
      return;
    }

    if (!text.trim()) {
      setVisualState('idle');
      return;
    }

    fetch(`${API_BASE}/api/tts/synthesize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        text,
        voice: 'F4',
        lang: 'id',
        speed: 1.05,
      }),
    })
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.blob();
      })
      .then((blob) => {
        const url = URL.createObjectURL(blob);
        playAudio(url, true);
      })
      .catch((err) => {
        console.warn('[Mobile Supertonic Audio] Synthesize fetch error:', err);
        setVisualState('idle');
      });
  };

  // Handler Terpusat Semua Aksi (SSE + Polling Fallback) dengan Deduplikasi ID
  const handleIncomingAction = (act: any) => {
    if (!act || !act.action) return;

    if (act.id !== undefined) {
      if (act.id <= lastActionId) return;
      lastActionId = act.id;
    }

    // 1. Suara respon Supertonic-3 AI atau Web Speech API HP Android
    if (act.action === 'assistant:speak') {
      let textToSpeak = '';
      let em = 'neutral';
      let audioUrl: string | null = null;
      try {
        const parsed = typeof act.payload === 'string' ? JSON.parse(act.payload) : act.payload;
        textToSpeak = parsed.text || '';
        em = parsed.emotion || 'neutral';
        audioUrl = parsed.audio_url || null;
      } catch {
        textToSpeak = typeof act.payload === 'string' ? act.payload : '';
      }
      if (em) {
        setEmotion(em);
      }
      if ((textToSpeak.trim() || audioUrl) && isHpAudioActive()) {
        speakOnMobile(textToSpeak.trim(), em, audioUrl);
      }
    }

    // 2. Musik streaming dialirkan ke HP
    if (act.action === 'music:play' && isHpAudioActive()) {
      audioSync.playMusic(act.payload);
    } else if (act.action === 'music:pause') {
      audioSync.pauseMusic();
    } else if (act.action === 'music:resume' && isHpAudioActive()) {
      audioSync.resumeMusic();
    } else if (act.action === 'music:stop') {
      audioSync.stopMusic();
    }
  };

  const setupSpeechRecognition = () => {
    if (typeof window === 'undefined') return;
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    speechRec = new SpeechRecognition();
    speechRec.lang = 'id-ID';
    speechRec.interimResults = false;
    speechRec.continuous = false;

    speechRec.onstart = () => {
      setIsHpMicListening(true);
      setVisualState('listening');
    };

    speechRec.onresult = (event: any) => {
      const transcript = event.results[0][0].transcript;
      if (transcript && transcript.trim()) {
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
      props.showToast('Gunakan ikon mikrofon di keyboard HP Anda.');
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
    notifyRemoteActive();
    if (!silent) {
      props.showToast('🔊 Audio HP aktif & tersinkronisasi');
    }
  };

  const switchAudioMode = async (mode: MobileAudioMode) => {
    if (audioMode() === mode) return;
    setAudioMode(mode);
    try {
      localStorage.setItem('ragasukma_audio_mode', mode);
    } catch {}

    if (mode === 'remote_desktop') {
      audioSync.stopAll();
      await api.sendCommand('mode_remote_desktop');
      props.showToast('🖥️ Mode Remote Desktop: Suara di Desktop, Mic PC Mute');
    } else {
      // Full HP: Suara TTS & musik dialihkan ke smartphone
      if (!hasUnlockedAudio()) {
        unlockAudioContext(true);
      }
      await api.sendCommand('mode_full_mobile');
      props.showToast('📱 Mode Full Mobile: Suara di HP, Desktop Senyap');
    }
  };

  onMount(() => {
    notifyRemoteActive();
    // Sinkronkan mode awal audio ke server (memastikan mic desktop langsung di-mute)
    const initMode = audioMode();
    api.sendCommand(initMode === 'full_mobile' ? 'mode_full_mobile' : 'mode_remote_desktop').catch(() => {});
    setupSpeechRecognition();
    fetchLogs();
    pollServerState();
    pollServerActions();

    // Auto-unlock audio HP pada sentuhan/interaksi pertama pengguna
    const handleFirstInteraction = () => {
      if (!hasUnlockedAudio()) {
        unlockAudioContext(true);
      }
    };
    window.addEventListener('touchstart', handleFirstInteraction, { once: true, passive: true });
    window.addEventListener('click', handleFirstInteraction, { once: true });

    // Loop polling log, state heartbeat, actions polling, dan remote keep-alive
    const logsInterval = setInterval(fetchLogs, 2500);
    const stateInterval = setInterval(pollServerState, 800);
    const actionsInterval = setInterval(pollServerActions, 600);
    const keepAliveInterval = setInterval(notifyRemoteActive, 10000);

    audioSync.setOnStateChange((st) => {
      if (['idle', 'listening', 'speaking', 'thinking'].includes(st)) {
        setVisualState(st as any);
      }
    });

    audioSync.setOnTrackStateChange((st) => {
      setMobileMusicState(st);
    });

    // 1. Subscribe Realtime SSE State
    const unsubState = api.subscribeState((st) => {
      if (st && st.state) {
        const s = st.state.toLowerCase();
        if (['idle', 'listening', 'speaking', 'thinking'].includes(s)) {
          setVisualState(s as any);
        }
        if ((st as any).emotion) {
          setEmotion((st as any).emotion);
        }
      }
    });

    // 2. Subscribe Action Events dari SSE
    const unsubActions = api.subscribeActions((act) => {
      handleIncomingAction(act);
    });

    onCleanup(() => {
      window.removeEventListener('touchstart', handleFirstInteraction);
      window.removeEventListener('click', handleFirstInteraction);
      clearInterval(logsInterval);
      clearInterval(stateInterval);
      clearInterval(actionsInterval);
      clearInterval(keepAliveInterval);
      unsubState();
      unsubActions();
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
    });
  });

  createEffect(() => {
    logs();
    if (logsContainerRef) {
      logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
    }
  });

  const sendTextToHost = async (customText?: string) => {
    if (!hasUnlockedAudio()) {
      unlockAudioContext(true);
    }
    const textToSend = (customText !== undefined ? customText : inputText()).trim();
    if (!textToSend || isSending()) return;

    audioSync.stopAll();

    setIsSending(true);
    try {
      const res = await api.sendRemoteText(textToSend, props.authKey);
      if (res.success) {
        setLastSentText(textToSend);
        setInputText('');
        setVisualState('thinking');
        props.showToast(`✅ Terkirim: "${textToSend}"`);
        setTimeout(fetchLogs, 300);
        setTimeout(pollServerActions, 500);
      } else {
        props.showToast(`⚠️ ${res.error || 'Gagal mengirim pesan'}`);
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
      props.showToast('📋 Log berhasil disalin');
    } catch {
      props.showToast('⚠️ Gagal menyalin');
    }
  };

  const formatMessageStyle = (msg: string, level: string) => {
    if (level === 'error' || msg.includes('[ERROR]') || msg.includes('GAGAL')) {
      return 'text-red-400 font-semibold';
    }
    if (msg.includes('[User]')) {
      return 'text-theme-cyan font-bold';
    }
    if (msg.includes('[LLM]') || msg.includes('merespons')) {
      return 'text-theme-rose font-medium';
    }
    if (msg.includes('✨') || msg.includes('▶️')) {
      return 'text-theme-cyan';
    }
    return 'text-theme-text/80';
  };

  return (
    <div class="relative flex flex-col h-full w-full bg-theme-bg select-text max-w-lg mx-auto shadow-2xl overflow-hidden">
      {/* 1. Modal Overlay Sambutan: Aktifkan Audio HP (Hanya saat mode Full HP & belum unlock) */}
      <Show when={isHpAudioActive() && !hasUnlockedAudio()}>
        <div class="absolute inset-0 z-50 bg-black/85 backdrop-blur-md flex items-center justify-center p-6 text-center select-none">
          <div class="max-w-xs w-full bg-theme-surface border border-theme-cyan/50 p-6 rounded-2xl shadow-2xl flex flex-col items-center gap-4">
            <div class="w-16 h-16 rounded-full bg-theme-cyan/20 border border-theme-cyan/40 flex items-center justify-center text-theme-cyan shadow-[0_0_20px_var(--cyan)]">
              <Volume2 size={32} />
            </div>

            <div class="space-y-1">
              <h2 class="text-base font-heading font-bold text-theme-text">
                Hubungkan Audio HP
              </h2>
              <p class="text-xs text-theme-text/60 leading-relaxed">
                Browser smartphone memerlukan 1x izin sentuhan agar suara asisten & musik otomatis dialihkan ke speaker smartphone ini.
              </p>
            </div>

            <button
              type="button"
              onClick={() => unlockAudioContext(false)}
              class="w-full py-3 px-4 bg-theme-cyan text-theme-card font-bold text-xs uppercase tracking-wider rounded-xl shadow-[0_0_15px_var(--cyan)] active:scale-95 transition-transform"
            >
              Aktifkan Audio HP
            </button>
          </div>
        </div>
      </Show>

      {/* 2. Top Bar Mobile */}
      <header class="h-12 px-3 bg-theme-surface border-b border-theme-border flex items-center justify-between shrink-0 shadow-sm select-none z-10">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse" />
          <h1 class="text-xs sm:text-sm font-heading font-bold text-theme-text tracking-wide">
            Ragasukma Remote
          </h1>
        </div>

        {/* Audio Mode Switcher: Remote Desktop vs Full Mobile */}
        <div class="flex items-center bg-theme-card/90 p-0.5 rounded-lg border border-theme-border text-[10px] font-mono">
          <button
            type="button"
            onClick={() => switchAudioMode('remote_desktop')}
            class={`flex items-center gap-1 px-2.5 py-1 rounded-md transition-all ${
              audioMode() === 'remote_desktop'
                ? 'bg-theme-cyan text-theme-card font-bold shadow-sm'
                : 'text-theme-text/60 hover:text-theme-text'
            }`}
            title="Mode Remote Desktop: Suara TTS & Musik di Desktop, Smartphone hening, Mic Desktop mute"
          >
            <Monitor size={12} />
            <span>Remote PC</span>
          </button>
          <button
            type="button"
            onClick={() => switchAudioMode('full_mobile')}
            class={`flex items-center gap-1 px-2.5 py-1 rounded-md transition-all ${
              audioMode() === 'full_mobile'
                ? 'bg-emerald-500 text-white font-bold shadow-sm'
                : 'text-theme-text/60 hover:text-theme-text'
            }`}
            title="Mode Full Mobile: Suara TTS & Musik di HP, Desktop senyap/mute, Mic Desktop mute"
          >
            <Smartphone size={12} />
            <span>Full HP</span>
          </button>
        </div>
      </header>

      {/* 3. Avatar Karakter Area (3D VRM GPU dengan Pinch-to-Zoom & Orbit di HP) */}
      <div class="p-3 bg-theme-surface/40 shrink-0">
        <AvatarViewer 
          state={visualState()} 
          emotion={emotion()} 
          isAudioStreaming={isHpAudioActive()} 
        />
      </div>

      {/* 3b. Now Playing Music Player Mini Card (Hanya saat Full HP) */}
      <Show when={isHpAudioActive() && mobileMusicState().track}>
        <div class="px-3 py-2 bg-theme-card/90 border-y border-theme-border flex items-center justify-between gap-3 shrink-0">
          <div class="flex items-center gap-2.5 min-w-0">
            <div class="w-8 h-8 rounded-lg bg-theme-cyan/20 border border-theme-cyan/30 flex items-center justify-center shrink-0">
              <Show when={mobileMusicState().isPlaying} fallback={<Music size={14} class="text-theme-cyan" />}>
                <div class="flex items-end justify-center gap-0.5 h-3">
                  <span class="w-1 bg-theme-cyan animate-[pulse_0.6s_ease-in-out_infinite] h-2" />
                  <span class="w-1 bg-theme-rose animate-[pulse_0.9s_ease-in-out_infinite] h-3" />
                  <span class="w-1 bg-theme-cyan animate-[pulse_0.7s_ease-in-out_infinite] h-2" />
                </div>
              </Show>
            </div>
            <div class="min-w-0">
              <div class="flex items-center gap-1.5">
                <span class="text-[9px] font-mono font-bold px-1.5 py-0.5 bg-theme-cyan/20 text-theme-cyan rounded">
                  {mobileMusicState().isPlaying ? 'PLAYING' : 'PAUSED'}
                </span>
                <p class="text-xs font-bold text-theme-text truncate">
                  {mobileMusicState().track?.title}
                </p>
              </div>
              <p class="text-[10px] text-theme-text/50 truncate">
                {mobileMusicState().track?.uploader || 'Local Library'}
              </p>
            </div>
          </div>

          <div class="flex items-center gap-1.5 shrink-0">
            <button
              onClick={() => {
                if (mobileMusicState().isPlaying) {
                  audioSync.pauseMusic();
                } else {
                  audioSync.resumeMusic();
                }
              }}
              class="p-2 rounded-lg bg-theme-surface hover:bg-theme-border border border-theme-border text-theme-cyan transition-all active:scale-95"
              title={mobileMusicState().isPlaying ? 'Jeda' : 'Lanjutkan'}
            >
              <Show when={mobileMusicState().isPlaying} fallback={<Play size={13} class="fill-current" />}>
                <Pause size={13} />
              </Show>
            </button>
            <button
              onClick={() => audioSync.stopMusic()}
              class="p-2 rounded-lg bg-theme-surface hover:bg-theme-rose/20 border border-theme-border text-theme-text/60 hover:text-theme-rose transition-all active:scale-95"
              title="Stop"
            >
              <Square size={13} />
            </button>
          </div>
        </div>
      </Show>

      {/* 4. Text & Voice Input Section */}
      <div class="p-3 bg-gradient-to-b from-theme-surface to-theme-bg border-y border-theme-border flex flex-col shrink-0 space-y-2.5">
        <Show when={lastSentText()}>
          <div class="w-full px-3 py-1.5 bg-theme-card border border-theme-border rounded-lg flex items-center gap-2 shadow-inner">
            <CheckCircle2 size={13} class="text-theme-cyan shrink-0" />
            <div class="min-w-0 flex-1">
              <span class="text-[9px] text-theme-text/50 font-mono block">Perintah Terakhir:</span>
              <p class="text-xs font-semibold text-theme-cyan truncate">
                "{lastSentText()}"
              </p>
            </div>
          </div>
        </Show>

        {/* Input Box & Action Buttons */}
        <div class="relative w-full">
          <textarea
            value={inputText()}
            onInput={(e) => setInputText(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ketik perintah atau gunakan suara..."
            rows={2}
            disabled={isSending()}
            class="w-full p-2.5 pr-9 bg-theme-card border border-theme-border rounded-xl text-xs text-theme-text placeholder:text-theme-text/40 focus:outline-none focus:border-theme-cyan transition-colors resize-none shadow-inner leading-relaxed"
          />

          <Show when={inputText().length > 0}>
            <button
              type="button"
              onClick={() => setInputText('')}
              class="absolute top-2 right-2 p-1 text-theme-text/40 hover:text-theme-text transition-colors rounded-md"
              title="Hapus teks"
            >
              <Trash2 size={13} />
            </button>
          </Show>
        </div>

        <div class="flex items-center justify-between gap-2 select-none">
          {/* Tombol Bicara PTT Suara Langsung HP */}
          <button
            type="button"
            onClick={toggleHpMic}
            class={`px-3.5 py-2 rounded-xl text-xs font-bold flex items-center gap-1.5 transition-all border shadow-sm ${
              isHpMicListening()
                ? 'bg-rose-500 text-white border-rose-400 animate-pulse shadow-[0_0_12px_rgba(244,63,94,0.5)]'
                : 'bg-theme-card text-theme-text border-theme-border hover:border-theme-cyan/50'
            }`}
          >
            {isHpMicListening() ? <MicOff size={14} /> : <Mic size={14} class="text-theme-cyan" />}
            <span>{isHpMicListening() ? 'Mendengar...' : 'Bicara (Mic)'}</span>
          </button>

          {/* Tombol Kirim Teks */}
          <button
            type="button"
            onClick={() => sendTextToHost()}
            disabled={!inputText().trim() || isSending()}
            class="flex-1 py-2 px-4 bg-theme-cyan text-theme-card rounded-xl text-xs font-bold flex items-center justify-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed active:scale-95 transition-all shadow-[0_0_12px_var(--cyan)]"
          >
            <Send size={13} class={isSending() ? 'animate-pulse' : ''} />
            <span>{isSending() ? 'Mengirim...' : 'Kirim'}</span>
          </button>
        </div>
      </div>

      {/* 5. Live Console Logs Section */}
      <div class="flex-1 flex flex-col min-h-0 bg-theme-surface">
        <div class="h-8 px-3 bg-theme-surface border-b border-theme-border flex items-center justify-between shrink-0 select-none">
          <div class="flex items-center gap-2 text-xs font-mono font-bold text-theme-text/80">
            <Terminal size={12} class="text-theme-cyan" />
            <span class="text-[11px]">CONSOLE LOGS</span>
            <span class="text-[10px] text-theme-text/40 font-normal">({logs().length})</span>
          </div>

          <div class="flex items-center gap-1">
            <button
              onClick={() => {
                const text = logs().map(l => `[${l.timestamp}] ${l.message}`).join('\n');
                copyToClipboard(text);
              }}
              class="p-1 text-theme-text/50 hover:text-theme-text transition-colors"
              title="Salin Semua Log"
            >
              <Copy size={12} />
            </button>
            <button
              onClick={() => {
                setIsRefreshingLogs(true);
                fetchLogs();
                setTimeout(() => setIsRefreshingLogs(false), 300);
              }}
              class="p-1 text-theme-text/50 hover:text-theme-text transition-colors"
              title="Refresh Logs"
            >
              <RefreshCw size={12} class={isRefreshingLogs() ? 'animate-spin' : ''} />
            </button>
          </div>
        </div>

        {/* Scrollable Logs Body */}
        <div
          ref={logsContainerRef}
          class="flex-1 overflow-y-auto p-2.5 font-mono text-[10.5px] leading-relaxed space-y-1 bg-theme-card/50 text-theme-text select-text"
        >
          <Show
            when={logs().length > 0}
            fallback={
              <div class="h-full flex items-center justify-center text-xs text-theme-text/40">
                Belum ada aktivitas log...
              </div>
            }
          >
            <For each={logs()}>
              {(log) => (
                <div class="break-words border-b border-theme-border/20 pb-1">
                  <span class="text-[9.5px] text-theme-text/40 mr-1.5 font-mono">
                    {log.timestamp ? log.timestamp.split(' ')[1] || log.timestamp : ''}
                  </span>
                  <span class={formatMessageStyle(log.message, log.level)}>
                    {log.message}
                  </span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </div>
  );
};
