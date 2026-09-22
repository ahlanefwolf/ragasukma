import { Component, For, Show, createSignal, onCleanup, onMount } from 'solid-js';
import {
  MousePointer,
  Pencil,
  Square,
  Circle as CircleIcon,
  Minus,
  MoveRight,
  Type,
  Eraser,
  Undo2,
  Redo2,
  Trash2,
  Download,
  Image as ImageIcon,
  Bot,
  Maximize2,
  Minimize2,
  ZoomIn,
  ZoomOut,
  Magnet,
  Layers,
  Keyboard,
  Settings,
  ChevronDown,
} from 'lucide-solid';
import { CanvasTool } from '../types';
import {
  activeTool,
  setActiveTool,
  selectedObjectType,
  canUndo,
  canRedo,
  isAiDrawing,
  showAiSimulator,
  setShowAiSimulator,
  isFullscreen,
  setIsFullscreen,
  zoomLevel,
  snapToGrid,
  setSnapToGrid,
  showLayersPanel,
  setShowLayersPanel,
  layersList,
} from '../canvasStore';
import { fabricService } from '../fabricService';
import { InspectorToolbar } from './InspectorToolbar';

interface CanvasToolbarProps {
  onOpenExportModal: () => void;
  onOpenShortcutsModal?: () => void;
  showToast: (msg: string) => void;
}

