import { Component, createSignal, createEffect, Show } from 'solid-js';
import { X, Globe, Check, Puzzle, ExternalLink, Folder } from 'lucide-solid';
import { api } from '../api';

interface BrowserExtensionModalProps {
  isOpen: boolean;
  onClose: () => void;
  showToast: (msg: string) => void;
}

export const BrowserExtensionModal: Component<BrowserExtensionModalProps> = (props) => {
  const [copiedType, setCopiedType] = createSignal<'crx' | null>(null);
  const [crxPath, setCrxPath] = createSignal<string>('assets/ragasukma-companion.crx');

  createEffect(() => {
    if (props.isOpen) {
      api.getExtensionPath().then((data) => {
        if (data?.crx) {
          setCrxPath(data.crx);
        } else if (data?.folder) {
          setCrxPath(`${data.folder}/ragasukma-companion.crx`);
        }
      });
    }
  });

  const copyCrx = () => {
    navigator.clipboard.writeText(crxPath());
    setCopiedType('crx');
    props.showToast('📁 Path file ragasukma-companion.crx disalin!');
    setTimeout(() => setCopiedType(null), 2500);
  };

  const openFileManager = async () => {
    props.showToast('Membuka folder assets di File Manager...');
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
          {/* Modal Header */}
          <div class="flex items-center justify-between p-4 sm:p-5 border-b border-theme-border bg-theme-bg shrink-0">
            <div class="flex items-center gap-3">
              <div class="p-2.5 bg-theme-cyan/20 text-[#29483A] border border-theme-cyan/40">
                <Puzzle size={20} />
              </div>
              <div>
                <h3 class="text-base sm:text-lg font-bold text-theme-text flex items-center gap-2">
                  Pasang Ekstensi Browser (.CRX)
                  <span class="text-[10px] font-mono font-bold px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan">
                    Single File Packaged
                  </span>
                </h3>
                <p class="text-xs text-theme-text/70 mt-0.5">
                  Ekstensi paket Chromium untuk kontrol penuh tab, YouTube & musik
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
            {/* File .CRX Banner */}
            <div class="p-4 bg-theme-card border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-3 shadow-sm">
              <div class="space-y-1">
                <div class="font-bold text-theme-text flex items-center gap-2">
                  <Puzzle size={16} class="text-theme-cyan" />
                  <span class="text-xs uppercase tracking-wider font-mono">File Paket Ekstensi (.CRX):</span>
                </div>
                <code class="text-[11px] font-mono text-theme-cyan block select-all bg-theme-bg px-2.5 py-1.5 border border-theme-border font-medium">
                  {crxPath()}
                </code>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <button
                  type="button"
                  onClick={copyCrx}
                  class="px-3.5 py-2 bg-theme-bg hover:bg-theme-surface border border-theme-border text-theme-text font-bold flex items-center gap-1.5 transition-all cursor-pointer shadow-sm active:scale-95"
                  title="Salin path file .crx"
                >
                  <Show when={copiedType() === 'crx'} fallback={<Check size={14} class="opacity-40" />}>
                    <Check size={14} class="text-theme-cyan" />
                  </Show>
                  <span>{copiedType() === 'crx' ? 'Disalin!' : 'Salin Path CRX'}</span>
                </button>

                <button
                  type="button"
                  onClick={openFileManager}
                  class="px-3.5 py-2 bg-theme-cyan hover:bg-[#4a5a3a] text-theme-card border border-theme-cyan font-bold flex items-center gap-1.5 transition-all shadow-sm cursor-pointer active:scale-95"
                  title="Buka folder di File Manager OS"
                >
                  <ExternalLink size={14} />
                  <span>Buka Folder</span>
                </button>
              </div>
            </div>

            {/* Chromium Card */}
            <div class="p-4 sm:p-5 bg-theme-bg border border-theme-border hover:border-theme-cyan/50 transition-all flex flex-col justify-between gap-4 shadow-sm">
              <div class="space-y-3.5">
                <div class="pb-2 border-b border-theme-border/70 flex items-center justify-between">
                  <span class="px-2.5 py-1 text-xs font-bold uppercase bg-theme-cyan/20 text-[#29483A] border border-theme-cyan/40 tracking-wide font-mono">
                    Cara Pasang di Chrome / Brave / Edge
                  </span>
                  <span class="text-[11px] font-mono text-theme-cyan font-semibold">1-Step Drag & Drop</span>
                </div>

                <div class="space-y-2.5 text-theme-text/85 leading-relaxed text-xs sm:text-sm">
                  <div class="flex items-start gap-2.5">
                    <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-[#29483A] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                      1
                    </span>
                    <span>Buka tab baru dan ketik alamat <code class="px-1.5 py-0.5 bg-theme-card border border-theme-border font-mono text-xs font-bold text-theme-cyan">chrome://extensions</code> di browser Chromium.</span>
                  </div>

                  <div class="flex items-start gap-2.5">
                    <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-[#29483A] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                      2
                    </span>
                    <span>Aktifkan toggle <strong>Developer mode</strong> di pojok kanan atas jendela browser.</span>
                  </div>

                  <div class="flex items-start gap-2.5">
                    <span class="w-5 h-5 rounded-full bg-theme-cyan/20 text-[#29483A] font-mono font-bold text-[10px] flex items-center justify-center shrink-0 mt-0.5 border border-theme-cyan/30">
                      3
                    </span>
                    <div>
                      <span><strong>Drag & Drop</strong> (seret dan lepas) file <code class="px-1 py-0.5 bg-theme-card border border-theme-border font-mono text-[11px] text-theme-cyan">ragasukma-companion.crx</code> langsung ke halaman extension browser, lalu klik <strong>Add extension</strong>.</span>
                      <p class="text-[11px] text-theme-text/60 mt-1 italic">
                        *Jika muncul label "Chrome can't verify where this extension comes from", ini 100% normal untuk file CRX lokal dan ekstensi tetap aktif berjalan (status On).
                      </p>
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
                    props.showToast("🌐 URL chrome://extensions disalin! Jika tidak terbuka otomatis, tempel di tab baru.");
                  }}
                  class="w-full py-2.5 px-3 bg-theme-cyan/15 hover:bg-theme-cyan text-[#29483A] hover:text-theme-card border border-theme-cyan/30 font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer shadow-sm"
                >
                  <Globe size={14} />
                  <span>Buka Halaman chrome://extensions</span>
                </button>
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
