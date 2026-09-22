import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { 
  Save, 
  Brain, 
  Mic, 
  User, 
  Monitor 
} from 'lucide-solid';
import { api, UserConfig, AssetInfo } from '../api';
import { PersonaAiSection } from './settings/PersonaAiSection';
import { AudioTtsSection } from './settings/AudioTtsSection';
import { AvatarModelSection } from './settings/AvatarModelSection';
import { SystemOverlaySection } from './settings/SystemOverlaySection';

interface SettingsTabProps {
  showToast: (msg: string) => void;
}

type SettingsSubTab = 'persona_ai' | 'audio_tts' | 'avatar_model' | 'system_overlay';

export const SettingsTab: Component<SettingsTabProps> = (props) => {
  const [activeSubTab, setActiveSubTab] = createSignal<SettingsSubTab>('persona_ai');

  // Assistant Persona & Call
  const [assistantName, setAssistantName] = createSignal('');
  const [userCallsign, setUserCallsign] = createSignal('');
  const [assistantGender, setAssistantGender] = createSignal('');
  const [personalityRole, setPersonalityRole] = createSignal('');

  // AI LLM Provider
  const [llmProvider, setLlmProvider] = createSignal('');
  const [llmBaseUrl, setLlmBaseUrl] = createSignal('');
  const [llmModel, setLlmModel] = createSignal('');
  const [llmApiKey, setLlmApiKey] = createSignal('');
  const [groqApiKey, setGroqApiKey] = createSignal('');

  // Audio & STT
  const [sttEngine, setSttEngine] = createSignal('');
  const [voiceInterruption, setVoiceInterruption] = createSignal(false);
  const [voiceMode, setVoiceMode] = createSignal('');
  const [micSensitivity, setMicSensitivity] = createSignal('');
  const [vadSilenceLimit, setVadSilenceLimit] = createSignal('');
  const [noiseReduction, setNoiseReduction] = createSignal(0);

  // TTS Engine
  const [ttsEngine, setTtsEngine] = createSignal('');
  const [ttsLang, setTtsLang] = createSignal('');
  const [ttsSpeed, setTtsSpeed] = createSignal(1.0);
  const [ttsElevenLabsKey, setTtsElevenLabsKey] = createSignal('');
  const [elevenLabsVoiceId, setElevenLabsVoiceId] = createSignal('');

  // Avatar & System
  const [avatarType, setAvatarType] = createSignal('');
  const [alwaysOnTop, setAlwaysOnTop] = createSignal(false);
  const [isSaving, setIsSaving] = createSignal(false);
  const [assetInfo, setAssetInfo] = createSignal<AssetInfo | null>(null);
  const [isUploadingVrm, setIsUploadingVrm] = createSignal(false);
  const [isUploadingAvatar, setIsUploadingAvatar] = createSignal(false);
  const [selectedAvatarSlot, setSelectedAvatarSlot] = createSignal('idle');

  let isComponentMounted = true;

  onMount(() => {
    isComponentMounted = true;
    loadConfigData();
  });

  onCleanup(() => {
    isComponentMounted = false;
  });

  const loadAssetInfoData = async () => {
    const info = await api.getAssetsInfo();
    if (isComponentMounted && info) {
      setAssetInfo(info);
    }
  };

  const loadConfigData = async () => {
    const cfg = await api.getConfig();
    if (!isComponentMounted) return;
    if (cfg) {
      setAssistantName(cfg.assistant_name);
      setUserCallsign(cfg.user_callsign);
      setAssistantGender(cfg.assistant_gender);
      setPersonalityRole(cfg.personality_role);

      setLlmProvider(cfg.llm_provider);
      setLlmBaseUrl(cfg.llm_base_url);
      setLlmModel(cfg.llm_model);
      setLlmApiKey(cfg.llm_api_key);
      setGroqApiKey(cfg.groq_api_key);

      setSttEngine(cfg.stt_engine);
      setVoiceInterruption(cfg.voice_interruption);
      setVoiceMode(cfg.voice_mode);
      setMicSensitivity(cfg.mic_sensitivity);
      setVadSilenceLimit(cfg.vad_silence_limit);
      setNoiseReduction(cfg.noise_reduction_level);

      setTtsEngine(cfg.tts_engine);
      setTtsLang(cfg.tts_lang);
      setTtsSpeed(cfg.tts_speed);
      setTtsElevenLabsKey(cfg.tts_eleven_labs_key);
      setElevenLabsVoiceId(cfg.elevenlabs_voice_id);

      setAvatarType(cfg.avatar_type);
      setAlwaysOnTop(cfg.always_on_top);
    }
    if (isComponentMounted) {
      await loadAssetInfoData();
    }
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
      assistant_gender: assistantGender(),
      personality_role: personalityRole(),
      stt_engine: sttEngine(),
      voice_interruption: voiceInterruption(),
      groq_api_key: groqApiKey().trim(),
      llm_provider: llmProvider(),
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
      <div class="bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-4 sm:p-6 border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <span class="px-2.5 py-1 text-xs font-bold uppercase tracking-wider bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
            System & Engine Preferences
          </span>
          <h2 class="text-2xl sm:text-3xl font-carter tracking-wide text-theme-text mt-2">
            PENGATURAN SISTEM
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
            Kelola persona AI, sensitivitas audio, model VRM 3D/2D, serta preferensi jendela overlay.
          </p>
        </div>

        <button
          type="submit"
          disabled={isSaving()}
          class="w-full sm:w-auto px-6 py-3 font-bold text-sm bg-theme-cyan hover:bg-[#4a5a3a] text-theme-card shadow-md anime-shadow-green flex items-center justify-center gap-2 transition-all shrink-0"
        >
          <Save size={16} />
          <span>{isSaving() ? 'Menyimpan...' : 'Simpan Semua'}</span>
        </button>
      </div>

      {/* Sub-Tab Navigation Bar - Segmented Precision Control */}
      <div class="p-1.5 bg-theme-surface border border-theme-border/80 grid grid-cols-2 sm:grid-cols-4 gap-1.5 shadow-sm">
        <button
          type="button"
          onClick={() => setActiveSubTab('persona_ai')}
          class={`flex items-center justify-center gap-2 py-3 px-3 text-xs sm:text-sm font-semibold transition-all duration-150 ${
            activeSubTab() === 'persona_ai'
              ? 'bg-theme-cyan text-theme-card font-bold shadow-md anime-shadow-green border border-theme-cyan/40'
              : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card/80 border border-transparent'
          }`}
        >
          <Brain size={16} class={activeSubTab() === 'persona_ai' ? 'text-theme-card' : 'text-theme-cyan'} />
          <span class="truncate">Persona & AI Brain</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveSubTab('audio_tts')}
          class={`flex items-center justify-center gap-2 py-3 px-3 text-xs sm:text-sm font-semibold transition-all duration-150 ${
            activeSubTab() === 'audio_tts'
              ? 'bg-theme-cyan text-theme-card font-bold shadow-md anime-shadow-green border border-theme-cyan/40'
              : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card/80 border border-transparent'
          }`}
        >
          <Mic size={16} class={activeSubTab() === 'audio_tts' ? 'text-theme-card' : 'text-theme-rose'} />
          <span class="truncate">Audio & TTS Voice</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveSubTab('avatar_model')}
          class={`flex items-center justify-center gap-2 py-3 px-3 text-xs sm:text-sm font-semibold transition-all duration-150 ${
            activeSubTab() === 'avatar_model'
              ? 'bg-theme-cyan text-theme-card font-bold shadow-md anime-shadow-green border border-theme-cyan/40'
              : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card/80 border border-transparent'
          }`}
        >
          <User size={16} class={activeSubTab() === 'avatar_model' ? 'text-theme-card' : 'text-theme-cyan'} />
          <span class="truncate">Model & Avatar</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveSubTab('system_overlay')}
          class={`flex items-center justify-center gap-2 py-3 px-3 text-xs sm:text-sm font-semibold transition-all duration-150 ${
            activeSubTab() === 'system_overlay'
              ? 'bg-theme-cyan text-theme-card font-bold shadow-md anime-shadow-green border border-theme-cyan/40'
              : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card/80 border border-transparent'
          }`}
        >
          <Monitor size={16} class={activeSubTab() === 'system_overlay' ? 'text-theme-card' : 'text-theme-rose'} />
          <span class="truncate">Jendela & Sistem</span>
        </button>
      </div>

      {/* Sub-Tab Content Sections */}
      {activeSubTab() === 'persona_ai' && (
        <PersonaAiSection
          assistantName={assistantName()}
          setAssistantName={setAssistantName}
          userCallsign={userCallsign()}
          setUserCallsign={setUserCallsign}
          assistantGender={assistantGender()}
          setAssistantGender={setAssistantGender}
          personalityRole={personalityRole()}
          setPersonalityRole={setPersonalityRole}
          llmProvider={llmProvider()}
          setLlmProvider={setLlmProvider}
          llmBaseUrl={llmBaseUrl()}
          setLlmBaseUrl={setLlmBaseUrl}
          llmModel={llmModel()}
          setLlmModel={setLlmModel}
          llmApiKey={llmApiKey()}
          setLlmApiKey={setLlmApiKey}
        />
      )}

      {activeSubTab() === 'audio_tts' && (
        <AudioTtsSection
          sttEngine={sttEngine()}
          setSttEngine={setSttEngine}
          groqApiKey={groqApiKey()}
          setGroqApiKey={setGroqApiKey}
          voiceInterruption={voiceInterruption()}
          setVoiceInterruption={setVoiceInterruption}
          micSensitivity={micSensitivity()}
          setMicSensitivity={setMicSensitivity}
          vadSilenceLimit={vadSilenceLimit()}
          setVadSilenceLimit={setVadSilenceLimit}
          noiseReduction={noiseReduction()}
          setNoiseReduction={setNoiseReduction}
          ttsEngine={ttsEngine()}
          setTtsEngine={setTtsEngine}
          ttsSpeed={ttsSpeed()}
          setTtsSpeed={setTtsSpeed}
          ttsElevenLabsKey={ttsElevenLabsKey()}
          setTtsElevenLabsKey={setTtsElevenLabsKey}
          elevenLabsVoiceId={elevenLabsVoiceId()}
          setElevenLabsVoiceId={setElevenLabsVoiceId}
        />
      )}

      {activeSubTab() === 'avatar_model' && (
        <AvatarModelSection
          avatarType={avatarType()}
          setAvatarType={setAvatarType}
          assetInfo={assetInfo()}
          loadAssetInfoData={loadAssetInfoData}
          isUploadingVrm={isUploadingVrm()}
          handleVrmFileChange={handleVrmFileChange}
          isUploadingAvatar={isUploadingAvatar()}
          selectedAvatarSlot={selectedAvatarSlot()}
          setSelectedAvatarSlot={setSelectedAvatarSlot}
          handleAvatarImageChange={handleAvatarImageChange}
        />
      )}

      {activeSubTab() === 'system_overlay' && (
        <SystemOverlaySection
          alwaysOnTop={alwaysOnTop()}
          setAlwaysOnTop={setAlwaysOnTop}
        />
      )}
    </form>
  );
};
