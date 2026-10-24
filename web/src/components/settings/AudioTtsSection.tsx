import { Component, createSignal, onCleanup, createEffect, onMount } from 'solid-js';
import { Mic, Volume2, Play, Square, Sparkles, Zap, Shield, Flame, User, Upload, Trash2, Music, CheckCircle2 } from 'lucide-solid';
import { API_BASE } from '../../services/base';
import { SearchableSelect } from '../SearchableSelect';

interface SupertonicVoice {
  id: string; // M1..M5, F1..F5
  name: string;
  label: string;
  gender: 'male' | 'female';
}

const SUPERTONIC_VOICES: SupertonicVoice[] = [
  { id: 'M1', name: 'M1', label: 'Warm Male', gender: 'male' },
  { id: 'M2', name: 'M2', label: 'Clear Male', gender: 'male' },
  { id: 'M3', name: 'M3', label: 'Deep Male', gender: 'male' },
  { id: 'M4', name: 'M4', label: 'Bright Male', gender: 'male' },
  { id: 'M5', name: 'M5', label: 'Narrator Male', gender: 'male' },
  { id: 'F1', name: 'F1', label: 'Warm Female', gender: 'female' },
  { id: 'F2', name: 'F2', label: 'Clear Female', gender: 'female' },
  { id: 'F3', name: 'F3', label: 'Bright Female', gender: 'female' },
  { id: 'F4', name: 'F4', label: 'Narrator Female', gender: 'female' },
  { id: 'F5', name: 'F5', label: 'Soft Female', gender: 'female' },
];

interface LanguageOption {
  code: string;       // ISO-639-1 (Supertonic tag: <id>, <en>, <ja>, ...)
  chromeCode: string; // BCP-47 for Chrome Web STT (id-ID, en-US, ja-JP, ...)
  label: string;
}

const SUPPORTED_LANGUAGES: LanguageOption[] = [
  { code: 'id', chromeCode: 'id-ID', label: 'Indonesian' },
  { code: 'en', chromeCode: 'en-US', label: 'English' },
  { code: 'ja', chromeCode: 'ja-JP', label: 'Japanese' },
  { code: 'ko', chromeCode: 'ko-KR', label: 'Korean' },
  { code: 'ar', chromeCode: 'ar-SA', label: 'Arabic' },
  { code: 'bg', chromeCode: 'bg-BG', label: 'Bulgarian' },
  { code: 'cs', chromeCode: 'cs-CZ', label: 'Czech' },
  { code: 'da', chromeCode: 'da-DK', label: 'Danish' },
  { code: 'de', chromeCode: 'de-DE', label: 'German' },
  { code: 'el', chromeCode: 'el-GR', label: 'Greek' },
  { code: 'es', chromeCode: 'es-ES', label: 'Spanish' },
  { code: 'et', chromeCode: 'et-EE', label: 'Estonian' },
  { code: 'fi', chromeCode: 'fi-FI', label: 'Finnish' },
  { code: 'fr', chromeCode: 'fr-FR', label: 'French' },
  { code: 'hi', chromeCode: 'hi-IN', label: 'Hindi' },
  { code: 'hr', chromeCode: 'hr-HR', label: 'Croatian' },
  { code: 'hu', chromeCode: 'hu-HU', label: 'Hungarian' },
  { code: 'it', chromeCode: 'it-IT', label: 'Italian' },
  { code: 'lt', chromeCode: 'lt-LT', label: 'Lithuanian' },
  { code: 'lv', chromeCode: 'lv-LV', label: 'Latvian' },
  { code: 'nl', chromeCode: 'nl-NL', label: 'Dutch' },
  { code: 'pl', chromeCode: 'pl-PL', label: 'Polish' },
  { code: 'pt', chromeCode: 'pt-PT', label: 'Portuguese' },
  { code: 'ro', chromeCode: 'ro-RO', label: 'Romanian' },
  { code: 'ru', chromeCode: 'ru-RU', label: 'Russian' },
  { code: 'sk', chromeCode: 'sk-SK', label: 'Slovak' },
  { code: 'sl', chromeCode: 'sl-SI', label: 'Slovenian' },
  { code: 'sv', chromeCode: 'sv-SE', label: 'Swedish' },
  { code: 'tr', chromeCode: 'tr-TR', label: 'Turkish' },
  { code: 'uk', chromeCode: 'uk-UA', label: 'Ukrainian' },
  { code: 'vi', chromeCode: 'vi-VN', label: 'Vietnamese' },
];

