import { Component, createSignal, Show } from 'solid-js';
import { 
  Globe, 
  Music, 
  Terminal, 
  CheckSquare, 
  MousePointerClick, 
  Wand2,
  ChevronDown,
  ChevronUp,
  Palette,
  Puzzle,
  Layers
} from 'lucide-solid';

interface CapabilityItem {
  id: string;
  icon: typeof Globe;
  title: string;
  category: string;
  description: string;
  examples: string[];
  badgeColor: string;
}

interface AiCapabilitiesCardProps {
  onOpenExtensionModal?: () => void;
  onOpenAppsModal?: () => void;
}

export const AiCapabilitiesCard: Component<AiCapabilitiesCardProps> = (props) => {
  const [isExpanded, setIsExpanded] = createSignal(true);
  const [selectedCategory, setSelectedCategory] = createSignal<string>('all');

  const capabilities: CapabilityItem[] = [
    {
      id: 'browser',
      icon: Globe,
      title: 'Navigasi Browser & Multi-Tab',
      category: 'web',
      description: 'Membuka website, berpindah antar tab yang sedang aktif, dan mencari kata kunci langsung di situs apa pun.',
      examples: [
        '"Buka tab YouTube" / "Buka Tokopedia"',
        '"Pindah ke tab YouTube" / "Tutup tab Google"',
        '"Cari video tutorial Rust di YouTube"'
      ],
      badgeColor: 'text-theme-cyan bg-theme-cyan/15 border-theme-cyan/30'
    },
    {
      id: 'dom',
      icon: MousePointerClick,
      title: 'Interaksi Web (Live Click & Type)',
      category: 'web',
      description: 'Mengklik tulisan/link yang tampil di layar, menekan tombol website, mengetik ke kolom form, dan scroll halaman.',
      examples: [
        '"Klik tulisan Highlights" / "Klik judul video"',
        '"Klik tombol Skip" / "Klik Beli Sekarang"',
        '"Ketik berita teknologi lalu tekan enter"',
        '"Scroll ke bawah"'
      ],
      badgeColor: 'text-[#29483A] bg-[#29483A]/12 border-[#29483A]/30'
    },
    {
      id: 'media',
      icon: Music,
      title: 'Pemutar Musik & Audio Background',
      category: 'media',
      description: 'Memutar musik offline atau mengunduh otomatis dari YouTube di background worker tanpa freeze.',
      examples: [
        '"Putar lagu Denny Caknan Sugeng Dalu"',
        '"Jeda musik" / "Lanjutkan musik"',
        '"Lagu berikutnya" / "Matikan musik"'
      ],
      badgeColor: 'text-[#B08A4A] bg-[#B08A4A]/15 border-[#B08A4A]/30'
    },
    {
      id: 'system',
      icon: Terminal,
      title: 'Kontrol Sistem & Aplikasi Desktop',
      category: 'system',
      description: 'Membuka aplikasi OS secara langsung, kontrol volume audio komputer, screenshot, dan kunci layar.',
      examples: [
        '"Buka Steam" / "Buka Spotify"',
        '"Buka terminal" / "Buka VS Code"',
        '"Buka kalkulator" / "Buka File Manager"',
        '"Naikkan volume suara" / "Mute audio"'
      ],
      badgeColor: 'text-[#876447] bg-[#876447]/15 border-[#876447]/30'
    },
    {
      id: 'todo',
      icon: CheckSquare,
      title: 'Catatan & To-Do List Cerdas',
      category: 'tools',
      description: 'Mencatat agenda baru ke daftar to-do dan menandai tugas yang sudah selesai secara otomatis.',
      examples: [
        '"Catat tugas besok meeting jam 9"',
        '"Catat beli perlengkapan kantor"',
        '"Tugas meeting jam 9 sudah selesai"'
      ],
      badgeColor: 'text-[#634936] bg-[#634936]/15 border-[#634936]/30'
    },
    {
      id: 'canvas',
      icon: Palette,
      title: 'Whiteboard 2D & Diagram Visual',
      category: 'canvas',
      description: 'Menginstruksikan AI untuk menggambar flowchart, mindmap, atau diagram visual secara interaktif di kanvas whiteboard.',
      examples: [
        '"Gambarkan flowchart alur login"',
        '"Buat diagram arsitektur sistem"',
        '"Gambarkan mindmap rencana proyek"'
      ],
      badgeColor: 'text-[#B08A4A] bg-[#B08A4A]/15 border-[#B08A4A]/30'
    }
  ];

  const filtered = () => {
    const cat = selectedCategory();
    if (cat === 'all') return capabilities;
    return capabilities.filter(c => c.category === cat);
  };

  return (
    <div class="p-5 bg-theme-surface border border-theme-border shadow-sm space-y-4">
      {/* Header Card */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
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
              Contoh kalimat perintah suara yang didukung
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 self-end sm:self-center">
          <button
            type="button"
            onClick={() => props.onOpenAppsModal?.()}
            class="px-3 py-1.5 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
            title="Buka Daftar Aplikasi OS yang Terdeteksi"
          >
            <Layers size={14} class="text-theme-cyan" />
            <span>Aplikasi OS</span>
          </button>

          <button
            type="button"
            onClick={() => props.onOpenExtensionModal?.()}
            class="px-3 py-1.5 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
            title="Buka Petunjuk Pasang Ekstensi Browser"
          >
            <Puzzle size={14} class="text-[#B08A4A]" />
            <span>Pasang Ekstensi Web</span>
          </button>

          <button
            onClick={() => setIsExpanded(!isExpanded())}
            class="p-1.5 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan transition-all"
            title={isExpanded() ? 'Sembunyikan Panduan' : 'Tampilkan Panduan'}
          >
            {isExpanded() ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
          </button>
        </div>
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
            <button
              onClick={() => setSelectedCategory('canvas')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'canvas'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Whiteboard 2D
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
                        <div class="flex flex-col">
                          <h4 class="text-sm font-bold text-theme-text">
                            {cap.title}
                          </h4>
                          {cap.category === 'web' && (
                            <span class="text-[10px] font-mono text-[#876447] font-semibold mt-0.5 flex items-center gap-1">
                              <Puzzle size={11} class="text-[#B08A4A]" />
                              Perlu Ekstensi Browser
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                    <p class="text-xs text-theme-text/75 leading-relaxed">
                      {cap.description}
                    </p>
                  </div>

                  {/* Example Phrases */}
                  <div class="space-y-1.5 pt-2 border-t border-theme-border/60">
                    <div class="flex items-center justify-between">
                      <span class="text-xs font-mono uppercase tracking-wider text-theme-text/60 font-bold">
                        Contoh Ucapan:
                      </span>
                    </div>
                    <div class="flex flex-wrap gap-1.5">
                      {cap.examples.map((ex) => (
                        <span class="text-xs px-2.5 py-1 bg-theme-card border border-theme-border text-theme-text/90 font-mono">
                          {ex}
                        </span>
                      ))}
                    </div>

                    {cap.category === 'web' && (
                      <button
                        type="button"
                        onClick={() => props.onOpenExtensionModal?.()}
                        class="w-full mt-2.5 py-2 px-3 bg-theme-card hover:bg-[#876447] hover:text-[#FAF7F2] text-[#876447] hover:border-[#876447] border border-[#B08A4A]/40 flex items-center justify-center gap-2 text-xs font-bold transition-all shadow-sm cursor-pointer"
                      >
                        <Puzzle size={14} class="text-[#B08A4A]" />
                        <span>Cara Pasang Ekstensi Browser</span>
                      </button>
                    )}

                    {cap.id === 'system' && (
                      <button
                        type="button"
                        onClick={() => props.onOpenAppsModal?.()}
                        class="w-full mt-2.5 py-2 px-3 bg-theme-card hover:bg-[#29483A] hover:text-[#FAF7F2] text-theme-cyan border border-theme-cyan/40 flex items-center justify-center gap-2 text-xs font-bold transition-all shadow-sm cursor-pointer"
                      >
                        <Layers size={14} />
                        <span>Buka Daftar Aplikasi OS (Terdeteksi)</span>
                      </button>
                    )}
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
