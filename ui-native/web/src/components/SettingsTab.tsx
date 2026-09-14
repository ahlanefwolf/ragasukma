import { Component, createSignal, onMount } from 'solid-js';
import { 
  Brain, 
  Mic, 
  Volume2, 
  Monitor, 
  Save, 
  Upload
} from 'lucide-solid';
import { api, UserConfig } from '../api';

interface SettingsTabProps {
  showToast: (msg: string) => void;
}

export const SettingsTab: Component<SettingsTabProps> = (props) => {
  // Assistant Persona & Call
  const [assistantName, setAssistantName] = createSignal('Laras');
  const [userCallsign, setUserCallsign] = createSignal('Mas');
  const [personalityRole, setPersonalityRole] = createSignal('partner');

  // AI LLM Provider
  const [llmBaseUrl, setLlmBaseUrl] = createSignal('http://localhost:20128/v1');
  const [llmModel, setLlmModel] = createSignal('ag/gemini-3.8-flash-low');
  const [llmApiKey, setLlmApiKey] = createSignal('');
  const [groqApiKey, setGroqApiKey] = createSignal('');

  // Audio & STT
  const [sttEngine, setSttEngine] = createSignal('google');
  const [voiceMode, setVoiceMode] = createSignal('auto');
  const [micSensitivity, setMicSensitivity] = createSignal('2');
  const [vadSilenceLimit, setVadSilenceLimit] = createSignal('1');
  const [noiseReduction, setNoiseReduction] = createSignal(0);

  // TTS Engine
  const [ttsEngine, setTtsEngine] = createSignal('google');
  const [ttsLang, setTtsLang] = createSignal('id');
  const [ttsSpeed, setTtsSpeed] = createSignal(1.15);
  const [ttsElevenLabsKey, setTtsElevenLabsKey] = createSignal('');
  const [elevenLabsVoiceId, setElevenLabsVoiceId] = createSignal('');

  // Avatar & System
  const [avatarType, setAvatarType] = createSignal('vrm');
  const [alwaysOnTop, setAlwaysOnTop] = createSignal(true);
  const [enableTerminalLog, setEnableTerminalLog] = createSignal(false);
  const [isSaving, setIsSaving] = createSignal(false);

  const loadConfigData = async () => {
    const cfg = await api.getConfig();
    if (cfg) {
      setAssistantName(cfg.assistant_name || 'Laras');
      setUserCallsign(cfg.user_callsign || 'Mas');
      setPersonalityRole(cfg.personality_role || 'partner');

      setLlmBaseUrl(cfg.llm_base_url || 'http://localhost:20128/v1');
      setLlmModel(cfg.llm_model || 'ag/gemini-3.8-flash-low');
      setLlmApiKey(cfg.llm_api_key || '');
      setGroqApiKey(cfg.groq_api_key || '');

      setSttEngine(cfg.stt_engine || 'google');
      setVoiceMode(cfg.voice_mode || 'auto');
      setMicSensitivity(cfg.mic_sensitivity || '2');
      setVadSilenceLimit(cfg.vad_silence_limit || '1');
      setNoiseReduction(cfg.noise_reduction_level || 0);

      setTtsEngine(cfg.tts_engine || 'google');
      setTtsLang(cfg.tts_lang || 'id');
      setTtsSpeed(cfg.tts_speed || 1.15);
      setTtsElevenLabsKey(cfg.tts_eleven_labs_key || '');
      setElevenLabsVoiceId(cfg.elevenlabs_voice_id || '');

      setAvatarType(cfg.avatar_type || 'vrm');
      setAlwaysOnTop(cfg.always_on_top !== undefined ? cfg.always_on_top : true);
      setEnableTerminalLog(cfg.enable_terminal_log || false);
    }
  };

  onMount(() => {
    loadConfigData();
  });

  const handleSave = async (e: Event) => {
    e.preventDefault();
    setIsSaving(true);

    const payload: UserConfig = {
      assistant_name: assistantName().trim(),
      user_callsign: userCallsign().trim(),
      personality_role: personalityRole(),
      stt_engine: sttEngine(),
      groq_api_key: groqApiKey().trim(),
      llm_base_url: llmBaseUrl().trim(),
      llm_model: llmModel().trim(),
      llm_api_key: llmApiKey().trim(),
      voice_mode: voiceMode(),
      mic_sensitivity: micSensitivity(),
      vad_silence_limit: vadSilenceLimit(),
      noise_reduction_level: noiseReduction(),
      tts_engine: ttsEngine(),
      tts_lang: ttsLang(),
      tts_speed: ttsSpeed(),
      tts_eleven_labs_key: ttsElevenLabsKey().trim(),
      elevenlabs_voice_id: elevenLabsVoiceId().trim(),
      avatar_type: avatarType(),
      always_on_top: alwaysOnTop(),
      enable_terminal_log: enableTerminalLog(),
    };

    const ok = await api.saveConfig(payload);
    setIsSaving(false);
    if (ok) {
      props.showToast('Semua konfigurasi berhasil disimpan ke config-ai.json!');
    } else {
      props.showToast('Gagal menyimpan ke config-ai.json');
    }
  };

  return (
    <form onSubmit={handleSave} class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#232D36] via-[#1D252E] to-[#161D24] p-6 border border-[#2F3B46] flex items-center justify-between">
        <div>
          <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30">
            System & Engine Preferences
          </span>
          <h2 class="text-3xl font-bangers tracking-wide text-[#EFF7F6] mt-2">
            PENGATURAN LENGKAP SISTEM
          </h2>
          <p class="text-sm text-[#EFF7F6]/70 mt-1 max-w-xl">
            Atur persona nama, LLM Brain endpoint, sensitivitas mikrofon STT, mesin TTS, dan logging config-ai.json.
          </p>
        </div>

        <button
          type="submit"
          disabled={isSaving()}
          class="px-6 py-3 rounded-xl font-bold text-sm bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] shadow-md anime-shadow-green flex items-center gap-2 transition-all shrink-0 font-semibold"
        >
          <Save size={16} />
          <span>{isSaving() ? 'Menyimpan...' : 'Simpan Semua'}</span>
        </button>
      </div>

      {/* 1. Persona & Panggilan */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
          1. PERSONA & IDENTITAS ASISTEN
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Nama Asisten</label>
            <input
              type="text"
              value={assistantName()}
              onInput={(e) => setAssistantName(e.currentTarget.value)}
              placeholder="Laras"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Panggilan Pengguna</label>
            <input
              type="text"
              value={userCallsign()}
              onInput={(e) => setUserCallsign(e.currentTarget.value)}
              placeholder="Mas"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Peran Kepribadian</label>
            <select
              value={personalityRole()}
              onChange={(e) => setPersonalityRole(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="partner">Partner (Hangat & Santai)</option>
              <option value="assistant">Assistant (Formal & Rapi)</option>
              <option value="tsundere">Tsundere (Khas Anime)</option>
              <option value="maid">Maid (Pelayan Santun)</option>
            </select>
          </div>
        </div>
      </div>

      {/* 2. Pengaturan AI Brain (LLM Provider) */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-lg font-bangers text-[#F2B5D4] flex items-center gap-2">
          <Brain size={18} />
          2. AI BRAIN (LLM PROVIDER)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">LLM Base URL (OpenAI Compatible)</label>
            <input
              type="text"
              value={llmBaseUrl()}
              onInput={(e) => setLlmBaseUrl(e.currentTarget.value)}
              placeholder="http://localhost:20128/v1"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Model Name</label>
            <input
              type="text"
              value={llmModel()}
              onInput={(e) => setLlmModel(e.currentTarget.value)}
              placeholder="ag/gemini-3.8-flash-low"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            />
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">LLM API Key (Opsional / Gateway)</label>
            <input
              type="password"
              value={llmApiKey()}
              onInput={(e) => setLlmApiKey(e.currentTarget.value)}
              placeholder="sk-..."
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Groq API Key (Fast Whisper STT / LLM)</label>
            <input
              type="password"
              value={groqApiKey()}
              onInput={(e) => setGroqApiKey(e.currentTarget.value)}
              placeholder="gsk_..."
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            />
          </div>
        </div>
      </div>

      {/* 3. Audio & Speech-to-Text (STT) */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
          <Mic size={18} />
          3. AUDIO & SPEECH-TO-TEXT (STT)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">STT Engine</label>
            <select
              value={sttEngine()}
              onChange={(e) => setSttEngine(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="google">Google Cloud STT (Cepat & Akurat ID)</option>
              <option value="groq">Groq Whisper Large v3</option>
              <option value="vosk">Vosk Local Offline</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Sensitivitas VAD Mikrofon</label>
            <select
              value={micSensitivity()}
              onChange={(e) => setMicSensitivity(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="0">0 - Ekstra Kebal (Sangat Ramai)</option>
              <option value="1">1 - Rendah</option>
              <option value="2">2 - Normal (Standar)</option>
              <option value="3">3 - Responsif</option>
              <option value="4">4 - Sangat Sensitif</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Batas Jeda Hening VAD</label>
            <select
              value={vadSilenceLimit()}
              onChange={(e) => setVadSilenceLimit(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="0">Cepat (0.6 Detik)</option>
              <option value="1">Sedang (1.0 Detik)</option>
              <option value="2">Panjang (1.5 Detik)</option>
            </select>
          </div>
        </div>
      </div>

      {/* 4. Text-to-Speech (TTS) Voice */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-lg font-bangers text-[#F2B5D4] flex items-center gap-2">
          <Volume2 size={18} />
          4. SUARA KARAKTER (TEXT-TO-SPEECH)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">TTS Engine</label>
            <select
              value={ttsEngine()}
              onChange={(e) => setTtsEngine(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="google">Google TTS (Ringan & Cepat)</option>
              <option value="edge">Edge TTS Indonesian</option>
              <option value="elevenlabs">ElevenLabs Neural Voice</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Bahasa Voice</label>
            <select
              value={ttsLang()}
              onChange={(e) => setTtsLang(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="id">Indonesian (id)</option>
              <option value="ja">Japanese (ja)</option>
              <option value="en">English (en)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between text-xs font-semibold text-[#EFF7F6]/80">
              <span>Kecepatan Bicara</span>
              <span class="text-[#7BDFF2] font-mono">{ttsSpeed().toFixed(2)}x</span>
            </div>
            <input
              type="range"
              min="0.8"
              max="2.0"
              step="0.05"
              value={ttsSpeed()}
              onInput={(e) => setTtsSpeed(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#232D36] rounded-lg appearance-none cursor-pointer accent-[#7BDFF2] mt-2"
            />
          </div>
        </div>
      </div>
      {/* 5. Desktop Overlay & Window Preferences */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
          <Monitor size={18} />
          5. DESKTOP OVERLAY & WINDOW
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="flex items-center justify-between p-4 rounded-xl bg-[#232D36] border border-[#2F3B46]">
            <div>
              <span class="block text-sm font-bold text-[#EFF7F6]">Always On Top (Selalu di Depan)</span>
              <span class="text-xs text-[#EFF7F6]/60">Overlay 3D avatar tetap berada di atas jendela aplikasi lain</span>
            </div>
            <button
              type="button"
              onClick={() => setAlwaysOnTop(!alwaysOnTop())}
              class={`px-3 py-1.5 rounded-lg text-xs font-bold uppercase font-mono transition-all ${
                alwaysOnTop()
                  ? 'bg-[#7BDFF2] text-[#232D36] shadow anime-shadow-green'
                  : 'bg-[#1D252E] text-[#EFF7F6]/60 border border-[#2F3B46]'
              }`}
            >
              {alwaysOnTop() ? 'ON' : 'OFF'}
            </button>
          </div>

          <div class="flex items-center justify-between p-4 rounded-xl bg-[#232D36] border border-[#2F3B46]">
            <div>
              <span class="block text-sm font-bold text-[#EFF7F6]">Terminal Debug Logging</span>
              <span class="text-xs text-[#EFF7F6]/60">Tampilkan stream log STT, LLM, dan TTS di konsol terminal</span>
            </div>
            <button
              type="button"
              onClick={() => setEnableTerminalLog(!enableTerminalLog())}
              class={`px-3 py-1.5 rounded-lg text-xs font-bold uppercase font-mono transition-all ${
                enableTerminalLog()
                  ? 'bg-[#7BDFF2] text-[#232D36] shadow anime-shadow-green'
                  : 'bg-[#1D252E] text-[#EFF7F6]/60 border border-[#2F3B46]'
              }`}
            >
              {enableTerminalLog() ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>
      </div>
    </form>
  );
};
