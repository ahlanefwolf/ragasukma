import { Component, createSignal } from 'solid-js';
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
  Power,
  Sparkles,
  Bot,
  Layers,
  ArrowUpRight
} from 'lucide-solid';

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

  const handleStart = () => {
    setIsPaused(false);
    props.setCurrentState('listening');
    props.showToast('Voice Assistant mendengarkan...');
  };

  const handlePause = () => {
    setIsPaused(true);
    props.setCurrentState('idle');
    props.showToast('Voice Assistant dijeda');
  };

  const handleStop = () => {
    setIsPaused(false);
    props.setCurrentState('idle');
    props.showToast('Voice Assistant dihentikan');
  };

  const toggleVoice = () => {
    setVoiceMuted(!voiceMuted());
    props.showToast(voiceMuted() ? 'Suara TTS Dinonaktifkan' : 'Suara TTS Diaktifkan');
  };

  const toggleMic = () => {
    setMicMuted(!micMuted());
    props.showToast(micMuted() ? 'Mikrofon STT Dinonaktifkan' : 'Mikrofon STT Diaktifkan');
  };

  const handleCheckUpdate = () => {
    setIsUpdating(true);
    setTimeout(() => {
      setIsUpdating(false);
      props.showToast('Ragasukma sudah versi terbaru (v0.1.1)');
    }, 1500);
  };

  const handleResetSession = () => {
    props.setCurrentState('idle');
    props.showToast('Memory session & context AI berhasil di-reset');
  };

  const handleRestartEngine = () => {
    props.showToast('Merestart 3D native engine...');
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
            RAGASUKMA VOICE ASSISTANT
          </h2>
          <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-lg">
            Kendalikan status percakapan, mikrofon, pemutaran suara TTS, dan fungsi restart engine 3D native.
          </p>
        </div>

        {/* State Indicator Pill */}
        <div class="p-4 rounded-xl bg-[#181820] border border-[#3B3B48] flex items-center gap-4 shrink-0">
          <div class="w-12 h-12 rounded-xl bg-[#282832] flex items-center justify-center text-[#91AE6E]">
            <Bot size={24} />
          </div>
          <div>
            <span class="text-xs text-[#F2F2F2]/50 block">Voice Engine</span>
            <span class="text-sm font-bold text-[#F2F2F2]">Edge TTS (Indonesian)</span>
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
            <span>Restart 3D Engine</span>
          </button>
        </div>
      </div>
    </div>
  );
};
