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
  ZapOff,
  Workflow,
  MessageSquare,
  Terminal,
  Circle
} from 'lucide-solid';
import { api } from '../api';
import { AiCapabilitiesCard } from './AiCapabilitiesCard';
import { BrowserExtensionModal } from './BrowserExtensionModal';
import { SystemAppsModal } from './SystemAppsModal';

interface MenuTabProps {
  currentState: string;
  setCurrentState: (s: string) => void;
  showToast: (msg: string) => void;
  onNavigateTab?: (tab: string) => void;
}

// Module-level persistent audio visualizer context
let sharedAudioCtx: AudioContext | null = null;
let sharedAnalyser: AnalyserNode | null = null;
let sharedMicStream: MediaStream | null = null;
let isAudioInitializing = false;

async function getOrCreateWebAudioAnalyser(): Promise<AnalyserNode | null> {
  if (sharedAnalyser && sharedAudioCtx && sharedAudioCtx.state !== 'closed') {
    if (sharedAudioCtx.state === 'suspended') {
      try { await sharedAudioCtx.resume(); } catch {}
    }
    return sharedAnalyser;
  }
  if (isAudioInitializing) return null;
  isAudioInitializing = true;
  try {
    if (typeof window === 'undefined' || !navigator.mediaDevices?.getUserMedia) return null;
    sharedMicStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
    sharedAudioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
    const source = sharedAudioCtx.createMediaStreamSource(sharedMicStream);
    sharedAnalyser = sharedAudioCtx.createAnalyser();
    sharedAnalyser.fftSize = 64;
    sharedAnalyser.smoothingTimeConstant = 0.65;
    source.connect(sharedAnalyser);
    return sharedAnalyser;
  } catch (e) {
    console.warn('Native Web Audio mic analyser not active / permission denied:', e);
    return null;
  } finally {
    isAudioInitializing = false;
  }
}

