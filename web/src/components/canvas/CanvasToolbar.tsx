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
import { CanvasTool } from './types';
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
} from './canvasStore';
import { fabricService } from './fabricService';
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
    <div class="bg-theme-card border border-theme-border p-3 flex flex-wrap items-center justify-between gap-3 shadow-md">
      {/* 1. Compact Drawing Tool Selector with Active Shape Dropdown */}
      <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60">
        {/* Select & Brush */}
        <For each={primaryTools}>
          {(t) => {
            const Icon = t.icon;
            const isSelected = () => activeTool() === t.id;
            return (
              <button
                type="button"
                onClick={() => handleToolChange(t.id)}
                title={t.label}
                class={`p-2 transition-all flex items-center justify-center ${
                  isSelected()
                    ? 'bg-theme-cyan text-theme-card font-bold shadow-sm anime-shadow-green'
                    : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card'
                }`}
              >
                <Icon size={16} class={isSelected() ? 'stroke-[2.5]' : 'stroke-[2]'} />
              </button>
            );
          }}
        </For>

        {/* Shapes / Objects Dropdown (Shows Current Active Shape) */}
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
                title={`Bentuk Objek: ${activeShapeInfo.label} (Klik untuk ganti bentuk)`}
                class={`p-2 transition-all flex items-center gap-1 ${
                  isShapeActive()
                    ? 'bg-theme-cyan text-theme-card font-bold shadow-sm anime-shadow-green'
                    : 'text-theme-text/70 hover:text-theme-text hover:bg-theme-card'
                }`}
              >
                <ActiveIcon size={16} class={isShapeActive() ? 'stroke-[2.5]' : 'stroke-[2]'} />
                <ChevronDown size={11} class={`transition-transform duration-150 ${showShapeMenu() ? 'rotate-180' : ''}`} />
              </button>
            );
          })()}

          {/* Shape Dropdown Popover */}
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
                      class={`w-full flex items-center justify-between px-2.5 py-1.5 text-xs font-mono transition-all text-left ${
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

        {/* Eraser Tool */}
        <button
          type="button"
          onClick={() => handleToolChange('eraser')}
          title="Penghapus Objek (Klik objek untuk menghapus)"
          class={`p-2 transition-all flex items-center justify-center ${
            activeTool() === 'eraser'
              ? 'bg-theme-rose text-theme-card font-bold shadow-sm'
              : 'text-theme-text/70 hover:text-theme-rose hover:bg-theme-card'
          }`}
        >
          <Eraser size={16} class={activeTool() === 'eraser' ? 'stroke-[2.5]' : 'stroke-[2]'} />
        </button>
      </div>

      {/* 2. Dynamic Object Contextual Inspector Toolbar */}
      <InspectorToolbar effectiveType={effectiveType()} />

      {/* 3. Actions: Undo, Redo, Clear, AI Drawer, Export */}
      <div class="flex items-center gap-1.5">
        {/* Zoom & Snap Grid Controls */}
        <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60">
          <button
            type="button"
            onClick={() => setSnapToGrid(!snapToGrid())}
            title={snapToGrid() ? 'Snap to Grid Dot: Aktif (Klik untuk matikan)' : 'Snap to Grid Dot: Mati (Klik untuk aktifkan)'}
            class={`p-2 transition-all flex items-center justify-center ${
              snapToGrid()
                ? 'bg-theme-cyan text-theme-card shadow-sm anime-shadow-green'
                : 'text-theme-text/60 hover:text-theme-text hover:bg-theme-card'
            }`}
          >
            <Magnet size={15} class={snapToGrid() ? 'stroke-[2.5]' : 'stroke-[2]'} />
          </button>

          <button
            type="button"
            onClick={() => fabricService.zoomOut()}
            title="Zoom Out (Ctrl - / Scroll Down)"
            class="p-1.5 text-theme-text/80 hover:text-theme-text hover:bg-theme-card transition-all"
          >
            <ZoomOut size={14} />
          </button>
          <button
            type="button"
            onClick={() => fabricService.resetZoom()}
            title="Reset Zoom ke 100% (Ctrl 0)"
            class="px-1.5 py-0.5 text-xs font-mono font-bold text-theme-cyan hover:bg-theme-card transition-all"
          >
            {zoomLevel()}%
          </button>
          <button
            type="button"
            onClick={() => fabricService.zoomIn()}
            title="Zoom In (Ctrl + / Scroll Up)"
            class="p-1.5 text-theme-text/80 hover:text-theme-text hover:bg-theme-card transition-all"
          >
            <ZoomIn size={14} />
          </button>
        </div>

        <button
          type="button"
          onClick={() => fabricService.undo()}
          disabled={!canUndo() || isAiDrawing()}
          title="Undo (Ctrl+Z)"
          class="p-2 bg-theme-surface text-theme-text/80 hover:text-theme-text border border-theme-border disabled:opacity-40 disabled:pointer-events-none hover:bg-theme-card transition-all"
        >
          <Undo2 size={16} />
        </button>

        <button
          type="button"
          onClick={() => fabricService.redo()}
          disabled={!canRedo() || isAiDrawing()}
          title="Redo (Ctrl+Y)"
          class="p-2 bg-theme-surface text-theme-text/80 hover:text-theme-text border border-theme-border disabled:opacity-40 disabled:pointer-events-none hover:bg-theme-card transition-all"
        >
          <Redo2 size={16} />
        </button>

        <button
          type="button"
          onClick={() => fabricService.clear()}
          disabled={isAiDrawing()}
          title="Bersihkan Semua Canvas"
          class="p-2 bg-theme-surface text-theme-rose hover:bg-theme-rose/20 border border-theme-border hover:border-theme-rose/40 disabled:opacity-40 disabled:pointer-events-none transition-all"
        >
          <Trash2 size={16} />
        </button>

        {/* ── Dropdown Menu Pengaturan & Aksi ── */}
        <div class="relative" ref={dropdownRef}>
          <button
            type="button"
            onClick={() => setShowSettingsMenu(!showSettingsMenu())}
            title="Pengaturan & Aksi Tambahan"
            class={`px-3 py-2 text-xs font-bold border transition-all flex items-center gap-1.5 ${
              showSettingsMenu()
                ? 'bg-theme-cyan text-theme-card border-theme-cyan shadow-sm anime-shadow-green font-bold'
                : 'bg-theme-surface text-theme-text/80 hover:text-theme-text border-theme-border hover:bg-theme-card'
            }`}
          >
            <Settings size={15} class={showSettingsMenu() ? 'animate-spin-slow' : ''} />
            <span class="hidden sm:inline">Pengaturan</span>
            <ChevronDown size={13} class={`transition-transform duration-200 ${showSettingsMenu() ? 'rotate-180' : ''}`} />
          </button>

          {/* Hidden file input for Image import */}
          <input
            ref={fileInputRef}
            type="file"
            accept="image/png,image/jpeg,image/webp,image/svg+xml"
            onChange={handleImageFileChange}
            class="hidden"
          />

          <Show when={showSettingsMenu()}>
            <div class="absolute right-0 top-full mt-1.5 w-60 bg-theme-card border border-theme-border shadow-2xl p-1.5 z-50 flex flex-col gap-1">
              {/* 1. Ekspor Kanvas (Featured) */}
              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  props.onOpenExportModal();
                }}
                class="w-full px-3 py-2 text-xs font-bold bg-theme-cyan/20 hover:bg-theme-cyan text-theme-cyan hover:text-theme-card border border-theme-cyan/40 flex items-center gap-2.5 transition-all text-left"
              >
                <Download size={15} />
                <span>Ekspor Kanvas...</span>
              </button>

              <div class="h-px bg-theme-border/60 my-0.5" />

              {/* 2. AI MCP Console */}
              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  setShowAiSimulator(!showAiSimulator());
                }}
                class="w-full px-3 py-2 text-xs font-semibold text-theme-text hover:bg-theme-surface hover:text-amber-300 flex items-center justify-between transition-all text-left"
              >
                <div class="flex items-center gap-2.5">
                  <Bot size={15} class="text-amber-400" />
                  <span>AI MCP Live Console</span>
                </div>
                <Show when={isAiDrawing()}>
                  <span class="w-2 h-2 rounded-full bg-amber-400 animate-ping" />
                </Show>
              </button>

              {/* 3. Panel Layers */}
              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  setShowLayersPanel(!showLayersPanel());
                }}
                class="w-full px-3 py-2 text-xs font-semibold text-theme-text hover:bg-theme-surface hover:text-theme-cyan flex items-center justify-between transition-all text-left"
              >
                <div class="flex items-center gap-2.5">
                  <Layers size={15} class="text-theme-cyan" />
                  <span>{showLayersPanel() ? 'Sembunyikan Layers' : 'Tampilkan Layers'}</span>
                </div>
                <Show when={layersList().length > 0}>
                  <span class="text-[10px] px-1 font-mono font-bold bg-theme-surface border border-theme-border text-theme-cyan">
                    {layersList().length}
                  </span>
                </Show>
              </button>

              {/* 4. Import Image */}
              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  fileInputRef?.click();
                }}
                class="w-full px-3 py-2 text-xs font-semibold text-theme-text hover:bg-theme-surface hover:text-theme-cyan flex items-center gap-2.5 transition-all text-left"
              >
                <ImageIcon size={15} class="text-theme-cyan" />
                <span>Import Gambar...</span>
              </button>

              {/* 5. Fullscreen */}
              <button
                type="button"
                onClick={() => {
                  setShowSettingsMenu(false);
                  setIsFullscreen(!isFullscreen());
                }}
                class="w-full px-3 py-2 text-xs font-semibold text-theme-text hover:bg-theme-surface hover:text-theme-cyan flex items-center gap-2.5 transition-all text-left"
              >
                {isFullscreen() ? <Minimize2 size={15} class="text-theme-rose" /> : <Maximize2 size={15} class="text-theme-cyan" />}
                <span>{isFullscreen() ? 'Keluar Fullscreen' : 'Layar Penuh (F)'}</span>
              </button>

              {/* 6. Shortcuts Keyboard Modal */}
              <Show when={props.onOpenShortcutsModal}>
                <button
                  type="button"
                  onClick={() => {
                    setShowSettingsMenu(false);
                    props.onOpenShortcutsModal?.();
                  }}
                  class="w-full px-3 py-2 text-xs font-semibold text-theme-text hover:bg-theme-surface hover:text-theme-cyan flex items-center gap-2.5 transition-all text-left border-t border-theme-border/40 mt-0.5 pt-1.5"
                >
                  <Keyboard size={15} class="text-theme-text/70" />
                  <span>Daftar Shortcut...</span>
                </button>
              </Show>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};
