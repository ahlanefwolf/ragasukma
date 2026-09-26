import { Component, createSignal, onMount, onCleanup, createEffect } from 'solid-js';
import { Mic, Volume2, Play, Square } from 'lucide-solid';

interface AudioTtsSectionProps {
  assistantGender: string;
  setAssistantGender?: (val: string) => void;
  sttLang: string;
  setSttLang: (val: string) => void;
  ttsVoiceName: string;
  setTtsVoiceName: (val: string) => void;
  ttsSpeed: number;
  setTtsSpeed: (val: number) => void;
  ttsPitch: number;
  setTtsPitch: (val: number) => void;
  voiceInterruption: boolean;
  setVoiceInterruption: (val: boolean) => void;
}

export const AudioTtsSection: Component<AudioTtsSectionProps> = (props) => {
  const [browserVoices, setBrowserVoices] = createSignal<SpeechSynthesisVoice[]>([]);
  const [isPlayingTest, setIsPlayingTest] = createSignal(false);
  const [testText, setTestText] = createSignal('Halo Mas! Aku adalah asisten AI Ragasukma.');
  const [genderFilter, setGenderFilter] = createSignal<'all' | 'female' | 'male'>('all');

  // Sinkronkan filter awal dengan gender asisten jika ada
  createEffect(() => {
    const g = (props.assistantGender || '').toLowerCase();
    if (g.includes('wanita') || g.includes('female') || g.includes('cewek')) {
      setGenderFilter('female');
    } else if (g.includes('pria') || g.includes('male') || g.includes('cowok')) {
      setGenderFilter('male');
    } else {
      setGenderFilter('all');
    }
  });

  const MALE_KEYWORDS = [
    'male', 'david', 'mark', 'george', 'james', 'richard', 'daniel', 'paul',
    'ardi', 'guy', 'stefan', 'martin', 'claude', 'henri', 'pablo',
    'alva', 'diego', 'jorge', 'cosimo', 'yuri', 'alex', 'fred', 'albert',
    'bad news', 'bahh', 'bells', 'boing', 'bubbles', 'cellos', 'deranged',
    'good news', 'hysterical', 'pipe organ', 'trinoids', 'whisper', 'zarvox',
    'junior', 'ralph', 'bruce', 'oliver', 'liam', 'noah', 'william', 'lucas',
    'tom', 'ken', 'hiroshi', 'ryo', 'min-ho', 'jun-seo'
  ];

  const FEMALE_KEYWORDS = [
    'female', 'woman', 'girl', 'zira', 'susan', 'hazel', 'catherine',
    'linda', 'heather', 'jenny', 'aria', 'sara', 'damayanti', 'siti',
    'gadis', 'indonesian female', 'google bahasa indonesia', 'natural',
    'victoria', 'samantha', 'karen', 'moira', 'tessa', 'fiona', 'veena',
    'alice', 'amelie', 'anna', 'helena', 'kyoko', 'yuna', 'sin-ji', 'ting-ting',
    'ayumi', 'haruka', 'sayaka', 'mei-ling', 'ji-young', 'so-yeon'
  ];

  const matchesGender = (voice: SpeechSynthesisVoice, gender: 'all' | 'female' | 'male'): boolean => {
    if (gender === 'all') return true;
    const nameLower = voice.name.toLowerCase();

    if (gender === 'female') {
      if (MALE_KEYWORDS.some(kw => nameLower.includes(kw))) return false;
      if (FEMALE_KEYWORDS.some(hint => nameLower.includes(hint))) return true;
      if (voice.lang.toLowerCase().includes('id')) return true; // Default Google Bahasa Indonesia adalah vokal cewek
      return true; // Suara umum yang tidak terindikasi pria tetap diikutsertakan
    }

    if (gender === 'male') {
      if (FEMALE_KEYWORDS.some(kw => nameLower.includes(kw))) return false;
      if (MALE_KEYWORDS.some(hint => nameLower.includes(hint))) return true;
      return false;
    }

    return true;
  };

  const isLocalLang = (code: string) => {
    const c = (code || '').toLowerCase();
    return c.includes('id') || c.includes('jv') || c.includes('su');
  };

  const filteredVoices = () => {
    const all = browserVoices();
    const filter = genderFilter();
    const matched = all.filter(v => matchesGender(v, filter));
    const list = matched.length > 0 ? matched : all;

    return [...list].sort((a, b) => {
      const aIsLocal = isLocalLang(a.lang);
      const bIsLocal = isLocalLang(b.lang);
      if (aIsLocal && !bIsLocal) return -1;
      if (!aIsLocal && bIsLocal) return 1;
      return a.name.localeCompare(b.name);
    });
  };

  const updateVoiceList = () => {
    if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
      const allVoices = window.speechSynthesis.getVoices();
      if (allVoices.length > 0) {
        setBrowserVoices(allVoices);

        if (!props.ttsVoiceName) {
          const idVoice = allVoices.find(v => isLocalLang(v.lang));
          const defaultVoice = idVoice || allVoices[0];
          props.setTtsVoiceName(defaultVoice.name);
        }
      }
    }
  };

  onMount(() => {
    updateVoiceList();
    if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
      window.speechSynthesis.onvoiceschanged = updateVoiceList;
      // Polling voices selama 3 detik pertama untuk antisipasi Chromium delayed voice loading
      const timer = setInterval(() => {
        if (browserVoices().length === 0) {
          updateVoiceList();
        } else {
          clearInterval(timer);
        }
      }, 300);
      setTimeout(() => clearInterval(timer), 3000);
    }
  });

  onCleanup(() => {
    if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
      window.speechSynthesis.cancel();
    }
  });

  const handleTestVoice = () => {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) return;

    if (isPlayingTest()) {
      window.speechSynthesis.cancel();
      setIsPlayingTest(false);
      return;
    }

    window.speechSynthesis.cancel();
    const utter = new SpeechSynthesisUtterance(testText());
    
    // Pilih voice langsung dari browser
    const voices = browserVoices();
    const selected = voices.find(v => v.name === props.ttsVoiceName);
    if (selected) {
      utter.voice = selected;
      utter.lang = selected.lang;
    } else if (props.sttLang) {
      utter.lang = props.sttLang;
    }

    utter.rate = props.ttsSpeed || 1.0;
    utter.pitch = props.ttsPitch || 1.0;

    utter.onstart = () => setIsPlayingTest(true);
    utter.onend = () => setIsPlayingTest(false);
    utter.onerror = () => setIsPlayingTest(false);

    window.speechSynthesis.speak(utter);
  };

  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 3. Audio & Speech-to-Text (STT Google Chrome) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-bold text-theme-cyan flex items-center gap-2">
          <Mic size={18} />
          3. Mikrofon & Speech-to-Text (Ekosistem Google Chrome)
        </h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 items-center">
          <div class="space-y-1.5">
            <div class="flex justify-between items-center">
              <label class="text-xs font-semibold text-theme-text/80">Bahasa Pengenalan Suara (STT)</label>
              <div class="flex gap-1">
                <button
                  type="button"
                  onClick={() => props.setSttLang('id-ID')}
                  class={`px-1.5 py-0.5 text-[10px] font-mono border ${
                    props.sttLang === 'id-ID'
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                      : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-cyan'
                  }`}
                >
                  id-ID
                </button>
                <button
                  type="button"
                  onClick={() => props.setSttLang('jv-ID')}
                  class={`px-1.5 py-0.5 text-[10px] font-mono border ${
                    props.sttLang === 'jv-ID'
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                      : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-cyan'
                  }`}
                >
                  jv-ID (Jawa)
                </button>
                <button
                  type="button"
                  onClick={() => props.setSttLang('su-ID')}
                  class={`px-1.5 py-0.5 text-[10px] font-mono border ${
                    props.sttLang === 'su-ID'
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                      : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-cyan'
                  }`}
                >
                  su-ID (Sunda)
                </button>
              </div>
            </div>
            <select
              value={props.sttLang || 'id-ID'}
              onChange={(e) => props.setSttLang(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="id-ID">🇮🇩 Bahasa Indonesia (id-ID)</option>
              <option value="jv-ID">🇮🇩 Basa Jawa (jv-ID)</option>
              <option value="su-ID">🇮🇩 Basa Sunda (su-ID)</option>
              <option value="en-US">🇺🇸 English (en-US)</option>
              <option value="ja-JP">🇯🇵 Japanese (ja-JP)</option>
              {Array.from(new Set(browserVoices().map((v) => v.lang)))
                .filter((l) => !['id-ID', 'jv-ID', 'su-ID', 'en-US', 'ja-JP'].includes(l))
                .sort()
                .map((lang) => (
                  <option value={lang}>🌐 {lang}</option>
                ))}
            </select>
          </div>

          <div class="p-3 bg-theme-card border border-theme-border flex items-center justify-between gap-4">
            <div>
              <label class="text-xs sm:text-sm font-semibold text-theme-text block">⚡ Interupsi Suara (Barge-in)</label>
              <p class="text-xs text-theme-text/60 mt-0.5">
                Potong pembicaraan asisten seketika saat kamu bicara.
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
      </div>

      {/* 4. Text-to-Speech (TTS Google Chrome Native) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <h3 class="text-base sm:text-lg font-bold text-theme-rose flex items-center gap-2">
            <Volume2 size={18} />
            4. Suara Asisten (Speech Synthesis Google Chrome)
          </h3>
          <div class="flex items-center gap-2">
            <span class="text-xs text-theme-text/60">Filter Suara:</span>
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
                Semua ({browserVoices().length})
              </button>
              <button
                type="button"
                onClick={() => setGenderFilter('female')}
                class={`px-2 py-1 text-xs transition-colors ${
                  genderFilter() === 'female'
                    ? 'bg-theme-rose text-white font-bold'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                👩 Wanita
              </button>
              <button
                type="button"
                onClick={() => setGenderFilter('male')}
                class={`px-2 py-1 text-xs transition-colors ${
                  genderFilter() === 'male'
                    ? 'bg-theme-blue text-white font-bold'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                👨 Pria
              </button>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 items-start">
          <div class="space-y-1.5 sm:col-span-1">
            <div class="flex justify-between items-center">
              <label class="text-xs font-semibold text-theme-text/80">Pilih Suara Vokal (Dari Browser)</label>
              <span class="text-[10px] text-theme-text/50 font-mono">
                {filteredVoices().length} suara
              </span>
            </div>
            <select
              value={props.ttsVoiceName}
              onChange={(e) => props.setTtsVoiceName(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              {filteredVoices().map((v) => {
                const isIndo = isLocalLang(v.lang);
                const isMale = matchesGender(v, 'male');
                const icon = isMale ? '👨' : '👩';
                return (
                  <option
                    value={v.name}
                    class={isIndo ? 'font-bold text-theme-cyan' : ''}
                  >
                    {icon} {v.name} ({v.lang}){v.default ? ' — DEFAULT' : ''}
                  </option>
                );
              })}
              {filteredVoices().length === 0 && (
                <option value="">Memuat suara browser...</option>
              )}
            </select>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between items-center text-xs font-semibold text-theme-text/80">
              <span>Kecepatan Bicara (Speed)</span>
              <span class="text-xs font-mono font-bold text-theme-cyan bg-theme-card px-2 py-0.5 border border-theme-border">
                {(props.ttsSpeed || 1.0).toFixed(2)}x
              </span>
            </div>
            <div class="h-[42px] px-3.5 bg-theme-card border border-theme-border flex items-center">
              <input
                type="range"
                min="0.5"
                max="2.0"
                step="0.05"
                value={props.ttsSpeed || 1.0}
                onInput={(e) => props.setTtsSpeed(parseFloat(e.currentTarget.value))}
                class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-cyan"
              />
            </div>
          </div>

          <div class="space-y-1.5">
            <div class="flex justify-between items-center text-xs font-semibold text-theme-text/80">
              <span>Nada Suara (Pitch)</span>
              <span class="text-xs font-mono font-bold text-theme-cyan bg-theme-card px-2 py-0.5 border border-theme-border">
                {(props.ttsPitch || 1.0).toFixed(2)}x
              </span>
            </div>
            <div class="h-[42px] px-3.5 bg-theme-card border border-theme-border flex items-center">
              <input
                type="range"
                min="0.5"
                max="2.0"
                step="0.05"
                value={props.ttsPitch || 1.0}
                onInput={(e) => props.setTtsPitch(parseFloat(e.currentTarget.value))}
                class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-cyan"
              />
            </div>
          </div>
        </div>

        {/* Test Suara Banner */}
        <div class="p-3.5 bg-theme-card border border-theme-border flex flex-col sm:flex-row items-center gap-3 justify-between">
          <input
            type="text"
            value={testText()}
            onInput={(e) => setTestText(e.currentTarget.value)}
            placeholder="Tulis kalimat uji suara..."
            class="w-full sm:flex-1 bg-theme-bg border border-theme-border px-3 py-2 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
          />
          <button
            type="button"
            onClick={handleTestVoice}
            class={`w-full sm:w-auto px-4 py-2 text-xs font-semibold flex items-center justify-center gap-2 border transition-all cursor-pointer ${
              isPlayingTest()
                ? 'bg-red-500/20 text-red-400 border-red-500/40 hover:bg-red-500/30'
                : 'bg-theme-cyan text-theme-card border-theme-cyan hover:opacity-90'
            }`}
          >
            {isPlayingTest() ? (
              <>
                <Square size={14} />
                <span>Hentikan Tes</span>
              </>
            ) : (
              <>
                <Play size={14} />
                <span>Uji Suara Chrome</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