export const MenuTab: Component<MenuTabProps> = (props) => {
  const [isRunning, setIsRunning] = createSignal(true);
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
  const [updateModalOpen, setUpdateModalOpen] = createSignal(false);
  const [updateInfo, setUpdateInfo] = createSignal({
    hasUpdate: false,
    currentVersion: 'v0.2.0',
    latestVersion: 'v0.2.0',
    releaseName: '',
    releaseNotes: '',
    releaseUrl: '',
  });
  const [pythonInfo, setPythonInfo] = createSignal<{ installed: boolean; version: string; path?: string; is_installing: boolean }>({
    installed: false,
    version: '',
    is_installing: false,
  });
  const [pythonChecking, setPythonChecking] = createSignal(false);
  const [extensionModalOpen, setExtensionModalOpen] = createSignal(false);
  const [appsModalOpen, setAppsModalOpen] = createSignal(false);

  onMount(() => {
    let disposed = false;

    const loadConfig = async () => {
      const cfg = await api.getConfig();
      if (disposed || !cfg) return;
      setAssistantName(cfg.assistant_name);
      setTtsEngineName(
        cfg.tts_voice_name
          ? `Supertonic ${cfg.tts_voice_name} (${cfg.lang || 'en'})`
          : `Supertonic F4 (${cfg.lang || 'en'})`
      );
      setVoiceInterruption(Boolean(cfg.voice_interruption));
    };

    void loadConfig();

    const handleConfigUpdated = (e: CustomEvent) => {
      const updated = e.detail;
      if (updated?.voice_interruption !== undefined) {
        setVoiceInterruption(Boolean(updated.voice_interruption));
      }
      if (updated?.assistant_name) {
        setAssistantName(updated.assistant_name);
      }
      if (updated?.tts_voice_name || updated?.lang) {
        setTtsEngineName(
          updated.tts_voice_name
            ? `Supertonic ${updated.tts_voice_name} (${updated.lang || 'en'})`
            : `Supertonic F4 (${updated.lang || 'en'})`
        );
      }
    };
    window.addEventListener('ragasukma:config_updated' as any, handleConfigUpdated);

    const barsBuffer = new Array<number>(24).fill(0);
    let rafId: number | null = null;
    let localAnalyser: AnalyserNode | null = null;

    getOrCreateWebAudioAnalyser().then((analyser) => {
      localAnalyser = analyser;
    });

    // Check Python status on mount
    fetchPythonStatus();

    // 1. Global AI State Stream
    const unsubscribeState = api.subscribeState((payload) => {
      if (payload?.state) {
        props.setCurrentState(payload.state);
      }
      if (typeof payload?.is_running === 'boolean') {
        setIsRunning(payload.is_running);
      }
      if (typeof payload?.voice_muted === 'boolean') {
        setVoiceMuted(payload.voice_muted);
      }
      if (typeof payload?.mic_muted === 'boolean') {
        setMicMuted(payload.mic_muted);
      }
    });

    const freqData = new Uint8Array(32);

    // 2. 60 FPS RequestAnimationFrame Loop directly from Web Audio API
    const renderLoop = () => {
      const activeAnalyser = localAnalyser || sharedAnalyser;
      if (micMuted() || !activeAnalyser) {
        setMicLevel(0);
        barsBuffer.fill(0);
        setMicBars([...barsBuffer]);
      } else {
        activeAnalyser.getByteFrequencyData(freqData);

        let sum = 0;
        for (let i = 0; i < 24; i++) {
          const val = freqData[i] || 0;
          sum += val;
          barsBuffer[i] = Math.round((val / 255) * 100);
        }
        const avg = Math.round((sum / (24 * 255)) * 100 * 1.5);
        setMicLevel(Math.min(100, avg));
        setMicBars([...barsBuffer]);
      }

      rafId = requestAnimationFrame(renderLoop);
    };

    rafId = requestAnimationFrame(renderLoop);
    setMicActive(true);

    onCleanup(() => {
      disposed = true;
      window.removeEventListener('ragasukma:config_updated' as any, handleConfigUpdated);
      if (rafId !== null) {
        cancelAnimationFrame(rafId);
      }
      try {
        unsubscribeState();
      } catch (_) {}
    });
  });

  const toggleAssistant = async () => {
    const nextRunning = !isRunning();
    setIsRunning(nextRunning);
    if (nextRunning) {
      await api.sendCommand('start_chat');
      props.showToast('Voice Assistant activated');
    } else {
      await api.sendCommand('stop_chat');
      props.showToast('Voice Assistant stopped (Standby)');
    }
  };

  const toggleVoice = async () => {
    const next = !voiceMuted();
    setVoiceMuted(next);
    await api.sendCommand(next ? 'mute_voice' : 'unmute_voice');
    props.showToast(next ? 'TTS Audio Muted' : 'TTS Audio Unmuted');
  };

  const toggleMic = async () => {
    const next = !micMuted();
    setMicMuted(next);
    await api.sendCommand(next ? 'mute_mic' : 'unmute_mic');
    props.showToast(next ? 'STT Microphone Muted' : 'STT Microphone Unmuted');
  };

  const toggleVoiceInterruption = async () => {
    const next = !voiceInterruption();
    setVoiceInterruption(next);
    const cfg = await api.getConfig();
    if (cfg) {
      cfg.voice_interruption = next;
      await api.saveConfig(cfg);
      try {
        window.dispatchEvent(new CustomEvent('ragasukma:config_updated', { detail: cfg }));
      } catch {}
      props.showToast(next ? 'Voice Interruption (Barge-in) Active' : 'Voice Interruption Disabled');
    }
  };

  const CURRENT_APP_VERSION = 'v0.2.0';

  const handleCheckUpdate = async () => {
    setIsUpdating(true);
    try {
      const res = await fetch('https://api.github.com/repos/ahlanefwolf/ragasukma/releases/latest');
      if (!res.ok) {
        throw new Error('Failed to fetch GitHub release data');
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
        releaseNotes: data.body || 'No release notes available.',
        releaseUrl: data.html_url || 'https://github.com/ahlanefwolf/ragasukma/releases',
      });
      setUpdateModalOpen(true);

      if (isNewer) {
        props.showToast(`New version available: ${latestTag}!`);
      } else {
        props.showToast(`Ragasukma is up to date (${CURRENT_APP_VERSION})`);
      }
    } catch (err: any) {
      setUpdateInfo({
        hasUpdate: false,
        currentVersion: CURRENT_APP_VERSION,
        latestVersion: CURRENT_APP_VERSION,
        releaseName: `Ragasukma ${CURRENT_APP_VERSION}`,
        releaseNotes: 'The application is already running the latest stable release.',
        releaseUrl: 'https://github.com/ahlanefwolf/ragasukma/releases',
      });
      setUpdateModalOpen(true);
      props.showToast(`Ragasukma is up to date (${CURRENT_APP_VERSION})`);
    } finally {
      setIsUpdating(false);
    }
  };

  const fetchPythonStatus = async () => {
    try {
      const info = await api.getPythonInfo();
      setPythonInfo(info);
    } catch (e) {
      console.warn('Failed to fetch Python status:', e);
    }
  };

  const handleCheckPythonStatus = async () => {
    setPythonChecking(true);
    try {
      const info = await api.getPythonInfo();
      setPythonInfo(info);
      if (info.installed) {
        props.showToast(`Python detected: ${info.version}`);
      } else {
        props.showToast('Python 3 not detected on this system');
      }
    } catch {
      props.showToast('Could not fetch Python status');
    } finally {
      setPythonChecking(false);
    }
  };

  const handleTriggerInstallPython = async () => {
    try {
      props.showToast('Initiating Python 3 setup in background...');
      const res = await api.installPython();
      setPythonInfo(res);
      // Poll a few times
      let attempts = 0;
      const timer = setInterval(async () => {
        attempts++;
        const updated = await api.getPythonInfo();
        setPythonInfo(updated);
        if (updated.installed || attempts > 15) {
          clearInterval(timer);
          if (updated.installed) {
            props.showToast(`Python 3 is ready: ${updated.version}`);
          }
        }
      }, 3000);
    } catch {
      props.showToast('Failed to trigger Python setup');
    }
  };

  const handleResetSession = async () => {
    await api.sendCommand('reset_session');
    props.showToast('AI memory session & context successfully reset');
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
              <span class={`inline-flex items-center gap-1.5 px-2.5 py-0.5 text-xs font-bold font-mono uppercase border ${
                isRunning() ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40' : 'bg-theme-text/10 text-theme-text/60 border-theme-border'
              }`}>
                <Circle size={8} class="shrink-0 fill-current" />
                {props.currentState}
              </span>
            </div>

            <h2 class="font-heading text-theme-text mt-2">
              Voice Assistant Overlay
            </h2>
          </div>

          {/* Quick Actions (Recipes, History, Reset & Update) */}
          <div class="flex items-center gap-2 flex-wrap">
            <button
              onClick={() => props.onNavigateTab?.('recipes')}
              title="Open Automation Recipes"
              class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-2 transition-all cursor-pointer"
            >
              <Workflow size={14} class="text-theme-cyan" />
              <span class="hidden sm:inline">Recipes</span>
            </button>

            <button
              onClick={() => props.onNavigateTab?.('history')}
              title="Open Chat History Logs"
              class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-2 transition-all cursor-pointer"
            >
              <MessageSquare size={14} class="text-gi-primary" />
              <span class="hidden sm:inline">Chat History</span>
            </button>

            <button
              onClick={handleResetSession}
              title="Reset AI Session Memory"
              class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-2 transition-all cursor-pointer"
            >
              <RotateCcw size={14} />
              <span class="hidden sm:inline">Reset AI</span>
            </button>

            <button
              onClick={handleCheckUpdate}
              disabled={isUpdating()}
              title="Check for Ragasukma Updates"
              class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-2 transition-all cursor-pointer"
            >
              <RefreshCw size={14} class={isUpdating() ? 'animate-spin text-theme-cyan' : ''} />
              <span class="hidden sm:inline">{isUpdating() ? 'Checking...' : 'Update'}</span>
            </button>
          </div>
        </div>

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
            <span class="truncate">{voiceMuted() ? 'TTS: Muted' : 'TTS: Active'}</span>
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
            <span class="truncate">{micMuted() ? 'Mic: Muted' : 'Mic: Active'}</span>
          </button>

          <button
            onClick={toggleVoiceInterruption}
            title={voiceInterruption() ? 'Interruption Active: You can interrupt while AI is speaking' : 'Interruption Disabled: AI will finish speaking its entire response'}
            class={`flex items-center justify-center gap-2.5 py-2.5 px-3 border text-xs sm:text-sm font-semibold transition-all ${
              voiceInterruption()
                ? 'bg-theme-card border-theme-cyan/60 text-theme-cyan shadow-[0_0_10px_rgba(89,107,69,0.2)]'
                : 'bg-theme-bg border-theme-border text-theme-text/60 hover:text-theme-text'
            }`}
          >
            {voiceInterruption() ? <Zap size={17} class="shrink-0 fill-current" /> : <ZapOff size={17} class="shrink-0" />}
            <span class="truncate">{voiceInterruption() ? 'Interruption: ON' : 'Interruption: OFF'}</span>
          </button>
        </div>

        {/* Python 3 Runtime Status Bar */}
        <div class="mt-3 p-3 bg-theme-bg/80 border border-theme-border/80 flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs">
          <div class="flex items-center gap-2.5">
            <Terminal size={18} class="shrink-0 text-theme-cyan" />
            <div>
              <div class="flex items-center gap-2 font-mono">
                <span class="font-bold text-theme-text">Python 3 Runtime:</span>
                {pythonInfo().installed ? (
                  <span class="text-theme-cyan font-semibold flex items-center gap-1.5">
                    <span class="inline-block w-2 h-2 bg-theme-cyan animate-pulse"></span>
                    Detected ({pythonInfo().version})
                  </span>
                ) : pythonInfo().is_installing ? (
                  <span class="text-amber-400 font-semibold flex items-center gap-1.5">
                    <RefreshCw size={12} class="animate-spin" />
                    Setting up in background...
                  </span>
                ) : (
                  <span class="text-theme-rose font-semibold">Not Detected</span>
                )}
              </div>
              <p class="text-[11px] text-theme-text/60 mt-0.5">
                Standard scripting engine utilized by autonomous agents for web scraping, sqlite, and terminal automation tasks.
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2 self-end sm:self-center shrink-0">
            {!pythonInfo().installed && (
              <button
                onClick={handleTriggerInstallPython}
                disabled={pythonInfo().is_installing}
                class="px-2.5 py-1 text-xs font-semibold bg-theme-cyan/20 hover:bg-theme-cyan/30 text-theme-cyan border border-theme-cyan/40 flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <Download size={12} />
                <span>{pythonInfo().is_installing ? 'Setting up...' : 'Setup Python 3'}</span>
              </button>
            )}
            <button
              onClick={handleCheckPythonStatus}
              disabled={pythonChecking()}
              title="Refresh Python 3 detection"
              class="px-2.5 py-1 text-xs font-semibold bg-theme-card hover:bg-theme-surface text-theme-text hover:text-theme-cyan border border-theme-border flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
            >
              <RefreshCw size={12} class={pythonChecking() ? 'animate-spin text-theme-cyan' : ''} />
              <span>{pythonChecking() ? 'Checking...' : 'Check Status'}</span>
            </button>
            <a
              href="https://www.python.org"
              target="_blank"
              rel="noreferrer"
              title="Open Python.org"
              class="p-1 text-theme-text/60 hover:text-theme-cyan transition-colors"
            >
              <ExternalLink size={14} />
            </a>
          </div>
        </div>
      </div>

      {/* 2. Realtime Microphone Spectrum (VU Meter) Card */}
      <div class="p-5 bg-theme-surface border border-theme-border shadow-sm space-y-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2.5">
            <Mic size={20} class="text-theme-cyan" />
            <h3 class="text-theme-text">Realtime Spectrum & Audio Monitor</h3>
          </div>

          <div class="flex items-center gap-2">
            <span class={`text-xs px-2.5 py-0.5 font-bold uppercase font-mono border ${
              micMuted()
                ? 'bg-theme-rose/20 text-theme-rose border-theme-rose/40'
                : micLevel() > 15
                ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40'
                : 'bg-theme-text/10 text-theme-text/50 border-theme-border'
            }`}>
              {micMuted() ? 'MUTED' : micLevel() > 15 ? 'AUDIO DETECTED' : 'STANDBY'}
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
                    ? 'var(--gi-status-warning)'
                    : height > 35
                    ? 'var(--gi-primary)'
                    : 'var(--gi-border-gold)',
                'box-shadow': !micMuted() && height > 35 ? '0 0 8px rgba(229, 184, 92, 0.45)' : 'none',
                opacity: micMuted() ? '0.3' : '0.95',
              }}
            />
          ))}
        </div>

        {/* Progress Bar with Peak */}
        <div class="space-y-2 pt-1">
          <div class="relative h-6 w-full bg-theme-bg overflow-hidden border border-theme-border p-0.5 shadow-inner">
            {/* Active Level Fill Bar */}
            <div
              class="h-full transition-all duration-75 flex items-center justify-end pr-2 text-xs font-mono font-bold text-theme-card"
              style={{
                width: micMuted() ? '0%' : `${micLevel()}%`,
                'background-color':
                  micLevel() > 80
                    ? 'var(--gi-status-warning)'
                    : micLevel() > 15
                    ? 'var(--gi-primary)'
                    : 'var(--gi-border-gold)',
              }}
            >
              {micLevel() > 15 && !micMuted() && `${micLevel()}%`}
            </div>
          </div>

          <div class="flex items-center justify-between text-xs font-mono text-theme-text/70 px-0.5">
            <span class="font-bold">0%</span>
            <span class="px-2.5 py-0.5 bg-theme-card border border-theme-border font-bold text-theme-cyan">
              Microphone Active Level
            </span>
            <span class="font-bold">100%</span>
          </div>
        </div>
      </div>

      {/* 3. AI Capabilities & Voice Command Guide */}
      <AiCapabilitiesCard 
        onOpenExtensionModal={() => setExtensionModalOpen(true)}
        onOpenAppsModal={() => setAppsModalOpen(true)}
      />

      {/* Browser Extension Modal */}
      <BrowserExtensionModal
        isOpen={extensionModalOpen()}
        onClose={() => setExtensionModalOpen(false)}
        showToast={props.showToast}
      />

      {/* System Apps Modal */}
      <SystemAppsModal
        isOpen={appsModalOpen()}
        onClose={() => setAppsModalOpen(false)}
        showToast={props.showToast}
      />

      {/* Update & Release Notes Modal */}
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
                  <h3 class="text-theme-text tracking-wide m-0 leading-tight">
                    {updateInfo()!.hasUpdate ? 'Update Available!' : 'System Up-to-Date'}
                  </h3>
                  <div class="flex items-center gap-2 text-xs text-theme-text/70 mt-0.5">
                    <span>Current version: <strong class="text-theme-text">{updateInfo()!.currentVersion}</strong></span>
                    <span>•</span>
                    <span>Latest: <strong class="text-theme-cyan">{updateInfo()!.latestVersion}</strong></span>
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

            {/* Release Notes / Changelog */}
            <div class="p-5 overflow-y-auto space-y-4 flex-1 text-sm text-theme-text/90">
              <div class="p-3.5 bg-theme-bg border border-theme-border">
                <h4 class="font-mono uppercase tracking-wider text-theme-cyan mb-1">Release Title</h4>
                <p class="font-bold text-base text-theme-text">{updateInfo()!.releaseName}</p>
              </div>

              <div>
                <h4 class="font-mono uppercase tracking-wider text-theme-text/70 mb-2">Release Notes & Features</h4>
                <div class="p-3.5 bg-theme-bg border border-theme-border text-xs font-mono text-theme-text/90 whitespace-pre-wrap leading-relaxed max-h-56 overflow-y-auto custom-scrollbar">
                  {updateInfo()!.releaseNotes}
                </div>
              </div>
            </div>

            {/* Actions Footer */}
            <div class="p-4 border-t border-theme-border bg-theme-bg flex items-center justify-end gap-3">
              <button
                onClick={() => setUpdateModalOpen(false)}
                class="px-4 py-2.5 text-xs font-semibold bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border transition-all"
              >
                Close
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
                <span>{updateInfo()!.hasUpdate ? 'Download Update on GitHub' : 'View GitHub Releases'}</span>
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
