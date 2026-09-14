import { Component, createSignal, onMount } from 'solid-js';
import { 
  Brain, 
  Mic, 
  Volume2, 
  Monitor, 
  Save, 
  Upload,
  Sparkles,
  Image as ImageIcon,
  CheckCircle2,
  RefreshCw
} from 'lucide-solid';
import { api, UserConfig, AssetInfo } from '../api';

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
  const [isSaving, setIsSaving] = createSignal(false);
  const [assetInfo, setAssetInfo] = createSignal<AssetInfo | null>(null);
  const [isUploadingVrm, setIsUploadingVrm] = createSignal(false);
  const [isUploadingAvatar, setIsUploadingAvatar] = createSignal(false);
  const [selectedAvatarSlot, setSelectedAvatarSlot] = createSignal('idle');

  const loadAssetInfoData = async () => {
    const info = await api.getAssetsInfo();
    if (info) {
      setAssetInfo(info);
    }
  };

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
    }
    await loadAssetInfoData();
  };

  const handleVrmFileChange = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];

    setIsUploadingVrm(true);
    props.showToast(`Mengunggah model VRM: ${file.name}...`);

    const res = await api.uploadVrm(file);
    setIsUploadingVrm(false);
    target.value = '';

    if (res.ok) {
      props.showToast('✨ Model VRM berhasil diunggah & langsung aktif di layar!');
      await loadAssetInfoData();
    } else {
      props.showToast(`⚠️ ${res.message || 'Gagal mengunggah file VRM'}`);
    }
  };

  const handleAvatarImageChange = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];
    const slot = selectedAvatarSlot();

    setIsUploadingAvatar(true);
    props.showToast(`Mengunggah gambar avatar (${slot}): ${file.name}...`);

    const ok = await api.uploadAvatarImage(slot, file);
    setIsUploadingAvatar(false);
    target.value = '';

    if (ok) {
      props.showToast(`🎨 Gambar avatar [${slot}] berhasil disimpan ke assets/!`);
      await loadAssetInfoData();
    } else {
      props.showToast('⚠️ Gagal mengunggah gambar avatar');
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

        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">STT Engine</label>
            <select
              value={sttEngine()}
              onChange={(e) => setSttEngine(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="google">Google Cloud STT (Cepat & Akurat ID)</option>
              <option value="groq">Groq Whisper Large v3</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Sensitivitas VAD Mikrofon</label>
            <select
              value={micSensitivity()}
              onChange={(e) => setMicSensitivity(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
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
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Batas Jeda Hening VAD</label>
            <select
              value={vadSilenceLimit()}
              onChange={(e) => setVadSilenceLimit(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="0">Sangat Cepat (~0.5 Detik)</option>
              <option value="1">Cepat (~0.7 Detik)</option>
              <option value="2">Normal (~1.0 Detik)</option>
              <option value="3">Santai (~1.3 Detik)</option>
              <option value="4">Panjang (~1.7 Detik)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Noise Reduction (DSP Filter)</label>
            <select
              value={noiseReduction()}
              onChange={(e) => setNoiseReduction(parseInt(e.currentTarget.value, 10))}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value={0}>0 - Nonaktif (Direct Mic)</option>
              <option value={1}>1 - Ringan (High-Pass + Noise Gate)</option>
              <option value={2}>2 - Agresif (Deep Noise Gate)</option>
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

        {ttsEngine() === 'elevenlabs' && (
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 pt-2 border-t border-[#2F3B46]">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-[#EFF7F6]/80">ElevenLabs API Key</label>
              <input
                type="password"
                value={ttsElevenLabsKey()}
                onInput={(e) => setTtsElevenLabsKey(e.currentTarget.value)}
                placeholder="sk_..."
                class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
              />
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-[#EFF7F6]/80">ElevenLabs Voice ID</label>
              <input
                type="text"
                value={elevenLabsVoiceId()}
                onInput={(e) => setElevenLabsVoiceId(e.currentTarget.value)}
                placeholder="cgSgspJ2msm6clMCkdW9"
                class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
              />
            </div>
          </div>
        )}
      </div>
      {/* 5. Avatar & Model Management */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-5">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
            <Sparkles size={18} />
            5. AVATAR & MODEL ASSETS (VRM / 2D)
          </h3>

          <button
            type="button"
            onClick={loadAssetInfoData}
            class="text-xs text-[#EFF7F6]/60 hover:text-[#7BDFF2] flex items-center gap-1.5 transition-colors"
          >
            <RefreshCw size={13} />
            <span>Refresh Info</span>
          </button>
        </div>

        {/* Avatar Type Selector */}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Tipe Avatar Utama</label>
            <select
              value={avatarType()}
              onChange={(e) => setAvatarType(e.currentTarget.value)}
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2.5 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
            >
              <option value="vrm">VRM 3D Model (Native WGPU GPU Shader)</option>
              <option value="2d">2D Image / Vector Avatar</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Status Model Aktif</label>
            <div class="p-2.5 rounded-xl bg-[#232D36] border border-[#2F3B46] flex items-center justify-between text-xs">
              <div class="flex items-center gap-2 truncate">
                <CheckCircle2 size={15} class="text-[#7BDFF2] shrink-0" />
                <span class="text-[#EFF7F6] font-mono truncate">
                  {assetInfo()?.vrm_exists ? `char.vrm (${assetInfo()?.vrm_size_mb.toFixed(2)} MB)` : 'char.vrm tidak ditemukan'}
                </span>
              </div>
              <span class="text-[10px] px-2 py-0.5 rounded bg-[#7BDFF2]/20 text-[#7BDFF2] font-bold uppercase">
                {assetInfo()?.vrm_exists ? 'Aktif' : 'Kosong'}
              </span>
            </div>
          </div>
        </div>

        {/* VRM 3D Model Upload Card */}
        <div class="p-4 rounded-xl bg-[#232D36] border border-[#2F3B46] space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-bold text-[#EFF7F6] flex items-center gap-2">
              <Upload size={16} class="text-[#7BDFF2]" />
              Upload Model VRM 3D (.vrm / .glb)
            </span>
            <span class="text-xs font-mono text-[#7BDFF2] bg-[#7BDFF2]/10 border border-[#7BDFF2]/20 px-2 py-0.5 rounded">
              Wajib .vrm / .glb
            </span>
          </div>

          <p class="text-xs text-[#EFF7F6]/70 leading-relaxed">
            Format yang didukung: <strong class="text-[#7BDFF2]">.vrm</strong> (standar VRM 0.x/1.0 dari VRoid Studio / Blender) atau <strong class="text-[#7BDFF2]">.glb</strong>. File otomatis disimpan ke <code class="text-[#7BDFF2] font-mono">assets/char.vrm</code> dan langsung dimuat ulang (hot-reload).
          </p>

          <div class="flex items-center gap-3 pt-1">
            <label class="relative cursor-pointer">
              <input
                type="file"
                accept=".vrm,.glb"
                disabled={isUploadingVrm()}
                onChange={handleVrmFileChange}
                class="sr-only"
              />
              <span class={`inline-flex items-center gap-2 px-4 py-2.5 rounded-xl font-bold text-xs transition-all ${
                isUploadingVrm()
                  ? 'bg-[#2F3B46] text-[#EFF7F6]/50 cursor-not-allowed'
                  : 'bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] shadow anime-shadow-green cursor-pointer'
              }`}>
                <Upload size={14} />
                <span>{isUploadingVrm() ? 'Mengunggah & Memuat...' : 'Pilih File VRM (.vrm / .glb)'}</span>
              </span>
            </label>

            {assetInfo()?.vrm_exists && (
              <span class="text-xs text-[#EFF7F6]/60 font-mono">
                Ukuran: {assetInfo()?.vrm_size_mb.toFixed(2)} MB
              </span>
            )}
          </div>
        </div>

        {/* 2D Image Avatar Upload Card */}
        <div class="p-4 rounded-xl bg-[#232D36] border border-[#2F3B46] space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-bold text-[#EFF7F6] flex items-center gap-2">
              <ImageIcon size={16} class="text-[#F2B5D4]" />
              Upload Gambar Avatar 2D (State-Based)
            </span>
            <span class="text-xs font-mono text-[#F2B5D4] bg-[#F2B5D4]/10 border border-[#F2B5D4]/20 px-2 py-0.5 rounded">
              Format: PNG (Transparan) / JPG
            </span>
          </div>

          <div class="p-2.5 rounded-lg bg-[#161D24] border border-[#2F3B46] text-xs text-[#EFF7F6]/80 leading-relaxed space-y-1">
            <div class="flex items-center gap-1.5 font-bold text-[#F2B5D4]">
              <span>Ketentuan Format & Gambar:</span>
            </div>
            <ul class="list-disc list-inside space-y-0.5 text-[11px] text-[#EFF7F6]/70">
              <li><strong class="text-[#EFF7F6]">PNG Transparan (Disarankan)</strong> agar karakter tampil tanpa kotak background.</li>
              <li>Mendukung file <strong class="text-[#EFF7F6]">.png</strong>, <strong class="text-[#EFF7F6]">.jpg</strong>, <strong class="text-[#EFF7F6]">.jpeg</strong> (Resolusi disarankan min 512x512 px).</li>
              <li>Setiap slot ekspresi (<code class="font-mono text-[#7BDFF2]">idle</code>, <code class="font-mono text-[#7BDFF2]">listening</code>, <code class="font-mono text-[#7BDFF2]">talking</code>, <code class="font-mono text-[#7BDFF2]">thinking</code>) dapat memiliki gambar tersendiri.</li>
            </ul>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-[#EFF7F6]/80">Pilih Slot Ekspresi</label>
              <select
                value={selectedAvatarSlot()}
                onChange={(e) => setSelectedAvatarSlot(e.currentTarget.value)}
                class="w-full bg-[#1D252E] border border-[#2F3B46] rounded-xl px-3 py-2 text-xs text-[#EFF7F6] outline-none focus:border-[#F2B5D4]"
              >
                <option value="idle">Idle (Diam / Normal)</option>
                <option value="listening">Listening (Mendengarkan)</option>
                <option value="talking">Talking / Speaking (Bicara)</option>
                <option value="thinking">Thinking (Berpikir)</option>
              </select>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-[#EFF7F6]/80">Pilih File (.png / .jpg / .jpeg)</label>
              <label class="relative block cursor-pointer">
                <input
                  type="file"
                  accept="image/png,image/jpeg,image/jpg"
                  disabled={isUploadingAvatar()}
                  onChange={handleAvatarImageChange}
                  class="sr-only"
                />
                <span class={`inline-flex w-full items-center justify-center gap-2 px-4 py-2 rounded-xl font-bold text-xs transition-all ${
                  isUploadingAvatar()
                    ? 'bg-[#2F3B46] text-[#EFF7F6]/50 cursor-not-allowed'
                    : 'bg-[#F2B5D4] hover:bg-[#e4a4c4] text-[#232D36] shadow cursor-pointer'
                }`}>
                  <Upload size={14} />
                  <span>{isUploadingAvatar() ? 'Mengunggah...' : `Unggah PNG/JPG untuk '${selectedAvatarSlot()}'`}</span>
                </span>
              </label>
            </div>
          </div>

          {assetInfo()?.images && assetInfo()!.images.length > 0 && (
            <div class="pt-2 border-t border-[#2F3B46]/60">
              <span class="text-[11px] text-[#EFF7F6]/60 block mb-1.5">File gambar tersimpan di assets/:</span>
              <div class="flex flex-wrap gap-2">
                {assetInfo()!.images.map((imgName) => (
                  <span class="text-[10px] font-mono px-2 py-1 rounded bg-[#1D252E] border border-[#2F3B46] text-[#7BDFF2]">
                    {imgName}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* 6. Desktop Overlay & Window Preferences */}
      <div class="p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
          <Monitor size={18} />
          6. DESKTOP OVERLAY & WINDOW
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
        </div>
      </div>
    </form>
  );
};