const getChromeSttCode = (langCode: string): string => {
  const clean = (langCode || 'id').toLowerCase().split('-')[0];
  const found = SUPPORTED_LANGUAGES.find((l) => l.code === clean);
  return found ? found.chromeCode : 'id-ID';
};

interface AudioTtsSectionProps {
  assistantGender: string;
  setAssistantGender?: (val: string) => void;
  lang: string;
  setLang: (val: string) => void;
  ttsVoiceName: string;
  setTtsVoiceName: (val: string) => void;
  ttsSpeed: number;
  setTtsSpeed: (val: number) => void;
  ttsVolume?: number;
  setTtsVolume?: (val: number) => void;
  voiceInterruption: boolean;
  setVoiceInterruption: (val: boolean) => void;
  vadThreshold: number;
  setVadThreshold: (val: number) => void;
  vadNoiseSuppression: boolean;
  setVadNoiseSuppression: (val: boolean) => void;
}

export const AudioTtsSection: Component<AudioTtsSectionProps> = (props) => {
  const [isPlayingTest, setIsPlayingTest] = createSignal(false);
  const [testText, setTestText] = createSignal('');
  const [genderFilter, setGenderFilter] = createSignal<'all' | 'female' | 'male'>('all');

  // 🎙️ Reference Voice (OpenVoice V2 Tone Converter) State
  const [refVoiceExists, setRefVoiceExists] = createSignal(false);
  const [refVoiceFilename, setRefVoiceFilename] = createSignal('');
  const [refVoiceSizeMb, setRefVoiceSizeMb] = createSignal(0);
  const [isUploadingRef, setIsUploadingRef] = createSignal(false);
  const [isPlayingRefPreview, setIsPlayingRefPreview] = createSignal(false);
  let refFileInputRef: HTMLInputElement | undefined;
  let activeRefAudio: HTMLAudioElement | null = null;

  const loadRefVoiceInfo = async () => {
    try {
      const res = await fetch(`${API_BASE}/api/assets/info`);
      if (res.ok) {
        const data = await res.json();
        setRefVoiceExists(Boolean(data.refvoice_exists));
        setRefVoiceFilename(data.refvoice_filename || '');
        setRefVoiceSizeMb(data.refvoice_size_mb || 0);
      }
    } catch (err) {
      console.warn('[Assets Info Error]:', err);
    }
  };

  onMount(() => {
    loadRefVoiceInfo();
  });

  let activeTestAudio: HTMLAudioElement | null = null;
  let voiceSelectRef: HTMLSelectElement | undefined;
  let langSelectRef: HTMLSelectElement | undefined;

  // 🎙️ Live VAD & Audio Level Meter State
  const [isMicTesting, setIsMicTesting] = createSignal(false);
  const [currentMicLevel, setCurrentMicLevel] = createSignal(0);
  let micAudioCtx: AudioContext | null = null;
  let micStream: MediaStream | null = null;
  let micAnimId: number | null = null;

  const currentLangCode = () => {
    return (props.lang || 'id').toLowerCase().split('-')[0];
  };

  const handleLanguageChange = (code: string) => {
    const cleanCode = code.toLowerCase().split('-')[0];
    props.setLang(cleanCode);
  };

  // Ensure language select ref always syncs with state
  createEffect(() => {
    const code = currentLangCode();
    if (langSelectRef && code) {
      langSelectRef.value = code;
    }
  });

  // Ensure voice select ref & gender filter are always synchronized bidirectionally without revert
  createEffect(() => {
    const v = props.ttsVoiceName || 'F4';
    if (voiceSelectRef) {
      voiceSelectRef.value = v;
    }
    if (v.startsWith('M')) {
      setGenderFilter('male');
    } else if (v.startsWith('F')) {
      setGenderFilter('female');
    }
  });

  const stopMicMeter = () => {
    if (micAnimId) {
      cancelAnimationFrame(micAnimId);
      micAnimId = null;
    }
    if (micStream) {
      micStream.getTracks().forEach((t) => t.stop());
      micStream = null;
    }
    if (micAudioCtx) {
      micAudioCtx.close().catch(() => {});
      micAudioCtx = null;
    }
    setCurrentMicLevel(0);
    setIsMicTesting(false);
  };

  const startMicMeter = async () => {
    stopMicMeter();
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          noiseSuppression: props.vadNoiseSuppression,
          echoCancellation: true,
          autoGainControl: true,
        },
      });
      micStream = stream;
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      micAudioCtx = ctx;

      const source = ctx.createMediaStreamSource(stream);
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 512;
      analyser.smoothingTimeConstant = 0.3;
      source.connect(analyser);

      const buffer = new Uint8Array(analyser.frequencyBinCount);
      setIsMicTesting(true);

      const loop = () => {
        analyser.getByteTimeDomainData(buffer);
        let sumSquares = 0;
        for (let i = 0; i < buffer.length; i++) {
          const norm = (buffer[i] - 128) / 128;
          sumSquares += norm * norm;
        }
        const rms = Math.sqrt(sumSquares / buffer.length);
        const levelPct = Math.min(100, Math.round(rms * 250));
        setCurrentMicLevel(levelPct);
        micAnimId = requestAnimationFrame(loop);
      };
      loop();
    } catch (err) {
      console.warn('[VAD Meter] Failed to access microphone:', err);
      setIsMicTesting(false);
    }
  };

  const toggleMicMeter = () => {
    if (isMicTesting()) {
      stopMicMeter();
    } else {
      startMicMeter();
    }
  };


  onCleanup(() => {
    stopMicMeter();
    if (activeTestAudio) {
      activeTestAudio.pause();
      activeTestAudio = null;
    }
    if (activeRefAudio) {
      activeRefAudio.pause();
      activeRefAudio = null;
    }
  });

  const handleUploadRefVoice = async (file: File) => {
    if (!file) return;
    setIsUploadingRef(true);
    try {
      const ext = file.name.split('.').pop()?.toLowerCase() || 'wav';
      const arrayBuffer = await file.arrayBuffer();
      const res = await fetch(`${API_BASE}/api/upload/refvoice?ext=${ext}&filename=${encodeURIComponent(file.name)}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
        },
        body: arrayBuffer,
      });

      if (!res.ok) {
        throw new Error(`Upload reference voice failed: HTTP ${res.status}`);
      }

      await loadRefVoiceInfo();
    } catch (err) {
      console.error('[Upload Ref Voice Error]:', err);
      alert('Gagal mengunggah file audio referensi: ' + (err instanceof Error ? err.message : String(err)));
    } finally {
      setIsUploadingRef(false);
      if (refFileInputRef) {
        refFileInputRef.value = '';
      }
    }
  };

  const handleDeleteRefVoice = async () => {
    if (isPlayingRefPreview() && activeRefAudio) {
      activeRefAudio.pause();
      activeRefAudio = null;
      setIsPlayingRefPreview(false);
    }
    try {
      const res = await fetch(`${API_BASE}/api/delete/refvoice`, {
        method: 'POST',
      });
      if (!res.ok) {
        throw new Error(`Delete reference voice failed: HTTP ${res.status}`);
      }
      await loadRefVoiceInfo();
    } catch (err) {
      console.error('[Delete Ref Voice Error]:', err);
    }
  };

  const handleToggleRefPreview = () => {
    if (isPlayingRefPreview()) {
      if (activeRefAudio) {
        activeRefAudio.pause();
        activeRefAudio = null;
      }
      setIsPlayingRefPreview(false);
      return;
    }

    if (!refVoiceFilename()) return;

    const audioUrl = `${API_BASE}/api/assets/file/${refVoiceFilename()}?t=${Date.now()}`;
    const audio = new Audio(audioUrl);
    activeRefAudio = audio;
    setIsPlayingRefPreview(true);

    audio.onended = () => {
      setIsPlayingRefPreview(false);
      activeRefAudio = null;
    };
    audio.onerror = () => {
      setIsPlayingRefPreview(false);
      activeRefAudio = null;
    };

    audio.play().catch((e) => {
      console.warn('[Play Ref Audio Error]:', e);
      setIsPlayingRefPreview(false);
    });
  };

  const handleTestVoice = async () => {
    if (isPlayingTest()) {
      if (activeTestAudio) {
        activeTestAudio.pause();
        activeTestAudio = null;
      }
      setIsPlayingTest(false);
      try {
        window.dispatchEvent(new CustomEvent('ragasukma:test_voice', { detail: { active: false } }));
      } catch {}
      return;
    }

    setIsPlayingTest(true);
    try {
      window.dispatchEvent(new CustomEvent('ragasukma:test_voice', { detail: { active: true } }));
    } catch {}

    try {
      const voice = props.ttsVoiceName || 'F4';
      const lang = currentLangCode();
      const speed = props.ttsSpeed || 1.05;

      const res = await fetch(`${API_BASE}/api/tts/synthesize`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          text: testText().trim() || 'Hello! I am your AI assistant Ragasukma.',
          voice,
          speed,
          lang,
        }),
      });

      if (!res.ok) {
        throw new Error(`Sintesis Supertonic HTTP ${res.status}`);
      }

      const blob = await res.blob();
      const audioUrl = URL.createObjectURL(blob);
      const audio = new Audio(audioUrl);
      activeTestAudio = audio;

      audio.onended = () => {
        setIsPlayingTest(false);
        activeTestAudio = null;
        URL.revokeObjectURL(audioUrl);
        try {
          window.dispatchEvent(new CustomEvent('ragasukma:test_voice', { detail: { active: false } }));
        } catch {}
      };

      audio.onerror = () => {
        setIsPlayingTest(false);
        activeTestAudio = null;
        URL.revokeObjectURL(audioUrl);
        try {
          window.dispatchEvent(new CustomEvent('ragasukma:test_voice', { detail: { active: false } }));
        } catch {}
      };

      await audio.play();
    } catch (err) {
      console.warn('[Test Supertonic Error]:', err);
      setIsPlayingTest(false);
      try {
        window.dispatchEvent(new CustomEvent('ragasukma:test_voice', { detail: { active: false } }));
      } catch {}
    }
  };

  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 3. Audio & Speech-to-Text (STT Google Chrome) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-theme-cyan flex items-center gap-2">
          <Mic size={18} />
          3. Microphone & Spoken Language (Chrome STT + Supertonic)
        </h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 items-center">
          <div class="space-y-1.5">
            <div class="flex justify-between items-center">
              <label class="text-xs font-semibold text-theme-text/80">Select Language (Unified & Synced)</label>
              <div class="flex gap-1">
                <button
                  type="button"
                  onClick={() => handleLanguageChange('id')}
                  class={`px-1.5 py-0.5 text-2xs font-mono border ${
                    currentLangCode() === 'id'
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                      : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-cyan'
                  }`}
                >
                  id
                </button>
                <button
                  type="button"
                  onClick={() => handleLanguageChange('en')}
                  class={`px-1.5 py-0.5 text-2xs font-mono border ${
                    currentLangCode() === 'en'
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                      : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-cyan'
                  }`}
                >
                  en
                </button>
                <button
                  type="button"
                  onClick={() => handleLanguageChange('ja')}
                  class={`px-1.5 py-0.5 text-2xs font-mono border ${
                    currentLangCode() === 'ja'
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                      : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-cyan'
                  }`}
                >
                  ja
                </button>
              </div>
            </div>
            <SearchableSelect
              value={currentLangCode()}
              onChange={(code) => handleLanguageChange(code)}
              options={SUPPORTED_LANGUAGES.map((l) => ({
                value: l.code,
                label: `${l.label} (${l.code})`,
                sublabel: `STT Chrome Code: ${l.chromeCode}`,
              }))}
              searchPlaceholder="Search language name or code..."
              buttonClass="w-full flex items-center justify-between px-3.5 py-2.5 bg-theme-card border border-theme-border text-sm text-theme-text font-mono hover:border-theme-cyan transition-colors"
              dropdownClass="absolute left-0 mt-1.5 w-full bg-theme-surface border border-theme-border shadow-2xl z-50 overflow-hidden flex flex-col max-h-72"
            />
          </div>

          <div class="p-3 bg-theme-card border border-theme-border flex items-center justify-between gap-4">
            <div>
              <label class="text-xs sm:text-sm font-semibold text-theme-text flex items-center gap-1.5">
                <Zap size={14} class="text-theme-cyan" />
                <span>Voice Interruption (Barge-in)</span>
              </label>
              <p class="text-xs text-theme-text/60 mt-0.5">
                Instantly interrupt assistant speech when you speak.
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

        {/* 🎚️ VAD (Voice Activity Detection) & Noise Gate Visualizer */}
        <div class="p-4 bg-theme-card/80 border border-theme-border space-y-3.5">
          <div class="flex items-center justify-between flex-wrap gap-2">
            <div>
              <div class="flex items-center gap-2">
                <span class="text-xs sm:text-sm font-bold text-theme-text flex items-center gap-1.5">
                  <Mic size={14} class="text-theme-cyan" />
                  Voice Threshold (VAD Noise Gate)
                </span>
                <span class="text-2xs px-1.5 py-0.5 border border-theme-cyan/40 bg-theme-cyan/10 text-theme-cyan font-mono font-bold">
                  {Math.round((props.vadThreshold || 0.15) * 100)}%
                </span>
              </div>
              <p class="text-xs text-theme-text/60 mt-0.5">
                Only sounds above the threshold trigger interruptions. Background fan/speaker noise below the line is automatically filtered.
              </p>
            </div>

            <button
              type="button"
              onClick={toggleMicMeter}
              class={`px-3 py-1.5 text-xs font-semibold border transition-all flex items-center gap-1.5 ${
                isMicTesting()
                  ? 'bg-gi-status-danger/20 text-gi-status-danger border-gi-status-danger/50 animate-pulse'
                  : 'bg-theme-surface text-theme-cyan border-theme-cyan/40 hover:bg-theme-cyan/10'
              }`}
            >
              <Mic size={13} />
              {isMicTesting() ? 'Stop Mic Test' : 'Live Mic Test'}
            </button>
          </div>

          {/* Meter Audio & Garis Threshold Line */}
          <div class="space-y-1.5">
            <div class="relative w-full h-8 bg-black/40 border border-theme-border overflow-hidden select-none">
              <div
                class={`h-full transition-all duration-75 ${
                  currentMicLevel() >= Math.round((props.vadThreshold || 0.15) * 100)
                    ? 'bg-gradient-to-r from-gi-status-success via-gi-status-warning to-gi-status-danger shadow-[0_0_12px_rgba(224,83,69,0.6)]'
                    : 'bg-gi-status-success/60'
                }`}
                style={{ width: `${currentMicLevel()}%` }}
              ></div>

              <div
                class="absolute top-0 bottom-0 w-0.5 bg-gi-primary z-10 shadow-[0_0_8px_rgba(229,184,92,0.6)]"
                style={{ left: `${Math.round((props.vadThreshold || 0.15) * 100)}%` }}
              >
                <div class="absolute -top-1 -left-1.5 w-3.5 h-3.5 bg-gi-primary rotate-45 border border-black shadow-sm"></div>
              </div>

              <div class="absolute inset-0 flex items-center justify-between px-3 text-xs font-mono pointer-events-none">
                <span class="text-white/60">
                  {isMicTesting() ? `Mic: ${currentMicLevel()}%` : 'Click "Live Mic Test" to calibrate'}
                </span>
                <span
                  class={`font-bold transition-colors ${
                    currentMicLevel() >= Math.round((props.vadThreshold || 0.15) * 100) && isMicTesting()
                      ? 'text-gi-status-danger drop-shadow-md'
                      : 'text-gi-status-warning/80'
                  }`}
                >
                  {currentMicLevel() >= Math.round((props.vadThreshold || 0.15) * 100) && isMicTesting()
                    ? 'ABOVE THRESHOLD (TRIGGER)'
                    : `Threshold: ${Math.round((props.vadThreshold || 0.15) * 100)}%`}
                </span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <span class="text-xs font-mono text-theme-text/50">Sensitive (3%)</span>
              <input
                type="range"
                min="0.03"
                max="0.50"
                step="0.01"
                value={props.vadThreshold || 0.15}
                onInput={(e) => props.setVadThreshold(parseFloat(e.currentTarget.value))}
                class="flex-1 accent-gi-primary cursor-pointer h-1.5 bg-theme-border rounded-none"
              />
              <span class="text-xs font-mono text-theme-text/50">Strict (50%)</span>
            </div>
          </div>

          <div class="flex items-center justify-between pt-2 border-t border-theme-border/60">
            <div>
              <label class="text-xs font-semibold text-theme-text flex items-center gap-1.5">
                <Shield size={14} class="text-theme-cyan" />
                <span>Noise Suppression</span>
              </label>
              <p class="text-xs text-theme-text/60">
                WebRTC filter to suppress laptop fan hum and room reverb.
              </p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer shrink-0">
              <input
                type="checkbox"
                checked={props.vadNoiseSuppression}
                onChange={(e) => props.setVadNoiseSuppression(e.currentTarget.checked)}
                class="sr-only peer"
              />
              <div class="w-9 h-5 bg-theme-border peer-focus:outline-none peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-theme-text after:h-4 after:w-4 after:transition-all peer-checked:bg-theme-cyan"></div>
            </label>
          </div>
        </div>
      </div>

      {/* 4. Text-to-Speech (Supertonic-3 Neural TTS ONNX Runtime) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <h3 class="text-theme-rose flex items-center gap-2">
            <Volume2 size={18} />
            4. Assistant Voice (Supertonic-3 Neural TTS Native Rust)
          </h3>
          <div class="flex items-center gap-2">
            <span class="text-xs text-theme-text/60">Voice Filter:</span>
            <div class="inline-flex border border-theme-border bg-theme-card">
              <button
                type="button"
                onClick={() => setGenderFilter('all')}
                class={`px-2 py-1 text-xs transition-colors ${
                  genderFilter() === 'all'
                    ? 'bg-theme-cyan text-theme-card font-bold'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                All (10)
              </button>
              <button
                type="button"
                onClick={() => {
                  setGenderFilter('female');
                  if (props.ttsVoiceName?.startsWith('M')) {
                    props.setTtsVoiceName('F4');
                  }
                  props.setAssistantGender?.('female');
                }}
                class={`px-2 py-1 text-xs transition-colors ${
                  genderFilter() === 'female'
                    ? 'bg-theme-rose text-white font-bold'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                Female (5)
              </button>
              <button
                type="button"
                onClick={() => {
                  setGenderFilter('male');
                  if (props.ttsVoiceName?.startsWith('F')) {
                    props.setTtsVoiceName('M1');
                  }
                  props.setAssistantGender?.('male');
                }}
                class={`px-2 py-1 text-xs transition-colors ${
                  genderFilter() === 'male'
                    ? 'bg-theme-blue text-white font-bold'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                Male (5)
              </button>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 items-start">
          <div class="space-y-1.5">
            <div class="flex justify-between items-center">
              <label class="text-xs font-semibold text-theme-text/80">Select Supertonic Voice Character</label>
              <span class="text-2xs text-theme-rose font-mono flex items-center gap-1 font-bold">
                <Sparkles size={11} /> 44.1kHz Hi-Fi
              </span>
            </div>
            <select
              ref={voiceSelectRef}
              value={props.ttsVoiceName || 'F4'}
              onChange={(e) => {
                const voiceId = e.currentTarget.value;
                props.setTtsVoiceName(voiceId);
                if (voiceId.startsWith('M')) {
                  setGenderFilter('male');
                  props.setAssistantGender?.('male');
                } else if (voiceId.startsWith('F')) {
                  setGenderFilter('female');
                  props.setAssistantGender?.('female');
                }
              }}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-rose transition-colors font-medium"
            >
              <optgroup label="Female Voices">
                {SUPERTONIC_VOICES.filter((v) => v.gender === 'female').map((v) => (
                  <option value={v.id}>
                    {v.label} ({v.id})
                  </option>
                ))}
              </optgroup>
              <optgroup label="Male Voices">
                {SUPERTONIC_VOICES.filter((v) => v.gender === 'male').map((v) => (
                  <option value={v.id}>
                    {v.label} ({v.id})
                  </option>
                ))}
              </optgroup>
            </select>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between items-center text-xs font-semibold text-theme-text/80">
              <span>Speech Speed</span>
              <span class="text-xs font-mono font-bold text-theme-cyan bg-theme-card px-2 py-0.5 border border-theme-border">
                {(props.ttsSpeed || 1.05).toFixed(2)}x
              </span>
            </div>
            <div class="h-[42px] px-3.5 bg-theme-card border border-theme-border flex items-center">
              <input
                type="range"
                min="0.5"
                max="2.0"
                step="0.05"
                value={props.ttsSpeed || 1.05}
                onInput={(e) => props.setTtsSpeed(parseFloat(e.currentTarget.value))}
                class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-rose"
              />
            </div>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between items-center text-xs font-semibold text-theme-text/80">
              <span>Voice Volume Gain</span>
              <span class="text-xs font-mono font-bold text-theme-rose bg-theme-card px-2 py-0.5 border border-theme-border">
                {Math.round((props.ttsVolume !== undefined ? props.ttsVolume : 1.6) * 100)}%
              </span>
            </div>
            <div class="h-[42px] px-3.5 bg-theme-card border border-theme-border flex items-center">
              <input
                type="range"
                min="0.5"
                max="3.0"
                step="0.1"
                value={props.ttsVolume !== undefined ? props.ttsVolume : 1.6}
                onInput={(e) => props.setTtsVolume?.(parseFloat(e.currentTarget.value))}
                class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-rose"
              />
            </div>
          </div>
        </div>

        {/* Test Voice Banner */}
        <div class="p-3.5 bg-theme-card border border-theme-border flex flex-col sm:flex-row items-center gap-3 justify-between">
          <input
            type="text"
            value={testText()}
            onInput={(e) => setTestText(e.currentTarget.value)}
            placeholder="Write a test sentence... (e.g. Hello! I am your AI assistant.)"
            class="w-full sm:flex-1 bg-theme-bg border border-theme-border px-3 py-2 text-sm text-theme-text outline-none focus:border-theme-rose transition-colors"
          />
          <button
            type="button"
            onClick={handleTestVoice}
            class={`w-full sm:w-auto px-4 py-2 text-xs font-semibold flex items-center justify-center gap-2 border transition-all cursor-pointer ${
              isPlayingTest()
                ? 'bg-gi-status-danger/20 text-gi-status-danger border-gi-status-danger/40 hover:bg-gi-status-danger/30 animate-pulse'
                : 'bg-theme-rose text-white border-theme-rose hover:opacity-90'
            }`}
          >
            {isPlayingTest() ? (
              <>
                <Square size={14} />
                <span>Stop Audio</span>
              </>
            ) : (
              <>
                <Play size={14} />
                <span>Test Supertonic Voice</span>
              </>
            )}
          </button>
        </div>

        {/* 🎙️ Reference Voice (Zero-Shot Clone / Tone Color Converter) */}
        <div class="p-4 bg-theme-card border border-theme-border space-y-3">
          <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
            <div class="space-y-0.5">
              <div class="flex items-center gap-2">
                <Music size={16} class="text-theme-cyan" />
                <span class="text-xs sm:text-sm font-bold text-theme-text">
                  Reference Voice Timbre (Zero-Shot OpenVoice V2)
                </span>
                {refVoiceExists() ? (
                  <span class="inline-flex items-center gap-1 px-1.5 py-0.5 text-2xs font-mono bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 font-semibold">
                    <CheckCircle2 size={10} /> Active Clone
                  </span>
                ) : (
                  <span class="inline-flex items-center gap-1 px-1.5 py-0.5 text-2xs font-mono bg-theme-bg text-theme-text/60 border border-theme-border">
                    Native Supertonic
                  </span>
                )}
              </div>
              <p class="text-xs text-theme-text/60">
                Reference audio is automatically saved as <code class="text-theme-cyan font-mono font-bold">assets/refvoice.*</code>. Leave empty to use native Supertonic 3 voice timbre.
              </p>
            </div>

            <div class="flex items-center gap-2">
              <input
                ref={refFileInputRef}
                type="file"
                accept="audio/wav,audio/mp3,audio/ogg,audio/flac,audio/m4a,audio/aac,.wav,.mp3,.ogg,.flac,.m4a,.aac"
                class="hidden"
                onChange={(e) => {
                  const files = e.currentTarget.files;
                  if (files && files[0]) {
                    handleUploadRefVoice(files[0]);
                  }
                }}
              />

              <button
                type="button"
                disabled={isUploadingRef()}
                onClick={() => refFileInputRef?.click()}
                class="px-3 py-1.5 text-xs font-semibold bg-theme-cyan text-theme-card border border-theme-cyan hover:opacity-90 transition-all flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
              >
                <Upload size={13} />
                <span>{isUploadingRef() ? 'Uploading...' : refVoiceExists() ? 'Replace Audio' : 'Upload Reference Audio'}</span>
              </button>

              {refVoiceExists() && (
                <button
                  type="button"
                  onClick={handleDeleteRefVoice}
                  title="Remove reference voice (fallback to native Supertonic)"
                  class="p-1.5 text-xs font-semibold bg-gi-status-danger/10 text-gi-status-danger border border-gi-status-danger/30 hover:bg-gi-status-danger/20 transition-all cursor-pointer"
                >
                  <Trash2 size={14} />
                </button>
              )}
            </div>
          </div>

          {refVoiceExists() && (
            <div class="p-2.5 bg-theme-bg border border-theme-border flex items-center justify-between gap-3 text-xs">
              <div class="flex items-center gap-2 min-w-0">
                <button
                  type="button"
                  onClick={handleToggleRefPreview}
                  class={`p-1.5 border transition-all cursor-pointer ${
                    isPlayingRefPreview()
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan'
                      : 'bg-theme-card text-theme-cyan border-theme-border hover:border-theme-cyan'
                  }`}
                >
                  {isPlayingRefPreview() ? <Square size={12} /> : <Play size={12} />}
                </button>
                <div class="truncate">
                  <span class="font-mono font-bold text-theme-text">{refVoiceFilename()}</span>
                  <span class="text-theme-text/50 text-xs ml-2">({refVoiceSizeMb().toFixed(2)} MB)</span>
                </div>
              </div>
              <span class="text-xs text-theme-text/60 italic hidden sm:inline">
                OpenVoice V2 Tone Color Converter ONNX Active
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