export const CanvasToolbar: Component<CanvasToolbarProps> = (props) => {
  const primaryTools: { id: CanvasTool; label: string; icon: any }[] = [
    { id: 'select', label: 'Pilih / Geser', icon: MousePointer },
    { id: 'brush', label: 'Kuas Bebas', icon: Pencil },
  ];

  const shapeTools: { id: CanvasTool; label: string; icon: any; shortcut: string }[] = [
    { id: 'rectangle', label: 'Kotak (Rect)', icon: Square, shortcut: 'R' },
    { id: 'circle', label: 'Lingkaran', icon: CircleIcon, shortcut: 'C' },
    { id: 'arrow', label: 'Panah Relasi', icon: MoveRight, shortcut: 'A' },
    { id: 'line', label: 'Garis Lurus', icon: Minus, shortcut: 'L' },
    { id: 'text', label: 'Teks Label', icon: Type, shortcut: 'T' },
  ];

  const [selectedShape, setSelectedShape] = createSignal<CanvasTool>('rectangle');
  const [showShapeMenu, setShowShapeMenu] = createSignal(false);
  let shapeDropdownRef: HTMLDivElement | undefined;

  const currentActiveShape = () => {
    const active = activeTool();
    const found = shapeTools.find((s) => s.id === active);
    return found || shapeTools.find((s) => s.id === selectedShape()) || shapeTools[0];
  };

  const handleToolChange = (tool: CanvasTool) => {
    setActiveTool(tool);
    if (shapeTools.some((s) => s.id === tool)) {
      setSelectedShape(tool);
    }
    fabricService.updateTool(tool);
  };

  let fileInputRef: HTMLInputElement | undefined;
  const [showSettingsMenu, setShowSettingsMenu] = createSignal(false);
  let dropdownRef: HTMLDivElement | undefined;

  const handleClickOutside = (e: MouseEvent) => {
    if (dropdownRef && !dropdownRef.contains(e.target as Node)) {
      setShowSettingsMenu(false);
    }
    if (shapeDropdownRef && !shapeDropdownRef.contains(e.target as Node)) {
      setShowShapeMenu(false);
    }
  };

  onMount(() => {
    document.addEventListener('pointerdown', handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener('pointerdown', handleClickOutside);
  });

  const handleImageFileChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;
    const file = target.files[0];
    const reader = new FileReader();

    reader.onload = async (ev) => {
      const dataUrl = ev.target?.result as string;
      if (dataUrl) {
        props.showToast(`Memuat gambar: ${file.name}...`);
        const ok = await fabricService.addImageFromDataUrl(dataUrl);
        if (ok) {
          props.showToast('✨ Gambar berhasil ditambahkan ke kanvas!');
        } else {
          props.showToast('⚠️ Gagal memuat gambar ke kanvas');
        }
      }
    };
    reader.readAsDataURL(file);
    target.value = '';
  };

  const effectiveType = () => {
    const sel = selectedObjectType();
    if (sel) {
      if (sel === 'rect') return 'rect';
      if (sel === 'circle') return 'circle';
      if (sel === 'line' || sel === 'arrow') return 'line';
      if (sel === 'i-text' || sel === 'text') return 'text';
      if (sel === 'path') return 'brush';
      if (sel === 'activeselection' || sel === 'group') return 'group';
      return sel;
    }
    const tool = activeTool();
    if (tool === 'rectangle') return 'rect';
    if (tool === 'circle') return 'circle';
    if (tool === 'arrow' || tool === 'line') return 'line';
    if (tool === 'text') return 'text';
    if (tool === 'brush') return 'brush';
    if (tool === 'eraser') return 'eraser';
    return 'select';
  };

  return (
    <div class="relative z-30 bg-theme-card border-b border-theme-border px-3 py-1.5 flex items-center justify-between gap-2 shadow-sm select-none">
      {/* ── LEFT: Tools & Contextual Inspector (Photoshop/Illustrator Options Bar) ── */}
      <div class="flex items-center gap-2 py-0.5">
        {/* Drawing Tools Group */}
        <div class="flex items-center gap-0.5 bg-theme-surface p-0.5 border border-theme-border/70 shrink-0">
          <For each={primaryTools}>
            {(t) => {
              const Icon = t.icon;
              const isSelected = () => activeTool() === t.id;
              return (
                <button
                  type="button"
                  onClick={() => handleToolChange(t.id)}
                  title={t.label}
                  class={`p-1.5 transition-all flex items-center justify-center ${
                    isSelected()
                      ? 'bg-theme-cyan text-theme-card font-bold shadow-sm'
                      : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card'
                  }`}
                >
                  <Icon size={15} class={isSelected() ? 'stroke-[2.5]' : 'stroke-[2]'} />
                </button>
              );
            }}
          </For>

          {/* Shapes Dropdown */}
          <div class="relative" ref={shapeDropdownRef}>
            {(() => {
              const activeShapeInfo = currentActiveShape();
              const ActiveIcon = activeShapeInfo.icon;
              const isShapeActive = () => shapeTools.some((s) => s.id === activeTool());
              return (
                <button
                  type="button"
                  onClick={() => {
                    if (!isShapeActive()) {
                      handleToolChange(activeShapeInfo.id);
                    }
                    setShowShapeMenu(!showShapeMenu());
                  }}
                  title={`Bentuk: ${activeShapeInfo.label} (Klik untuk ganti)`}
                  class={`p-1.5 transition-all flex items-center gap-0.5 ${
                    isShapeActive()
                      ? 'bg-theme-cyan text-theme-card font-bold shadow-sm'
                      : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card'
                  }`}
                >
                  <ActiveIcon size={15} class={isShapeActive() ? 'stroke-[2.5]' : 'stroke-[2]'} />
                  <ChevronDown size={10} class={`transition-transform duration-150 ${showShapeMenu() ? 'rotate-180' : ''}`} />
                </button>
              );
            })()}

            <Show when={showShapeMenu()}>
              <div class="absolute left-0 top-full mt-1.5 w-44 bg-theme-card border border-theme-border shadow-2xl p-1 z-50 flex flex-col gap-0.5 animate-in fade-in zoom-in-95 duration-100">
                <For each={shapeTools}>
                  {(shape) => {
                    const Icon = shape.icon;
                    const isCurrent = () => activeTool() === shape.id;
                    return (
                      <button
                        type="button"
                        onClick={() => {
                          handleToolChange(shape.id);
                          setShowShapeMenu(false);
                        }}
                        class={`w-full flex items-center justify-between px-2 py-1.5 text-xs font-mono transition-all text-left ${
                          isCurrent()
                            ? 'bg-theme-cyan text-theme-card font-bold shadow-sm'
                            : 'text-theme-text/80 hover:text-theme-cyan hover:bg-theme-surface'
                        }`}
                      >
                        <div class="flex items-center gap-2">
                          <Icon size={14} class={isCurrent() ? 'text-theme-card' : 'text-theme-cyan'} />
                          <span>{shape.label}</span>
                        </div>
                        <span class={`text-[10px] ${isCurrent() ? 'text-theme-card/80' : 'text-theme-text/40'}`}>
                          {shape.shortcut}
                        </span>
                      </button>
                    );
                  }}
                </For>
              </div>
            </Show>
          </div>

          {/* Eraser */}
          <button
            type="button"
            onClick={() => handleToolChange('eraser')}
            title="Penghapus"
            class={`p-1.5 transition-all flex items-center justify-center ${
              activeTool() === 'eraser'
                ? 'bg-theme-rose text-theme-card font-bold shadow-sm'
                : 'text-theme-text/70 hover:text-theme-rose hover:bg-theme-card'
            }`}
          >
            <Eraser size={15} class={activeTool() === 'eraser' ? 'stroke-[2.5]' : 'stroke-[2]'} />
          </button>
        </div>

        {/* Separator */}
        <div class="h-5 w-[1px] bg-theme-border/60 shrink-0" />

        {/* Contextual Properties Bar (Illustrator-style Inspector) */}
        <div class="shrink-0">
          <InspectorToolbar />
        </div>
      </div>

      {/* ── RIGHT: Navigation & Workspace Actions (Compact Icon Row) ── */}
      <div class="flex items-center gap-1 shrink-0">
        {/* Snap Grid Toggle */}
        <button
          type="button"
          onClick={() => setSnapToGrid(!snapToGrid())}
          title={snapToGrid() ? 'Snap to Grid: Aktif' : 'Snap to Grid: Mati'}
          class={`p-1.5 transition-all flex items-center justify-center border ${
            snapToGrid()
              ? 'bg-theme-cyan text-theme-card border-theme-cyan shadow-sm font-bold'
              : 'bg-theme-surface text-theme-text/60 border-theme-border hover:bg-theme-card'
          }`}
        >
          <Magnet size={14} class={snapToGrid() ? 'stroke-[2.5]' : 'stroke-[2]'} />
        </button>

        {/* Zoom Controls */}
        <div class="flex items-center bg-theme-surface border border-theme-border/80">
          <button
            type="button"
            onClick={() => fabricService.zoomOut()}
            title="Zoom Out (Ctrl -)"
            class="p-1 text-theme-text/70 hover:text-theme-text hover:bg-theme-card"
          >
            <ZoomOut size={13} />
          </button>
          <button
            type="button"
            onClick={() => fabricService.resetZoom()}
            title="Reset Zoom 100% (Ctrl 0)"
            class="px-1 text-[11px] font-mono font-bold text-theme-cyan hover:bg-theme-card"
          >
            {zoomLevel()}%
          </button>
          <button
            type="button"
            onClick={() => fabricService.zoomIn()}
            title="Zoom In (Ctrl +)"
            class="p-1 text-theme-text/70 hover:text-theme-text hover:bg-theme-card"
          >
            <ZoomIn size={13} />
          </button>
        </div>

        {/* Undo / Redo */}
        <div class="flex items-center bg-theme-surface border border-theme-border/80">
          <button
            type="button"
            onClick={() => fabricService.undo()}
            disabled={!canUndo() || isAiDrawing()}
            title="Undo (Ctrl+Z)"
            class="p-1.5 text-theme-text/80 hover:text-theme-text disabled:opacity-30 disabled:pointer-events-none hover:bg-theme-card"
          >
            <Undo2 size={14} />
          </button>
          <button
            type="button"
            onClick={() => fabricService.redo()}
            disabled={!canRedo() || isAiDrawing()}
            title="Redo (Ctrl+Y)"
            class="p-1.5 text-theme-text/80 hover:text-theme-text disabled:opacity-30 disabled:pointer-events-none hover:bg-theme-card"
          >
            <Redo2 size={14} />
          </button>
        </div>

        {/* Clear Canvas */}
        <button
          type="button"
          onClick={() => fabricService.clear()}
          disabled={isAiDrawing()}
          title="Bersihkan Semua Kanvas"
          class="p-1.5 bg-theme-surface text-theme-rose hover:bg-theme-rose/20 border border-theme-border hover:border-theme-rose/40 disabled:opacity-30 disabled:pointer-events-none transition-all"
        >
          <Trash2 size={14} />
        </button>

        {/* Fullscreen */}
        <button
          type="button"
          onClick={() => setIsFullscreen(!isFullscreen())}
          title={isFullscreen() ? 'Keluar Fullscreen (F)' : 'Fullscreen (F)'}
          class={`p-1.5 transition-all flex items-center justify-center border ${
            isFullscreen()
              ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
              : 'bg-theme-surface text-theme-text/80 border-theme-border hover:bg-theme-card'
          }`}
        >
          {isFullscreen() ? <Minimize2 size={14} class="stroke-[2.5]" /> : <Maximize2 size={14} />}
        </button>

        {/* Settings & Extra Menu */}
        <div class="relative" ref={dropdownRef}>
          <button
            type="button"
            onClick={() => setShowSettingsMenu(!showSettingsMenu())}
            title="Menu & Opsi Tambahan"
            class={`p-1.5 transition-all flex items-center justify-center border ${
              showSettingsMenu()
                ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold'
                : 'bg-theme-surface text-theme-text/80 border-theme-border hover:bg-theme-card'
            }`}
          >
            <Settings size={14} class={showSettingsMenu() ? 'animate-spin-slow' : ''} />
          </button>

          <input
            ref={fileInputRef}
            type="file"
            accept="image/png,image/jpeg,image/webp,image/svg+xml"
            onChange={handleImageFileChange}
            class="hidden"
          />

          <Show when={showSettingsMenu()}>
            <div class="absolute right-0 top-full mt-1.5 w-56 bg-theme-card border border-theme-border shadow-2xl p-1 z-50 flex flex-col gap-0.5">
              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  props.onOpenExportModal();
                }}
                class="w-full px-2.5 py-1.5 text-xs font-bold bg-theme-cyan/20 hover:bg-theme-cyan text-theme-cyan hover:text-theme-card border border-theme-cyan/40 flex items-center gap-2 transition-all text-left"
              >
                <Download size={14} />
                <span>Ekspor Kanvas (PNG/SVG)</span>
              </button>

              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  fileInputRef?.click();
                }}
                class="w-full px-2.5 py-1.5 text-xs text-theme-text hover:text-theme-cyan hover:bg-theme-surface flex items-center gap-2 transition-all text-left font-mono"
              >
                <ImageIcon size={14} class="text-theme-cyan" />
                <span>Unggah Gambar...</span>
              </button>

              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  setShowLayersPanel(!showLayersPanel());
                }}
                class="w-full px-2.5 py-1.5 text-xs text-theme-text hover:text-theme-cyan hover:bg-theme-surface flex items-center justify-between transition-all font-mono"
              >
                <div class="flex items-center gap-2">
                  <Layers size={14} class="text-theme-cyan" />
                  <span>Panel Layer</span>
                </div>
                <span class="text-[10px] bg-theme-surface px-1.5 py-0.5 border border-theme-border">
                  {layersList().length}
                </span>
              </button>

              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  setShowAiSimulator(!showAiSimulator());
                }}
                class="w-full px-2.5 py-1.5 text-xs text-theme-text hover:text-amber-300 hover:bg-theme-surface flex items-center gap-2 transition-all text-left font-mono"
              >
                <Bot size={14} class="text-amber-400" />
                <span>Demo AI MCP Drawer</span>
              </button>

              <Show when={props.onOpenShortcutsModal}>
                <button
                  type="button"
                  onClick={() => {
                    setShowSettingsMenu(false);
                    props.onOpenShortcutsModal?.();
                  }}
                  class="w-full px-2.5 py-1.5 text-xs text-theme-text hover:text-theme-cyan hover:bg-theme-surface flex items-center gap-2 transition-all text-left font-mono border-t border-theme-border/60 mt-0.5"
                >
                  <Keyboard size={14} class="text-theme-text/60" />
                  <span>Shortcut Keyboard</span>
                </button>
              </Show>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};
