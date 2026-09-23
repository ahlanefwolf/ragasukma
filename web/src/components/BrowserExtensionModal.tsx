import { Component, createSignal, createEffect, Show } from 'solid-js';
import { X, Globe, Check, Puzzle, ExternalLink, Folder } from 'lucide-solid';
import { api } from '../api';

interface BrowserExtensionModalProps {
  isOpen: boolean;
  onClose: () => void;
  showToast: (msg: string) => void;
}

export const BrowserExtensionModal: Component<BrowserExtensionModalProps> = (props) => {
  const [copiedType, setCopiedType] = createSignal<'folder' | 'manifest' | null>(null);
  const [folderPath, setFolderPath] = createSignal<string>('extension');
  const [manifestPath, setManifestPath] = createSignal<string>('extension/manifest.json');

  createEffect(() => {
    if (props.isOpen) {
      api.getExtensionPath().then((data) => {
        if (data?.folder) {
          setFolderPath(data.folder);
          setManifestPath(data.manifest);
        }
      });
    }
  });

  const copyFolder = () => {
    navigator.clipboard.writeText(folderPath());
    setCopiedType('folder');
    props.showToast('📁 Path folder ekstensi disalin!');
    setTimeout(() => setCopiedType(null), 2500);
  };

  const copyManifest = () => {
    navigator.clipboard.writeText(manifestPath());
    setCopiedType('manifest');
    props.showToast('📁 Path manifest.json Firefox disalin!');
    setTimeout(() => setCopiedType(null), 2500);
  };

  const openFileManager = async () => {
    props.showToast('Membuka folder ekstensi di File Manager...');
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
        class="fixed inset-0 z-[150] flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-fade-in select-none"
        onClick={props.onClose}
      >
        <div 
          class="bg-theme-surface border border-theme-border w-full max-w-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh] text-theme-text"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div class="flex items-center justify-between p-4 sm:p-5 border-b border-theme-border bg-theme-bg shrink-0">
            <div class="flex items-center gap-3">
              <div class="p-2.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
                <Puzzle size={20} />
              </div>
              <div>
                <h3 class="text-sm sm:text-base font-bold text-theme-text font-heading">
                  Pasang Ekstensi Browser Ragasukma
                </h3>
                <p class="text-xs text-theme-text/70 mt-0.5">
                  Dibutuhkan agar AI bisa membaca tab browser aktif, scroll, klik DOM, dan mencari di web.
                </p>
              </div>
            </div>

            <button
              onClick={props.onClose}
              class="p-2 text-theme-text/60 hover:text-theme-text hover:bg-theme-card border border-transparent hover:border-theme-border transition-colors cursor-pointer"
              title="Tutup Modal"
            >
              <X size={18} />
            </button>
          </div>

          {/* Modal Body */}
          <div class="p-4 sm:p-6 overflow-y-auto space-y-5 text-xs select-text">
            {/* Folder Opener Banner */}
            <div class="p-4 bg-theme-card border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-3 shadow-sm">
              <div class="space-y-1">
                <div class="font-bold text-theme-text flex items-center gap-2">
                  <Folder size={16} class="text-theme-cyan" />
                  <span class="text-xs uppercase tracking-wider font-mono">Folder Sumber Ekstensi:</span>
                </div>
                <code class="text-[11px] font-mono text-theme-cyan block select-all bg-theme-bg px-2.5 py-1.5 border border-theme-border font-medium">
                  {folderPath()}
                </code>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  onClick={copyFolder}
                  class="px-3.5 py-2 bg-theme-bg hover:bg-theme-surface border border-theme-border text-theme-text font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-sm active:scale-95"
                  title="Salin path direktori ekstensi"
                >
                  <Show when={copiedType() === 'folder'} fallback={<Check size={14} class="opacity-40" />}>
                    <Check size={14} class="text-theme-cyan" />
                  </Show>
                  <span>{copiedType() === 'folder' ? 'Disalin!' : 'Salin Path'}</span>
                </button>

                <button
                  type="button"
                  onClick={openFileManager}
                  class="px-3.5 py-2 bg-theme-cyan hover:bg-[#4a5a3a] text-theme-card border border-theme-cyan font-bold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer active:scale-95"
                  title="Buka langsung folder ini di File Manager OS"
                >
                  <ExternalLink size={14} />
                  <span>Buka Folder</span>
                </button>
              </div>
            </div>

            {/* Browser Cards */}
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              {/* Chrome / Brave / Edge */}
              <div class="p-4 sm:p-5 bg-theme-bg border border-theme-border hover:border-theme-cyan/50 transition-all flex flex-col justify-between gap-4 shadow-sm">
                <div class="space-y-3.5">
                  <div class="pb-2 border-b border-theme-border/70">
                    <span class="px-2.5 py-1 text-xs font-bold uppercase bg-theme-cyan/20 text-[#29483A] border border-theme-cyan/40 tracking-wide font-mono">
                      Chrome / Brave / Edge
                    </span>
                  </div>

                  <div class="space-y-2.5 text-theme-text/85 leading-relaxed">
                    <div class="flex items-start gap-2.5">
                      <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-[#29483A] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                        1
                      </span>
                      <span>Buka halaman <code class="px-1 py-0.5 bg-theme-card border border-theme-border font-mono text-[11px]">chrome://extensions</code> di browser.</span>
                    </div>

                    <div class="flex items-start gap-2.5">
                      <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-[#29483A] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                        2
                      </span>
                      <span>Nyalakan toggle <strong>Developer mode</strong> di pojok kanan atas.</span>
                    </div>

                    <div class="flex items-start gap-2.5">
                      <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-[#29483A] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                        3
                      </span>
                      <span>Klik <strong>Load unpacked</strong>, lalu pilih folder ekstensi di atas.</span>
                    </div>
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2 pt-2 border-t border-theme-border/60">
                  <button
                    type="button"
                    onClick={() => {
                      navigator.clipboard.writeText("chrome://extensions");
                      window.open("chrome://extensions", "_blank");
                      props.showToast("🌐 URL chrome://extensions disalin! Jika tidak terbuka otomatis, tempel di tab baru.");
                    }}
                    class="py-2 px-3 bg-theme-cyan/15 hover:bg-theme-cyan text-[#29483A] hover:text-theme-card border border-theme-cyan/30 font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-sm"
                  >
                    <Globe size={14} />
                    <span>Buka Extensions</span>
                  </button>

                  <button
                    type="button"
                    onClick={copyFolder}
                    class="py-2 px-3 bg-theme-card hover:bg-theme-surface border border-theme-border text-theme-text font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-sm"
                    title="Salin path folder ekstensi"
                  >
                    <Show when={copiedType() === 'folder'} fallback={<Check size={14} class="opacity-40" />}>
                      <Check size={14} class="text-theme-cyan" />
                    </Show>
                    <span>Salin Folder</span>
                  </button>
                </div>
              </div>

              {/* Firefox */}
              <div class="p-4 sm:p-5 bg-theme-bg border border-theme-border hover:border-[#B08A4A]/50 transition-all flex flex-col justify-between gap-4 shadow-sm">
                <div class="space-y-3.5">
                  <div class="pb-2 border-b border-theme-border/70">
                    <span class="px-2.5 py-1 text-xs font-bold uppercase bg-[#B08A4A]/15 text-[#876447] border border-[#B08A4A]/30 tracking-wide font-mono">
                      Mozilla Firefox
                    </span>
                  </div>

                  <div class="space-y-2.5 text-theme-text/85 leading-relaxed">
                    <div class="flex items-start gap-2.5">
                      <span class="w-5 h-5 rounded-full bg-[#B08A4A]/20 text-[#876447] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-[#B08A4A]/30">
                        1
                      </span>
                      <span>Buka halaman <code class="px-1 py-0.5 bg-theme-card border border-theme-border font-mono text-[11px]">about:debugging</code> di browser.</span>
                    </div>

                    <div class="flex items-start gap-2.5">
                      <span class="w-5 h-5 rounded-full bg-[#B08A4A]/20 text-[#876447] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-[#B08A4A]/30">
                        2
                      </span>
                      <span>Pilih menu <strong>This Firefox</strong> pada panel sebelah kiri.</span>
                    </div>

                    <div class="flex items-start gap-2.5">
                      <span class="w-5 h-5 rounded-full bg-[#B08A4A]/20 text-[#876447] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-[#B08A4A]/30">
                        3
                      </span>
                      <span>Klik <strong>Load Temporary Add-on</strong>, pilih file <code class="px-1 py-0.5 bg-theme-card border border-theme-border font-mono text-[11px]">manifest.json</code>.</span>
                    </div>
                  </div>
                </div>

                <div class="grid grid-cols-2 gap-2 pt-2 border-t border-theme-border/60">
                  <button
                    type="button"
                    onClick={() => {
                      navigator.clipboard.writeText("about:debugging#/runtime/this-firefox");
                      window.open("about:debugging#/runtime/this-firefox", "_blank");
                      props.showToast("🦊 URL about:debugging disalin! Jika tidak terbuka otomatis, tempel di tab baru.");
                    }}
                    class="py-2 px-3 bg-[#B08A4A]/15 hover:bg-[#B08A4A] text-[#876447] hover:text-theme-card border border-[#B08A4A]/30 font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-sm"
                  >
                    <Globe size={14} />
                    <span>Buka Debugging</span>
                  </button>

                  <button
                    type="button"
                    onClick={copyManifest}
                    class="py-2 px-3 bg-theme-card hover:bg-theme-surface border border-theme-border text-theme-text font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-sm"
                    title="Salin path file manifest.json"
                  >
                    <Show when={copiedType() === 'manifest'} fallback={<Check size={14} class="opacity-40" />}>
                      <Check size={14} class="text-[#B08A4A]" />
                    </Show>
                    <span>Salin Manifest</span>
                  </button>
                </div>
              </div>
            </div>
          </div>

          {/* Modal Footer */}
          <div class="p-3.5 sm:p-4 bg-theme-bg border-t border-theme-border flex items-center justify-between text-xs text-theme-text/70 shrink-0">
            <span>Ekstensi aman & hanya membaca tab aktif saat diberi perintah.</span>
            <button
              type="button"
              onClick={props.onClose}
              class="px-4 py-1.5 bg-theme-card hover:bg-theme-surface border border-theme-border text-theme-text font-semibold transition-all cursor-pointer"
            >
              Tutup
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
