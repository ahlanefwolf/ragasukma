import { Component, createSignal } from 'solid-js';
import { 
  Settings, 
  Brain, 
  Mic, 
  Volume2, 
  Monitor, 
  FolderOpen, 
  Save, 
  Check, 
  Key,
  Upload,
  Layers
} from 'lucide-solid';

interface SettingsTabProps {
  showToast: (msg: string) => void;
}

export const SettingsTab: Component<SettingsTabProps> = (props) => {
  // General & Overlay
  const [overlayFps, setOverlayFps] = createSignal('60');
  const [alwaysOnTop, setAlwaysOnTop] = createSignal(true);
  const [autoLaunch, setAutoLaunch] = createSignal(true);

  // AI LLM Provider
  const [aiProvider, setAiProvider] = createSignal('gemini');
  const [aiModel, setAiModel] = createSignal('gemini-1.5-flash');
  const [apiKey, setApiKey] = createSignal('');

  // Audio & STT
  const [sttProvider, setSttProvider] = createSignal('web_speech');
  const [micDevice, setMicDevice] = createSignal('default');
  const [micThreshold, setMicThreshold] = createSignal(35);

  // TTS Engine
  const [ttsProvider, setTtsProvider] = createSignal('edge_tts');
  const [ttsVoice, setTtsVoice] = createSignal('id-ID-ArdiNeural');
  const [ttsSpeed, setTtsSpeed] = createSignal(1.0);
  const [ttsPitch, setTtsPitch] = createSignal(1.0);

  // Model & Asset Path
  const [vrmModelPath, setVrmModelPath] = createSignal('assets/char.vrm');
  const [isSaving, setIsSaving] = createSignal(false);

  const handleSave = (e: Event) => {
    e.preventDefault();
    setIsSaving(true);
    setTimeout(() => {
      setIsSaving(false);
      props.showToast('Semua konfigurasi berhasil disimpan ke config.json!');
    }, 800);
  };

  const handleVrmUpload = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) {
      setVrmModelPath(`assets/${target.files[0].name}`);
      props.showToast(`Model "${target.files[0].name}" siap dimuat ke engine`);
    }
  };

  return (
    <form onSubmit={handleSave} class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48] flex items-center justify-between">
        <div>
          <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#689D4B]/20 text-[#91AE6E] border border-[#689D4B]/30">
            System & Engine Preferences
          </span>
          <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
            PENGATURAN LENGKAP SISTEM
          </h2>
          <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-xl">
            Atur provider AI LLM, engine Speech-to-Text (STT), model suara TTS, dan parameter overlay desktop native.
          </p>
        </div>

        <button
          type="submit"
          class="px-6 py-3 rounded-xl font-bold text-sm bg-[#D96868] hover:bg-[#c45a5a] text-white shadow-md anime-shadow-red flex items-center gap-2 transition-all shrink-0"
        >
          <Save size={16} />
          <span>{isSaving() ? 'Menyimpan...' : 'Simpan Semua'}</span>
        </button>
      </div>

      {/* 1. Pengaturan AI Brain (LLM Provider) */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#D96868] flex items-center gap-2">
          <Brain size={18} />
          1. AI BRAIN (LLM PROVIDER)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">AI Provider</label>
            <select
              value={aiProvider()}
              onChange={(e) => setAiProvider(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none"
            >
              <option value="gemini">Google Gemini (Disarankan / Cepat)</option>
              <option value="openai">OpenAI (GPT-4o / GPT-3.5)</option>
              <option value="groq">Groq / Ollama (Local Offline)</option>
              <option value="claude">Anthropic Claude</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Model Name</label>
            <input
              type="text"
              value={aiModel()}
              onInput={(e) => setAiModel(e.currentTarget.value)}
              placeholder="gemini-1.5-flash"
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none"
            />
          </div>
        </div>

        <div class="space-y-1.5">
          <label class="text-xs font-semibold text-[#F2F2F2]/80">API Key Token</label>
          <div class="relative">
            <input
              type="password"
              value={apiKey()}
              onInput={(e) => setApiKey(e.currentTarget.value)}
              placeholder="Masukkan API Key (cth: AIzaSy... / sk-...)"
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none font-mono"
            />
          </div>
        </div>
      </div>

      {/* 2. Audio & Speech-to-Text (STT) */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Mic size={18} />
          2. AUDIO & SPEECH-TO-TEXT (STT)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">STT Engine Provider</label>
            <select
              value={sttProvider()}
              onChange={(e) => setSttProvider(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none"
            >
              <option value="web_speech">WebSpeech API (Real-time Browser)</option>
              <option value="whisper">OpenAI Whisper (Akurat)</option>
              <option value="vosk">Vosk Offline (On-Device)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/80">Sensitivitas Input Mic</span>
              <span class="text-[#91AE6E] font-mono">{micThreshold()} dB</span>
            </div>
            <input
              type="range"
              min="10"
              max="80"
              value={micThreshold()}
              onInput={(e) => setMicThreshold(parseInt(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#689D4B] mt-2"
            />
          </div>
        </div>
      </div>

      {/* 3. Text-to-Speech (TTS) Voice */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#D96868] flex items-center gap-2">
          <Volume2 size={18} />
          3. SUARA KARAKTER (TEXT-TO-SPEECH)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">TTS Engine</label>
            <select
              value={ttsProvider()}
              onChange={(e) => setTtsProvider(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none"
            >
              <option value="edge_tts">Microsoft Edge TTS (Sangat Jernih & Ringan)</option>
              <option value="piper">Piper Local TTS (Offline Cepat)</option>
              <option value="elevenlabs">ElevenLabs (High-Fidelity AI)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Pilihan Suara Karakter</label>
            <select
              value={ttsVoice()}
              onChange={(e) => setTtsVoice(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none"
            >
              <option value="id-ID-ArdiNeural">id-ID-ArdiNeural (Pria Casual Indonesia)</option>
              <option value="id-ID-GadisNeural">id-ID-GadisNeural (Wanita Ramah Indonesia)</option>
              <option value="ja-JP-NanamiNeural">ja-JP-NanamiNeural (Anime Voice Jepang)</option>
              <option value="en-US-JennyNeural">en-US-JennyNeural (English Natural)</option>
            </select>
          </div>
        </div>
      </div>

      {/* 4. Desktop Overlay & Model VRM Path */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Monitor size={18} />
          4. DESKTOP OVERLAY & ASSET VRM
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Target Frame Rate (FPS)</label>
            <select
              value={overlayFps()}
              onChange={(e) => setOverlayFps(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2.5 text-sm text-[#F2F2F2] outline-none"
            >
              <option value="60">60 FPS (Standar Halus)</option>
              <option value="120">120 FPS (High Refresh Rate)</option>
              <option value="30">30 FPS (Hemat Baterai Laptop)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">File Model VRM Aktif</label>
            <div class="flex gap-2">
              <input
                type="text"
                value={vrmModelPath()}
                readOnly
                class="flex-1 bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2 text-xs font-mono text-[#91AE6E] outline-none"
              />
              <label class="px-3 py-2 rounded-xl bg-[#282832] hover:bg-[#323240] text-xs font-bold text-[#F2F2F2] border border-[#3B3B48] cursor-pointer flex items-center gap-1.5">
                <Upload size={14} />
                <span>Upload</span>
                <input type="file" accept=".vrm,.glb" onChange={handleVrmUpload} class="hidden" />
              </label>
            </div>
          </div>
        </div>
      </div>
    </form>
  );
};
