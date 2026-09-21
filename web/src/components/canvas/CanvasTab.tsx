import { Component, onMount, onCleanup, Show, createSignal } from 'solid-js';
import { Palette, Sparkles, Keyboard, Info, X, Image as ImageIcon } from 'lucide-solid';
import { CanvasToolbar } from './CanvasToolbar';
import { LayersPanel } from './LayersPanel';
import { ExportModal } from './ExportModal';
import { ShortcutsModal } from './ShortcutsModal';
import { AiDrawingPanel } from './AiDrawingPanel';
import { fabricService } from './fabricService';
import {
  showAiSimulator,
  isAiDrawing,
  setActiveTool,
  isFullscreen,
  setIsFullscreen,
  showLayersPanel,
} from './canvasStore';
import { CanvasTool } from './types';

interface CanvasTabProps {
  showToast: (msg: string) => void;
}

export const CanvasTab: Component<CanvasTabProps> = (props) => {
  let canvasContainerRef: HTMLDivElement | undefined;
  let canvasElRef: HTMLCanvasElement | undefined;
  let resizeObserver: ResizeObserver | null = null;
  const [showShortcutsModal, setShowShortcutsModal] = createSignal(false);
  const [showExportModal, setShowExportModal] = createSignal(false);
  const [isDragOver, setIsDragOver] = createSignal(false);

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      if (file.type.startsWith('image/')) {
        const reader = new FileReader();
        reader.onload = async (ev) => {
          const dataUrl = ev.target?.result as string;
          if (dataUrl) {
            props.showToast(`Memuat gambar: ${file.name}...`);
            const ok = await fabricService.addImageFromDataUrl(dataUrl);
            if (ok) {
              props.showToast('✨ Gambar berhasil ditambahkan ke kanvas!');
            }
          }
        };
        reader.readAsDataURL(file);
      }
    }
  };

  const handlePaste = (e: ClipboardEvent) => {
    // If user is editing text in an input or canvas text, let native paste work
    const activeEl = document.activeElement;
    const isInputFocused =
      activeEl &&
      (activeEl.tagName === 'INPUT' ||
        activeEl.tagName === 'TEXTAREA' ||
        activeEl.getAttribute('contenteditable') === 'true');

    if (isInputFocused || fabricService.isEditingText()) {
      return;
    }

    const items = e.clipboardData?.items;
    let foundImage = false;

    if (items) {
      for (let i = 0; i < items.length; i++) {
        if (items[i].type.indexOf('image') !== -1) {
          foundImage = true;
          const blob = items[i].getAsFile();
          if (blob) {
            const reader = new FileReader();
            reader.onload = async (ev) => {
              const dataUrl = ev.target?.result as string;
              if (dataUrl) {
                props.showToast('Menempelkan gambar dari clipboard...');
                const ok = await fabricService.addImageFromDataUrl(dataUrl);
                if (ok) {
                  props.showToast('✨ Gambar dari clipboard berhasil ditempelkan!');
                }
              }
            };
            reader.readAsDataURL(blob);
          }
          break;
        }
      }
    }

    // If not an image and internal canvas has copied object, paste internal object
    if (!foundImage && fabricService.hasClipboard()) {
      e.preventDefault();
      fabricService.paste().then((ok) => {
        if (ok) {
          props.showToast('✨ Objek berhasil ditempelkan!');
        }
      });
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    // If typing in input / textarea or editing canvas text, ignore tool shortcuts
    const activeEl = document.activeElement;
    const isInputFocused =
      activeEl &&
      (activeEl.tagName === 'INPUT' ||
        activeEl.tagName === 'TEXTAREA' ||
        activeEl.getAttribute('contenteditable') === 'true');

    if (isInputFocused || fabricService.isEditingText()) {
      return;
    }

    const isCtrlOrMeta = e.ctrlKey || e.metaKey;

    // Zoom shortcuts
    if (isCtrlOrMeta && (e.key === '=' || e.key === '+')) {
      e.preventDefault();
      fabricService.zoomIn();
      return;
    }
    if (isCtrlOrMeta && e.key === '-') {
      e.preventDefault();
      fabricService.zoomOut();
      return;
    }
    if (isCtrlOrMeta && e.key === '0') {
      e.preventDefault();
      fabricService.resetZoom();
      return;
    }

    // Spacebar to pan
    if (e.code === 'Space' && !e.repeat) {
      fabricService.setSpacePressed(true);
    }

    // Copy: Ctrl+C
    if (isCtrlOrMeta && e.key.toLowerCase() === 'c' && !e.shiftKey) {
      fabricService.copy().then((ok) => {
        if (ok) {
          props.showToast('📋 Objek disalin ke clipboard!');
        }
      });
      return;
    }

    // Duplicate: Ctrl+D
    if (isCtrlOrMeta && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      fabricService.duplicate().then((ok) => {
        if (ok) {
          props.showToast('✨ Objek berhasil diduplikasi!');
        }
      });
      return;
    }

    // Paste: Ctrl+V (fallback if clipboard event doesn't fire or for internal objects)
    if (isCtrlOrMeta && e.key.toLowerCase() === 'v') {
      if (fabricService.hasClipboard()) {
        e.preventDefault();
        fabricService.paste().then((ok) => {
          if (ok) {
            props.showToast('✨ Objek berhasil ditempelkan!');
          }
        });
        return;
      }
    }

    // Undo: Ctrl+Z (without Shift)
    if (isCtrlOrMeta && e.key.toLowerCase() === 'z' && !e.shiftKey) {
      e.preventDefault();
      fabricService.undo();
      return;
    }

    // Redo: Ctrl+Y or Ctrl+Shift+Z
    if (
      (isCtrlOrMeta && e.key.toLowerCase() === 'y') ||
      (isCtrlOrMeta && e.shiftKey && e.key.toLowerCase() === 'z')
    ) {
      e.preventDefault();
      fabricService.redo();
      return;
    }

    // Grouping: Ctrl+G
    if (isCtrlOrMeta && e.key.toLowerCase() === 'g' && !e.shiftKey) {
      e.preventDefault();
      fabricService.groupSelected();
      props.showToast('🔗 Objek berhasil digabung menjadi 1 Group!');
      return;
    }

    // Ungrouping: Ctrl+Shift+G
    if (isCtrlOrMeta && e.key.toLowerCase() === 'g' && e.shiftKey) {
      e.preventDefault();
      fabricService.ungroupSelected();
      props.showToast('🔓 Group berhasil dipisahkan menjadi objek mandiri.');
      return;
    }

    // Delete selected object
    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault();
      fabricService.deleteSelected();
      return;
    }

    // Escape to exit fullscreen
    if (e.key === 'Escape' && isFullscreen()) {
      e.preventDefault();
      setIsFullscreen(false);
      return;
    }

    // Toggle Fullscreen: F key
    if (e.key.toLowerCase() === 'f' && !isCtrlOrMeta && !e.altKey) {
      e.preventDefault();
      setIsFullscreen(!isFullscreen());
      return;
    }

    // Tool Switch Shortcuts (Single key, no Ctrl/Meta)
    if (!isCtrlOrMeta && !e.altKey) {
      const key = e.key.toLowerCase();
      let selectedTool: CanvasTool | null = null;

      if (key === 'b' || key === 'p') selectedTool = 'brush';
      else if (key === 'v' || key === 's') selectedTool = 'select';
      else if (key === 'r') selectedTool = 'rectangle';
      else if (key === 'c') selectedTool = 'circle';
      else if (key === 'a') selectedTool = 'arrow';
      else if (key === 'l') selectedTool = 'line';
      else if (key === 't') selectedTool = 'text';
      else if (key === 'e') selectedTool = 'eraser';

      if (selectedTool) {
        e.preventDefault();
        setActiveTool(selectedTool);
        fabricService.updateTool(selectedTool);
      }
    }
  };

  onMount(() => {
    if (!canvasContainerRef || !canvasElRef) return;

    const width = canvasContainerRef.clientWidth || 800;
    const height = Math.max(500, canvasContainerRef.clientHeight || 550);

    fabricService.init(canvasElRef, width, height);

    // Attach global keyboard shortcuts
    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        fabricService.setSpacePressed(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    window.addEventListener('paste', handlePaste);

    // Responsive Canvas Resize Observer
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const w = entry.contentRect.width;
        const h = entry.contentRect.height;
        if (w > 0 && h > 0) {
          fabricService.resize(w, h);
        }
      }
    });

    resizeObserver.observe(canvasContainerRef);
  });

  onCleanup(() => {
    window.removeEventListener('keydown', handleKeyDown);
    window.removeEventListener('paste', handlePaste);
    fabricService.setSpacePressed(false);
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    fabricService.destroy();
  });

  const handleExport = () => {
    const dataUrl = fabricService.exportImage('png');
    if (!dataUrl) {
      props.showToast('⚠️ Gagal mengekspor gambar');
      return;
    }

    const link = document.createElement('a');
    link.download = `ragasukma-canvas-${Date.now()}.png`;
    link.href = dataUrl;
    link.click();
    props.showToast('✨ Gambar canvas berhasil diekspor (PNG)!');
  };

  return (
    <div
      class={
        isFullscreen()
          ? 'fixed inset-0 z-[120] bg-[#0F1218] p-3 sm:p-4 flex flex-col gap-3 select-none'
          : 'space-y-4 flex flex-col h-full'
      }
    >
      {/* Header Banner (Hidden in Fullscreen for maximum workspace, but toolbar remains accessible) */}
      <Show when={!isFullscreen()}>
        <div class="bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-4 sm:p-5 border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-3 shrink-0">
          <div>
            <span class="px-2.5 py-1 text-xs font-bold uppercase tracking-wider bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
              2D Whiteboard & MCP Drawing Engine
            </span>
            <h2 class="text-2xl sm:text-3xl font-carter tracking-wide text-theme-text mt-1.5 flex items-center gap-2.5">
              <Palette class="text-theme-cyan" size={28} />
              PAPAN GAMBAR 2D
            </h2>
            <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
              Whiteboard interaktif dengan pondasi tool-calling MCP sehingga AI dapat menggambar diagram dan flowchart secara langsung di depan pengguna.
            </p>
          </div>

          <div class="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setShowShortcutsModal(true)}
              class="px-3 py-1.5 bg-theme-surface hover:bg-theme-card border border-theme-border text-xs font-mono font-bold text-theme-text/80 hover:text-theme-cyan flex items-center gap-1.5 transition-all shadow-sm"
            >
              <Keyboard size={14} class="text-theme-cyan" />
              <span>Shortcuts</span>
            </button>
            <div class="hidden sm:flex px-3 py-1.5 bg-theme-surface border border-theme-border text-xs font-mono font-bold text-theme-cyan items-center gap-2">
              <Sparkles size={14} />
              <span>Fabric.js v7 + MCP Ready</span>
            </div>
          </div>
        </div>
      </Show>

      {/* Canvas Toolbar */}
      <div class="shrink-0">
        <CanvasToolbar 
          onOpenExportModal={() => setShowExportModal(true)}
          onOpenShortcutsModal={() => setShowShortcutsModal(true)}
          showToast={props.showToast} 
        />
      </div>

      {/* AI Live Drawer Simulator (Collapsible) */}
      <Show when={showAiSimulator() || isAiDrawing()}>
        <div class="shrink-0 animate-in fade-in duration-200">
          <AiDrawingPanel showToast={props.showToast} />
        </div>
      </Show>

      {/* Canvas Viewport & Sidebar Workspace Area */}
      <div class="flex-1 min-h-[500px] w-full flex overflow-hidden border border-theme-border shadow-inner">
        {/* Interactive Fabric Canvas Viewport */}
        <div
          ref={canvasContainerRef}
          onDragOver={handleDragOver}
          onDragLeave={handleDragLeave}
          onDrop={handleDrop}
          class={`flex-1 w-full bg-[#10141D] relative overflow-hidden flex items-center justify-center select-none bg-[radial-gradient(rgba(123,223,242,0.28)_1.5px,transparent_1.5px)] [background-size:24px_24px] transition-colors ${
            isDragOver() ? 'border-2 border-theme-cyan bg-theme-cyan/5' : ''
          }`}
        >
          <Show when={isDragOver()}>
            <div class="absolute inset-0 bg-theme-card/85 backdrop-blur-sm z-30 flex flex-col items-center justify-center pointer-events-none border-2 border-dashed border-theme-cyan animate-pulse">
              <ImageIcon size={48} class="text-theme-cyan mb-2" />
              <p class="text-sm font-bold text-theme-cyan uppercase tracking-wider font-mono">
                Lepaskan file gambar untuk menambahkan ke kanvas
              </p>
            </div>
          </Show>
          <canvas ref={canvasElRef} />
        </div>

        {/* Collapsible Layers & Objects Sidebar */}
        <Show when={showLayersPanel()}>
          <LayersPanel showToast={props.showToast} />
        </Show>
      </div>

      {/* Advanced Export Modal */}
      <ExportModal
        isOpen={showExportModal()}
        onClose={() => setShowExportModal(false)}
        showToast={props.showToast}
      />

      {/* Keyboard Shortcuts Modal */}
      <ShortcutsModal
        isOpen={showShortcutsModal()}
        onClose={() => setShowShortcutsModal(false)}
      />
    </div>
  );
};

export default CanvasTab;
