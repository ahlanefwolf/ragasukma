import { Component, createSignal } from 'solid-js';
import { 
  Globe, 
  Music, 
  Zap, 
  Terminal, 
  CheckSquare, 
  MousePointerClick, 
  Wand2,
  ChevronDown,
  ChevronUp,
  Command
} from 'lucide-solid';

interface CapabilityItem {
  icon: typeof Globe;
  title: string;
  category: string;
  description: string;
  canonicalCmd: string;
  examples: string[];
  badgeColor: string;
}

export const AiCapabilitiesCard: Component = () => {
  const [isExpanded, setIsExpanded] = createSignal(true);
  const [selectedCategory, setSelectedCategory] = createSignal<string>('all');

  const capabilities: CapabilityItem[] = [
    {
      icon: Globe,
      title: 'Navigasi Browser & Multi-Tab',
      category: 'web',
      description: 'Membuka website, berpindah antar tab yang sedang aktif, dan mencari kata kunci langsung di situs apa pun.',
      canonicalCmd: '[CMD:browser:open|url] / [CMD:browser:switch|tab]',
      examples: [
        '"Buka tab YouTube" / "Buka Tokopedia"',
        '"Pindah ke tab YouTube" / "Tutup tab Google"',
        '"Cari video tutorial Rust di YouTube"'
      ],
      badgeColor: 'text-theme-cyan bg-theme-cyan/15 border-theme-cyan/30'
    },
    {
      icon: MousePointerClick,
      title: 'Interaksi Web (DOM Live Click & Type)',
      category: 'web',
      description: 'Mengklik tulisan/link yang tampil di layar, menekan tombol website, mengetik ke kolom form, dan scroll halaman.',
      canonicalCmd: '[CMD:browser:click_text|teks] / [CMD:browser:type|teks]',
      examples: [
        '"Klik tulisan HIGHLIGHTS" / "Klik judul video"',
        '"Klik tombol Skip" / "Klik Beli Sekarang"',
        '"Ketik berita teknologi lalu tekan enter"',
        '"Scroll ke bawah"'
      ],
      badgeColor: 'text-emerald-400 bg-emerald-500/15 border-emerald-500/30'
    },
    {
      icon: Music,
      title: 'Pemutar Musik & Audio Background',
      category: 'media',
      description: 'Memutar musik offline atau mengunduh otomatis dari YouTube di background worker tanpa freeze.',
      canonicalCmd: '[CMD:media:play|judul] / [CMD:media:pause]',
      examples: [
        '"Putar lagu Denny Caknan Sugeng Dalu"',
        '"Jeda musik" / "Lanjutkan musik"',
        '"Lagu berikutnya" / "Matikan musik"'
      ],
      badgeColor: 'text-theme-rose bg-theme-rose/15 border-theme-rose/30'
    },
    {
      icon: Terminal,
      title: 'Kontrol Sistem & Aplikasi Desktop',
      category: 'system',
      description: 'Membuka aplikasi OS secara langsung, kontrol volume audio komputer, screenshot, dan kunci layar.',
      canonicalCmd: '[CMD:app:open|NamaApp] / [CMD:os:volume_up]',
      examples: [
        '"Buka Steam" / "Buka Spotify"',
        '"Buka terminal" / "Buka VS Code"',
        '"Buka kalkulator" / "Buka File Manager"',
        '"Naikkan volume suara" / "Mute audio"'
      ],
      badgeColor: 'text-sky-400 bg-sky-500/15 border-sky-500/30'
    },
    {
      icon: CheckSquare,
      title: 'Catatan & To-Do List Cerdas',
      category: 'tools',
      description: 'Mencatat agenda baru ke daftar to-do dan menandai tugas yang sudah selesai secara otomatis.',
      canonicalCmd: '[CMD:todo:add|tugas] / [CMD:todo:done|tugas]',
      examples: [
        '"Catat tugas besok meeting jam 9"',
        '"Catat beli perlengkapan kantor"',
        '"Tugas meeting jam 9 sudah selesai"'
      ],
      badgeColor: 'text-purple-400 bg-purple-500/15 border-purple-500/30'
    },
    {
      icon: Zap,
      title: 'Voice Barge-in & Respon Realtime',
      category: 'voice',
      description: 'Bebas memotong atau menyela pembicaraan AI kapan saja dengan VAD (Voice Activity Detection) realtime.',
      canonicalCmd: 'Interupsi Suara Realtime',
      examples: [
        '"Langsung bicara untuk memotong AI"',
        '"Respon suara alami via Piper/Kokoro TTS"',
        '"Deteksi emosi wajah 3D dinamis"'
      ],
      badgeColor: 'text-amber-400 bg-amber-500/15 border-amber-500/30'
    }
  ];

  const filtered = () => {
    const cat = selectedCategory();
    if (cat === 'all') return capabilities;
    return capabilities.filter(c => c.category === cat);
  };

  return (
    <div class="bg-theme-surface border border-theme-border p-5 space-y-4 shadow-sm">
      {/* Header Card */}
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
            <Wand2 size={18} />
          </div>
          <div>
            <h3 class="text-sm sm:text-base font-bold text-theme-text flex items-center gap-2">
              Kemampuan & Panduan Perintah AI
              <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan font-bold">
                {capabilities.length} Fitur
              </span>
            </h3>
            <p class="text-xs text-theme-text/70 mt-0.5">
              Contoh kalimat perintah suara dan tag eksekusi canonical yang didukung
            </p>
          </div>
        </div>

        <button
          onClick={() => setIsExpanded(!isExpanded())}
          class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan transition-all"
          title={isExpanded() ? 'Sembunyikan Panduan' : 'Tampilkan Panduan'}
        >
          {isExpanded() ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        </button>
      </div>

      {isExpanded() && (
        <div class="space-y-4 pt-1 animate-fade-in">
          {/* Category Filter Chips */}
          <div class="flex items-center gap-2 overflow-x-auto pb-1 custom-scrollbar text-xs font-semibold">
            <button
              onClick={() => setSelectedCategory('all')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'all'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Semua ({capabilities.length})
            </button>
            <button
              onClick={() => setSelectedCategory('web')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'web'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Web & Browser
            </button>
            <button
              onClick={() => setSelectedCategory('media')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'media'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Musik & Media
            </button>
            <button
              onClick={() => setSelectedCategory('system')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'system'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Kontrol OS
            </button>
            <button
              onClick={() => setSelectedCategory('tools')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'tools'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              To-Do & Catatan
            </button>
          </div>

          {/* Capabilities Grid */}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
            {filtered().map((cap) => {
              const IconComponent = cap.icon;
              return (
                <div class="p-4 bg-theme-bg border border-theme-border hover:border-theme-cyan/40 transition-all flex flex-col justify-between space-y-3">
                  <div class="space-y-2">
                    <div class="flex items-center justify-between gap-2">
                      <div class="flex items-center gap-2.5">
                        <span class={`p-2 border ${cap.badgeColor}`}>
                          <IconComponent size={16} />
                        </span>
                        <h4 class="text-sm font-bold text-theme-text">
                          {cap.title}
                        </h4>
                      </div>
                      <span class="text-xs font-mono text-theme-cyan flex items-center gap-1 opacity-90">
                        <Command size={12} /> {cap.canonicalCmd.split(' ')[0]}
                      </span>
                    </div>
                    <p class="text-xs text-theme-text/75 leading-relaxed">
                      {cap.description}
                    </p>
                  </div>

                  {/* Example Phrases */}
                  <div class="space-y-1.5 pt-2 border-t border-theme-border/60">
                    <span class="text-xs font-mono uppercase tracking-wider text-theme-text/60 font-bold">
                      Contoh Ucapan:
                    </span>
                    <div class="flex flex-wrap gap-1.5">
                      {cap.examples.map((ex) => (
                        <span class="text-xs px-2.5 py-1 bg-theme-card border border-theme-border text-theme-text/90 font-mono">
                          {ex}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};
