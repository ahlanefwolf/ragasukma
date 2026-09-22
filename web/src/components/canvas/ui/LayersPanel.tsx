import { Component, createSignal, For, Show } from 'solid-js';
import { 
  Layers, 
  Eye, 
  EyeOff, 
  Lock, 
  Unlock, 
  ChevronUp, 
  ChevronDown, 
  Trash2, 
  Square, 
  Circle as CircleIcon, 
  Minus, 
  MoveRight, 
  Type, 
  Image as ImageIcon, 
  Pencil, 
  X, 
  Search, 
  GripVertical, 
  Edit2, 
  Check,
  FolderPlus,
  FolderMinus,
  Layers2,
  Scissors
} from 'lucide-solid';
import { layersList, showLayersPanel, setShowLayersPanel, selectedObjectType } from '../canvasStore';
import { fabricService } from '../fabricService';
import { CanvasLayerItem } from '../types';

interface LayersPanelProps {
  showToast: (msg: string) => void;
}

export const LayersPanel: Component<LayersPanelProps> = (props) => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [editingName, setEditingName] = createSignal('');
  const [draggedIndex, setDraggedIndex] = createSignal<number | null>(null);
  const [dragOverIndex, setDragOverIndex] = createSignal<number | null>(null);

  const filteredLayers = () => {
    const q = searchQuery().toLowerCase().trim();
    if (!q) return layersList();
    return layersList().filter(
      (l) => l.name.toLowerCase().includes(q) || l.type.toLowerCase().includes(q)
    );
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'rect':
        return Square;
      case 'circle':
        return CircleIcon;
      case 'line':
        return Minus;
      case 'arrow':
        return MoveRight;
      case 'text':
        return Type;
      case 'image':
        return ImageIcon;
      case 'path':
        return Pencil;
      case 'group':
        return Layers;
      default:
        return Square;
    }
  };

  const selectedCount = () => layersList().filter((l) => l.selected).length;
  const isGroupSelected = () => {
    const selected = layersList().filter((l) => l.selected);
    return selected.length === 1 && selected[0].type === 'group';
  };

  const handleSelect = (e: MouseEvent, item: CanvasLayerItem) => {
    if (item.locked || !item.visible || editingId() === item.id) return;
    fabricService.selectLayer(item.id, {
      ctrlKey: e.ctrlKey,
      metaKey: e.metaKey,
      shiftKey: e.shiftKey,
    });
  };

  const handleGroup = () => {
    fabricService.groupSelected();
    props.showToast('🔗 Objek berhasil digabung menjadi 1 Group!');
  };

  const handleUngroup = () => {
    fabricService.ungroupSelected();
    props.showToast('🔓 Group berhasil dipisahkan menjadi objek mandiri.');
  };

  const handleStartRename = (e: MouseEvent, item: CanvasLayerItem) => {
    e.stopPropagation();
    setEditingId(item.id);
    setEditingName(item.name);
  };

  const handleSaveRename = (id: string) => {
    if (editingName().trim()) {
      fabricService.renameLayer(id, editingName().trim());
      props.showToast(`✨ Nama layer diubah: "${editingName().trim()}"`);
    }
    setEditingId(null);
  };

  const handleKeyDownRename = (e: KeyboardEvent, id: string) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSaveRename(id);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      setEditingId(null);
    }
  };

  const handleToggleVisibility = (e: MouseEvent, id: string) => {
    e.stopPropagation();
    fabricService.toggleLayerVisibility(id);
  };

  const handleToggleLock = (e: MouseEvent, id: string) => {
    e.stopPropagation();
    fabricService.toggleLayerLock(id);
  };

  const handleMoveUp = (e: MouseEvent, id: string) => {
    e.stopPropagation();
    fabricService.moveLayerUp(id);
  };

  const handleMoveDown = (e: MouseEvent, id: string) => {
    e.stopPropagation();
    fabricService.moveLayerDown(id);
  };

  const handleDelete = (e: MouseEvent, id: string) => {
    e.stopPropagation();
    fabricService.deleteLayer(id);
  };

  // Drag & Drop Handlers for Layer Reordering
  const handleDragStart = (e: DragEvent, index: number) => {
    setDraggedIndex(index);
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(index));
    }
  };

  const handleDragOver = (e: DragEvent, index: number) => {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    setDragOverIndex(index);
  };

  const handleDragLeave = () => {
    setDragOverIndex(null);
  };

  const handleDrop = (e: DragEvent, targetIndex: number) => {
    e.preventDefault();
    const sourceIndex = draggedIndex();
    if (sourceIndex !== null && sourceIndex !== targetIndex) {
      fabricService.reorderLayer(sourceIndex, targetIndex);
    }
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
    setDragOverIndex(null);
  };

  return (
    <div class="w-80 bg-theme-card border-l border-theme-border flex flex-col h-full select-none shrink-0 shadow-lg">
      {/* Panel Header */}
      <div class="p-3 bg-theme-surface border-b border-theme-border flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Layers size={16} class="text-theme-cyan" />
          <span class="text-xs font-mono font-bold text-theme-text uppercase tracking-wider">
            Layer & Objek
          </span>
          <span class="px-1.5 py-0.2 text-[10px] font-mono font-bold bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
            {layersList().length}
          </span>
        </div>
        <div class="flex items-center gap-1">
          <Show when={selectedCount() > 1}>
            <button
              type="button"
              onClick={handleGroup}
              title="Gabung Objek Terpilih Menjadi 1 Group (Ctrl+G)"
              class="px-2 py-1 text-[11px] font-mono font-bold bg-theme-cyan text-theme-card hover:bg-theme-cyan/80 flex items-center gap-1 transition-all shadow-sm anime-shadow-green"
            >
              <FolderPlus size={12} />
              <span>Grup</span>
            </button>
          </Show>
          <Show when={isGroupSelected()}>
            <button
              type="button"
              onClick={handleUngroup}
              title="Pisahkan Group Menjadi Objek Mandiri (Ctrl+Shift+G)"
              class="px-2 py-1 text-[11px] font-mono font-bold bg-theme-rose/20 text-theme-rose border border-theme-rose/40 hover:bg-theme-rose/30 flex items-center gap-1 transition-all"
            >
              <FolderMinus size={12} />
              <span>Pisah</span>
            </button>
          </Show>
          <button
            type="button"
            onClick={() => setShowLayersPanel(false)}
            title="Tutup Panel Layer"
            class="p-1 text-theme-text/60 hover:text-theme-rose hover:bg-theme-card transition-all"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Search Bar */}
      <Show when={layersList().length > 4}>
        <div class="p-2 border-b border-theme-border/60 bg-theme-bg/60">
          <div class="flex items-center bg-theme-surface border border-theme-border px-2 py-1 gap-1.5">
            <Search size={12} class="text-theme-text/40" />
            <input
              type="text"
              placeholder="Cari nama layer..."
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
              class="w-full bg-transparent text-xs text-theme-text placeholder-theme-text/40 focus:outline-none font-mono"
            />
          </div>
        </div>
      </Show>

      {/* Layer List with Drag and Drop Reordering */}
      <div class="flex-1 overflow-y-auto p-2 space-y-1.5 pb-3">
        <Show
          when={filteredLayers().length > 0}
          fallback={
            <div class="p-6 text-center text-theme-text/40 flex flex-col items-center justify-center h-48 space-y-2">
              <Layers size={28} class="opacity-30" />
              <p class="text-xs font-mono">
                {searchQuery() ? 'Tidak ada layer yang cocok' : 'Belum ada objek di kanvas'}
              </p>
              <p class="text-[11px] text-theme-text/30">
                Gunakan alat gambar di atas untuk menambahkan objek.
              </p>
            </div>
          }
        >
          <For each={filteredLayers()}>
            {(layer, idx) => {
              const Icon = getTypeIcon(layer.type);
              const isSelected = () => layer.selected;
              const isEditing = () => editingId() === layer.id;
              const isDragging = () => draggedIndex() === idx();
              const isTargetOver = () => dragOverIndex() === idx();

              return (
                <div
                  draggable={!isEditing()}
                  onDragStart={(e) => handleDragStart(e, idx())}
                  onDragOver={(e) => handleDragOver(e, idx())}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, idx())}
                  onDragEnd={handleDragEnd}
                  onClick={(e) => handleSelect(e, layer)}
                  class={`group p-2 flex items-center justify-between gap-1.5 text-xs transition-all border relative rounded-none shrink-0 ${
                    isTargetOver()
                      ? 'border-t-2 border-t-theme-cyan bg-theme-cyan/20'
                      : isSelected()
                      ? 'bg-theme-cyan/20 !border-theme-cyan text-white font-bold shadow-sm anime-shadow-green'
                      : layer.locked
                      ? 'bg-theme-surface/50 border-theme-border/40 text-theme-text/50 opacity-75'
                      : !layer.visible
                      ? 'bg-theme-surface/30 border-theme-border/30 text-theme-text/40 opacity-50'
                      : 'bg-theme-surface hover:bg-theme-card border-theme-border/60 hover:border-theme-border text-theme-text'
                  } ${isDragging() ? 'opacity-40 scale-[0.98]' : ''}`}
                >
                  {/* Drag Handle */}
                  <div
                    title="Tahan & geser untuk mengubah urutan layer (Z-index)"
                    class="cursor-grab active:cursor-grabbing text-theme-text/40 hover:text-theme-cyan p-0.5 shrink-0"
                  >
                    <GripVertical size={13} />
                  </div>

                  {/* Icon & Accurate Fill / Stroke Pip (Figma/Illustrator Style) */}
                  <div
                    title={`Fill: ${layer.fillColor || 'None'}, Border: ${layer.strokeColor || 'None'}`}
                    class="w-3.5 h-3.5 rounded-none shrink-0 border-2 relative flex items-center justify-center shadow-sm"
                    style={{
                      'border-color': layer.strokeColor && layer.strokeColor !== 'transparent' ? layer.strokeColor : 'rgba(255,255,255,0.2)',
                      'background-color': layer.fillColor && layer.fillColor !== 'transparent' ? layer.fillColor : 'transparent',
                    }}
                  >
                    <Show when={!layer.fillColor || layer.fillColor === 'transparent'}>
                      <span class="text-[8px] font-bold text-theme-rose leading-none">/</span>
                    </Show>
                  </div>
                  <Icon size={14} class="shrink-0 text-theme-cyan" />

                  {/* Layer Name / Inline Rename Input */}
                  <div class="min-w-0 flex-1">
                    <Show
                      when={isEditing()}
                      fallback={
                        <div
                          onDblClick={(e) => handleStartRename(e, layer)}
                          title="Klik ganda untuk mengubah nama layer"
                          class="truncate text-xs font-mono font-medium cursor-text"
                        >
                          {layer.name}
                        </div>
                      }
                    >
                      <div class="flex items-center gap-1" onClick={(e) => e.stopPropagation()}>
                        <input
                          type="text"
                          value={editingName()}
                          onInput={(e) => setEditingName(e.currentTarget.value)}
                          onKeyDown={(e) => handleKeyDownRename(e, layer.id)}
                          autofocus
                          class="w-full bg-theme-bg border border-theme-cyan px-1.5 py-0.5 text-xs font-mono text-white focus:outline-none"
                        />
                        <button
                          type="button"
                          onClick={() => handleSaveRename(layer.id)}
                          title="Simpan Nama"
                          class="p-0.5 text-emerald-400 hover:bg-theme-bg"
                        >
                          <Check size={12} />
                        </button>
                      </div>
                    </Show>

                    <Show when={layer.details && !isEditing()}>
                      <div class="text-[10px] font-mono text-theme-text/50 truncate">
                        {layer.details}
                      </div>
                    </Show>
                  </div>

                  {/* Right Action Icons: Rename, Visibility, Lock, Order, Delete */}
                  <div class="flex items-center gap-0.5 shrink-0 opacity-80 group-hover:opacity-100">
                    <Show when={!isEditing()}>
                      <button
                        type="button"
                        onClick={(e) => handleStartRename(e, layer)}
                        title="Beri Nama / Rename Objek"
                        class="p-1 text-theme-text/50 hover:text-theme-cyan hover:bg-theme-bg transition-colors"
                      >
                        <Edit2 size={12} />
                      </button>
                    </Show>

                    <button
                      type="button"
                      onClick={(e) => handleToggleVisibility(e, layer.id)}
                      title={layer.visible ? 'Sembunyikan Layer' : 'Tampilkan Layer'}
                      class={`p-1 hover:bg-theme-bg transition-colors ${
                        layer.visible ? 'text-theme-text/80 hover:text-white' : 'text-theme-rose'
                      }`}
                    >
                      {layer.visible ? <Eye size={13} /> : <EyeOff size={13} />}
                    </button>

                    <button
                      type="button"
                      onClick={(e) => handleToggleLock(e, layer.id)}
                      title={layer.locked ? 'Buka Kunci Layer' : 'Kunci Layer (Cegah Geser)'}
                      class={`p-1 hover:bg-theme-bg transition-colors ${
                        layer.locked ? 'text-amber-400' : 'text-theme-text/60 hover:text-white'
                      }`}
                    >
                      {layer.locked ? <Lock size={13} /> : <Unlock size={13} />}
                    </button>

                    <button
                      type="button"
                      onClick={(e) => handleMoveUp(e, layer.id)}
                      title="Pindahkan Layer ke Atas (Bring Forward)"
                      class="p-1 text-theme-text/60 hover:text-theme-cyan hover:bg-theme-bg transition-colors"
                    >
                      <ChevronUp size={13} />
                    </button>

                    <button
                      type="button"
                      onClick={(e) => handleMoveDown(e, layer.id)}
                      title="Pindahkan Layer ke Bawah (Send Backward)"
                      class="p-1 text-theme-text/60 hover:text-theme-cyan hover:bg-theme-bg transition-colors"
                    >
                      <ChevronDown size={13} />
                    </button>

                    <button
                      type="button"
                      onClick={(e) => handleDelete(e, layer.id)}
                      title="Hapus Objek"
                      class="p-1 text-theme-text/50 hover:text-theme-rose hover:bg-theme-bg transition-colors"
                    >
                      <Trash2 size={13} />
                    </button>
                  </div>
                </div>
              );
            }}
          </For>
        </Show>
      </div>

      {/* Footer Info */}
      <div class="p-2.5 bg-theme-surface border-t border-theme-border flex items-center justify-between text-[10.5px] font-mono text-theme-text/60">
        <span>Tahan & geser untuk atur urutan</span>
        <button
          type="button"
          onClick={() => fabricService.syncLayers()}
          class="text-theme-cyan hover:underline"
        >
          Refresh
        </button>
      </div>
    </div>
  );
};
