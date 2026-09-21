import { Component, createSignal, Show } from 'solid-js';
import { 
  Menu as MenuIcon, 
  Zap, 
  Music,
  Sliders, 
  Settings as SettingsIcon,
  Terminal,
  X,
  Sparkles,
  Palette,
  Bot,
  Rocket,
  ChevronDown
} from 'lucide-solid';
import { BrandLogo } from './BrandLogo';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  overlayConnected: boolean;
  currentState: string;
  isMobileOpen?: boolean;
  onCloseMobile?: () => void;
}

export const Sidebar: Component<SidebarProps> = (props) => {
  const [providerMenuOpen, setProviderMenuOpen] = createSignal(true);

  const menuItems = [
    { id: 'menu', label: 'Main Menu', icon: MenuIcon, badge: props.currentState.toUpperCase() },
    { id: 'canvas', label: '2D Whiteboard', icon: Palette },
    { id: 'skills', label: 'Skill Commands', icon: Zap },
    { id: 'music', label: 'Music Player', icon: Music },
    { id: 'tuner', label: 'Pose Tuner', icon: Sliders },
    { id: 'logs', label: 'Console Logs', icon: Terminal },
    { id: 'settings', label: 'Pengaturan', icon: SettingsIcon },
  ];

  const handleSelectTab = (id: string) => {
    props.setActiveTab(id);
    if (props.onCloseMobile) {
      props.onCloseMobile();
    }
  };

  const isProviderActive = () => props.activeTab.startsWith('provider-');

  const SidebarContent = () => (
    <div class="flex flex-col justify-between h-full select-none">
      <div>
        {/* Brand Header */}
        <div class="flex items-center justify-between mb-6 px-1">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 overflow-hidden shadow-lg anime-shadow-green shrink-0 flex items-center justify-center border border-theme-border">
              <BrandLogo size={40} />
            </div>
            <div>
              <h1 class="text-2xl font-heading tracking-wide text-theme-text leading-none">
                RAGASUKMA
              </h1>
              <p class="text-xs text-theme-cyan font-semibold tracking-wider flex items-center gap-1.5 mt-1">
                <Sparkles size={13} /> NATIVE AI DESKTOP
              </p>
            </div>
          </div>

          {/* Close button on mobile drawer */}
          <Show when={props.isMobileOpen}>
            <button
              onClick={props.onCloseMobile}
              class="p-2 text-theme-text/60 hover:text-theme-text hover:bg-theme-card md:hidden transition-colors border border-theme-border"
              aria-label="Tutup Menu"
            >
              <X size={20} />
            </button>
          </Show>
        </div>

        {/* Live Overlay Status Pill */}
        <div class="mb-5 p-3.5 bg-theme-card border border-theme-border flex items-center justify-between shadow-sm">
          <div class="flex items-center gap-2.5">
            <span 
              class={`w-2.5 h-2.5 ${
                props.overlayConnected 
                  ? 'bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse' 
                  : 'bg-theme-rose'
              }`} 
            />
            <span class="text-xs font-semibold text-theme-text tracking-wide">
              {props.overlayConnected ? '3D Native Aktif' : 'Offline'}
            </span>
          </div>
          <span class="text-xs uppercase font-mono font-bold px-2 py-0.5 bg-theme-bg text-theme-cyan border border-theme-border">
            WGPU 9:16
          </span>
        </div>

        {/* Main Tabs Navigation */}
        <nav class="space-y-1">
          {menuItems.map((item) => {
            const Icon = item.icon;
            const isActive = () => props.activeTab === item.id;

            return (
              <button
                onClick={() => handleSelectTab(item.id)}
                class={`w-full flex items-center justify-between px-3.5 py-3 text-sm font-semibold transition-all duration-150 border ${
                  isActive()
                    ? 'bg-theme-cyan text-theme-card border-theme-cyan shadow-md anime-shadow-green font-bold'
                    : 'bg-transparent border-transparent text-theme-text/80 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
                }`}
              >
                <div class="flex items-center gap-3">
                  <Icon size={18} class={isActive() ? 'stroke-[2.5]' : 'stroke-[2]'} />
                  <span class="tracking-wide">{item.label}</span>
                </div>
                {item.badge && (
                  <span class={`text-xs px-2 py-0.5 font-bold font-mono uppercase border ${
                    isActive() 
                      ? 'bg-theme-card/20 text-theme-card border-theme-card/30' 
                      : 'bg-theme-rose/20 text-theme-rose border-theme-rose/40'
                  }`}>
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}

          {/* Provider LLM Group with Submenu */}
          <div class="pt-2">
            <button
              onClick={() => setProviderMenuOpen(!providerMenuOpen())}
              class={`w-full flex items-center justify-between px-3.5 py-2.5 text-sm font-semibold transition-all duration-150 border ${
                isProviderActive()
                  ? 'text-theme-cyan bg-theme-card/80 border-theme-cyan/40 font-bold'
                  : 'bg-transparent border-transparent text-theme-text/80 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
              }`}
            >
              <div class="flex items-center gap-3">
                <Bot size={18} class={isProviderActive() ? 'stroke-[2.5] text-theme-cyan' : 'stroke-[2]'} />
                <span class="tracking-wide">Provider LLM</span>
              </div>
              <ChevronDown 
                size={16} 
                class={`transition-transform duration-200 ${providerMenuOpen() ? 'rotate-180 text-theme-cyan' : 'text-theme-text/50'}`} 
              />
            </button>

            {/* Submenu Items */}
            <Show when={providerMenuOpen()}>
              <div class="pl-4 pr-1 py-1 space-y-1 mt-1 border-l-2 border-theme-border/70 ml-4">
                <button
                  onClick={() => handleSelectTab('provider-antigravity')}
                  class={`w-full flex items-center justify-between px-3 py-2 text-xs font-semibold transition-all border ${
                    props.activeTab === 'provider-antigravity'
                      ? 'bg-amber-500/20 text-amber-300 font-bold border-amber-500/40'
                      : 'bg-transparent border-transparent text-theme-text/70 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
                  }`}
                >
                  <div class="flex items-center gap-2.5">
                    <Rocket size={15} class={props.activeTab === 'provider-antigravity' ? 'text-amber-400' : 'text-theme-text/60'} />
                    <span>Antigravity</span>
                  </div>
                  <span class="text-[10px] font-mono px-1.5 py-0.5 bg-amber-500/15 text-amber-400 border border-amber-500/30 uppercase font-bold">
                    OAuth
                  </span>
                </button>
              </div>
            </Show>
          </div>
        </nav>
      </div>

      {/* Footer System Info */}
      <div class="pt-4 border-t border-theme-border/70 text-xs text-theme-text/60 px-1 space-y-2">
        <div class="flex items-center justify-between">
          <span>Engine Status</span>
          <span class="text-theme-cyan font-semibold text-xs">WGPU 1:1 Three-VRM</span>
        </div>
        <div class="flex items-center justify-between text-xs">
          <span>Dashboard Port</span>
          <span class="font-mono text-theme-text font-bold">42128</span>
        </div>
      </div>
    </div>
  );

  return (
    <>
      {/* Desktop Persistent Sidebar */}
      <aside class="hidden md:flex w-64 bg-theme-surface border-r border-theme-border p-5 shrink-0 flex-col justify-between">
        <SidebarContent />
      </aside>

      {/* Mobile Drawer Backdrop */}
      <Show when={props.isMobileOpen}>
        <div 
          class="fixed inset-0 bg-black/70 backdrop-blur-sm z-40 md:hidden animate-fade-in"
          onClick={props.onCloseMobile}
        />
        <aside class="fixed inset-y-0 left-0 w-72 bg-theme-surface border-r border-theme-border p-5 z-50 md:hidden flex flex-col justify-between shadow-2xl animate-slide-right">
          <SidebarContent />
        </aside>
      </Show>
    </>
  );
};
