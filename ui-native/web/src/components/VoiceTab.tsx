import { Component, createSignal } from 'solid-js';
import { 
  Volume2, 
  Mic, 
  Sparkles, 
  Sliders, 
  Cpu, 
  Headphones,
  Activity,
  Play
} from 'lucide-solid';

export const VoiceTab: Component = () => {
  const [selectedVoice, setSelectedVoice] = createSignal('id-ID-ArdiNeural');
  const [speechSpeed, setSpeechSpeed] = createSignal(1.0);
  const [speechPitch, setSpeechPitch] = createSignal(1.0);
  const [volume, setVolume] = createSignal(85);
  const [micThreshold, setMicThreshold] = createSignal(35);
  const [isPlayingPreview, setIsPlayingPreview] = createSignal(false);

  const voiceOptions = [
    { id: 'id-ID-ArdiNeural', name: 'Ardi (Indonesian - Natural Casual)', type: 'Edge TTS' },
    { id: 'id-ID-GadisNeural', name: 'Gadis (Indonesian - Anime Kawaii/Warm)', type: 'Edge TTS' },
    { id: 'ja-JP-NanamiNeural', name: 'Nanami (Japanese - Anime Tsundere)', type: 'Edge TTS' },
    { id: 'custom_piper', name: 'Piper TTS (Local Offline Ultra-Fast)', type: 'Offline On-Device' },
  ];

  const handleTestAudio = () => {
    setIsPlayingPreview(true);
    setTimeout(() => setIsPlayingPreview(false), 2500);
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48]">
        <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#D96868]/20 text-[#D96868] border border-[#D96868]/30">
          Neural Audio Engine & Lip-Sync
        </span>
        <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
          PENGATURAN SUARA & RESPON AI
        </h2>
        <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-xl">
          Atur model Text-to-Speech (TTS), kecepatan berbicara, nada suara anime, serta sensitivitas input mikrofon untuk lip-sync real-time.
        </p>
      </div>

      {/* Voice Model Selection */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
            <Cpu size={18} />
            MODEL SUARA AI (TTS VOICE)
          </h3>
          <span class="text-xs text-[#91AE6E] font-medium">Auto Lip-Sync Terhubung</span>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          {voiceOptions.map((v) => {
            const isSelected = () => selectedVoice() === v.id;

            return (
              <button
                onClick={() => setSelectedVoice(v.id)}
                class={`p-4 rounded-xl border text-left transition-all ${
                  isSelected()
                    ? 'bg-[#282832] border-[#D96868] ring-2 ring-[#D96868]/30 shadow-md'
                    : 'bg-[#282832]/50 border-[#3B3B48] hover:border-[#F2F2F2]/30'
                }`}
              >
                <div class="flex items-center justify-between">
                  <span class="font-bold text-sm text-[#F2F2F2]">
                    {v.name}
                  </span>
                  <span class="text-[10px] px-2 py-0.5 rounded bg-[#181820] text-[#91AE6E] font-semibold border border-[#3B3B48]">
                    {v.type}
                  </span>
                </div>
                <p class="text-xs text-[#F2F2F2]/50 font-mono mt-1">
                  {v.id}
                </p>
              </button>
            );
          })}
        </div>
      </div>

      {/* Sliders Control Panel */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-5">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Sliders size={18} />
          TUNING PARAMETER AUDIO
        </h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Speed Slider */}
          <div class="space-y-2">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/80">Kecepatan Bicara (Speed)</span>
              <span class="text-[#91AE6E] font-mono">{speechSpeed()}x</span>
            </div>
            <input 
              type="range" 
              min="0.5" 
              max="2.0" 
              step="0.05"
              value={speechSpeed()}
              onInput={(e) => setSpeechSpeed(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#D96868]"
            />
          </div>

          {/* Pitch Slider */}
          <div class="space-y-2">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/80">Nada Suara (Pitch Tone)</span>
              <span class="text-[#91AE6E] font-mono">{speechPitch()}x</span>
            </div>
            <input 
              type="range" 
              min="0.5" 
              max="1.5" 
              step="0.05"
              value={speechPitch()}
              onInput={(e) => setSpeechPitch(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#689D4B]"
            />
          </div>

          {/* Volume Slider */}
          <div class="space-y-2">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/80">Volume Output</span>
              <span class="text-[#91AE6E] font-mono">{volume()}%</span>
            </div>
            <input 
              type="range" 
              min="0" 
              max="100"
              value={volume()}
              onInput={(e) => setVolume(parseInt(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#D96868]"
            />
          </div>

          {/* Mic Threshold Slider */}
          <div class="space-y-2">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/80">Sensitivitas Mikrofon (STT Threshold)</span>
              <span class="text-[#91AE6E] font-mono">{micThreshold()} dB</span>
            </div>
            <input 
              type="range" 
              min="10" 
              max="80"
              value={micThreshold()}
              onInput={(e) => setMicThreshold(parseInt(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#689D4B]"
            />
          </div>
        </div>

        {/* Test Voice Button */}
        <div class="pt-2 flex justify-end">
          <button
            onClick={handleTestAudio}
            disabled={isPlayingPreview()}
            class="px-5 py-2.5 rounded-xl font-bold text-sm bg-[#689D4B] hover:bg-[#5b8c40] text-white shadow-md anime-shadow-green flex items-center gap-2 transition-all"
          >
            <Play size={16} class={isPlayingPreview() ? 'animate-pulse fill-current' : 'fill-current'} />
            <span>{isPlayingPreview() ? 'Memutar Preview...' : 'Tes Suara TTS'}</span>
          </button>
        </div>
      </div>
    </div>
  );
};
