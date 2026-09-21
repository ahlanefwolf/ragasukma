import { Component, createSignal, createEffect, onMount, Show, For } from 'solid-js';
import { 
  Download, 
  X, 
  Check, 
  Lock, 
  Unlock, 
  Layers, 
  Crop, 
  Sliders, 
  Monitor, 
  Sparkles 
} from 'lucide-solid';
import { fabricService } from './fabricService';

interface ExportModalProps {
  isOpen: boolean;
  onClose: () => void;
  showToast: (msg: string) => void;
}

export const ExportModal: Component<ExportModalProps> = (props) => {
  const [format, setFormat] = createSignal<'png' | 'jpeg' | 'svg'>('png');
  const [scope, setScope] = createSignal<'selection' | 'canvas'>('canvas');
  const [hasSelection, setHasSelection] = createSignal<boolean>(false);
  const [background, setBackground] = createSignal<'transparent' | 'white' | 'dark'>('transparent');
  
  // Base dimensions from detected source
  const [baseWidth, setBaseWidth] = createSignal<number>(800);
  const [baseHeight, setBaseHeight] = createSignal<number>(600);

  // User target export dimensions
  const [targetWidth, setTargetWidth] = createSignal<number>(800);
  const [targetHeight, setTargetHeight] = createSignal<number>(600);
  const [isRatioLocked, setIsRatioLocked] = createSignal<boolean>(true);

  // Sync state when modal is opened
  createEffect(() => {
    if (props.isOpen) {
      const info = fabricService.getExportScopeInfo();
      setHasSelection(info.hasSelection);

      // Default to selection if user actively selected an object
      const defaultScope = info.hasSelection ? 'selection' : 'canvas';
      setScope(defaultScope);

      const bW = defaultScope === 'selection' ? info.selectionWidth : info.canvasWidth;
      const bH = defaultScope === 'selection' ? info.selectionHeight : info.canvasHeight;
      setBaseWidth(bW);
      setBaseHeight(bH);
      setTargetWidth(bW);
      setTargetHeight(bH);
    }
  });

  const handleScopeChange = (newScope: 'selection' | 'canvas') => {
    setScope(newScope);
    const info = fabricService.getExportScopeInfo();
    const bW = newScope === 'selection' ? info.selectionWidth : info.canvasWidth;
    const bH = newScope === 'selection' ? info.selectionHeight : info.canvasHeight;
    setBaseWidth(bW);
    setBaseHeight(bH);
    setTargetWidth(bW);
    setTargetHeight(bH);
  };

  const getAspectRatio = () => {
    const w = baseWidth();
    const h = baseHeight();
    return h > 0 ? w / h : 1;
  };

  const handleWidthChange = (w: number) => {
    const validW = Math.max(10, Math.min(8192, Math.round(w)));
    setTargetWidth(validW);
    if (isRatioLocked()) {
      const ratio = getAspectRatio();
      setTargetHeight(Math.max(10, Math.round(validW / ratio)));
    }
  };

  const handleHeightChange = (h: number) => {
    const validH = Math.max(10, Math.min(8192, Math.round(h)));
    setTargetHeight(validH);
    if (isRatioLocked()) {
      const ratio = getAspectRatio();
      setTargetWidth(Math.max(10, Math.round(validH * ratio)));
    }
  };

  const handleScaleMultiplier = (mult: number) => {
    const newW = Math.round(baseWidth() * mult);
    const newH = Math.round(baseHeight() * mult);
    setTargetWidth(newW);
    setTargetHeight(newH);
  };

  const handleSetExactPreset = (presetWidth: number) => {
    handleWidthChange(presetWidth);
  };

  const formats = [
    {
      id: 'png' as const,
      label: 'PNG Image',
      tag: 'Lossless + Alpha',
      desc: 'Transparan default, tajam untuk dokumen & slide presentasi.',
    },
    {
      id: 'jpeg' as const,
      label: 'JPG Image',
      tag: 'Standard Web',
      desc: 'Ukuran file ringan dengan latar belakang solid (putih/gelap).',
    },
    {
      id: 'svg' as const,
      label: 'SVG Vector',
      tag: 'Infinite Scale',
      desc: 'Format vektor berbasis XML, tidak pecah di-zoom tak terbatas.',
    },
  ];

  const bgOptions = [
    { id: 'transparent' as const, label: 'Transparan (Alpha)', preview: 'bg-[radial-gradient(#555_1px,transparent_1px)] [background-size:6px_6px] bg-[#1a1a1a]' },
    { id: 'dark' as const, label: 'Tema Gelap (#10141D)', preview: 'bg-[#10141D] border border-theme-cyan/30' },
    { id: 'white' as const, label: 'Putih Bersih (#FFFFFF)', preview: 'bg-white text-black' },
  ];

  const handleExport = () => {
    const fmt = format();
    const sc = scope();
    const bg = background();
    const tW = targetWidth();
    const tH = targetHeight();

    const output = fabricService.exportCanvas({
      format: fmt,
      scope: sc,
      width: tW,
      height: tH,
      background: bg,
    });

    if (!output) {
      props.showToast('⚠️ Gagal mengekspor kanvas');
      return;
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
    const scopeTag = sc === 'selection' ? 'selection' : 'canvas';
    const filename = `ragasukma-${scopeTag}-${tW}x${tH}-${timestamp}.${fmt === 'jpeg' ? 'jpg' : fmt}`;

    if (fmt === 'svg') {
      const blob = new Blob([output], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      link.click();
      URL.revokeObjectURL(url);
    } else {
      const link = document.createElement('a');
      link.href = output;
      link.download = filename;
      link.click();
    }

    props.showToast(`✨ Berhasil mengekspor (${tW}x${tH} px): ${filename}`);
    props.onClose();
  };

  const getResolutionBadge = () => {
    const w = targetWidth();
    if (w >= 3840) return { label: '4K Ultra HD', color: 'text-theme-rose bg-theme-rose/15 border-theme-rose/30' };
    if (w >= 2560) return { label: '2K QHD', color: 'text-amber-400 bg-amber-400/15 border-amber-400/30' };
    if (w >= 1920) return { label: 'Full HD 1080p', color: 'text-theme-cyan bg-theme-cyan/15 border-theme-cyan/30' };
    if (w >= 1280) return { label: 'HD 720p', color: 'text-emerald-400 bg-emerald-400/15 border-emerald-400/30' };
    return { label: 'Standard Web', color: 'text-theme-text/70 bg-theme-surface border-theme-border' };
  };

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-[150] flex items-center justify-center bg-black/75 backdrop-blur-sm p-4 animate-in fade-in duration-150">
        <div class="bg-theme-card border border-theme-cyan/50 w-full max-w-2xl shadow-2xl overflow-hidden flex flex-col max-h-[92vh]">
          {/* Modal Header */}
          <div class="bg-gradient-to-r from-theme-surface to-theme-bg p-4 border-b border-theme-border flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <div class="p-2 bg-theme-cyan/20 border border-theme-cyan/40 text-theme-cyan">
                <Download size={18} />
              </div>
              <div>
                <h3 class="text-base font-bold text-theme-text font-heading uppercase tracking-wide">
                  Ekspor Kanvas & Objek Terpilih
                </h3>
                <p class="text-xs text-theme-text/60">
                  Kustomisasi format vektor/raster, rasio terkunci, hingga resolusi 4K.
                </p>
              </div>
            </div>
            <button
              type="button"
              onClick={props.onClose}
              class="text-theme-text/60 hover:text-theme-rose p-1.5 transition-colors"
            >
              <X size={18} />
            </button>
          </div>

          {/* Modal Body */}
          <div class="p-4 sm:p-5 space-y-4 overflow-y-auto">
            {/* 1. Scope Selection (Selection vs Whole Canvas) */}
            <div>
              <div class="flex items-center justify-between mb-2">
                <label class="text-xs font-mono font-bold text-theme-cyan uppercase flex items-center gap-1.5">
                  <Crop size={14} />
                  1. Area / Objek yang Diekspor
                </label>
                <Show when={hasSelection()}>
                  <span class="text-[11px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 border border-emerald-500/30">
                    ✓ Objek Aktif Terdeteksi ({baseWidth()}x{baseHeight()} px)
                  </span>
                </Show>
              </div>

              <div class="grid grid-cols-2 gap-2">
                <button
                  type="button"
                  disabled={!hasSelection()}
                  onClick={() => handleScopeChange('selection')}
                  class={`p-3 text-left border transition-all flex items-center justify-between ${
                    !hasSelection()
                      ? 'opacity-40 cursor-not-allowed bg-theme-surface border-theme-border'
                      : scope() === 'selection'
                      ? 'bg-theme-cyan/15 border-theme-cyan font-bold shadow-sm anime-shadow-green'
                      : 'bg-theme-surface border-theme-border hover:bg-theme-card'
                  }`}
                >
                  <div class="flex items-center gap-2.5">
                    <Crop size={16} class={scope() === 'selection' ? 'text-theme-cyan' : 'text-theme-text/60'} />
                    <div>
                      <div class="text-xs font-mono font-bold text-theme-text uppercase">
                        Hanya Objek Terpilih
                      </div>
                      <div class="text-[10.5px] text-theme-text/60 mt-0.5">
                        {hasSelection() ? `Crop sesuai bounding box (${baseWidth()}x{baseHeight()} px)` : 'Tidak ada objek yang dipilih di kanvas'}
                      </div>
                    </div>
                  </div>
                  <Show when={scope() === 'selection'}>
                    <Check size={14} class="text-theme-cyan" />
                  </Show>
                </button>

                <button
                  type="button"
                  onClick={() => handleScopeChange('canvas')}
                  class={`p-3 text-left border transition-all flex items-center justify-between ${
                    scope() === 'canvas'
                      ? 'bg-theme-cyan/15 border-theme-cyan font-bold shadow-sm anime-shadow-green'
                      : 'bg-theme-surface border-theme-border hover:bg-theme-card'
                  }`}
                >
                  <div class="flex items-center gap-2.5">
                    <Layers size={16} class={scope() === 'canvas' ? 'text-theme-cyan' : 'text-theme-text/60'} />
                    <div>
                      <div class="text-xs font-mono font-bold text-theme-text uppercase">
                        Seluruh Area Kanvas
                      </div>
                      <div class="text-[10.5px] text-theme-text/60 mt-0.5">
                        Mengekspor seluruh tampilan papan gambar
                      </div>
                    </div>
                  </div>
                  <Show when={scope() === 'canvas'}>
                    <Check size={14} class="text-theme-cyan" />
                  </Show>
                </button>
              </div>
            </div>

            {/* 2. Format Selection */}
            <div>
              <label class="block text-xs font-mono font-bold text-theme-cyan uppercase mb-2">
                2. Format Berkas
              </label>
              <div class="grid grid-cols-3 gap-2">
                <For each={formats}>
                  {(f) => (
                    <button
                      type="button"
                      onClick={() => setFormat(f.id)}
                      class={`p-2.5 text-left border transition-all flex flex-col justify-between ${
                        format() === f.id
                          ? 'bg-theme-cyan/15 border-theme-cyan shadow-sm anime-shadow-green'
                          : 'bg-theme-surface border-theme-border/80 hover:bg-theme-card'
                      }`}
                    >
                      <div class="flex items-center justify-between w-full mb-1">
                        <span class="text-xs font-bold font-mono text-theme-text uppercase">
                          {f.label}
                        </span>
                        <Show when={format() === f.id}>
                          <Check size={14} class="text-theme-cyan" />
                        </Show>
                      </div>
                      <span class="text-[9.5px] px-1.5 py-0.5 w-max font-mono bg-theme-bg/80 border border-theme-border text-theme-cyan mb-1">
                        {f.tag}
                      </span>
                      <p class="text-[10px] text-theme-text/60 leading-tight">
                        {f.desc}
                      </p>
                    </button>
                  )}
                </For>
              </div>
            </div>

            {/* 3. Pixel Dimensions, Aspect Ratio Lock & 4K Slider */}
            <div class="bg-theme-surface p-3.5 border border-theme-border/80 space-y-3">
              <div class="flex items-center justify-between">
                <label class="text-xs font-mono font-bold text-theme-cyan uppercase flex items-center gap-1.5">
                  <Sliders size={14} />
                  3. Ukuran Piksel & Kunci Rasio (Hingga 4K)
                </label>
                <div class={`text-[11px] font-mono px-2 py-0.5 border ${getResolutionBadge().color}`}>
                  {getResolutionBadge().label} ({targetWidth()} x {targetHeight()} px)
                </div>
              </div>

              {/* Number Inputs with Lock toggle */}
              <div class="flex items-center gap-3">
                <div class="flex-1">
                  <span class="text-[10px] font-mono text-theme-text/60 uppercase block mb-1">
                    Lebar (Width px):
                  </span>
                  <div class="flex items-center bg-theme-card border border-theme-border px-2 py-1.5">
                    <input
                      type="number"
                      min="10"
                      max="8192"
                      value={targetWidth()}
                      onInput={(e) => handleWidthChange(parseInt(e.currentTarget.value || '0', 10))}
                      class="w-full bg-transparent text-sm font-mono font-bold text-theme-text focus:outline-none"
                    />
                    <span class="text-xs font-mono text-theme-text/50">px</span>
                  </div>
                </div>

                {/* Aspect Ratio Lock Toggle */}
                <button
                  type="button"
                  onClick={() => setIsRatioLocked(!isRatioLocked())}
                  title={isRatioLocked() ? 'Rasio Terkunci (Klik untuk buka)' : 'Rasio Bebas (Klik untuk kunci)'}
                  class={`p-2.5 mt-4 border transition-all flex items-center justify-center ${
                    isRatioLocked()
                      ? 'bg-theme-cyan text-theme-card border-theme-cyan shadow-sm anime-shadow-green'
                      : 'bg-theme-card text-theme-text/50 border-theme-border hover:text-theme-text'
                  }`}
                >
                  {isRatioLocked() ? <Lock size={16} /> : <Unlock size={16} />}
                </button>

                <div class="flex-1">
                  <span class="text-[10px] font-mono text-theme-text/60 uppercase block mb-1">
                    Tinggi (Height px):
                  </span>
                  <div class="flex items-center bg-theme-card border border-theme-border px-2 py-1.5">
                    <input
                      type="number"
                      min="10"
                      max="8192"
                      value={targetHeight()}
                      onInput={(e) => handleHeightChange(parseInt(e.currentTarget.value || '0', 10))}
                      class="w-full bg-transparent text-sm font-mono font-bold text-theme-text focus:outline-none"
                    />
                    <span class="text-xs font-mono text-theme-text/50">px</span>
                  </div>
                </div>
              </div>

              {/* Smooth Scale Slider (Up to 4K: 4096px Width) */}
              <div class="space-y-1.5 pt-1">
                <div class="flex items-center justify-between text-[11px] font-mono text-theme-text/60">
                  <span>Slider Resolusi:</span>
                  <span class="text-theme-cyan font-bold">
                    {targetWidth()} px ({((targetWidth() / Math.max(1, baseWidth())) * 100).toFixed(0)}%)
                  </span>
                </div>
                <input
                  type="range"
                  min={Math.max(100, Math.round(baseWidth() * 0.25))}
                  max="4096"
                  value={targetWidth()}
                  onInput={(e) => handleWidthChange(parseInt(e.currentTarget.value, 10))}
                  class="w-full accent-theme-cyan h-2 bg-theme-card cursor-pointer"
                />
                <div class="flex justify-between text-[9px] font-mono text-theme-text/40">
                  <span>0.5x</span>
                  <span>1x (Asli: {baseWidth()}px)</span>
                  <span>2x HD</span>
                  <span>4K UHD (3840-4096px)</span>
                </div>
              </div>

              {/* Quick Multiplier & Industry Standard Presets */}
              <div class="flex flex-wrap items-center gap-1.5 pt-1">
                <span class="text-[10.5px] font-mono text-theme-text/60 mr-1">Preset:</span>
                <button
                  type="button"
                  onClick={() => handleScaleMultiplier(1)}
                  class="px-2 py-1 text-[10.5px] font-mono bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:border-theme-cyan transition-all"
                >
                  1x ({baseWidth()}px)
                </button>
                <button
                  type="button"
                  onClick={() => handleScaleMultiplier(2)}
                  class="px-2 py-1 text-[10.5px] font-mono bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-cyan hover:border-theme-cyan transition-all font-bold"
                >
                  2x Retina
                </button>
                <button
                  type="button"
                  onClick={() => handleSetExactPreset(1920)}
                  class="px-2 py-1 text-[10.5px] font-mono bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:border-theme-cyan transition-all"
                >
                  1080p (1920px)
                </button>
                <button
                  type="button"
                  onClick={() => handleSetExactPreset(2560)}
                  class="px-2 py-1 text-[10.5px] font-mono bg-theme-card hover:bg-theme-bg border border-theme-border text-amber-300 hover:border-theme-cyan transition-all font-bold"
                >
                  2K (2560px)
                </button>
                <button
                  type="button"
                  onClick={() => handleSetExactPreset(3840)}
                  class="px-2 py-1 text-[10.5px] font-mono bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-rose hover:border-theme-rose transition-all font-bold"
                >
                  4K UHD (3840px)
                </button>
              </div>
            </div>

            {/* 4. Background Color */}
            <div>
              <label class="block text-xs font-mono font-bold text-theme-cyan uppercase mb-2">
                4. Latar Belakang (Background)
              </label>
              <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
                <For each={bgOptions}>
                  {(b) => {
                    const isSelected = () => background() === b.id;
                    const isDisabled = () => format() === 'jpeg' && b.id === 'transparent';
                    return (
                      <button
                        type="button"
                        disabled={isDisabled()}
                        onClick={() => setBackground(b.id)}
                        class={`p-2.5 border transition-all flex items-center gap-2.5 text-left ${
                          isDisabled() ? 'opacity-40 cursor-not-allowed' : ''
                        } ${
                          isSelected()
                            ? 'bg-theme-cyan/15 border-theme-cyan font-bold'
                            : 'bg-theme-surface border-theme-border hover:bg-theme-card'
                        }`}
                      >
                        <div class={`w-5 h-5 shrink-0 ${b.preview}`} />
                        <span class="text-xs text-theme-text font-medium">
                          {b.label}
                        </span>
                      </button>
                    );
                  }}
                </For>
              </div>
              <Show when={format() === 'jpeg'}>
                <p class="text-[11px] text-amber-400 mt-1">
                  * Format JPG tidak mendukung lapisan transparan, otomatis menggunakan latar solid.
                </p>
              </Show>
            </div>
          </div>

          {/* Modal Footer */}
          <div class="bg-theme-surface p-4 border-t border-theme-border flex items-center justify-between gap-3">
            <div class="text-xs font-mono text-theme-text/70 hidden sm:block">
              Target: <span class="text-theme-cyan font-bold">{targetWidth()} x {targetHeight()} px</span> | Format: <span class="text-theme-cyan uppercase font-bold">{format()}</span>
            </div>

            <div class="flex items-center gap-2 w-full sm:w-auto justify-end">
              <button
                type="button"
                onClick={props.onClose}
                class="px-4 py-2 text-xs font-bold text-theme-text/80 hover:text-theme-text bg-theme-card border border-theme-border hover:bg-theme-surface transition-all"
              >
                Batal
              </button>
              <button
                type="button"
                onClick={handleExport}
                class="px-5 py-2 text-xs font-bold bg-theme-cyan hover:bg-[#68cbdd] text-theme-card border border-theme-cyan shadow-md anime-shadow-green flex items-center gap-2 transition-all"
              >
                <Download size={15} />
                <span>Unduh ({targetWidth()}x{targetHeight()} px)</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};
