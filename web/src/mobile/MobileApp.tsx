import { Component, createSignal, onMount, onCleanup, createEffect } from 'solid-js';
import { api, LogEntry } from '../api';
import { API_BASE } from '../services/base';
import { getChromeSttCode } from '../services/configService';
import { AvatarViewer } from './AvatarViewer';
import { globalAudioSync, MobileTrackState } from './AudioSyncManager';
import { MobileAudioMode } from './components/MobileHeader';
import { MobileTopBar } from './components/MobileTopBar';
import { MobileSettingsSheet } from './components/MobileSettingsSheet';
import { MobileFloatingMusic } from './components/MobileFloatingMusic';
import { MobileInteractionDock } from './components/MobileInteractionDock';
import { MobileLogsDrawer } from './components/MobileLogsDrawer';
import { MobileAudioUnlockModal } from './components/MobileAudioUnlockModal';

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
  let speechRec: any = null;
  const audioSync = globalAudioSync;
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

  const notifyRemoteActive = () => {
    fetch(`${API_BASE}/api/remote/active`, { method: 'POST' }).catch(() => {});
  };

  let activeVoiceAudio: HTMLAudioElement | null = null;
  let isVoiceSpeakingOnMobile = false;
  let mobileSpeakSeq = 0;

  const speakOnMobile = (text: string, em: string, audioUrl?: string | null) => {
    if (!isHpAudioActive()) return;
    const mySeq = ++mobileSpeakSeq;

    // Hentikan suara TTS sebelumnya jika ada (jangan matikan musik!)
    if (activeVoiceAudio) {
      try {
        activeVoiceAudio.pause();
        activeVoiceAudio.currentTime = 0;
      } catch {}
      activeVoiceAudio = null;
    }

    // Kunci status speaking segera agar avatar di HP langsung merespon & tidak prematur idle
    isVoiceSpeakingOnMobile = true;
    setVisualState('speaking');
    setEmotion(em);

    // Audio Ducking: turunkan volume musik agar suara AI terdengar jelas
    audioSync.duckMusic(0.25);

    // Beritahu backend bahwa mode speaking aktif
    fetch(`${API_BASE}/api/tts/state`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ speaking: true, emotion: em }),
    }).catch(() => {});

    const playAudio = (url: string) => {
      const fullUrl = url.startsWith('http') || url.startsWith('blob:') ? url : `${API_BASE}${url}`;
      isVoiceSpeakingOnMobile = true;
      activeVoiceAudio = true as any;

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

      const finishSpeaking = () => {
        if (mySeq !== mobileSpeakSeq) return;
        activeVoiceAudio = null;
        isVoiceSpeakingOnMobile = false;
        audioSync.unduckMusic();
        setVisualState('idle');
        setEmotion('neutral');
        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
        }).catch(() => {});
      };

      audioSync.playVoice(fullUrl, {
        onPlaying: notifySpeaking,
        onEnded: finishSpeaking,
        onError: (err) => {
          console.warn('[Mobile Supertonic Audio] Playback error/blocked:', err);
          finishSpeaking();
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

  // Handler Terpusat Semua Aksi (SSE + Polling Fallback) dengan Deduplikasi ID
  const handleIncomingAction = (act: any) => {
    if (!act || !act.action) return;

    if (act.id !== undefined) {
      if (act.id <= lastActionId) return;
      lastActionId = act.id;
    }

    // 1. Suara respon Supertonic-3 Neural Audio (WAV stream)
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

    // 2. Musik streaming dialirkan ke HP atau dikontrol di PC
    if (act.action === 'music:play') {
      const trackTitle = typeof act.payload === 'string' ? act.payload : 'Lagu Desktop';
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
        // Segera rilis STT mic agar Android keluar dari Voice Telephony mode kembali ke Media Hi-Fi
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
    try {
      localStorage.setItem('ragasukma_audio_mode', mode);
    } catch {}

    if (mode === 'remote_desktop') {
      audioSync.stopAll();
      try {
        await api.sendCommand('mode_remote_desktop');
      } catch {}
      props.showToast('🖥️ Beralih ke Mode Remote Desktop... memuat ulang');
    } else {
      try {
        await api.sendCommand('mode_full_mobile');
      } catch {}
      props.showToast('📱 Beralih ke Mode Full Mobile... memuat ulang');
    }

    // Refresh halaman agar AudioContext, SSE, dan state audio ter-reinitialisasi secara bersih
    setTimeout(() => {
      window.location.reload();
    }, 250);
  };

  const handleDisconnectRemote = async () => {
    try {
      audioSync.stopAll();
      localStorage.setItem('ragasukma_audio_mode', 'remote_desktop');
      setAudioMode('remote_desktop');
      await api.sendCommand('mode_reset_desktop');
      props.showToast('🖥️ Sesi remote diputuskan, audio & mic dikembalikan penuh ke desktop.');
      setShowSettings(false);
      setTimeout(() => {
        window.location.hash = '';
        window.location.reload();
      }, 400);
    } catch {
      props.showToast('Gagal memutuskan remote.');
    }
  };

  onMount(() => {
    audioSync.setOnTrackStateChange((st) => {
      setMobileMusicState(st);
    });

    notifyRemoteActive();
    // Sinkronkan mode aktif ke backend (jika Full HP, kirim mode_full_mobile; jika Remote PC, kirim mode_remote_desktop)
    const currentMode = audioMode();
    if (currentMode === 'full_mobile') {
      api.sendCommand('mode_full_mobile').catch(() => {});
    } else {
      api.sendCommand('mode_remote_desktop').catch(() => {});
    }

    // Sinkronkan bahasa STT dari config server secara dinamis
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

    // Pantau Virtual Keyboard di HP via VisualViewport agar layout tidak tertutup keyboard
    const updateViewport = () => {
      if (window.visualViewport) {
        setViewportHeight(`${window.visualViewport.height}px`);
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

    // Auto-unlock audio HP pada sentuhan/interaksi pertama pengguna
    const handleFirstInteraction = () => {
      if (!hasUnlockedAudio()) {
        unlockAudioContext(true);
      }
    };
    window.addEventListener('touchstart', handleFirstInteraction, { once: true, passive: true });
    window.addEventListener('click', handleFirstInteraction, { once: true });

    // Polling Terpadu Super Cepat & Ringan (Murni Polling HTTP GET - 100% Kebal Blokir SSE)
    let pollTick = 0;
    const pollInterval = setInterval(async () => {
      if (document.hidden) return;
      pollTick++;

      // 1. Actions polling (assistant:speak & music:play) - Setiap 750ms
      try {
        const res = await api.getActionsSince(lastActionId);
        if (res && res.actions && Array.isArray(res.actions)) {
          if (lastActionId === 0 && res.last_id) {
            lastActionId = res.last_id;
          } else {
            for (const act of res.actions) {
              handleIncomingAction(act);
            }
            if (res.last_id) {
              lastActionId = Math.max(lastActionId, res.last_id);
            }
          }
        }
      } catch {}

      // 2. State polling (idle, speaking, listening, thinking, emotion) - Setiap 750ms
      try {
        const stRes = await fetch(`${API_BASE}/api/state`);
        if (stRes.ok) {
          const st = await stRes.json();
          if (st && st.state) {
            const s = st.state.toLowerCase();
            // JIKA HP sedang memutar suara TTS (activeVoiceAudio != null || isVoiceSpeakingOnMobile),
            // KUNCI status speaking! Jangan biarkan backend menimpa menjadi idle secara prematur sampai audio onended.
            if (activeVoiceAudio !== null || isVoiceSpeakingOnMobile) {
              setVisualState('speaking');
            } else if (isHpMicListening()) {
              setVisualState('listening');
            } else {
              if (['idle', 'listening', 'speaking', 'thinking'].includes(s)) {
                setVisualState(s as any);
              }
              if (st.emotion) {
                setEmotion(st.emotion);
              } else if (s === 'idle') {
                setEmotion('neutral');
              }
            }
          }
        }
      } catch {}

      // 3. Logs polling (jika drawer log dibuka, poll setiap ~2.2s; jika tertutup poll setiap ~6s)
      if (showLogsDrawer()) {
        if (pollTick % 3 === 0) fetchLogs();
      } else {
        if (pollTick % 8 === 0) fetchLogs();
      }

      // 4. Remote active heartbeat ke host setiap ~10 detik
      if (pollTick % 13 === 0) {
        notifyRemoteActive();
      }
    }, 750);

    // Helper pemulihan sesi: kembalikan mic & voice penuh ke desktop saat HP ditutup
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
    window.addEventListener('pagehide', restoreDesktopSession);

    onCleanup(() => {
      document.title = 'Ragasukma - Konfigurasi & Dashboard AI';
      window.removeEventListener('beforeunload', restoreDesktopSession);
      window.removeEventListener('pagehide', restoreDesktopSession);
      window.removeEventListener('touchstart', handleFirstInteraction);
      window.removeEventListener('click', handleFirstInteraction);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener('resize', updateViewport);
        window.visualViewport.removeEventListener('scroll', updateViewport);
      } else {
        window.removeEventListener('resize', updateViewport);
      }
      clearInterval(pollInterval);
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
      restoreDesktopSession();
    });
  });

  createEffect(() => {
    logs();
    if (logsContainerRef) {
      logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
    }
  });

  // Sinkronisasi title tab browser di mobile (Default: "Ragasukma", Saat musik: "▶ Judul - Ragasukma")
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

    // Pastikan mic STT di HP telah terlepas agar speaker HP tidak terjebak di filter telepon/radio
    if (speechRec && isHpMicListening()) {
      try {
        speechRec.abort();
      } catch {}
      setIsHpMicListening(false);
    }

    if (activeVoiceAudio) {
      try {
        activeVoiceAudio = null;
      } catch {}
    }
    // Hentikan suara asisten jika sedang bicara (interupsi user), jangan matikan musik!
    audioSync.stopVoice();

    setIsSending(true);
    try {
      const res = await api.sendRemoteText(textToSend, props.authKey);
      if (res.success) {
        setLastSentText(textToSend);
        setInputText('');
        setVisualState('thinking');
        props.showToast(`✅ Terkirim: "${textToSend}"`);
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
    <div
      class="relative w-full bg-theme-bg select-text max-w-lg mx-auto shadow-2xl overflow-hidden flex flex-col justify-between"
      style={{ height: viewportHeight() }}
    >
      {/* 1. Modal Unlock Audio HP (1x Interaksi) */}
      <MobileAudioUnlockModal
        show={isHpAudioActive() && !hasUnlockedAudio()}
        onUnlock={() => unlockAudioContext(false)}
      />

      {/* 2. Floating Top Controls (Ultra Clean: Floating Status Pill Kiri & Tombol Settings Kanan) */}
      <MobileTopBar
        visualState={visualState()}
        onOpenSettings={() => setShowSettings(true)}
        logsCount={logs().length}
      />

      {/* Sheet Pengaturan Companion */}
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

      {/* 3. Karakter 3D VRM 100% Fullscreen Penuh Tanpa Terpotong Padding */}
      <div class="absolute inset-0 z-0 w-full h-full">
        <AvatarViewer
          state={visualState()}
          emotion={emotion()}
          isAudioStreaming={isHpAudioActive()}
        />
      </div>

      {/* Spacer fleksibel tengah agar kontrol melayang di bawah */}
      <div class="flex-1 min-h-0 pointer-events-none" />

      {/* 4. Kontrol Bagian Bawah: Floating Music + Circular Interaction Dock */}
      <div class="relative z-20 w-full max-w-md mx-auto pointer-events-auto flex flex-col gap-1.5">
        {/* Floating Music Player Mini Card (Jika Ada Musik Diputar) */}
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

        {/* Floating Circular Interaction Dock (Tombol Lingkaran Mic PTT & Toggle Input Pesan) */}
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
    </div>
  );
};
