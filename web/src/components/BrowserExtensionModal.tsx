import { Component, createSignal, createEffect, Show } from 'solid-js';
import { X, Globe, Check, Puzzle, ExternalLink, Folder } from 'lucide-solid';
import { api } from '../api';

interface BrowserExtensionModalProps {
  isOpen: boolean;
  onClose: () => void;
  showToast: (msg: string) => void;
}

export const BrowserExtensionModal: Component<BrowserExtensionModalProps> = (props) => {
  const [copiedType, setCopiedType] = createSignal<'zip' | null>(null);
  const [zipPath, setZipPath] = createSignal<string>('assets/ragasukma-companion.zip');

  createEffect(() => {
    if (props.isOpen) {
      api.getExtensionPath().then((data) => {
        if (data?.zip) {
          setZipPath(data.zip);
        } else if (data?.folder) {
          setZipPath(`${data.folder}/ragasukma-companion.zip`);
        }
      });
    }
  });

  const copyZip = () => {
    navigator.clipboard.writeText(zipPath());
    setCopiedType('zip');
    props.showToast('ragasukma-companion.zip path copied!');
    setTimeout(() => setCopiedType(null), 2500);
  };

  const openFileManager = async () => {
    props.showToast('Opening assets folder in File Manager...');
    try {
      const ok = await api.runSkill('internal:open_extension_installer');
      if (!ok) {
        await api.runSkill('browser:open_ext_folder');
      }
    } catch {}
  };

  return (
    <Show when={props.isOpen}>
      <div 
        class="fixed inset-0 z-[150] !mt-0 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-fade-in select-none"
        onClick={props.onClose}
      >
        <div 
          class="bg-theme-surface border border-theme-border w-full max-w-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] text-theme-text"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Modal Header */}
          <div class="flex items-center justify-between p-4 sm:p-5 border-b border-theme-border bg-theme-bg shrink-0">
            <div class="flex items-start gap-3">
              <div class="p-2.5 bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 mt-0.5">
                <Puzzle size={20} />
              </div>
              <div>
                <h3 class="text-theme-text flex items-center gap-2 m-0 leading-tight">
                  Install Browser Extension (.ZIP)
                  <span class="text-2xs font-mono font-bold px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan">
                    Cross-Platform
                  </span>
                </h3>
                <p class="text-xs text-theme-text/70 mt-0.5">
                  Chromium extension package for tab control, YouTube, and media playback
                </p>
              </div>
            </div>
            <button
              onClick={props.onClose}
              class="p-2 text-theme-text/60 hover:text-theme-text hover:bg-theme-card border border-transparent hover:border-theme-border transition-colors cursor-pointer"
              title="Close Modal"
            >
              <X size={18} />
            </button>
          </div>

          {/* Modal Body */}
          <div class="p-4 sm:p-6 overflow-y-auto space-y-5 text-xs select-text">
            {/* File .ZIP Banner */}
            <div class="p-4 bg-theme-card border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-3 shadow-sm">
              <div class="space-y-1">
                <div class="font-bold text-theme-text flex items-center gap-2">
                  <Puzzle size={16} class="text-theme-cyan" />
                  <span class="text-xs uppercase tracking-wider font-mono">Extension Package (.ZIP):</span>
                </div>
                <code class="text-xs font-mono text-theme-cyan block select-all bg-theme-bg px-2.5 py-1.5 border border-theme-border font-medium">
                  {zipPath()}
                </code>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  onClick={copyZip}
                  class="px-3.5 py-2 bg-theme-bg hover:bg-theme-surface border border-theme-border text-theme-text font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-sm active:scale-95"
                  title="Copy .zip file path"
                >
                  <Show when={copiedType() === 'zip'} fallback={<Check size={14} class="opacity-40" />}>
                    <Check size={14} class="text-theme-cyan" />
                  </Show>
                  <span>{copiedType() === 'zip' ? 'Copied!' : 'Copy ZIP Path'}</span>
                </button>

                <button
                  type="button"
                  onClick={openFileManager}
                  class="p-2 bg-theme-cyan hover:bg-theme-cyan/85 text-theme-card border border-theme-cyan font-bold flex items-center justify-center transition-all shadow-sm cursor-pointer active:scale-95"
                  title="Open folder in OS File Manager"
                  aria-label="Open folder in OS File Manager"
                >
                  <Folder size={15} />
                </button>
              </div>
            </div>

            {/* Chromium Card */}
            <div class="p-4 sm:p-5 bg-theme-bg border border-theme-border hover:border-theme-cyan/50 transition-all flex flex-col justify-between gap-4 shadow-sm">
              <div class="space-y-3.5">
                <div class="pb-2 border-b border-theme-border/70 flex items-center justify-between">
                  <span class="px-2.5 py-1 text-xs font-bold uppercase bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 tracking-wide font-mono">
                    Installation for Chrome / Brave / Edge
                  </span>
                  <span class="text-xs font-mono text-theme-cyan font-semibold">1-Step Unpack</span>
                </div>

                <div class="space-y-2.5 text-theme-text/85 leading-relaxed text-xs sm:text-sm">
                  <div class="flex items-start gap-2.5">
                    <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-theme-cyan font-mono font-bold text-2xs flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                      1
                    </span>
                    <span>Extract <code class="px-1.5 py-0.5 bg-theme-card border border-theme-border font-mono text-xs font-bold text-theme-cyan">ragasukma-companion.zip</code> to a folder of your choice (e.g. the <code>extension</code> folder).</span>
                  </div>

                  <div class="flex items-start gap-2.5">
                    <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-theme-cyan font-mono font-bold text-2xs flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                      2
                    </span>
                    <span>Open a new tab and navigate to <code class="px-1.5 py-0.5 bg-theme-card border border-theme-border font-mono text-xs font-bold text-theme-cyan">chrome://extensions</code> in Chromium, then toggle on <strong>Developer mode</strong> in the top-right corner.</span>
                  </div>

                  <div class="flex items-start gap-2.5">
                    <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-theme-cyan font-mono font-bold text-2xs flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                      3
                    </span>
                    <div>
                      <span>Click <strong>Load unpacked</strong> and select the extracted folder. The extension will activate immediately!</span>
                    </div>
                  </div>
                </div>
              </div>

              <div class="pt-2 border-t border-theme-border/60">
                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText("chrome://extensions");
                    window.open("chrome://extensions", "_blank");
                    props.showToast("chrome://extensions URL copied! If it did not open automatically, paste it in a new tab.");
                  }}
                  class="w-full py-2.5 px-3 bg-theme-cyan/15 hover:bg-theme-cyan text-theme-text hover:text-theme-card border border-theme-cyan/30 font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-sm"
                >
                  <Globe size={14} />
                  <span>Open chrome://extensions Page</span>
                </button>
              </div>
            </div>
          </div>

          {/* Modal Footer */}
          <div class="p-3.5 sm:p-4 bg-theme-bg border-t border-theme-border flex items-center justify-between text-xs text-theme-text/70 shrink-0">
            <span>The extension runs locally and interacts with tabs only upon direct voice commands.</span>
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
