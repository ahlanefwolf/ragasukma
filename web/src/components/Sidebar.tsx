import { Component, createSignal, Show, For } from 'solid-js';
import {
  Menu as MenuIcon,
  Zap,
  Music,
  Sliders,
  Settings as SettingsIcon,
  Terminal,
  X,
  Palette,
  Bot,
  Rocket,
  Code2,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Radio,
  Workflow,
  MessageSquare,
  BarChart3
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
  const [isCollapsed, setIsCollapsed] = createSignal<boolean>((() => {
    try {
      return localStorage.getItem('sidebar_collapsed') === 'true';
    } catch {
      return false;
    }
  })());

  const toggleCollapsed = () => {
    const next = !isCollapsed();
    setIsCollapsed(next);
    try {
      localStorage.setItem('sidebar_collapsed', String(next));
    } catch {}
  };

  const navItems = [
    { id: 'menu', label: 'Main Menu', icon: MenuIcon },
    { id: 'agent', label: 'Agent Workspace', icon: Bot },
    { id: 'recipes', label: 'Recipes', icon: Workflow },
    { id: 'history', label: 'Chat History', icon: MessageSquare },
    { id: 'usage', label: 'Token Usage', icon: BarChart3 },
    { id: 'music', label: 'Music & Audio', icon: Music },
    { id: 'tuner', label: 'Pose Tuner', icon: Sliders },
    { id: 'remote', label: 'Mobile Remote', icon: Radio },
    { id: 'logs', label: 'Console Logs', icon: Terminal },
    { id: 'settings', label: 'Settings', icon: SettingsIcon },
  ];

  const providerSubItems = [
    { id: 'provider-antigravity', label: 'Antigravity Engine' },
    { id: 'provider-codex', label: 'Codex Provider' },
    { id: 'provider-claude', label: 'Claude Provider' },
    { id: 'provider-custom', label: 'Custom OpenAI-Compatible' },
  ];

  const isProviderActive = () => props.activeTab.startsWith('provider-');

  const [providerMenuOpen, setProviderMenuOpen] = createSignal(
    isProviderActive()
  );

  const handleSelectTab = (tabId: string) => {
    props.setActiveTab(tabId);
    if (props.onCloseMobile) {
      props.onCloseMobile();
    }
  };

  const SidebarContent = (contentProps: { collapsed?: boolean }) => {
    const collapsed = () => !!contentProps.collapsed;

    return (
      <div class="flex flex-col h-full justify-between">
        {/* Top Section */}
        <div>
          {/* Logo & Header */}
          <div
            class={`flex items-center mb-5 pb-3 border-b border-theme-border/60 ${
              collapsed() ? 'flex-col gap-2.5 items-center justify-center' : 'justify-between'
            }`}
          >
            <div
              onClick={() => handleSelectTab('menu')}
              class="flex items-center gap-3 cursor-pointer group select-none min-w-0"
              title="RAGASUKMA"
            >
              <div class="w-10 h-10 shrink-0 flex items-center justify-center">
                <BrandLogo size={40} />
              </div>
              <Show when={!collapsed()}>
                <div>
                  <span class="font-heading text-lg font-semibold tracking-wide text-theme-text leading-none block">
                    RAGASUKMA
                  </span>
                </div>
              </Show>
            </div>

            {/* Collapse toggle button on desktop */}
            <Show when={!props.isMobileOpen}>
              <button
                onClick={toggleCollapsed}
                class="hidden md:flex items-center justify-center w-7 h-7 text-theme-text/50 hover:text-theme-cyan bg-theme-card/50 hover:bg-theme-card border border-theme-border/60 hover:border-theme-cyan/40 transition-all duration-200 group/btn shadow-sm"
                title={collapsed() ? 'Expand Sidebar' : 'Collapse Sidebar'}
                aria-label={collapsed() ? 'Expand sidebar' : 'Collapse sidebar'}
              >
                {collapsed() ? (
                  <svg
                    class="w-4 h-4 transition-transform duration-200 group-hover/btn:scale-110"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <rect width="18" height="18" x="3" y="3" rx="2" />
                    <path d="M9 3v18" />
                    <path d="m14 15 3-3-3-3" />
                  </svg>
                ) : (
                  <svg
                    class="w-4 h-4 transition-transform duration-200 group-hover/btn:-translate-x-0.5"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="1.8"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <rect width="18" height="18" x="3" y="3" rx="2" />
                    <path d="M9 3v18" />
                    <path d="m14 9-3 3 3 3" />
                  </svg>
                )}
              </button>
            </Show>

            {/* Mobile close button */}
            <Show when={props.isMobileOpen}>
              <button
                onClick={props.onCloseMobile}
                class="p-2 text-theme-text hover:bg-theme-card border border-transparent hover:border-theme-border transition-colors md:hidden"
                aria-label="Close menu"
              >
                <X size={18} />
              </button>
            </Show>
          </div>

          {/* Clean State Status Indicator (Replaces Avatar Active box) */}
          <div
            class={`mb-4 px-3 py-2 bg-theme-card/60 border border-theme-border flex items-center transition-all ${
              collapsed() ? 'justify-center px-1 py-2' : 'justify-between'
            }`}
            title={`State: ${props.currentState || 'IDLE'}`}
          >
            <div class="flex items-center gap-2 min-w-0">
              <span
                class={`w-2 h-2 shrink-0 ${
                  props.overlayConnected
                    ? 'bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse'
                    : 'bg-theme-rose'
                }`}
              />
              <Show when={!collapsed()}>
                <span class="text-[10px] font-mono uppercase tracking-wider text-theme-text/60 font-semibold truncate">
                  STATE
                </span>
              </Show>
            </div>
            <Show when={!collapsed()}>
              <span class="text-xs font-mono font-bold text-theme-cyan uppercase tracking-wider truncate ml-2">
                {props.currentState || 'IDLE'}
              </span>
            </Show>
          </div>

          {/* Navigation Links */}
          <nav class="space-y-1">
            <For each={navItems}>
              {(item) => {
                const Icon = item.icon;
                const isActive = () => props.activeTab === item.id;
                return (
                  <button
                    onClick={() => handleSelectTab(item.id)}
                    title={collapsed() ? item.label : undefined}
                    class={`w-full flex items-center gap-3 px-3 py-2.5 text-xs font-semibold tracking-wide transition-all border ${
                      isActive()
                        ? 'bg-theme-cyan/15 text-theme-cyan font-bold border-theme-cyan/40 shadow-sm'
                        : 'bg-transparent border-transparent text-theme-text/75 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
                    } ${collapsed() ? 'justify-center px-2' : ''}`}
                  >
                    <Icon
                      size={18}
                      class={`shrink-0 transition-transform ${
                        isActive() ? 'text-theme-cyan scale-110' : 'text-theme-text/80'
                      }`}
                    />
                    <Show when={!collapsed()}>
                      <span class="truncate">{item.label}</span>
                    </Show>
                  </button>
                );
              }}
            </For>

            {/* Provider Section */}
            <div class="pt-2">
              <button
                onClick={() => {
                  if (collapsed()) {
                    toggleCollapsed();
                    setProviderMenuOpen(true);
                  } else {
                    setProviderMenuOpen(!providerMenuOpen());
                  }
                }}
                title={collapsed() ? 'Provider LLM' : undefined}
                class={`w-full flex items-center px-3 py-2.5 text-xs font-semibold tracking-wide transition-all border ${
                  isProviderActive()
                    ? 'bg-theme-card text-theme-text font-bold border-theme-border'
                    : 'bg-transparent border-transparent text-theme-text/75 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
                } ${collapsed() ? 'justify-center px-2' : 'justify-between'}`}
              >
                <div class="flex items-center gap-3 min-w-0">
                  <Bot
                    size={18}
                    class={`shrink-0 ${
                      isProviderActive() ? 'text-theme-cyan' : 'text-theme-text/80'
                    }`}
                  />
                  <Show when={!collapsed()}>
                    <span class="truncate">Provider LLM</span>
                  </Show>
                </div>
                <Show when={!collapsed()}>
                  <ChevronDown
                    size={14}
                    class={`shrink-0 transition-transform duration-200 text-theme-text/60 ${
                      providerMenuOpen() ? 'rotate-180 text-theme-cyan' : ''
                    }`}
                  />
                </Show>
              </button>

              {/* Submenu Items */}
              <Show when={providerMenuOpen() && !collapsed()}>
                <div class="pl-4 pr-1 py-1 space-y-1 mt-1 border-l-2 border-theme-border/70 ml-4 animate-in fade-in duration-150">
                  <For each={providerSubItems}>
                    {(subItem) => {
                      const isSubActive = () => props.activeTab === subItem.id;
                      return (
                        <button
                          onClick={() => handleSelectTab(subItem.id)}
                          class={`w-full flex items-center justify-between px-3 py-2 text-xs font-semibold transition-all border ${
                            isSubActive()
                              ? 'bg-theme-rose/15 text-theme-rose font-bold border-theme-rose/40'
                              : 'bg-transparent border-transparent text-theme-text/70 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
                          }`}
                        >
                          <span class="truncate">{subItem.label}</span>
                          <Show when={isSubActive()}>
                            <span class="w-1.5 h-1.5 bg-theme-rose shrink-0 ml-1.5" />
                          </Show>
                        </button>
                      );
                    }}
                  </For>
                </div>
              </Show>
            </div>
          </nav>
        </div>

        {/* Footer / Engine Status */}
        <div class="pt-4 border-t border-theme-border/60">
          <Show when={!collapsed()}>
            <div class="px-2 py-1 text-[11px] font-mono text-theme-text/50 flex items-center justify-between">
              <span>Engine</span>
              <span class="text-theme-cyan font-bold">Online</span>
            </div>
          </Show>
          <Show when={collapsed()}>
            <div class="flex justify-center" title="Engine: Online">
              <span class="w-2 h-2 bg-theme-cyan shadow-[0_0_6px_var(--cyan)]" />
            </div>
          </Show>
        </div>
      </div>
    );
  };

  return (
    <>
      {/* Desktop Persistent Sidebar */}
      <aside
        class={`hidden md:flex flex-col justify-between bg-theme-surface border-r border-theme-border shrink-0 transition-all duration-300 ease-in-out select-none ${
          isCollapsed() ? 'w-16 p-3' : 'w-64 p-5'
        }`}
      >
        <SidebarContent collapsed={isCollapsed()} />
      </aside>

      {/* Mobile Drawer (always full width menu when open) */}
      <Show when={props.isMobileOpen}>
        <div
          class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 md:hidden animate-fade-in"
          onClick={props.onCloseMobile}
        />
        <aside class="fixed inset-y-0 left-0 w-72 bg-theme-surface border-r border-theme-border p-5 z-50 md:hidden flex flex-col justify-between shadow-2xl animate-slide-right select-none">
          <SidebarContent collapsed={false} />
        </aside>
      </Show>
    </>
  );
};