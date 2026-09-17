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
            <div class="w-10 h-10 rounded-2xl overflow-hidden shadow-lg anime-shadow-green shrink-0 flex items-center justify-center">
              <BrandLogo size={40} />
            </div>
            <div>
              <h1 class="text-2xl font-carter tracking-wider text-theme-text leading-none">
                RAGASUKMA
              </h1>
              <p class="text-[11px] text-theme-cyan font-semibold tracking-wide flex items-center gap-1">
                <Sparkles size={11} /> NATIVE AI DESKTOP
              </p>
            </div>
          </div>

          {/* Close button on mobile drawer */}
          <Show when={props.isMobileOpen}>
            <button
              onClick={props.onCloseMobile}
              class="p-2 rounded-xl text-theme-text/60 hover:text-theme-text hover:bg-theme-card md:hidden transition-colors"
              aria-label="Tutup Menu"
            >
              <X size={20} />
            </button>
          </Show>
        </div>

        {/* Live Overlay Status Pill */}
        <div class="mb-5 p-3 rounded-xl bg-theme-card border border-theme-border flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span 
              class={`w-2.5 h-2.5 rounded-full ${
                props.overlayConnected 
                  ? 'bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse' 
                  : 'bg-theme-rose'
              }`} 
            />
            <span class="text-xs font-semibold text-theme-text">
              {props.overlayConnected ? '3D Native Aktif' : 'Offline'}
            </span>
          </div>
          <span class="text-[10px] uppercase font-bold px-2 py-0.5 rounded bg-theme-bg text-theme-cyan border border-theme-border">
            WGPU 9:16
          </span>
        </div>

        {/* Main Tabs Navigation */}
        <nav class="space-y-1.5">
          {menuItems.map((item) => {
            const Icon = item.icon;
            const isActive = () => props.activeTab === item.id;

            return (
              <button
                onClick={() => handleSelectTab(item.id)}
                class={`w-full flex items-center justify-between px-3.5 py-3 rounded-xl text-sm font-medium transition-all duration-200 ${
                  isActive()
                    ? 'bg-theme-cyan text-theme-card shadow-md anime-shadow-green font-bold'
                    : 'text-theme-text/75 hover:bg-theme-card hover:text-theme-text'
                }`}
              >
                <div class="flex items-center gap-3">
                  <Icon size={18} class={isActive() ? 'stroke-[2.5]' : 'stroke-[2]'} />
                  <span class="tracking-wide">{item.label}</span>
                </div>
                {item.badge && (
                  <span class={`text-[10px] px-2 py-0.5 rounded-full font-bold font-mono uppercase ${
                    isActive() ? 'bg-theme-card/20 text-theme-card' : 'bg-theme-rose/20 text-theme-rose border border-theme-rose/30'
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
              class={`w-full flex items-center justify-between px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 ${
                isProviderActive()
                  ? 'text-theme-cyan bg-theme-card/60 font-bold border border-theme-cyan/30'
                  : 'text-theme-text/75 hover:bg-theme-card hover:text-theme-text'
              }`}
            >
              <div class="flex items-center gap-3">
                <Bot size={18} class={isProviderActive() ? 'stroke-[2.5] text-theme-cyan' : 'stroke-[2]'} />
                <span class="tracking-wide">Provider LLM</span>
              </div>
              <ChevronDown 
                size={16} 
                class={`transition-transform duration-200 ${providerMenuOpen() ? 'rotate-180 text-theme-cyan' : 'text-theme-text/40'}`} 
              />
            </button>

            {/* Submenu Items */}
            <Show when={providerMenuOpen()}>
              <div class="pl-5 pr-1 py-1 space-y-1 mt-1 border-l-2 border-theme-border/60 ml-4">
                <button
                  onClick={() => handleSelectTab('provider-antigravity')}
                  class={`w-full flex items-center justify-between px-3 py-2 rounded-lg text-xs font-medium transition-all ${
                    props.activeTab === 'provider-antigravity'
                      ? 'bg-amber-500/20 text-amber-300 font-bold border border-amber-500/30'
                      : 'text-theme-text/65 hover:bg-theme-card hover:text-theme-text'
                  }`}
                >
                  <div class="flex items-center gap-2.5">
                    <Rocket size={14} class={props.activeTab === 'provider-antigravity' ? 'text-amber-400' : 'text-theme-text/50'} />
                    <span>Antigravity</span>
                  </div>
                  <span class="text-[9px] font-mono px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20 uppercase font-bold">
                    OAuth
                  </span>
                </button>
              </div>
            </Show>
          </div>
        </nav>
      </div>

      {/* Footer System Info */}
      <div class="pt-4 border-t border-theme-border/60 text-xs text-theme-text/50 px-1 space-y-1.5">
        <div class="flex items-center justify-between">
          <span>Engine Status</span>
          <span class="text-theme-cyan font-semibold text-[11px]">WGPU 1:1 Three-VRM</span>
        </div>
        <div class="flex items-center justify-between text-[11px]">
          <span>Dashboard Port</span>
          <span class="font-mono text-theme-text/80 font-bold">42128</span>
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
