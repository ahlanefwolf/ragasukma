import { Component, Show, For } from 'solid-js';
import {
  Download,
  RefreshCw,
  Copy,
  Check,
  AlertCircle,
  Loader,
  Cloud,
} from 'lucide-solid';
import { CloudflaredInfo } from '../api';

interface CloudflaredStatusCardProps {
  checking: () => boolean;
  info: () => CloudflaredInfo | null;
  onRefresh: () => void;
  onInstall: () => void;
  installing: () => boolean;
  installMsg: () => string | null;
  copiedCmd: () => string | null;
  onCopyCmd: (cmd: string) => void;
}

export const CloudflaredStatusCard: Component<CloudflaredStatusCardProps> = (props) => {
  const installCommands = [
    { os: 'Windows', cmd: 'winget install Cloudflare.cloudflared', color: 'var(--cyan)' },
    { os: 'macOS', cmd: 'brew install cloudflared', color: 'var(--rose)' },
    { os: 'Linux (Fedora/RHEL)', cmd: 'sudo dnf install cloudflared', color: 'var(--text)' },
    { os: 'Linux (Debian/Ubuntu)', cmd: 'sudo apt install cloudflared', color: 'var(--text)' },
  ];

  return (
    <div class="border border-theme-border bg-theme-surface overflow-hidden">
      <div class="px-4 py-3 border-b border-theme-border flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Cloud size={16} class="text-theme-cyan" />
          <h3 class="text-theme-text/80">cloudflared Binary Status</h3>
        </div>
        <Show when={!props.checking()}>
          <button
            onClick={props.onRefresh}
            class="text-xs px-3 py-1 bg-theme-card hover:bg-theme-border text-theme-text/80 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5"
          >
            <RefreshCw size={13} />
            <span>Rescan</span>
          </button>
        </Show>
      </div>

      <div class="p-4">
        <Show when={props.checking()}>
          <div class="flex items-center gap-2 text-xs text-theme-text/70">
            <Loader size={14} class="animate-spin" />
            <span>Checking cloudflared on system...</span>
          </div>
        </Show>

        <Show when={!props.checking() && props.info()?.available}>
          <div class="space-y-1.5">
            <div class="flex items-center gap-2 flex-wrap">
              <span class="px-3 py-1 text-xs font-bold bg-theme-emerald/20 text-theme-emerald border border-theme-emerald/30 inline-flex items-center gap-1.5">
                <Check size={13} class="stroke-[2.5]" /> cloudflared v{props.info()!.version}
              </span>
              <span class="text-xs text-theme-text/60">Installed and ready to use</span>
            </div>
            <Show when={props.info()?.path}>
              <p class="text-xs font-mono text-theme-text/50 truncate">
                Location: {props.info()!.path}
              </p>
            </Show>
          </div>
        </Show>

        <Show when={!props.checking() && !props.info()?.available}>
          <div class="space-y-3">
            <div class="flex items-center gap-2 text-xs text-gi-status-danger">
              <AlertCircle size={14} />
              <span class="font-semibold">cloudflared was not found on your system.</span>
            </div>

            {/* Auto-Install Button */}
            <button
              onClick={props.onInstall}
              disabled={props.installing()}
              class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-theme-cyan/20 hover:bg-theme-cyan/30 border border-theme-cyan/40 text-theme-cyan font-semibold text-sm transition-all disabled:opacity-50"
            >
              <Show when={props.installing()} fallback={<Download size={16} />}>
                <Loader size={16} class="animate-spin" />
              </Show>
              <span>{props.installing() ? 'Downloading / Installing cloudflared...' : 'Auto Install'}</span>
            </button>

            <Show when={props.installMsg()}>
              <p
                class={`text-xs px-3 py-2 border font-medium ${
                  props.installMsg()!.toLowerCase().includes('failed')
                    ? 'bg-gi-status-danger/10 border-gi-status-danger/30 text-gi-status-danger'
                    : 'bg-theme-cyan/15 border-theme-cyan/35 text-theme-cyan'
                }`}
              >
                {props.installMsg()}
              </p>
            </Show>

            {/* Manual fallback guide */}
            <details class="group">
              <summary class="text-xs text-theme-text/60 hover:text-theme-text cursor-pointer select-none">
                Manual installation guide (click to expand)
              </summary>
              <div class="mt-2 grid grid-cols-1 sm:grid-cols-2 gap-2">
                <For each={installCommands}>
                  {(item) => (
                    <div class="p-3 bg-theme-bg border border-theme-border space-y-2">
                      <span class="text-xs font-bold text-theme-text/70 uppercase tracking-wide">
                        {item.os}
                      </span>
                      <div class="flex items-center gap-1.5">
                        <code class="text-xs font-mono bg-theme-card px-2 py-1 flex-1 border border-theme-border text-theme-cyan truncate">
                          {item.cmd}
                        </code>
                        <button
                          onClick={() => props.onCopyCmd(item.cmd)}
                          class="p-1 text-theme-text/60 hover:text-theme-text hover:bg-theme-border transition-colors shrink-0"
                          title="Copy Command"
                        >
                          <Show when={props.copiedCmd() === item.cmd} fallback={<Copy size={13} />}>
                            <Check size={13} class="text-theme-cyan" />
                          </Show>
                        </button>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </details>
          </div>
        </Show>
      </div>
    </div>
  );
};