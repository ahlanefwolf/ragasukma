import { Component, createSignal, onMount, createEffect, For, Show } from 'solid-js';
import { X, Search, Layers, RefreshCw, Terminal } from 'lucide-solid';
import { api, InstalledApp } from '../api';

interface SystemAppsModalProps {
  isOpen: boolean;
  onClose: () => void;
  showToast: (msg: string) => void;
}

// Global cache to preserve application data across modal open/close
const [cachedApps, setCachedApps] = createSignal<InstalledApp[]>([]);

export const SystemAppsModal: Component<SystemAppsModalProps> = (props) => {
  const [apps, setApps] = createSignal<InstalledApp[]>(cachedApps());
  const [loading, setLoading] = createSignal(false);
  const [search, setSearch] = createSignal('');

  const loadApps = async (force = false) => {
    if (apps().length > 0 && !force) return;
    setLoading(true);
    try {
      const list = await api.getSystemApps();
      const validList = Array.isArray(list) ? list : [];
      setApps(validList);
      setCachedApps(validList);
    } catch {
      if (apps().length === 0) setApps([]);
      props.showToast('Failed to load system applications list');
    } finally {
      setLoading(false);
    }
  };

  // Prefetch immediately on component mount
  onMount(() => {
    if (cachedApps().length === 0) {
      loadApps();
    }
  });

  // Reactively fetch when opened if empty
  createEffect(() => {
    if (props.isOpen && apps().length === 0) {
      loadApps();
    }
  });

  const filtered = () => {
    const q = search().trim().toLowerCase();
    if (!q) return apps();
    return apps().filter(
      (app) =>
        app.name.toLowerCase().includes(q) ||
        app.exec.toLowerCase().includes(q)
    );
  };

  return (
    <Show when={props.isOpen}>
      <div
        class="fixed inset-0 z-[150] !mt-0 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-fade-in select-none"
        onClick={props.onClose}
      >
        <div
          class="bg-theme-surface border border-theme-border w-full max-w-2xl shadow-2xl overflow-hidden flex flex-col max-h-[85vh] text-theme-text"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div class="flex items-center justify-between p-4 sm:p-5 border-b border-theme-border bg-theme-bg shrink-0">
            <div class="flex items-start gap-3">
              <div class="p-2.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 mt-0.5">
                <Layers size={20} />
              </div>
              <div>
                <h3 class="text-theme-text font-heading flex items-center gap-2 m-0 leading-tight">
                  <span>System Applications (OS)</span>
                  <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan font-bold">
                    {apps().length} Detected
                  </span>
                </h3>
                <p class="text-xs text-theme-text/70 mt-0.5">
                  Desktop apps recognized by the AI and directly openable via voice commands (e.g. "Open Steam")
                </p>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <button
                type="button"
                onClick={() => loadApps(true)}
                class="p-2 text-theme-text/60 hover:text-theme-cyan hover:bg-theme-card border border-transparent hover:border-theme-border transition-colors cursor-pointer"
                title="Rescan Applications"
              >
                <RefreshCw size={16} class={loading() ? 'animate-spin text-theme-cyan' : ''} />
              </button>

              <button
                type="button"
                onClick={props.onClose}
                class="p-2 text-theme-text/60 hover:text-theme-text hover:bg-theme-card border border-transparent hover:border-theme-border transition-colors cursor-pointer"
                title="Close Modal"
              >
                <X size={18} />
              </button>
            </div>
          </div>

          {/* Search Box */}
          <div class="p-3 sm:p-4 border-b border-theme-border bg-theme-surface flex items-center gap-2 shrink-0">
            <div class="relative flex-1">
              <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-theme-text/50" />
              <input
                type="text"
                placeholder="Search app name or executable..."
                value={search()}
                onInput={(e) => setSearch(e.currentTarget.value)}
                class="w-full pl-9 pr-3 py-2 bg-theme-bg border border-theme-border text-xs text-theme-text outline-none focus:border-theme-cyan transition-colors"
                autofocus
              />
            </div>
            <Show when={search()}>
              <button
                type="button"
                onClick={() => setSearch('')}
                class="px-2.5 py-2 text-xs bg-theme-card border border-theme-border text-theme-text/70 hover:text-theme-text"
              >
                Reset
              </button>
            </Show>
          </div>

          {/* List Content */}
          <div class="p-4 sm:p-5 overflow-y-auto space-y-2 flex-1 select-text">
            <Show when={loading() && apps().length === 0}>
              <div class="p-12 text-center text-xs text-theme-text/50 space-y-2">
                <RefreshCw size={24} class="animate-spin text-theme-cyan mx-auto" />
                <p>Scanning desktop entries in system...</p>
              </div>
            </Show>

            <Show when={!loading() && filtered().length === 0 && apps().length > 0}>
              <div class="p-8 text-center text-xs text-theme-text/60 bg-theme-bg border border-theme-border">
                No applications matching "{search()}"
              </div>
            </Show>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              <For each={filtered()}>
                {(app) => (
                  <div class="p-3 bg-theme-bg border border-theme-border hover:border-theme-cyan/40 transition-all flex flex-col justify-between gap-1.5">
                    <div class="space-y-0.5">
                      <div class="font-bold text-xs text-theme-text truncate flex items-center justify-between gap-1">
                        <span class="truncate">{app.name}</span>
                      </div>
                    </div>

                    <div class="flex items-center justify-between gap-2 pt-1 border-t border-theme-border/50 text-[10px] font-mono text-theme-cyan">
                      <span class="truncate max-w-[200px]" title={app.exec}>
                        cmd: {app.exec.split(' ')[0]}
                      </span>
                      <span class="text-theme-text/40 shrink-0">
                        "Open {app.name.split(' ')[0]}"
                      </span>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* Footer */}
          <div class="p-3.5 sm:p-4 bg-theme-bg border-t border-theme-border flex items-center justify-between text-xs text-theme-text/70 shrink-0">
            <span>List of applications registered in operating system (OS)</span>
            <button
              type="button"
              onClick={props.onClose}
              class="px-4 py-1.5 bg-theme-card hover:bg-theme-surface border border-theme-border text-theme-text font-semibold transition-all cursor-pointer"
            >
              Close
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
