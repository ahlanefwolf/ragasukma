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
  Sparkles, 
  Bot, 
  Layers, 
  Headphones,
  Download,
  ExternalLink,
  X
} from 'lucide-solid';
import { api, AppStatePayload } from '../api';

interface MenuTabProps {
  currentState: string;
  setCurrentState: (s: string) => void;
  showToast: (msg: string) => void;
}

export const MenuTab: Component<MenuTabProps> = (props) => {
  const [isRunning, setIsRunning] = createSignal(false);
  const [isPaused, setIsPaused] = createSignal(false);
  const [voiceMuted, setVoiceMuted] = createSignal(false);
  const [micMuted, setMicMuted] = createSignal(false);
  const [isUpdating, setIsUpdating] = createSignal(false);
  const [assistantName, setAssistantName] = createSignal('Laras');
  const [ttsEngineName, setTtsEngineName] = createSignal('Google TTS');
  const [micLevel, setMicLevel] = createSignal(0);
  const [micBars, setMicBars] = createSignal<number[]>(new Array(24).fill(0));
  const [micActive, setMicActive] = createSignal(false);
  const [isMonitoring, setIsMonitoring] = createSignal(false);
  const [micSensitivity, setMicSensitivity] = createSignal('3');
  const [noiseReductionLevel, setNoiseReductionLevel] = createSignal(0);
  const [updateModalOpen, setUpdateModalOpen] = createSignal(false);
  const [updateInfo, setUpdateInfo] = createSignal<{
    hasUpdate: boolean;
    currentVersion: string;
    latestVersion: string;
    releaseName: string;
    releaseNotes: string;
    releaseUrl: string;
  } | null>(null);
  let monitorGainRef: GainNode | null = null;
  let audioCtxRef: AudioContext | null = null;

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
      setAssistantName(cfg.assistant_name || 'Laras');
      setTtsEngineName(`${cfg.tts_engine.toUpperCase()} (${cfg.tts_lang})`);
      setMicSensitivity(cfg.mic_sensitivity || '3');
      setNoiseReductionLevel(cfg.noise_reduction_level || 0);
    }

    const unsubscribe = api.subscribeState((state) => {
      if (state) {
        props.setCurrentState(state.state);
        setIsRunning(state.is_running);
        setIsPaused(state.is_paused);
        setVoiceMuted(state.voice_muted);
        setMicMuted(state.mic_muted);
      }
    });

    // Real-time Mic Level Analyser via Web Audio API + Realtime DSP High-Pass Filter Sync
    let audioCtx: AudioContext | null = null;
    let analyser: AnalyserNode | null = null;
    let filter: BiquadFilterNode | null = null;
    let source: MediaStreamAudioSourceNode | null = null;
    let stream: MediaStream | null = null;
    let animId: number | null = null;

    const initMic = async () => {
      try {
        stream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
        audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
        analyser = audioCtx.createAnalyser();
        analyser.fftSize = 64;
        analyser.smoothingTimeConstant = 0.5;
        source = audioCtx.createMediaStreamSource(stream);

        // Terapkan DSP High-Pass Filter real-time jika noise_reduction_level aktif (120Hz - 150Hz)
        if (noiseReductionLevel() > 0) {
          filter = audioCtx.createBiquadFilter();
          filter.type = 'highpass';
          filter.frequency.value = noiseReductionLevel() === 1 ? 120 : 150;
          source.connect(filter);
          filter.connect(analyser);
        } else {
          source.connect(analyser);
        }

        // Setup Loopback Gain Node untuk fitur Review / Monitor Real-time Suara DSP
        const monitorGain = audioCtx.createGain();
        monitorGain.gain.value = 0.0; // Default Muted
        if (filter) {
          filter.connect(monitorGain);
        } else {
          source.connect(monitorGain);
        }
        monitorGain.connect(audioCtx.destination);
        monitorGainRef = monitorGain;
        audioCtxRef = audioCtx;

        setMicActive(true);

        const bufferLength = analyser.frequencyBinCount;
        const dataArray = new Uint8Array(bufferLength);
        const barsBuffer = new Array<number>(24).fill(0);
        let lastUpdateTime = 0;

        const updateLevel = (timestamp: number) => {
          if (!analyser) return;

          // Throttle visualizer UI update ke ~30 FPS dan skip saat browser tab di background
          if (!document.hidden && timestamp - lastUpdateTime >= 33) {
            lastUpdateTime = timestamp;

            if (micMuted()) {
              setMicLevel(0);
              barsBuffer.fill(0);
              setMicBars([...barsBuffer]);
            } else {
              analyser.getByteFrequencyData(dataArray);
              let sum = 0;
              const step = Math.floor(bufferLength / 24) || 1;
              for (let i = 0; i < 24; i++) {
                const val = dataArray[i * step] || 0;
                const norm = Math.min(100, Math.round((val / 255) * 100));
                barsBuffer[i] = norm;
                sum += val;
              }
              const avg = sum / (bufferLength || 1);
              const percent = Math.min(100, Math.round((avg / 120) * 100));
              setMicLevel(percent);
              setMicBars([...barsBuffer]);
            }
          }
          animId = requestAnimationFrame(updateLevel);
        };
        animId = requestAnimationFrame(updateLevel);
      } catch (e) {
        console.warn('Microphone access for visualizer:', e);
        setMicActive(false);
      }
    };

    initMic();

    onCleanup(() => {
      unsubscribe();
      if (animId) cancelAnimationFrame(animId);
      if (source) source.disconnect();
      if (audioCtx) audioCtx.close();
      if (stream) {
        stream.getTracks().forEach((t) => t.stop());
      }
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
    await syncState(props.currentState, nextRunning, false, voiceMuted(), micMuted());
    props.showToast(nextRunning ? 'Voice Assistant diaktifkan' : 'Voice Assistant dihentikan (Standby)');
  };

  const toggleMonitor = () => {
    const next = !isMonitoring();
    setIsMonitoring(next);
    if (monitorGainRef && audioCtxRef) {
      monitorGainRef.gain.setValueAtTime(next ? 1.0 : 0.0, audioCtxRef.currentTime);
    }
    props.showToast(
      next
        ? '🎧 Audio Monitor Aktif (Gunakan headphone untuk hindari feedback/gema)'
        : 'Audio Monitor Dimatikan'
    );
  };

  const toggleVoice = async () => {
    const next = !voiceMuted();
    await syncState(props.currentState, isRunning(), isPaused(), next, micMuted());
    props.showToast(next ? 'Suara TTS Dinonaktifkan' : 'Suara TTS Diaktifkan');
  };

  const toggleMic = async () => {
    const next = !micMuted();
    await syncState(props.currentState, isRunning(), isPaused(), voiceMuted(), next);
    props.showToast(next ? 'Mikrofon STT Dinonaktifkan' : 'Mikrofon STT Diaktifkan');
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

  const handleRestartEngine = async () => {
    await syncState('idle', false, false);
    props.showToast('Sinkronisasi & reload state engine 3D selesai');
  };

  return (
    <div class="space-y-6">
      {/* Top Banner Status Header */}
      <div class="rounded-2xl bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-4 sm:p-6 border border-theme-border flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <div class="flex flex-wrap items-center gap-2">
            <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-theme-rose/20 text-theme-rose border border-theme-rose/30">
              Menu Kontrol Utama
            </span>
            <span class="px-2 py-0.5 rounded-full text-[11px] font-bold font-mono bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 uppercase">
              State: {props.currentState}
            </span>
          </div>
          <h2 class="text-2xl sm:text-3xl font-carter tracking-wide text-theme-text mt-2">
            RAGASUKMA - {assistantName().toUpperCase()}
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-lg">
            Kendalikan status percakapan, mikrofon, pemutaran suara TTS, dan fungsi sinkronisasi engine 3D native.
          </p>
        </div>

        {/* State Indicator Pill */}
        <div class="p-3.5 sm:p-4 rounded-xl bg-theme-bg border border-theme-border flex items-center gap-3.5 sm:gap-4 shrink-0">
          <div class="w-11 h-11 sm:w-12 sm:h-12 rounded-xl bg-theme-card flex items-center justify-center text-theme-cyan">
            <Bot size={22} />
          </div>
          <div>
            <span class="text-[11px] text-theme-text/50 block">Voice Engine</span>
            <span class="text-sm font-bold text-theme-text">{ttsEngineName()}</span>
          </div>
        </div>
      </div>

      {/* 1. Kontrol Voice Assistant */}
      <div class="p-4 sm:p-6 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-cyan flex items-center gap-2">
          <Sparkles size={18} />
          1. KONTROL VOICE ASSISTANT
        </h3>

        <div>
          <button
            onClick={toggleAssistant}
            class={`w-full flex items-center justify-center gap-3 p-4 rounded-xl font-bold text-sm sm:text-base transition-all shadow-md ${
              isRunning()
                ? 'bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/50'
                : 'bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card anime-shadow-green'
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
        </div>
      </div>

      {/* 2. Realtime Mic Input Level (VU Meter) */}
      <div class="p-4 sm:p-6 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div>
            <h3 class="text-base sm:text-lg font-carter text-theme-cyan flex items-center gap-2">
              <Mic size={18} />
              2. REALTIME INPUT MIKROFON (VU METER)
            </h3>
            <div class="flex flex-wrap items-center gap-2 text-xs text-theme-text/60 mt-0.5 font-mono">
              <span>{sensitivityLabel()}</span>
              <span>•</span>
              <span>
                DSP Filter: {noiseReductionLevel() === 0 ? 'Nonaktif' : noiseReductionLevel() === 1 ? 'Ringan (120Hz)' : 'Agresif (150Hz)'}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <span class={`text-[11px] sm:text-xs px-2.5 py-0.5 rounded-full font-bold uppercase font-mono ${
              micMuted()
                ? 'bg-theme-rose/20 text-theme-rose'
                : micLevel() > vadThresholdPercent()
                ? 'bg-theme-cyan/20 text-theme-cyan'
                : 'bg-theme-text/10 text-theme-text/60'
            }`}>
              {micMuted() ? 'MUTED' : micLevel() > vadThresholdPercent() ? 'VOICE DETECTED' : 'QUIET'}
            </span>
            <span class="text-xs sm:text-sm font-mono font-bold text-theme-text">
              {micMuted() ? '0%' : `${micLevel()}%`}
            </span>
          </div>
        </div>

        {/* Dynamic Spectrum Equalizer Bars */}
        <div class="flex items-end justify-between gap-1 h-12 px-3 py-2 bg-theme-bg rounded-xl border border-theme-border">
          {micBars().map((height, idx) => (
            <div
              class="flex-1 rounded-t-sm transition-all duration-75"
              style={{
                height: micMuted() ? '3px' : `${Math.max(4, height)}%`,
                'background-color':
                  micMuted()
                    ? 'var(--border)'
                    : height > 75
                    ? 'var(--rose)'
                    : height > 35
                    ? 'var(--cyan)'
                    : '#4ECDC4',
                opacity: micMuted() ? '0.3' : '0.9',
              }}
            />
          ))}
        </div>

        {/* Progress Bar with Peak & Dynamic VAD Activation Threshold Marker */}
        <div class="space-y-1.5">
          <div class="relative h-4 w-full bg-theme-bg rounded-full overflow-hidden border border-theme-border p-0.5">
            {/* Dynamic VAD Threshold Line Marker based on config */}
            <div
              class="absolute top-0 bottom-0 w-0.5 bg-theme-text/60 shadow-[0_0_4px_var(--text)] z-10"
              style={{ left: `${vadThresholdPercent()}%` }}
              title={`Batas Trigger VAD (${vadThresholdPercent()}%)`}
            />
            {/* Active Level Fill Bar */}
            <div
              class="h-full rounded-full transition-all duration-75 shadow-sm"
              style={{
                width: micMuted() ? '0%' : `${micLevel()}%`,
                'background-color':
                  micLevel() > 80 ? 'var(--rose)' : micLevel() > vadThresholdPercent() ? 'var(--cyan)' : '#4ECDC4',
                'box-shadow': micLevel() > vadThresholdPercent() ? '0 0 10px rgba(123, 223, 242, 0.5)' : 'none',
              }}
            />
          </div>
          <div class="flex justify-between text-[10px] font-mono text-theme-text/40 px-1">
            <span>0%</span>
            <span class="text-theme-cyan/80">Batas Trigger VAD: {vadThresholdPercent()}%</span>
            <span>100%</span>
          </div>
        </div>

        {/* Real-time Audio Passthrough Loopback Toggle */}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2 pt-3 border-t border-theme-border/60">
          <div class="flex items-center gap-2 text-xs text-theme-text/60">
            <Headphones size={15} class={isMonitoring() ? 'text-theme-cyan animate-pulse shrink-0' : 'text-theme-text/40 shrink-0'} />
            <span class="text-[11px] sm:text-xs">Review Passthrough (Dengarkan hasil filter DSP ke headphone)</span>
          </div>

          <button
            onClick={toggleMonitor}
            class={`px-3.5 py-1.5 rounded-xl text-xs font-bold border transition-all flex items-center justify-center gap-2 shadow-sm ${
              isMonitoring()
                ? 'bg-theme-cyan border-theme-cyan text-theme-card anime-shadow-green'
                : 'bg-theme-bg border-theme-border text-theme-text/70 hover:text-theme-text hover:border-theme-cyan/50'
            }`}
          >
            {isMonitoring() ? <Volume2 size={14} /> : <VolumeX size={14} />}
            <span>{isMonitoring() ? 'DSP Monitor: ON' : 'DSP Monitor: OFF'}</span>
          </button>
        </div>
      </div>

      {/* 3. Toggle Audio & Input */}
      <div class="p-4 sm:p-6 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-rose flex items-center gap-2">
          <Volume2 size={18} />
          3. TOGGLE SUARA & MIKROFON
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">
          <button
            onClick={toggleVoice}
            class={`flex items-center justify-between p-3.5 sm:p-4 rounded-xl border font-bold text-sm transition-all ${
              voiceMuted()
                ? 'bg-theme-card border-theme-rose text-theme-rose'
                : 'bg-theme-card border-theme-cyan text-theme-cyan'
            }`}
          >
            <div class="flex items-center gap-3">
              {voiceMuted() ? <VolumeX size={20} class="shrink-0" /> : <Volume2 size={20} class="shrink-0" />}
              <div class="text-left">
                <span class="block text-xs sm:text-sm text-theme-text">Suara Karakter (TTS)</span>
                <span class="text-[11px] sm:text-xs text-theme-text/60 font-normal">{voiceMuted() ? 'Muted' : 'Aktif Bersuara'}</span>
              </div>
            </div>
            <span class={`text-[10px] sm:text-[11px] px-2.5 py-1 rounded-full uppercase font-mono ${
              voiceMuted() ? 'bg-theme-rose/20 text-theme-rose' : 'bg-theme-cyan/20 text-theme-cyan'
            }`}>
              {voiceMuted() ? 'Muted' : 'Unmuted'}
            </span>
          </button>

          <button
            onClick={toggleMic}
            class={`flex items-center justify-between p-3.5 sm:p-4 rounded-xl border font-bold text-sm transition-all ${
              micMuted()
                ? 'bg-theme-card border-theme-rose text-theme-rose'
                : 'bg-theme-card border-theme-cyan text-theme-cyan'
            }`}
          >
            <div class="flex items-center gap-3">
              {micMuted() ? <MicOff size={20} class="shrink-0" /> : <Mic size={20} class="shrink-0" />}
              <div class="text-left">
                <span class="block text-xs sm:text-sm text-theme-text">Input Mikrofon (STT)</span>
                <span class="text-[11px] sm:text-xs text-theme-text/60 font-normal">{micMuted() ? 'Muted' : 'Mendengarkan'}</span>
              </div>
            </div>
            <span class={`text-[10px] sm:text-[11px] px-2.5 py-1 rounded-full uppercase font-mono ${
              micMuted() ? 'bg-theme-rose/20 text-theme-rose' : 'bg-theme-cyan/20 text-theme-cyan'
            }`}>
              {micMuted() ? 'Muted' : 'Active'}
            </span>
          </button>
        </div>
      </div>

      {/* 4. Tindakan Cepat (Quick Actions) */}
      <div class="p-4 sm:p-6 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-cyan flex items-center gap-2">
          <Layers size={18} />
          4. TINDAKAN CEPAT & SISTEM
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 sm:gap-3">
          <button
            onClick={handleCheckUpdate}
            disabled={isUpdating()}
            class="flex items-center justify-center gap-2 p-3 sm:p-3.5 rounded-xl text-xs sm:text-sm font-semibold bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border transition-all"
          >
            <RefreshCw size={16} class={isUpdating() ? 'animate-spin text-theme-cyan' : ''} />
            <span>{isUpdating() ? 'Memeriksa...' : 'Cek Pembaruan'}</span>
          </button>

          <button
            onClick={handleResetSession}
            class="flex items-center justify-center gap-2 p-3 sm:p-3.5 rounded-xl text-xs sm:text-sm font-semibold bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border transition-all"
          >
            <RotateCcw size={16} />
            <span>Reset AI Session</span>
          </button>

          <button
            onClick={handleRestartEngine}
            class="flex items-center justify-center gap-2 p-3 sm:p-3.5 rounded-xl text-xs sm:text-sm font-semibold bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-border transition-all"
          >
            <RefreshCw size={16} />
            <span>Sync 3D Engine</span>
          </button>
        </div>
      </div>

      {/* Modal Dialog Cek Pembaruan & Release Notes */}
      {updateModalOpen() && updateInfo() && (
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4 animate-fade-in">
          <div class="bg-theme-surface border border-theme-border rounded-2xl w-full max-w-lg shadow-2xl overflow-hidden flex flex-col max-h-[85vh]">
            {/* Header Modal */}
            <div class="flex items-center justify-between p-4 border-b border-theme-border bg-theme-bg">
              <div class="flex items-center gap-2">
                <span class={`p-2 rounded-xl ${updateInfo()!.hasUpdate ? 'bg-theme-cyan/20 text-theme-cyan' : 'bg-theme-border text-theme-text/70'}`}>
                  {updateInfo()!.hasUpdate ? <Download size={20} /> : <RefreshCw size={20} />}
                </span>
                <div>
                  <h3 class="font-carter text-lg text-theme-text tracking-wide">
                    {updateInfo()!.hasUpdate ? '🎉 PEMBARUAN TERSEDIA!' : '✨ SISTEM UP-TO-DATE'}
                  </h3>
                  <div class="flex items-center gap-2 text-xs text-theme-text/60">
                    <span>Versi saat ini: <strong class="text-theme-text">{updateInfo()!.currentVersion}</strong></span>
                    <span>•</span>
                    <span>Terbaru: <strong class="text-theme-cyan">{updateInfo()!.latestVersion}</strong></span>
                  </div>
                </div>
              </div>
              <button
                onClick={() => setUpdateModalOpen(false)}
                class="p-1.5 rounded-lg text-theme-text/60 hover:text-theme-text hover:bg-theme-border transition-all"
              >
                <X size={18} />
              </button>
            </div>

            {/* Isi Release Notes / Changelog */}
            <div class="p-5 overflow-y-auto space-y-4 flex-1 text-sm text-theme-text/85">
              <div class="p-3.5 rounded-xl bg-theme-bg border border-theme-border">
                <h4 class="text-xs font-mono uppercase tracking-wider text-theme-cyan mb-1">Nama Rilis</h4>
                <p class="font-bold text-base text-theme-text">{updateInfo()!.releaseName}</p>
              </div>

              <div>
                <h4 class="text-xs font-mono uppercase tracking-wider text-theme-text/60 mb-2">Catatan Pembaruan & Fitur Baru</h4>
                <div class="p-3.5 rounded-xl bg-theme-bg border border-theme-border text-xs font-mono text-theme-text/80 whitespace-pre-wrap leading-relaxed max-h-56 overflow-y-auto custom-scrollbar">
                  {updateInfo()!.releaseNotes}
                </div>
              </div>
            </div>

            {/* Footer Aksi */}
            <div class="p-4 border-t border-theme-border bg-theme-bg flex items-center justify-end gap-3">
              <button
                onClick={() => setUpdateModalOpen(false)}
                class="px-4 py-2 rounded-xl text-xs font-semibold bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border transition-all"
              >
                Tutup
              </button>

              <a
                href={updateInfo()!.releaseUrl}
                target="_blank"
                rel="noopener noreferrer"
                class={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all shadow-md ${
                  updateInfo()!.hasUpdate
                    ? 'bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card anime-shadow-green'
                    : 'bg-theme-card hover:bg-theme-border text-theme-cyan border border-theme-cyan/30'
                }`}
              >
                {updateInfo()!.hasUpdate ? <Download size={14} /> : <ExternalLink size={14} />}
                <span>{updateInfo()!.hasUpdate ? 'Unduh Rilis Baru di GitHub' : 'Buka GitHub Releases'}</span>
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
