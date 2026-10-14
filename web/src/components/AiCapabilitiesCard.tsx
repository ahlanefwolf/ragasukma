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
  Puzzle,
  Layers,
  ScanEye,
  AppWindow,
  AlertTriangle,
  Check,
  Copy,
  Workflow
} from 'lucide-solid';

interface CapabilityItem {
  id: string;
  icon: typeof Globe;
  title: string;
  category: string;
  description: string;
  linuxDeps?: string;
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
  const [copiedTesseractCmd, setCopiedTesseractCmd] = createSignal(false);

  const getClientOs = (): 'windows' | 'macos' | 'linux' => {
    if (typeof window === 'undefined') return 'windows';
    const ua = window.navigator.userAgent.toLowerCase();
    if (ua.includes('mac') || ua.includes('darwin')) return 'macos';
    if (ua.includes('win')) return 'windows';
    return 'linux';
  };

  const tesseractInstallScript = `# Fedora / RHEL
sudo dnf install tesseract tesseract-langpack-eng tesseract-langpack-ind

# Ubuntu / Debian
sudo apt install tesseract-ocr tesseract-ocr-eng tesseract-ocr-ind

# Arch Linux
sudo pacman -S tesseract tesseract-data-eng tesseract-data-ind`;

  const handleCopyTesseractCmd = () => {
    navigator.clipboard.writeText(tesseractInstallScript);
    setCopiedTesseractCmd(true);
    setTimeout(() => setCopiedTesseractCmd(false), 2000);
  };

