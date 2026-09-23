import { Component } from 'solid-js';
import { Mic, Volume2 } from 'lucide-solid';

interface AudioTtsSectionProps {
  sttEngine: string;
  setSttEngine: (val: string) => void;
  groqApiKey: string;
  setGroqApiKey: (val: string) => void;
  voiceInterruption: boolean;
  setVoiceInterruption: (val: boolean) => void;
  micSensitivity: string;
  setMicSensitivity: (val: string) => void;
  vadSilenceLimit: string;
  setVadSilenceLimit: (val: string) => void;
  noiseReduction: number;
  setNoiseReduction: (val: number) => void;
  ttsEngine: string;
  setTtsEngine: (val: string) => void;
  ttsSpeed: number;
  setTtsSpeed: (val: number) => void;
  ttsElevenLabsKey: string;
  setTtsElevenLabsKey: (val: string) => void;
  elevenLabsVoiceId: string;
  setElevenLabsVoiceId: (val: string) => void;
}

export const AudioTtsSection: Component<AudioTtsSectionProps> = (props) => {
  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 3. Audio & Speech-to-Text (STT) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-bold text-theme-cyan flex items-center gap-2">
          <Mic size={18} />
          3. AUDIO & SPEECH-TO-TEXT (STT)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">STT Engine</label>
            <select
              value={props.sttEngine}
              onChange={(e) => props.setSttEngine(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="google">Google Cloud STT (Cepat & Akurat ID)</option>
              <option value="groq">Groq Whisper Large v3</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Sensitivitas VAD Mikrofon</label>
            <select
              value={props.micSensitivity}
              onChange={(e) => props.setMicSensitivity(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="0">0 - Ekstra Kebal (Ruangan Sangat Ramai)</option>
              <option value="1">1 - Sangat Rendah</option>
              <option value="2">2 - Rendah</option>
              <option value="3">3 - Normal (Standar)</option>
              <option value="4">4 - Tinggi (Responsif)</option>
              <option value="5">5 - Sangat Tinggi</option>
              <option value="6">6 - Ultra Sensitif (Bisikan)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Batas Jeda Hening VAD</label>
            <select
              value={props.vadSilenceLimit}
              onChange={(e) => props.setVadSilenceLimit(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="0">Sangat Cepat (~0.5 Detik)</option>
              <option value="1">Cepat (~0.7 Detik)</option>
              <option value="2">Normal (~1.0 Detik)</option>
              <option value="3">Santai (~1.3 Detik)</option>
              <option value="4">Panjang (~1.7 Detik)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Noise Reduction (DSP Filter)</label>
            <select
              value={props.noiseReduction}
              onChange={(e) => props.setNoiseReduction(parseInt(e.currentTarget.value, 10))}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value={0}>0 - Nonaktif (Direct Mic)</option>
              <option value={1}>1 - Ringan (High-Pass + Noise Gate)</option>
              <option value={2}>2 - Agresif (Deep Noise Gate)</option>
            </select>
          </div>
        </div>

        {props.sttEngine === 'groq' && (
          <div class="p-3.5 bg-theme-card border border-theme-cyan/30 space-y-1.5 animate-in fade-in duration-150">
            <label class="text-xs font-semibold text-theme-cyan flex items-center justify-between">
              <span>Groq API Key (Fast Whisper STT)</span>
              <span class="text-[10px] text-theme-text/50 font-mono">Dibutuhkan untuk Whisper Cloud</span>
            </label>
            <input
              type="password"
              value={props.groqApiKey}
              onInput={(e) => props.setGroqApiKey(e.currentTarget.value)}
              placeholder="gsk_..."
              class="w-full bg-theme-bg border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>
        )}

        <div class="p-3 bg-theme-card border border-theme-border flex items-center justify-between gap-4">
          <div>
            <label class="text-xs sm:text-sm font-semibold text-theme-text block">⚡ Interupsi Suara (Barge-in Voice Interrupt)</label>
            <p class="text-xs text-theme-text/60 mt-0.5">
              Mengizinkan kamu memotong pembicaraan asisten seketika hanya dengan berbicara langsung ke mikrofon.
            </p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer shrink-0">
            <input
              type="checkbox"
              checked={props.voiceInterruption}
              onChange={(e) => props.setVoiceInterruption(e.currentTarget.checked)}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-theme-border peer-focus:outline-none peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-theme-text after:h-5 after:w-5 after:transition-all peer-checked:bg-theme-cyan"></div>
          </label>
        </div>
      </div>

      {/* 4. Text-to-Speech (TTS) Voice */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-bold text-theme-rose flex items-center gap-2">
          <Volume2 size={18} />
          4. SUARA KARAKTER (TEXT-TO-SPEECH)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 items-start">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">TTS Engine</label>
            <select
              value={props.ttsEngine}
              onChange={(e) => props.setTtsEngine(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="google">Google TTS (Ringan & Cepat)</option>
              <option value="elevenlabs">ElevenLabs Neural Voice</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between items-center text-xs font-semibold text-theme-text/80">
              <span>Kecepatan Bicara</span>
              <span class="text-xs font-mono font-bold text-theme-cyan bg-theme-card px-2 py-0.5 border border-theme-border">
                {props.ttsSpeed.toFixed(2)}x
              </span>
            </div>
            <div class="h-[42px] px-3.5 bg-theme-card border border-theme-border flex items-center">
              <input
                type="range"
                min="0.8"
                max="2.0"
                step="0.05"
                value={props.ttsSpeed}
                onInput={(e) => props.setTtsSpeed(parseFloat(e.currentTarget.value))}
                class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-cyan"
              />
            </div>
          </div>
        </div>

        {props.ttsEngine === 'elevenlabs' && (
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-theme-border">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">ElevenLabs API Key</label>
              <input
                type="password"
                value={props.ttsElevenLabsKey}
                onInput={(e) => props.setTtsElevenLabsKey(e.currentTarget.value)}
                placeholder="sk_..."
                class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
              />
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">ElevenLabs Voice ID</label>
              <input
                type="text"
                value={props.elevenLabsVoiceId}
                onInput={(e) => props.setElevenLabsVoiceId(e.currentTarget.value)}
                placeholder="cgSgspJ2msm6clMCkdW9"
                class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
