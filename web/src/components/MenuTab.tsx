import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { 
  Play, 
  Square, 
  Volume2, 
  VolumeX, 
  Mic, 
  MicOff, 
  RefreshCw, 
  RotateCcw, 
  Bot, 
  Layers, 
  Headphones,
  Download,
  ExternalLink,
  X,
  Zap,
  ZapOff
} from 'lucide-solid';
import { api, AppStatePayload } from '../api';
import { AiCapabilitiesCard } from './AiCapabilitiesCard';
import { BrowserExtensionModal } from './BrowserExtensionModal';
import { SystemAppsModal } from './SystemAppsModal';

interface MenuTabProps {
  currentState: string;
  setCurrentState: (s: string) => void;
  showToast: (msg: string) => void;
  onNavigateTab?: (tab: string) => void;
}

export const MenuTab: Component<MenuTabProps> = (props) => {
  const [isRunning, setIsRunning] = createSignal(false);
  const [isPaused, setIsPaused] = createSignal(false);
  const [voiceMuted, setVoiceMuted] = createSignal(false);
  const [micMuted, setMicMuted] = createSignal(false);
  const [voiceInterruption, setVoiceInterruption] = createSignal(true);
  const [isUpdating, setIsUpdating] = createSignal(false);
  const [assistantName, setAssistantName] = createSignal('');
  const [ttsEngineName, setTtsEngineName] = createSignal('');
  const [micLevel, setMicLevel] = createSignal(0);
  const [micBars, setMicBars] = createSignal<number[]>(new Array(24).fill(0));
  const [micActive, setMicActive] = createSignal(true);
  const [micSensitivity, setMicSensitivity] = createSignal('');
  const [noiseReductionLevel, setNoiseReductionLevel] = createSignal(0);
  const [updateModalOpen, setUpdateModalOpen] = createSignal(false);
  const [extensionModalOpen, setExtensionModalOpen] = createSignal(false);
  const [appsModalOpen, setAppsModalOpen] = createSignal(false);
  const [updateInfo, setUpdateInfo] = createSignal<{
    hasUpdate: boolean;
    currentVersion: string;
    latestVersion: string;
    releaseName: string;
    releaseNotes: string;
    releaseUrl: string;
  } | null>(null);

  const vadThresholdPercent = () => {
    switch (micSensitivity().trim()) {
      case '0': return 45; // Ekstra Kebal (Ruangan Ramai)
      case '1': return 32; // Sangat Rendah
      case '2': return 22; // Rendah
      case '3': return 15; // Normal (Standar)
      case '4': return 9;  // Tinggi (Responsif)
      case '5': return 5;  // Sangat Tinggi
      case '6': return 3;  // Ultra Sensitif (Bisikan)
      default: return 15;
    }
  };

  const sensitivityLabel = () => {
    switch (micSensitivity().trim()) {
      case '0': return 'Level 0 (Ekstra Kebal)';
      case '1': return 'Level 1 (Sangat Rendah)';
      case '2': return 'Level 2 (Rendah)';
      case '3': return 'Level 3 (Normal)';
      case '4': return 'Level 4 (Tinggi)';
      case '5': return 'Level 5 (Sangat Tinggi)';
      case '6': return 'Level 6 (Ultra Sensitif)';
      default: return `Level ${micSensitivity()}`;
    }
  };

  onMount(async () => {
    const cfg = await api.getConfig();
    if (cfg) {
      setAssistantName(cfg.assistant_name);
      setTtsEngineName(
        cfg.tts_engine ? `${cfg.tts_engine.toUpperCase()} (${cfg.tts_lang})` : ''
      );
      setMicSensitivity(cfg.mic_sensitivity);
      setNoiseReductionLevel(cfg.noise_reduction_level);
      setVoiceInterruption(Boolean(cfg.voice_interruption));
    }

    // Ambil initial state kontrol 1x via GET /api/state
    const initState = await api.getState();
    if (initState) {
      setIsRunning(initState.is_running);
      setIsPaused(initState.is_paused);
      setVoiceMuted(initState.voice_muted);
      setMicMuted(initState.mic_muted);
      if (initState.state) {
        props.setCurrentState(initState.state);
      }
    }

    const barsBuffer = new Array<number>(24).fill(0);
    let targetMicLevel = 0;
    let currentLerpLevel = 0;
    let rafId: number | null = null;

    // 1. Global AI State Stream
    const unsubscribeState = api.subscribeState((payload) => {
      if (payload?.state) {
        props.setCurrentState(payload.state);
      }
    });

    // 2. Realtime Mic Spectrum Level Stream (~16 FPS update dari backend)
    const unsubscribeMic = api.subscribeMicLevel((lvl) => {
      targetMicLevel = micMuted() ? 0 : lvl;
    });

    // 3. 60 FPS RequestAnimationFrame Loop (Lerp / Interpolasi Mulus)
    const renderLoop = () => {
      if (micMuted()) {
        targetMicLevel = 0;
        currentLerpLevel = 0;
        setMicLevel(0);
        barsBuffer.fill(0);
        setMicBars([...barsBuffer]);
      } else {
        // Attack / Decay Lerp: naik cepat (attack ~0.35), turun halus (decay ~0.18)
        const lerpFactor = targetMicLevel > currentLerpLevel ? 0.35 : 0.18;
        currentLerpLevel += (targetMicLevel - currentLerpLevel) * lerpFactor;
        if (Math.abs(targetMicLevel - currentLerpLevel) < 0.2) {
          currentLerpLevel = targetMicLevel;
        }

        const displayLevel = Math.round(currentLerpLevel);
        setMicLevel(displayLevel);

        // Dynamic 60 FPS organic spectrum bars
        const now = Date.now();
        for (let i = 0; i < 24; i++) {
          const bell = Math.sin((i / 23) * Math.PI);
          const noise = (Math.sin(i * 3.7 + now / 120) + 1) * 0.5;
          const barH = Math.min(100, Math.round(currentLerpLevel * (0.4 + 0.6 * bell * (0.7 + 0.3 * noise))));
          barsBuffer[i] = barH;
        }
        setMicBars([...barsBuffer]);
      }

      rafId = requestAnimationFrame(renderLoop);
    };

    rafId = requestAnimationFrame(renderLoop);
    setMicActive(true);

    onCleanup(() => {
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }
      try {
        unsubscribeState();
        unsubscribeMic();
      } catch {}
    });
  });

  const syncState = async (
    newState: string,
    running = isRunning(),
    paused = isPaused(),
    vMuted = voiceMuted(),
    mMuted = micMuted()
  ) => {
    props.setCurrentState(newState);
    setIsRunning(running);
    setIsPaused(paused);
    setVoiceMuted(vMuted);
    setMicMuted(mMuted);

    const payload: AppStatePayload = {
      state: newState,
      is_running: running,
      is_paused: paused,
      voice_muted: vMuted,
      mic_muted: mMuted,
    };
    await api.setState(payload);
  };

  const toggleAssistant = async () => {
    const nextRunning = !isRunning();
    setIsRunning(nextRunning);
    if (nextRunning) {
      await api.sendCommand('start_chat');
      props.showToast('Voice Assistant diaktifkan');
    } else {
      await api.sendCommand('stop_chat');
      props.showToast('Voice Assistant dihentikan (Standby)');
    }
  };

  const toggleVoice = async () => {
    const next = !voiceMuted();
    setVoiceMuted(next);
    await api.sendCommand(next ? 'mute_voice' : 'unmute_voice');
    props.showToast(next ? 'Suara TTS Dinonaktifkan' : 'Suara TTS Diaktifkan');
  };

  const toggleMic = async () => {
    const next = !micMuted();
    setMicMuted(next);
    await api.sendCommand(next ? 'mute_mic' : 'unmute_mic');
    props.showToast(next ? 'Mikrofon STT Dinonaktifkan' : 'Mikrofon STT Diaktifkan');
  };

  const toggleVoiceInterruption = async () => {
    const next = !voiceInterruption();
    setVoiceInterruption(next);
    const cfg = await api.getConfig();
    if (cfg) {
      cfg.voice_interruption = next;
      await api.saveConfig(cfg);
      props.showToast(next ? '⚡ Interupsi Suara (Barge-in) Aktif' : '🔇 Interupsi Suara Nonaktif');
    }
  };

  const CURRENT_APP_VERSION = 'v0.1.2';

  const handleCheckUpdate = async () => {
    setIsUpdating(true);
    try {
      const res = await fetch('https://api.github.com/repos/ahlanefwolf/ragasukma/releases/latest');
      if (!res.ok) {
        throw new Error('Gagal mengambil data rilis GitHub');
      }
      const data = await res.json();
      const latestTag = (data.tag_name || '').trim();
      const cleanLatest = latestTag.replace(/^v/, '');
      const cleanCurrent = CURRENT_APP_VERSION.replace(/^v/, '');

      const isNewer = cleanLatest !== cleanCurrent && cleanLatest > cleanCurrent;

      setUpdateInfo({
        hasUpdate: isNewer,
        currentVersion: CURRENT_APP_VERSION,
        latestVersion: latestTag || CURRENT_APP_VERSION,
        releaseName: data.name || latestTag,
        releaseNotes: data.body || 'Tidak ada catatan rilis.',
        releaseUrl: data.html_url || 'https://github.com/ahlanefwolf/ragasukma/releases',
      });
      setUpdateModalOpen(true);

      if (isNewer) {
        props.showToast(`🎉 Versi baru tersedia: ${latestTag}!`);
      } else {
        props.showToast(`✨ Ragasukma sudah versi terbaru (${CURRENT_APP_VERSION})`);
      }
    } catch (err: any) {
      // Fallback modal / toast jika offline atau belum ada rilis di repo
      setUpdateInfo({
        hasUpdate: false,
        currentVersion: CURRENT_APP_VERSION,
        latestVersion: CURRENT_APP_VERSION,
        releaseName: `Ragasukma ${CURRENT_APP_VERSION}`,
        releaseNotes: 'Aplikasi sudah menggunakan versi stabil terbaru.',
        releaseUrl: 'https://github.com/ahlanefwolf/ragasukma/releases',
      });
      setUpdateModalOpen(true);
      props.showToast(`✨ Ragasukma sudah versi terbaru (${CURRENT_APP_VERSION})`);
    } finally {
      setIsUpdating(false);
    }
  };

  const handleResetSession = async () => {
    await api.sendCommand('reset_session');
    props.showToast('Memory session & context AI berhasil di-reset');
  };

  return (
    <div class="space-y-5">
      {/* 1. Hero Status & Master Control Card */}
      <div class="bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-5 sm:p-6 border border-theme-border flex flex-col gap-4 shadow-sm">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div>
            <div class="flex items-center gap-2">
              <span class="px-2.5 py-0.5 text-xs font-bold font-mono bg-theme-rose/20 text-theme-rose border border-theme-rose/30 uppercase tracking-wider">
                {assistantName()}
              </span>
              <span class={`px-2.5 py-0.5 text-xs font-bold font-mono uppercase border ${
                isRunning() ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40' : 'bg-theme-text/10 text-theme-text/60 border-theme-border'
              }`}>
                ● {props.currentState}
              </span>
            </div>

            <h2 class="text-xl sm:text-2xl font-heading text-theme-text mt-2">
              Voice Assistant Overlay
            </h2>
          </div>

          {/* Quick Actions (Reset & Update) */}
          <div class="flex items-center gap-2">
            <button
              onClick={handleResetSession}
              title="Reset AI Session Memory"
              class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-2 transition-all"
            >
              <RotateCcw size={14} />
              <span class="hidden sm:inline">Reset AI</span>
            </button>

            <button
              onClick={handleCheckUpdate}
              disabled={isUpdating()}
              title="Cek Pembaruan Ragasukma"
              class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-2 transition-all"
            >
              <RefreshCw size={14} class={isUpdating() ? 'animate-spin text-theme-cyan' : ''} />
              <span class="hidden sm:inline">{isUpdating() ? 'Cek...' : 'Update'}</span>
            </button>
          </div>
        </div>

        {/* Master Action Button */}
        <button
          onClick={toggleAssistant}
          class={`w-full flex items-center justify-center gap-3 py-3.5 px-4 font-bold text-sm sm:text-base transition-all shadow-md active:scale-[0.99] border ${
            isRunning()
              ? 'bg-theme-card hover:bg-theme-rose/20 text-theme-rose border-theme-rose/50 shadow-rose-900/20'
              : 'bg-theme-cyan hover:bg-[#4a5a3a] text-theme-card border-theme-cyan anime-shadow-green'
          }`}
        >
          {isRunning() ? (
            <>
              <Square size={18} class="fill-current" />
              <span>Hentikan Voice Assistant</span>
            </>
          ) : (
            <>
              <Play size={18} class="fill-current" />
              <span>Mulai / Aktifkan Voice Assistant</span>
            </>
          )}
        </button>

        {/* Quick Toggles: TTS, STT & Voice Interruption */}
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 pt-1">
          <button
            onClick={toggleVoice}
            class={`flex items-center justify-center gap-2.5 py-2.5 px-3 border text-xs sm:text-sm font-semibold transition-all ${
              voiceMuted()
                ? 'bg-theme-bg border-theme-rose/60 text-theme-rose'
                : 'bg-theme-card border-theme-border hover:border-theme-cyan/50 text-theme-text'
            }`}
          >
            {voiceMuted() ? <VolumeX size={17} class="shrink-0" /> : <Volume2 size={17} class="text-theme-cyan shrink-0" />}
            <span class="truncate">{voiceMuted() ? 'TTS: Muted' : 'TTS: Aktif'}</span>
          </button>

          <button
            onClick={toggleMic}
            class={`flex items-center justify-center gap-2.5 py-2.5 px-3 border text-xs sm:text-sm font-semibold transition-all ${
              micMuted()
                ? 'bg-theme-bg border-theme-rose/60 text-theme-rose'
                : 'bg-theme-card border-theme-border hover:border-theme-cyan/50 text-theme-text'
            }`}
          >
            {micMuted() ? <MicOff size={17} class="shrink-0" /> : <Mic size={17} class="text-theme-cyan shrink-0" />}
            <span class="truncate">{micMuted() ? 'Mic: Muted' : 'Mic: Aktif'}</span>
          </button>

          <button
            onClick={toggleVoiceInterruption}
            title={voiceInterruption() ? 'Fitur Interupsi Aktif: Kamu bisa langsung memotong saat AI berbicara' : 'Fitur Interupsi Nonaktif: AI akan menyelesaikan seluruh responnya'}
            class={`flex items-center justify-center gap-2.5 py-2.5 px-3 border text-xs sm:text-sm font-semibold transition-all ${
              voiceInterruption()
                ? 'bg-theme-card border-theme-cyan/60 text-theme-cyan shadow-[0_0_10px_rgba(89,107,69,0.2)]'
                : 'bg-theme-bg border-theme-border text-theme-text/60 hover:text-theme-text'
            }`}
          >
            {voiceInterruption() ? <Zap size={17} class="shrink-0 fill-current" /> : <ZapOff size={17} class="shrink-0" />}
            <span class="truncate">{voiceInterruption() ? 'Interupsi: ON' : 'Interupsi: OFF'}</span>
          </button>
        </div>
      </div>

      {/* 2. Realtime Input Mikrofon (VU Meter) Card */}
      <div class="p-5 bg-theme-surface border border-theme-border shadow-sm space-y-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2.5">
            <Mic size={20} class="text-theme-cyan" />
            <h3 class="text-base sm:text-lg font-bold text-theme-text">Realtime Spectrum & VAD Input</h3>
            <span class="text-xs text-theme-text/60 font-mono hidden sm:inline">({sensitivityLabel()})</span>
          </div>

          <div class="flex items-center gap-2">
            <span class={`text-xs px-2.5 py-0.5 font-bold uppercase font-mono border ${
              micMuted()
                ? 'bg-theme-rose/20 text-theme-rose border-theme-rose/40'
                : micLevel() > vadThresholdPercent()
                ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40'
                : 'bg-theme-text/10 text-theme-text/50 border-theme-border'
            }`}>
              {micMuted() ? 'MUTED' : micLevel() > vadThresholdPercent() ? 'VOICE DETECTED' : 'STANDBY'}
            </span>
            <span class="text-sm font-mono font-bold text-theme-text min-w-[40px] text-right">
              {micMuted() ? '0%' : `${micLevel()}%`}
            </span>
          </div>
        </div>

        {/* Dynamic Equalizer Waveform Bars */}
        <div class="flex items-end justify-between gap-1.5 h-20 sm:h-24 px-3 py-2.5 bg-theme-bg border border-theme-border shadow-inner">
          {micBars().map((height) => (
            <div
              class="flex-1 transition-all duration-75"
              style={{
                height: micMuted() ? '4px' : `${Math.max(5, height)}%`,
                'background-color':
                  micMuted()
                    ? 'var(--border)'
                    : height > 75
                    ? '#B08A4A' /* Ochre untuk sinyal kuat/tinggi */
                    : height > 35
                    ? '#596B45' /* Moss untuk suara aktif */
                    : '#876447', /* Earth Brown untuk ambient/rendah */
                'box-shadow': !micMuted() && height > 35 ? '0 0 8px rgba(89, 107, 69, 0.4)' : 'none',
                opacity: micMuted() ? '0.3' : '0.95',
              }}
            />
          ))}
        </div>

        {/* Progress Bar with Peak & Dynamic VAD Marker */}
        <div class="space-y-2 pt-1">
          <div class="relative h-6 w-full bg-theme-bg overflow-hidden border border-theme-border p-0.5 shadow-inner">
            {/* Dynamic VAD Threshold Marker */}
            <div
              class="absolute top-0 bottom-0 w-1 bg-theme-text shadow-[0_0_8px_var(--text)] z-10"
              style={{ left: `${vadThresholdPercent()}%` }}
              title={`Batas Trigger VAD (${vadThresholdPercent()}%)`}
            />
            {/* Active Level Fill Bar */}
            <div
              class="h-full transition-all duration-75 flex items-center justify-end pr-2 text-xs font-mono font-bold text-theme-card"
              style={{
                width: micMuted() ? '0%' : `${micLevel()}%`,
                'background-color':
                  micLevel() > 80
                    ? '#B08A4A'
                    : micLevel() > vadThresholdPercent()
                    ? '#596B45'
                    : '#876447',
              }}
            >
              {micLevel() > 15 && !micMuted() && `${micLevel()}%`}
            </div>
          </div>

          <div class="flex items-center justify-between text-xs font-mono text-theme-text/70 px-0.5">
            <span class="font-bold">0%</span>
            <span class="px-2.5 py-0.5 bg-theme-card border border-theme-border font-bold text-theme-cyan">
              Batas Trigger VAD: {vadThresholdPercent()} %
            </span>
            <span class="font-bold">100%</span>
          </div>
        </div>
      </div>

      {/* 3. Kartu Kapabilitas & Panduan Perintah AI */}
      <AiCapabilitiesCard 
        onOpenExtensionModal={() => setExtensionModalOpen(true)}
        onOpenAppsModal={() => setAppsModalOpen(true)}
      />

      {/* Modal Dialog Pasang Ekstensi Browser */}
      <BrowserExtensionModal
        isOpen={extensionModalOpen()}
        onClose={() => setExtensionModalOpen(false)}
        showToast={props.showToast}
      />

      {/* Modal Dialog Daftar Aplikasi OS */}
      <SystemAppsModal
        isOpen={appsModalOpen()}
        onClose={() => setAppsModalOpen(false)}
        showToast={props.showToast}
      />

      {/* Modal Dialog Cek Pembaruan & Release Notes */}
      {updateModalOpen() && updateInfo() && (
        <div class="fixed inset-0 z-50 !mt-0 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-fade-in">
          <div class="bg-theme-surface border border-theme-border w-full max-w-lg shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
            {/* Header Modal */}
            <div class="flex items-center justify-between p-4 border-b border-theme-border bg-theme-bg">
              <div class="flex items-center gap-3">
                <span class={`p-2 border ${updateInfo()!.hasUpdate ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40' : 'bg-theme-border text-theme-text/70 border-theme-border'}`}>
                  {updateInfo()!.hasUpdate ? <Download size={20} /> : <RefreshCw size={20} />}
                </span>
                <div>
                  <h3 class="text-lg font-bold text-theme-text tracking-wide m-0 leading-tight">
                    {updateInfo()!.hasUpdate ? '🎉 Pembaruan Tersedia!' : '✨ Sistem Up-to-Date'}
                  </h3>
                  <div class="flex items-center gap-2 text-xs text-theme-text/70 mt-0.5">
                    <span>Versi saat ini: <strong class="text-theme-text">{updateInfo()!.currentVersion}</strong></span>
                    <span>•</span>
                    <span>Terbaru: <strong class="text-theme-cyan">{updateInfo()!.latestVersion}</strong></span>
                  </div>
                </div>
              </div>
              <button
                onClick={() => setUpdateModalOpen(false)}
                class="p-1.5 text-theme-text/60 hover:text-theme-text hover:bg-theme-border border border-theme-border transition-all"
              >
                <X size={18} />
              </button>
            </div>

            {/* Isi Release Notes / Changelog */}
            <div class="p-5 overflow-y-auto space-y-4 flex-1 text-sm text-theme-text/90">
              <div class="p-3.5 bg-theme-bg border border-theme-border">
                <h4 class="text-xs font-mono uppercase tracking-wider text-theme-cyan mb-1 font-bold">Nama Rilis</h4>
                <p class="font-bold text-base text-theme-text">{updateInfo()!.releaseName}</p>
              </div>

              <div>
                <h4 class="text-xs font-mono uppercase tracking-wider text-theme-text/70 mb-2 font-bold">Catatan Pembaruan & Fitur Baru</h4>
                <div class="p-3.5 bg-theme-bg border border-theme-border text-xs font-mono text-theme-text/90 whitespace-pre-wrap leading-relaxed max-h-56 overflow-y-auto custom-scrollbar">
                  {updateInfo()!.releaseNotes}
                </div>
              </div>
            </div>

            {/* Footer Aksi */}
            <div class="p-4 border-t border-theme-border bg-theme-bg flex items-center justify-end gap-3">
              <button
                onClick={() => setUpdateModalOpen(false)}
                class="px-4 py-2.5 text-xs font-semibold bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border transition-all"
              >
                Tutup
              </button>

              <a
                href={updateInfo()!.releaseUrl}
                target="_blank"
                rel="noopener noreferrer"
                class={`flex items-center gap-2 px-4 py-2.5 text-xs font-bold transition-all shadow-md border ${
                  updateInfo()!.hasUpdate
                    ? 'bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card border-theme-cyan anime-shadow-green'
                    : 'bg-theme-card hover:bg-theme-border text-theme-cyan border-theme-cyan/40'
                }`}
              >
                {updateInfo()!.hasUpdate ? <Download size={15} /> : <ExternalLink size={15} />}
                <span>{updateInfo()!.hasUpdate ? 'Unduh Rilis Baru di GitHub' : 'Buka GitHub Releases'}</span>
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