  const capabilities: CapabilityItem[] = [
    {
      id: 'vision',
      icon: ScanEye,
      title: 'Screen Scan & Visual OCR (Native OS / Tesseract)',
      category: 'desktop',
      description: 'Analyze monitor display and applications to detect visible text, labels, and buttons directly on screen.',
      examples: [
        '"Scan screen" / "Visual scan"',
        '"Look at what is on screen"',
        '"What buttons are on the screen?"',
        '"Click the text on the screen"'
      ],
      badgeColor: 'text-theme-cyan bg-theme-cyan/15 border-theme-cyan/30'
    },
    {
      id: 'desktop_window',
      icon: AppWindow,
      title: 'Window Control & Desktop Actions (Linux / Windows / macOS)',
      category: 'desktop',
      description: 'Switch application windows, click desktop buttons or text, minimize, maximize, or close windows via Ground of Truth (active_window_app.json).',
      linuxDeps: 'wlrctl (Sway/Wayland) • ydotool (GNOME/KDE) • wmctrl/xdotool (X11)',
      examples: [
        '"Switch to Zed" / "Focus Google Chrome"',
        '"Click Custom Skills on screen"',
        '"Click Save button on desktop"',
        '"Close window" / "Maximize window"'
      ],
      badgeColor: 'text-gi-status-info bg-gi-status-info/15 border-gi-status-info/30'
    },
    {
      id: 'browser',
      icon: Globe,
      title: 'Browser Navigation & Web Search (Google / Chromium)',
      category: 'web',
      description: 'Open websites, switch active tabs, and perform web searches directly with AI reading via Ground of Truth (active_tab_browser.json).',
      examples: [
        '"Search weather in Tokyo"',
        '"Search latest AI news on Google"',
        '"Open YouTube" / "Switch to GitHub tab"',
        '"Close Google tab"'
      ],
      badgeColor: 'text-theme-cyan bg-theme-cyan/15 border-theme-cyan/30'
    },
    {
      id: 'dom',
      icon: MousePointerClick,
      title: 'Web Page Interaction (Live Click & Type)',
      category: 'web',
      description: 'Click links or articles inside web pages, submit forms, type input, and scroll pages via Chromium extension.',
      examples: [
        '"Click Sign In link on web" / "Click video title"',
        '"Click Submit button on web page"',
        '"Type tech news then press enter"',
        '"Scroll down"'
      ],
      badgeColor: 'text-gi-status-info bg-gi-status-info/15 border-gi-status-info/30'
    },
    {
      id: 'recipe',
      icon: Workflow,
      title: 'Recipe Trainer & Runner (Autonomous Web Automation)',
      category: 'web',
      description: 'Train the AI once to navigate complex web workflows, then replay the recorded recipe instantly with 0 extra token cost.',
      examples: [
        '"Train recipe: open YouTube and play lofi"',
        '"Train recipe to search product on Tokopedia"',
        '"Run recipe youtube_music with query jazz"',
        '"Run recipe tokopedia_search"'
      ],
      badgeColor: 'text-theme-cyan bg-theme-cyan/15 border-theme-cyan/30'
    },
    {
      id: 'media',
      icon: Music,
      title: 'Music Player & Background Audio',
      category: 'media',
      description: 'Play local music tracks or stream/download from YouTube in background workers without freezing.',
      examples: [
        '"Play lofi chill beats"',
        '"Pause music" / "Resume music"',
        '"Next track" / "Stop music"'
      ],
      badgeColor: 'text-gi-primary bg-gi-primary/15 border-gi-primary/30'
    },
    {
      id: 'system',
      icon: Terminal,
      title: 'System Control & Desktop Applications',
      category: 'system',
      description: 'Open OS desktop applications directly, adjust master volume, take screenshots, and lock screen.',
      examples: [
        '"Open Steam" / "Open Spotify"',
        '"Open terminal" / "Open VS Code"',
        '"Open calculator" / "Open File Manager"',
        '"Increase volume" / "Mute audio"'
      ],
      badgeColor: 'text-gi-status-warning bg-gi-status-warning/15 border-gi-status-warning/30'
    },
    {
      id: 'todo',
      icon: CheckSquare,
      title: 'Smart Notes & To-Do List',
      category: 'tools',
      description: 'Record tasks to your agenda and mark items as finished automatically.',
      examples: [
        '"Note down: meeting tomorrow at 9 AM"',
        '"Add buy groceries to my to-do list"',
        '"Meeting at 9 AM is done"'
      ],
      badgeColor: 'text-gi-status-success bg-gi-status-success/15 border-gi-status-success/30'
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
        <div class="flex items-start gap-3">
          <div class="p-2.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 mt-0.5">
            <Wand2 size={18} />
          </div>
          <div>
            <h3 class="text-theme-text flex items-center gap-2">
              AI Capabilities & Voice Command Guide
              <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan font-bold">
                {capabilities.length} Features
              </span>
            </h3>
            <p class="text-xs text-theme-text/70 mt-0.5">
              Examples of supported voice commands
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 self-end sm:self-center">
          <button
            type="button"
            onClick={() => props.onOpenAppsModal?.()}
            class="px-3 py-1.5 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
            title="Open Detected OS Applications"
          >
            <Layers size={14} class="text-theme-cyan" />
            <span>OS Apps</span>
          </button>

          <button
            type="button"
            onClick={() => props.onOpenExtensionModal?.()}
            class="px-3 py-1.5 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan text-xs font-semibold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
            title="Install Browser Extension (.ZIP)"
          >
            <Puzzle size={14} class="text-gi-primary" />
            <span>Install Extension (.ZIP)</span>
          </button>

          <button
            onClick={() => setIsExpanded(!isExpanded())}
            class="p-1.5 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan transition-all"
            title={isExpanded() ? 'Hide Guide' : 'Show Guide'}
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
              All ({capabilities.length})
            </button>
            <button
              onClick={() => setSelectedCategory('desktop')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'desktop'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Desktop & Screen
            </button>
            <button
              onClick={() => setSelectedCategory('web')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'web'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Chromium & Browser
            </button>
            <button
              onClick={() => setSelectedCategory('media')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'media'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              Music & Media
            </button>
            <button
              onClick={() => setSelectedCategory('system')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'system'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              OS Control
            </button>
            <button
              onClick={() => setSelectedCategory('tools')}
              class={`px-3.5 py-1.5 border transition-all shrink-0 ${
                selectedCategory() === 'tools'
                  ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                  : 'bg-theme-card border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/40'
              }`}
            >
              To-Do & Notes
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
                          <h4 class="text-theme-text">
                            {cap.title}
                          </h4>
                          {cap.id === 'vision' && (
                            <span class={`text-[10px] font-mono font-semibold mt-0.5 flex items-center gap-1 ${
                              getClientOs() === 'linux' ? 'text-gi-status-warning' : 'text-theme-cyan'
                            }`}>
                              {getClientOs() === 'windows' && (
                                <>
                                  <Check size={11} class="text-theme-cyan" />
                                  Windows 10/11: Built-in Native OCR Active
                                </>
                              )}
                              {getClientOs() === 'macos' && (
                                <>
                                  <Check size={11} class="text-theme-cyan" />
                                  macOS: Built-in Apple Vision OCR Active
                                </>
                              )}
                              {getClientOs() === 'linux' && (
                                <>
                                  <AlertTriangle size={11} class="text-gi-status-warning" />
                                  Linux: Requires Tesseract OCR
                                </>
                              )}
                            </span>
                          )}
                          {cap.linuxDeps && (
                            <span class="text-[10px] font-mono text-theme-cyan font-semibold mt-0.5 flex items-center gap-1">
                              <Terminal size={11} />
                              Linux: {cap.linuxDeps}
                            </span>
                          )}
                          {cap.category === 'web' && (
                            <span class="text-[10px] font-mono text-theme-muted font-semibold mt-0.5 flex items-center gap-1">
                              <Puzzle size={11} class="text-gi-primary" />
                              Requires Extension (.zip)
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                    <p class="text-xs text-theme-text/75 leading-relaxed">
                      {cap.description}
                    </p>

                    {cap.id === 'vision' && getClientOs() === 'linux' && (
                      <div class="mt-2.5 p-3 bg-gi-status-warning/10 border border-gi-status-warning/30 text-xs space-y-2">
                        <div class="flex items-center gap-1.5 font-bold text-gi-status-warning">
                          <AlertTriangle size={14} class="shrink-0" />
                          <span>Notice: Linux Requires Tesseract OCR Package</span>
                        </div>
                        <p class="text-[11px] text-theme-text/80 leading-relaxed">
                          Linux distributions use the Tesseract OCR engine (streaming pipe in-memory). Install using your distribution's package manager:
                        </p>
                        <div class="p-2 bg-theme-card border border-theme-border font-mono text-[10.5px] space-y-1 relative group">
                          <div class="flex items-center justify-between pb-1 border-b border-theme-border/50 text-[10px] text-theme-text/60">
                            <span>Package Installation Command:</span>
                            <button
                              type="button"
                              onClick={handleCopyTesseractCmd}
                              class="px-2 py-0.5 bg-theme-bg hover:bg-theme-surface border border-theme-border text-[10px] font-bold flex items-center gap-1 cursor-pointer transition-all shrink-0"
                              title="Copy command"
                            >
                              {copiedTesseractCmd() ? (
                                <>
                                  <Check size={11} class="text-theme-cyan" />
                                  <span class="text-theme-cyan">Copied</span>
                                </>
                              ) : (
                                <>
                                  <Copy size={11} class="text-theme-text/70" />
                                  <span>Copy Command</span>
                                </>
                              )}
                            </button>
                          </div>
                          <pre class="overflow-x-auto text-theme-cyan font-bold select-all leading-relaxed custom-scrollbar pt-1">
{tesseractInstallScript}
                          </pre>
                        </div>
                      </div>
                    )}

                    {cap.id === 'vision' && getClientOs() !== 'linux' && (
                      <div class="mt-2 p-2 bg-theme-cyan/15 border border-theme-cyan/30 text-[11px] text-theme-text/90 flex items-center gap-1.5">
                        <Check size={13} class="text-theme-cyan shrink-0" />
                        <span>
                          {getClientOs() === 'macos'
                            ? 'Native Apple Vision OCR (Neural Engine) is active with zero background RAM footprint.'
                            : 'Windows Media Native OCR is active (0 MB RAM idle, no extra models needed).'}
                        </span>
                      </div>
                    )}
                  </div>

                  {/* Example Phrases */}
                  <div class="space-y-1.5 pt-2 border-t border-theme-border/60">
                    <div class="flex items-center justify-between">
                      <span class="text-xs font-mono uppercase tracking-wider text-theme-text/60 font-bold">
                        Example Voice Queries:
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
                        class="w-full mt-2.5 py-2 px-3 bg-theme-card hover:bg-theme-surface text-theme-muted hover:text-theme-cyan hover:border-theme-cyan/50 border border-theme-border flex items-center justify-center gap-2 text-xs font-bold transition-all shadow-sm cursor-pointer"
                      >
                        <Puzzle size={14} class="text-gi-primary" />
                        <span>How to Install Extension (.ZIP)</span>
                      </button>
                    )}

                    {cap.id === 'system' && (
                      <button
                        type="button"
                        onClick={() => props.onOpenAppsModal?.()}
                        class="w-full mt-2.5 py-2 px-3 bg-theme-card hover:bg-theme-surface text-theme-cyan hover:border-theme-cyan/60 border border-theme-cyan/40 flex items-center justify-center gap-2 text-xs font-bold transition-all shadow-sm cursor-pointer"
                      >
                        <Layers size={14} />
                        <span>Open Detected OS Applications</span>
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
