import { Component, createSignal } from 'solid-js';
import { 
  Play, 
  Volume2, 
  Mic, 
  Brain, 
  Coffee, 
  Smile, 
  Heart, 
  Zap,
  Eye,
  RefreshCw
} from 'lucide-solid';

interface StatusTabProps {
  currentState: string;
  setCurrentState: (state: string) => void;
}

export const StatusTab: Component<StatusTabProps> = (props) => {
  const [testSpeechText, setTestSpeechText] = createSignal('');
  const [selectedEmotion, setSelectedEmotion] = createSignal('normal');
  const [isSpeakingTest, setIsSpeakingTest] = createSignal(false);

  const states = [
    { id: 'idle', label: 'Idle / Standby', icon: Coffee, desc: 'Pernapasan santai & kedip natural', color: '#91AE6E' },
    { id: 'listening', label: 'Mendengarkan', icon: Mic, desc: 'Bahu tegak & fokus ke user', color: '#689D4B' },
    { id: 'thinking', label: 'Berpikir / AI Processing', icon: Brain, desc: 'Menunduk & ekspresi menganalisa', color: '#D96868' },
    { id: 'speaking', label: 'Berbicara / Respon', icon: Volume2, desc: 'Gestur tangan aktif & sinkron bibir', color: '#E58383' },
  ];

  const emotions = [
    { id: 'normal', label: 'Natural / Tenang', icon: Smile },
    { id: 'shy', label: 'Malu / Shy', icon: Heart },
    { id: 'attentive', label: 'Fokus / Attentive', icon: Eye },
    { id: 'cheerful', label: 'Ceria / Happy', icon: Zap },
  ];

  const handleTriggerSpeech = () => {
    if (!testSpeechText().trim()) return;
    setIsSpeakingTest(true);
    props.setCurrentState('speaking');

    // Simulasi durasi bicara
    setTimeout(() => {
      setIsSpeakingTest(false);
      props.setCurrentState('idle');
      setTestSpeechText('');
    }, 4000);
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="relative overflow-hidden rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48]">
        <div class="relative z-10 max-w-xl">
          <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#D96868]/20 text-[#D96868] border border-[#D96868]/30">
            Real-Time AI State Switcher
          </span>
          <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
            KONTROL STATUS VISUAL AVATAR
          </h2>
          <p class="text-sm text-[#F2F2F2]/70 mt-1 leading-relaxed">
            Pilih status visual secara langsung atau biarkan AI backend mengubah gestur avatar secara dinamis saat mendengarkan dan merespon.
          </p>
        </div>
        <div class="absolute -right-6 -bottom-8 w-44 h-44 bg-[#D96868]/10 rounded-full blur-3xl pointer-events-none" />
      </div>

      {/* Grid Status Cards */}
      <div>
        <h3 class="text-lg font-bangers text-[#91AE6E] mb-3">
          1. PILIH STATUS AKTIF (STATE)
        </h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          {states.map((st) => {
            const Icon = st.icon;
            const isSelected = () => props.currentState === st.id;

            return (
              <button
                onClick={() => props.setCurrentState(st.id)}
                class={`flex items-start gap-4 p-4 rounded-xl border text-left transition-all duration-200 ${
                  isSelected()
                    ? 'bg-[#282832] border-[#D96868] ring-2 ring-[#D96868]/30 shadow-lg anime-shadow-red'
                    : 'bg-[#1E1E24] border-[#3B3B48] hover:border-[#F2F2F2]/30 hover:bg-[#282832]/60'
                }`}
              >
                <div 
                  class="p-3 rounded-xl shrink-0 transition-transform duration-200"
                  style={{
                    background: isSelected() ? `${st.color}25` : '#181820',
                    color: isSelected() ? st.color : '#F2F2F2',
                    border: `1px solid ${isSelected() ? st.color : '#3B3B48'}`
                  }}
                >
                  <Icon size={24} class="stroke-[2]" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center justify-between">
                    <span class="font-bold text-sm text-[#F2F2F2] truncate">
                      {st.label}
                    </span>
                    {isSelected() && (
                      <span class="text-[10px] px-2 py-0.5 rounded-full font-bold uppercase bg-[#D96868] text-white">
                        Aktif
                      </span>
                    )}
                  </div>
                  <p class="text-xs text-[#F2F2F2]/60 mt-1 leading-normal">
                    {st.desc}
                  </p>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Emotion Selector */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48]">
        <h3 class="text-lg font-bangers text-[#91AE6E] mb-3">
          2. EMOSI & EKSPRESI WAJAH
        </h3>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          {emotions.map((emo) => {
            const Icon = emo.icon;
            const isSelected = () => selectedEmotion() === emo.id;

            return (
              <button
                onClick={() => setSelectedEmotion(emo.id)}
                class={`flex items-center gap-2.5 p-3 rounded-xl border text-sm font-semibold transition-all ${
                  isSelected()
                    ? 'bg-[#689D4B] border-[#689D4B] text-white shadow-md anime-shadow-green'
                    : 'bg-[#282832] border-[#3B3B48] text-[#F2F2F2]/80 hover:bg-[#323240] hover:text-[#F2F2F2]'
                }`}
              >
                <Icon size={18} />
                <span>{emo.label}</span>
              </button>
            );
          })}
        </div>
      </div>

      {/* Test Interactive Speech Bar */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-3">
        <h3 class="text-lg font-bangers text-[#D96868]">
          3. TES RESPON SUARA & GERAK BIBIR
        </h3>
        <p class="text-xs text-[#F2F2F2]/60">
          Ketik kalimat uji coba di bawah untuk menguji animasi gestur bicara avatar.
        </p>

        <div class="flex gap-2 anime-border-focus rounded-xl">
          <input
            type="text"
            value={testSpeechText()}
            onInput={(e) => setTestSpeechText(e.currentTarget.value)}
            placeholder="Ketik sesuatu, contoh: Halo! Ada yang bisa saya bantu hari ini?"
            class="flex-1 bg-[#282832] border border-[#3B3B48] rounded-xl px-4 py-2.5 text-sm text-[#F2F2F2] placeholder-[#F2F2F2]/40 outline-none focus:border-[#D96868] transition-all"
            onKeyDown={(e) => e.key === 'Enter' && handleTriggerSpeech()}
          />
          <button
            onClick={handleTriggerSpeech}
            disabled={isSpeakingTest() || !testSpeechText().trim()}
            class={`px-5 py-2.5 rounded-xl font-bold text-sm flex items-center gap-2 transition-all ${
              isSpeakingTest()
                ? 'bg-[#689D4B] text-white animate-pulse'
                : 'bg-[#D96868] hover:bg-[#c45a5a] text-white shadow-md anime-shadow-red disabled:opacity-40 disabled:cursor-not-allowed'
            }`}
          >
            {isSpeakingTest() ? (
              <>
                <RefreshCw size={16} class="animate-spin" />
                <span>Bicara...</span>
              </>
            ) : (
              <>
                <Play size={16} class="fill-current" />
                <span>Uji Coba</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
