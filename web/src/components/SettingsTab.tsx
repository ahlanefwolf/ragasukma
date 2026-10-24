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
  isActive?: boolean;
}

type SettingsSubTab = 'persona_ai' | 'audio_tts' | 'avatar_model' | 'system_overlay';

export const SettingsTab: Component<SettingsTabProps> = (props) => {
  const [activeSubTab, setActiveSubTab] = createSignal<SettingsSubTab>('persona_ai');

  // Assistant Persona & Call
  const [assistantName, setAssistantName] = createSignal('Sukma');
  const [userCallsign, setUserCallsign] = createSignal('Mas');
  const [assistantGender, setAssistantGender] = createSignal('female');
  const [personalityRole, setPersonalityRole] = createSignal('tsundere');

  // AI LLM Provider
  const [llmProvider, setLlmProvider] = createSignal('custom');
  const [llmBaseUrl, setLlmBaseUrl] = createSignal('');
  const [llmModel, setLlmModel] = createSignal('');
  const [llmApiKey, setLlmApiKey] = createSignal('');

  // Audio & STT
  const [voiceInterruption, setVoiceInterruption] = createSignal(true);
  const [vadThreshold, setVadThreshold] = createSignal(0.5);
  const [vadNoiseSuppression, setVadNoiseSuppression] = createSignal(true);

  // TTS & STT (Chrome Web Speech API Ecosystem)
  const [lang, setLang] = createSignal('id');
  const [ttsVoiceName, setTtsVoiceName] = createSignal('F2');
  const [ttsSpeed, setTtsSpeed] = createSignal(1.05);
  const [ttsVolume, setTtsVolume] = createSignal(1.6);

  // Avatar & System
  const [avatarType, setAvatarType] = createSignal('vrm');
  const [alwaysOnTop, setAlwaysOnTop] = createSignal(true);
  const [isSaving, setIsSaving] = createSignal(false);
  const [assetInfo, setAssetInfo] = createSignal<AssetInfo | null>(null);
  const [isUploadingVrm, setIsUploadingVrm] = createSignal(false);
  const [isUploadingAvatar, setIsUploadingAvatar] = createSignal(false);
  const [selectedAvatarSlot, setSelectedAvatarSlot] = createSignal('idle');

  let isComponentMounted = true;

  onMount(() => {
    isComponentMounted = true;
    loadConfigData();

    const handleExternalConfigUpdate = (e: CustomEvent) => {
      const updated = e.detail;
      if (!isComponentMounted || !updated) return;
      if (updated.voice_interruption !== undefined) {
        setVoiceInterruption(Boolean(updated.voice_interruption));
      }
      if (updated.vad_threshold !== undefined) {
        setVadThreshold(Number(updated.vad_threshold));
      }
      if (updated.vad_noise_suppression !== undefined) {
        setVadNoiseSuppression(Boolean(updated.vad_noise_suppression));
      }
    };

    window.addEventListener('ragasukma:config_updated' as any, handleExternalConfigUpdate);
    onCleanup(() => {
      window.removeEventListener('ragasukma:config_updated' as any, handleExternalConfigUpdate);
    });
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
      if (cfg.assistant_name !== undefined) setAssistantName(cfg.assistant_name);
      if (cfg.user_callsign !== undefined) setUserCallsign(cfg.user_callsign);
      if (cfg.assistant_gender !== undefined) setAssistantGender(cfg.assistant_gender);
      if (cfg.personality_role !== undefined) setPersonalityRole(cfg.personality_role);

      if (cfg.llm_provider !== undefined) setLlmProvider(cfg.llm_provider);
      if (cfg.llm_base_url !== undefined) setLlmBaseUrl(cfg.llm_base_url);
      if (cfg.llm_model !== undefined) setLlmModel(cfg.llm_model);
      if (cfg.llm_api_key !== undefined) setLlmApiKey(cfg.llm_api_key);

      if (cfg.voice_interruption !== undefined) setVoiceInterruption(cfg.voice_interruption);
      if (cfg.vad_threshold !== undefined) setVadThreshold(cfg.vad_threshold);
      if (cfg.vad_noise_suppression !== undefined) setVadNoiseSuppression(cfg.vad_noise_suppression);

      if (cfg.lang !== undefined) setLang(cfg.lang);
      if (cfg.tts_voice_name !== undefined) setTtsVoiceName(cfg.tts_voice_name);
      if (cfg.tts_speed !== undefined) setTtsSpeed(cfg.tts_speed);
      if (cfg.tts_volume !== undefined) setTtsVolume(cfg.tts_volume);

      if (cfg.avatar_type !== undefined) setAvatarType(cfg.avatar_type);
      if (cfg.always_on_top !== undefined) setAlwaysOnTop(cfg.always_on_top);
    }
    if (isComponentMounted) {
      await loadAssetInfoData();
    }
  };

  const handleVrmFileChange = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];

    // Regex validation: 3D model is specifically VRM format only (.vrm)
    if (!/\.vrm$/i.test(file.name)) {
      props.showToast('Invalid file format: 3D model must be a .vrm file!');
      target.value = '';
      return;
    }

    setIsUploadingVrm(true);
    props.showToast(`Uploading VRM model: ${file.name}...`);

    const res = await api.uploadVrm(file);
    setIsUploadingVrm(false);
    target.value = '';

    if (res.ok) {
      props.showToast('VRM model uploaded successfully & active on screen!');
      await loadAssetInfoData();
    } else {
      props.showToast(res.message || 'Failed to upload VRM file');
    }
  };

  const handleAvatarImageChange = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];
    const slot = selectedAvatarSlot();

    setIsUploadingAvatar(true);
    props.showToast(`Uploading avatar image (${slot}): ${file.name}...`);

    const ok = await api.uploadAvatarImage(slot, file);
    setIsUploadingAvatar(false);
    target.value = '';

    if (ok) {
      props.showToast(`Avatar image [${slot}] successfully saved to assets/!`);
      await loadAssetInfoData();
    } else {
      props.showToast('Failed to upload avatar image');
    }
  };


  const handleSave = async (e: Event) => {
    e.preventDefault();
    setIsSaving(true);

    const payload: UserConfig = {
      assistant_name: assistantName().trim(),
      user_callsign: userCallsign().trim(),
      assistant_gender: assistantGender(),
      personality_role: personalityRole(),
      voice_interruption: voiceInterruption(),
      vad_threshold: vadThreshold(),
      vad_noise_suppression: vadNoiseSuppression(),
      llm_provider: llmProvider(),
      llm_base_url: llmBaseUrl().trim(),
      llm_model: llmModel().trim(),
      llm_api_key: llmApiKey().trim(),
      lang: lang(),
      tts_voice_name: ttsVoiceName(),
      tts_speed: ttsSpeed(),
      tts_volume: ttsVolume(),
      avatar_type: avatarType(),
      always_on_top: alwaysOnTop(),
    };

    const ok = await api.saveConfig(payload);
    setIsSaving(false);
    if (ok) {
      // Broadcast configuration update directly to Chrome Web Speech API runtime in App.tsx
      window.dispatchEvent(new CustomEvent('ragasukma:config_updated', { detail: payload }));
      props.showToast('All configurations saved to config-ai.json successfully!');
    } else {
      props.showToast('Failed to save to config-ai.json');
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
          <h2 class="font-heading tracking-tight text-theme-text mt-2">
            System Settings
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
            Configure AI persona, audio sensitivity, VRM 3D/2D models, and overlay window preferences.
          </p>
        </div>

        <button
          type="submit"
          disabled={isSaving()}
          class="w-full sm:w-auto px-6 py-3 font-bold text-sm bg-theme-cyan hover:bg-theme-cyan/85 text-theme-card shadow-md anime-shadow-green flex items-center justify-center gap-2 transition-all shrink-0"
        >
          <Save size={16} />
          <span>{isSaving() ? 'Saving...' : 'Save All'}</span>
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
          <span class="truncate">Window & System</span>
        </button>
      </div>

      {/* Sub-Tab Content Sections (Persistent DOM) */}
      <div class={activeSubTab() === 'persona_ai' ? 'block' : 'hidden'}>
        <PersonaAiSection
          isActive={Boolean(props.isActive && activeSubTab() === 'persona_ai')}
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
          ttsVoiceName={ttsVoiceName()}
          setTtsVoiceName={setTtsVoiceName}
        />
      </div>

      <div class={activeSubTab() === 'audio_tts' ? 'block' : 'hidden'}>
        <AudioTtsSection
          assistantGender={assistantGender()}
          setAssistantGender={setAssistantGender}
          lang={lang()}
          setLang={setLang}
          ttsVoiceName={ttsVoiceName()}
          setTtsVoiceName={setTtsVoiceName}
          ttsSpeed={ttsSpeed()}
          setTtsSpeed={setTtsSpeed}
          ttsVolume={ttsVolume()}
          setTtsVolume={setTtsVolume}
          voiceInterruption={voiceInterruption()}
          setVoiceInterruption={setVoiceInterruption}
          vadThreshold={vadThreshold()}
          setVadThreshold={setVadThreshold}
          vadNoiseSuppression={vadNoiseSuppression()}
          setVadNoiseSuppression={setVadNoiseSuppression}
        />
      </div>

      <div class={activeSubTab() === 'avatar_model' ? 'block' : 'hidden'}>
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
      </div>

      <div class={activeSubTab() === 'system_overlay' ? 'block' : 'hidden'}>
        <SystemOverlaySection
          alwaysOnTop={alwaysOnTop()}
          setAlwaysOnTop={setAlwaysOnTop}
        />
      </div>
    </form>
  );
};
