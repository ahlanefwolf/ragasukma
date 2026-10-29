import { Component, createEffect, createSignal, Show, For } from 'solid-js';
import type { JSX } from 'solid-js';
import {
  Home,
  Music,
  Sliders,
  Settings as SettingsIcon,
  Terminal,
  X,
  Bot,
  Cpu,
  ChevronDown,
  ChevronLeft,
  Radio,
  Workflow,
  MessageSquare,
  BarChart3,
  Puzzle,
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

interface NavItem {
  id: string;
  label: string;
  icon: (props: { size?: number; class?: string }) => JSX.Element;
}

interface NavGroup {
  id: string;
  label: string;
  items: NavItem[];
  /** Renders the collapsible LLM provider menu at the end of this group. */
  withProviders?: boolean;
}

const NAV_GROUPS: NavGroup[] = [
  {
    id: 'workspace',
    label: 'Workspace',
    items: [
      { id: 'menu', label: 'Home', icon: Home },
      { id: 'agent', label: 'Agent', icon: Bot },
      { id: 'skills', label: 'Skills', icon: Puzzle },
      { id: 'recipes', label: 'Recipes', icon: Workflow },
      { id: 'history', label: 'History', icon: MessageSquare },
    ],
  },
  {
    id: 'media',
    label: 'Media',
    items: [
      { id: 'music', label: 'Music', icon: Music },
      { id: 'tuner', label: 'Pose Tuner', icon: Sliders },
      { id: 'remote', label: 'Remote', icon: Radio },
    ],
  },
  {
    id: 'monitor',
    label: 'Monitor',
    items: [
      { id: 'usage', label: 'Usage', icon: BarChart3 },
      { id: 'logs', label: 'Logs', icon: Terminal },
    ],
  },
  {
    id: 'config',
    label: 'Config',
    items: [{ id: 'settings', label: 'Settings', icon: SettingsIcon }],
    withProviders: true,
  },
];

const PROVIDER_SUB_ITEMS = [
  { id: 'provider-antigravity', label: 'Antigravity' },
  { id: 'provider-codex', label: 'Codex' },
  { id: 'provider-claude', label: 'Claude' },
  { id: 'provider-custom', label: 'Custom API' },
];

const focusRing =
  'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-theme-cyan/60';

export const Sidebar: Component<SidebarProps> = (props) => {
  const [isCollapsed, setIsCollapsed] = createSignal<boolean>(
    (() => {
      try {
        return localStorage.getItem('sidebar_collapsed') === 'true';
      } catch {
        return false;
      }
    })()
  );

  const toggleCollapsed = () => {
    const next = !isCollapsed();
    setIsCollapsed(next);
    try {
      localStorage.setItem('sidebar_collapsed', String(next));
    } catch {
      /* storage unavailable, ignore */
    }
  };

  const isProviderActive = () => props.activeTab.startsWith('provider-');

  const [providerMenuOpen, setProviderMenuOpen] = createSignal(isProviderActive());

  // Keep the provider submenu open whenever navigation lands on a provider page.
  createEffect(() => {
    if (isProviderActive()) setProviderMenuOpen(true);
  });

  const handleSelectTab = (tabId: string) => {
    props.setActiveTab(tabId);
    props.onCloseMobile?.();
  };

  const stateLabel = () => (props.currentState || 'IDLE').toUpperCase();

  const SidebarContent = (contentProps: { collapsed?: boolean }) => {
    const collapsed = () => !!contentProps.collapsed;

    const NavButton = (btnProps: { item: NavItem }) => {
      const Icon = btnProps.item.icon;
      const isActive = () => props.activeTab === btnProps.item.id;
      return (
        <button
          type="button"
          onClick={() => handleSelectTab(btnProps.item.id)}
          title={collapsed() ? btnProps.item.label : undefined}
          aria-label={btnProps.item.label}
          aria-current={isActive() ? 'page' : undefined}
          class={`relative w-full flex items-center gap-3 px-3 py-2.5 text-xs font-semibold tracking-wide transition-all border ${focusRing} ${
            isActive()
              ? 'bg-theme-cyan/15 text-theme-cyan font-bold border-theme-cyan/40 shadow-sm'
              : 'bg-transparent border-transparent text-theme-text/75 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
          } ${collapsed() ? 'justify-center px-2' : ''}`}
        >
          <Show when={isActive()}>
            <span class="absolute left-0 top-1.5 bottom-1.5 w-0.5 bg-theme-cyan" />
          </Show>
          <Icon
            size={18}
            class={`shrink-0 transition-transform ${
              isActive() ? 'text-theme-cyan scale-110' : 'text-theme-text/80'
            }`}
          />
          <Show when={!collapsed()}>
            <span class="truncate">{btnProps.item.label}</span>
          </Show>
        </button>
      );
    };

    const ProviderMenu = () => (
      <div>
        <button
          type="button"
          onClick={() => {
            if (collapsed()) {
              toggleCollapsed();
              setProviderMenuOpen(true);
            } else {
              setProviderMenuOpen(!providerMenuOpen());
            }
          }}
          title={collapsed() ? 'LLM Providers' : undefined}
          aria-label="LLM Providers"
          aria-expanded={!collapsed() && providerMenuOpen()}
          aria-controls="sidebar-provider-menu"
          class={`relative w-full flex items-center px-3 py-2.5 text-xs font-semibold tracking-wide transition-all border ${focusRing} ${
            isProviderActive()
              ? 'bg-theme-card text-theme-text font-bold border-theme-border'
              : 'bg-transparent border-transparent text-theme-text/75 hover:bg-theme-card hover:text-theme-text hover:border-theme-border'
          } ${collapsed() ? 'justify-center px-2' : 'justify-between'}`}
        >
          <Show when={isProviderActive()}>
            <span class="absolute left-0 top-1.5 bottom-1.5 w-0.5 bg-theme-rose" />
          </Show>
          <div class="flex items-center gap-3 min-w-0">
            <Cpu
              size={18}
              class={`shrink-0 ${
                isProviderActive() ? 'text-theme-rose' : 'text-theme-text/80'
              }`}
            />
            <Show when={!collapsed()}>
              <span class="truncate">Providers</span>
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

        <Show when={providerMenuOpen() && !collapsed()}>
          <div
            id="sidebar-provider-menu"
            class="pl-3 pr-1 py-1 space-y-1 mt-1 border-l-2 border-theme-border/70 ml-4 animate-in fade-in duration-150"
          >
            <For each={PROVIDER_SUB_ITEMS}>
              {(subItem) => {
                const isSubActive = () => props.activeTab === subItem.id;
                return (
                  <button
                    type="button"
                    onClick={() => handleSelectTab(subItem.id)}
                    aria-current={isSubActive() ? 'page' : undefined}
                    class={`w-full flex items-center justify-between px-3 py-2 text-xs font-semibold transition-all border ${focusRing} ${
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
    );

    return (
      <div class="flex flex-col h-full min-h-0">
        {/* Logo & Header */}
        <div
          class={`shrink-0 flex items-center mb-4 pb-3 border-b border-theme-border/60 ${
            collapsed() ? 'justify-center' : 'justify-between'
          }`}
        >
          <button
            type="button"
            onClick={() => handleSelectTab('menu')}
            class={`flex items-center gap-3 cursor-pointer group select-none min-w-0 ${focusRing}`}
            title="RAGASUKMA"
            aria-label="RAGASUKMA, go to home"
          >
            <div class="w-10 h-10 shrink-0 flex items-center justify-center">
              <BrandLogo size={40} />
            </div>
            <Show when={!collapsed()}>
              <span class="font-heading text-lg font-semibold tracking-wide text-theme-text leading-none block truncate">
                RAGASUKMA
              </span>
            </Show>
          </button>

          {/* Mobile close button */}
          <Show when={props.isMobileOpen}>
            <button
              type="button"
              onClick={props.onCloseMobile}
              class={`p-2 text-theme-text hover:bg-theme-card border border-transparent hover:border-theme-border transition-colors md:hidden ${focusRing}`}
              aria-label="Close menu"
            >
              <X size={18} />
            </button>
          </Show>
        </div>

        {/* Navigation (scrolls on short screens) */}
        <nav
          aria-label="Main navigation"
          class="flex-1 min-h-0 overflow-y-auto overflow-x-hidden -mx-1 px-1 [scrollbar-width:thin]"
        >
          <For each={NAV_GROUPS}>
            {(group, index) => (
              <section
                aria-label={group.label}
                class={index() > 0 ? 'mt-3' : ''}
              >
                <Show
                  when={!collapsed()}
                  fallback={
                    <Show when={index() > 0}>
                      <div class="mx-2 mb-2 border-t border-theme-border/60" />
                    </Show>
                  }
                >
                  <h3 class="px-3 pb-1.5 text-[10px] font-mono font-semibold uppercase tracking-[0.18em] text-theme-text/40 select-none">
                    {group.label}
                  </h3>
                </Show>
                <div class="space-y-1">
                  <For each={group.items}>{(item) => <NavButton item={item} />}</For>
                  <Show when={group.withProviders}>
                    <ProviderMenu />
                  </Show>
                </div>
              </section>
            )}
          </For>
        </nav>

        {/* Footer: live engine & overlay status */}
        <div class="shrink-0 mt-3 pt-3 border-t border-theme-border/60">
          <div
            role="status"
            aria-live="polite"
            title={`Overlay ${props.overlayConnected ? 'connected' : 'disconnected'} · State: ${stateLabel()}`}
            class={`flex items-center ${collapsed() ? 'justify-center' : 'justify-between gap-2 px-2 py-1'}`}
          >
            <div class="flex items-center gap-2 min-w-0">
              <span
                class={`w-2 h-2 shrink-0 ${
                  props.overlayConnected
                    ? 'bg-theme-cyan shadow-[0_0_6px_var(--cyan)] animate-pulse'
                    : 'bg-theme-rose'
                }`}
              />
              <Show when={!collapsed()}>
                <span class="text-[11px] font-mono text-theme-text/50 truncate">Overlay</span>
              </Show>
            </div>
            <Show when={!collapsed()}>
              <div class="flex items-center gap-2 min-w-0">
                <span
                  class={`text-[11px] font-mono font-bold ${
                    props.overlayConnected ? 'text-theme-cyan' : 'text-theme-rose'
                  }`}
                >
                  {props.overlayConnected ? 'Online' : 'Offline'}
                </span>
                <span class="text-[10px] font-mono font-semibold uppercase tracking-wider text-theme-text/50 truncate">
                  {stateLabel()}
                </span>
              </div>
            </Show>
          </div>
        </div>
      </div>
    );
  };

  return (
    <>
      {/* Desktop Persistent Sidebar */}
      <aside
        class={`group/aside relative hidden md:flex flex-col min-h-0 bg-theme-surface border-r border-theme-border shrink-0 transition-all duration-300 ease-in-out select-none ${
          isCollapsed() ? 'w-16 p-3' : 'w-64 p-5'
        }`}
      >
        <SidebarContent collapsed={isCollapsed()} />

        {/* Edge collapse toggle: sits on the border, aligned with the logo */}
        <button
          type="button"
          onClick={toggleCollapsed}
          title={isCollapsed() ? 'Expand sidebar' : 'Collapse sidebar'}
          aria-label={isCollapsed() ? 'Expand sidebar' : 'Collapse sidebar'}
          aria-expanded={!isCollapsed()}
          class={`absolute -right-3 z-30 flex items-center justify-center w-6 h-6 rounded-full bg-theme-surface border border-theme-border text-theme-text/60 shadow-md transition-all duration-300 hover:text-theme-cyan hover:border-theme-cyan/60 hover:shadow-[0_0_10px_var(--cyan)] hover:scale-110 ${focusRing} ${
            isCollapsed() ? 'top-[1.25rem]' : 'top-[1.75rem]'
          }`}
        >
          <ChevronLeft
            size={14}
            stroke-width={2.5}
            class={`transition-transform duration-300 ${isCollapsed() ? 'rotate-180' : ''}`}
          />
        </button>
      </aside>

      {/* Mobile Drawer (always full width menu when open) */}
      <Show when={props.isMobileOpen}>
        <div
          class="fixed inset-0 bg-black/60 backdrop-blur-sm z-40 md:hidden animate-fade-in"
          onClick={props.onCloseMobile}
        />
        <aside class="fixed inset-y-0 left-0 w-72 bg-theme-surface border-r border-theme-border p-5 z-50 md:hidden flex flex-col min-h-0 shadow-2xl animate-slide-right select-none">
          <SidebarContent collapsed={false} />
        </aside>
      </Show>
    </>
  );
};
