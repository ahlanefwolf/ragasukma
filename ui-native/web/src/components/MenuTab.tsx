import { Component, createSignal, onMount } from 'solid-js';
import { 
  Play, 
  Pause, 
  Square, 
  Volume2, 
  VolumeX, 
  Mic, 
  MicOff, 
  RefreshCw, 
  RotateCcw, 
  Sparkles,
  Bot,
  Layers
} from 'lucide-solid';
import { api, AppStatePayload } from '../api';

interface MenuTabProps {
  currentState: string;
  setCurrentState: (s: string) => void;
  showToast: (msg: string) => void;
}

export const MenuTab: Component<MenuTabProps> = (props) => {
  const [isPaused, setIsPaused] = createSignal(false);
  const [voiceMuted, setVoiceMuted] = createSignal(false);
  const [micMuted, setMicMuted] = createSignal(false);
  const [isUpdating, setIsUpdating] = createSignal(false);
  const [assistantName, setAssistantName] = createSignal('Laras');
  const [ttsEngineName, setTtsEngineName] = createSignal('Google TTS');

  onMount(async () => {
    const cfg = await api.getConfig();
    if (cfg) {
      setAssistantName(cfg.assistant_name || 'Laras');
      setTtsEngineName(`${cfg.tts_engine.toUpperCase()} (${cfg.tts_lang})`);
    }

    const state = await api.getState();
    if (state) {
      props.setCurrentState(state.state);
      setIsPaused(state.is_paused);
      setVoiceMuted(state.voice_muted);
      setMicMuted(state.mic_muted);
    }
  });

  const syncState = async (newState: string, paused = isPaused(), vMuted = voiceMuted(), mMuted = micMuted()) => {
    props.setCurrentState(newState);
    setIsPaused(paused);
    setVoiceMuted(vMuted);
    setMicMuted(mMuted);

    const payload: AppStatePayload = {
      state: newState,
      is_paused: paused,
      voice_muted: vMuted,
      mic_muted: mMuted,
    };
    await api.setState(payload);
  };

  const handleStart = async () => {
    await syncState('listening', false);
    props.showToast('Voice Assistant mendengarkan...');
  };

  const handlePause = async () => {
    await syncState('idle', true);
    props.showToast('Voice Assistant dijeda');
  };

  const handleStop = async () => {
    await syncState('idle', false);
    props.showToast('Voice Assistant dihentikan');
  };

  const toggleVoice = async () => {
    const next = !voiceMuted();
    await syncState(props.currentState, isPaused(), next, micMuted());
    props.showToast(next ? 'Suara TTS Dinonaktifkan' : 'Suara TTS Diaktifkan');
  };

  const toggleMic = async () => {
    const next = !micMuted();
    await syncState(props.currentState, isPaused(), voiceMuted(), next);
    props.showToast(next ? 'Mikrofon STT Dinonaktifkan' : 'Mikrofon STT Diaktifkan');
  };

  const handleCheckUpdate = () => {
    setIsUpdating(true);
    setTimeout(() => {
      setIsUpdating(false);
      props.showToast('Ragasukma Engine sudah versi terbaru (v0.1.2)');
    }, 1200);
  };

  const handleResetSession = async () => {
    await syncState('idle', false);
    props.showToast('Memory session & context AI berhasil di-reset');
  };

  const handleRestartEngine = async () => {
    await syncState('idle', false);
    props.showToast('Sinkronisasi & reload state engine 3D selesai');
  };

  return (
    <div class="space-y-6">
      {/* Top Banner Status Header */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48] flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#D96868]/20 text-[#D96868] border border-[#D96868]/30">
              Menu Kontrol Utama
            </span>
            <span class="px-2 py-0.5 rounded-full text-[11px] font-bold font-mono bg-[#689D4B]/20 text-[#91AE6E] border border-[#689D4B]/30 uppercase">
              State: {props.currentState}
            </span>
          </div>
          <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
            RAGASUKMA - {assistantName().toUpperCase()}
          </h2>
          <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-lg">
            Kendalikan status percakapan, mikrofon, pemutaran suara TTS, dan fungsi sinkronisasi engine 3D native.
          </p>
        </div>

        {/* State Indicator Pill */}
        <div class="p-4 rounded-xl bg-[#181820] border border-[#3B3B48] flex items-center gap-4 shrink-0">
          <div class="w-12 h-12 rounded-xl bg-[#282832] flex items-center justify-center text-[#91AE6E]">
            <Bot size={24} />
          </div>
          <div>
            <span class="text-xs text-[#F2F2F2]/50 block">Voice Engine</span>
            <span class="text-sm font-bold text-[#F2F2F2]">{ttsEngineName()}</span>
          </div>
        </div>
      </div>

      {/* 1. Kontrol Voice Assistant */}
      <div class="p-6 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Sparkles size={18} />
          1. KONTROL VOICE ASSISTANT
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <button
            onClick={handleStart}
            disabled={props.currentState === 'listening'}
            class="flex items-center justify-center gap-2.5 p-4 rounded-xl font-bold text-sm bg-[#689D4B] hover:bg-[#5b8c40] text-white shadow-md anime-shadow-green transition-all disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <Play size={18} class="fill-current" />
            <span>Mulai / Dengarkan</span>
          </button>

          <button
            onClick={handlePause}
            disabled={isPaused()}
            class="flex items-center justify-center gap-2.5 p-4 rounded-xl font-bold text-sm bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48] transition-all disabled:opacity-40 disabled:cursor-not-allowed"
          >
            <Pause size={18} />
            <span>Jeda Percakapan</span>
          </button>

          <button
            onClick={handleStop}
            class="flex items-center justify-center gap-2.5 p-4 rounded-xl font-bold text-sm bg-[#282832] hover:bg-[#D96868]/30 hover:border-[#D96868] text-[#F2F2F2] border border-[#3B3B48] transition-all"
          >
            <Square size={18} />
            <span>Hentikan AI</span>
          </button>
        </div>
      </div>

      {/* 2. Toggle Audio & Input */}
      <div class="p-6 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#D96868] flex items-center gap-2">
          <Volume2 size={18} />
          2. TOGGLE SUARA & MIKROFON
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <button
            onClick={toggleVoice}
            class={`flex items-center justify-between p-4 rounded-xl border font-bold text-sm transition-all ${
              voiceMuted()
                ? 'bg-[#282832] border-[#D96868] text-[#D96868]'
                : 'bg-[#282832] border-[#689D4B] text-[#91AE6E]'
            }`}
          >
            <div class="flex items-center gap-3">
              {voiceMuted() ? <VolumeX size={20} /> : <Volume2 size={20} />}
              <div class="text-left">
                <span class="block text-sm text-[#F2F2F2]">Suara Karakter (TTS)</span>
                <span class="text-xs text-[#F2F2F2]/60 font-normal">{voiceMuted() ? 'Muted' : 'Aktif Bersuara'}</span>
              </div>
            </div>
            <span class={`text-[11px] px-2.5 py-1 rounded-full uppercase font-mono ${
              voiceMuted() ? 'bg-[#D96868]/20 text-[#D96868]' : 'bg-[#689D4B]/20 text-[#91AE6E]'
            }`}>
              {voiceMuted() ? 'Muted' : 'Unmuted'}
            </span>
          </button>

          <button
            onClick={toggleMic}
            class={`flex items-center justify-between p-4 rounded-xl border font-bold text-sm transition-all ${
              micMuted()
                ? 'bg-[#282832] border-[#D96868] text-[#D96868]'
                : 'bg-[#282832] border-[#689D4B] text-[#91AE6E]'
            }`}
          >
            <div class="flex items-center gap-3">
              {micMuted() ? <MicOff size={20} /> : <Mic size={20} />}
              <div class="text-left">
                <span class="block text-sm text-[#F2F2F2]">Input Mikrofon (STT)</span>
                <span class="text-xs text-[#F2F2F2]/60 font-normal">{micMuted() ? 'Muted' : 'Mendengarkan'}</span>
              </div>
            </div>
            <span class={`text-[11px] px-2.5 py-1 rounded-full uppercase font-mono ${
              micMuted() ? 'bg-[#D96868]/20 text-[#D96868]' : 'bg-[#689D4B]/20 text-[#91AE6E]'
            }`}>
              {micMuted() ? 'Muted' : 'Active'}
            </span>
          </button>
        </div>
      </div>

      {/* 3. Tindakan Cepat (Quick Actions) */}
      <div class="p-6 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Layers size={18} />
          3. TINDAKAN CEPAT & SISTEM
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <button
            onClick={handleCheckUpdate}
            disabled={isUpdating()}
            class="flex items-center justify-center gap-2 p-3.5 rounded-xl text-sm font-semibold bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48] transition-all"
          >
            <RefreshCw size={16} class={isUpdating() ? 'animate-spin text-[#91AE6E]' : ''} />
            <span>{isUpdating() ? 'Memeriksa...' : 'Cek Pembaruan'}</span>
          </button>

          <button
            onClick={handleResetSession}
            class="flex items-center justify-center gap-2 p-3.5 rounded-xl text-sm font-semibold bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48] transition-all"
          >
            <RotateCcw size={16} />
            <span>Reset AI Session</span>
          </button>

          <button
            onClick={handleRestartEngine}
            class="flex items-center justify-center gap-2 p-3.5 rounded-xl text-sm font-semibold bg-[#282832] hover:bg-[#D96868]/20 text-[#D96868] border border-[#3B3B48] transition-all"
          >
            <RefreshCw size={16} />
            <span>Sync 3D Engine</span>
          </button>
        </div>
      </div>
    </div>
  );
};
