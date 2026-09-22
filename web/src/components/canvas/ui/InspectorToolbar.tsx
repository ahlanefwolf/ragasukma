import { Component, For, Show, createSignal, onCleanup, onMount } from 'solid-js';
import {
  Square,
  Circle as CircleIcon,
  Triangle,
  MoveRight,
  Type,
  Pencil,
  Eraser,
  MousePointer,
  Layers,
  Split,
  FolderSearch,
  ChevronDown,
} from 'lucide-solid';
import {
  activeTool,
  selectedObjectType,
  strokeColor,
  setStrokeColor,
  fillColor,
  setFillColor,
  strokeWidth,
  setStrokeWidth,
  strokeAlign,
  setStrokeAlign,
  cornerRadius,
  setCornerRadius,
  fontSize,
  setFontSize,
  fontFamily,
  setFontFamily,
  availableFonts,
  queryOsLocalFonts,
  arrowHead,
  setArrowHead,
  lineStyle,
  setLineStyle,
  connectorRouting,
  setConnectorRouting,
} from '../canvasStore';
import { ArrowHead, ConnectorRouting, LineStyle, StrokeAlign } from '../types';
import { fabricService } from '../fabricService';
import { ColorPickerPopover } from './ColorPickerPopover';

export const InspectorToolbar: Component = () => {
  const [openDropdown, setOpenDropdown] = createSignal<string | null>(null);
  let containerRef: HTMLDivElement | undefined;

  const toggleDropdown = (key: string) => {
    setOpenDropdown((prev) => (prev === key ? null : key));
  };

  const handleClickOutside = (e: MouseEvent) => {
    if (containerRef && !containerRef.contains(e.target as Node)) {
      setOpenDropdown(null);
    }
  };

  onMount(() => {
    document.addEventListener('pointerdown', handleClickOutside);
  });

  onCleanup(() => {
    document.removeEventListener('pointerdown', handleClickOutside);
  });

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

  const handleStrokeWidthChange = (val: number) => {
    setStrokeWidth(val);
    if (val > 0 && strokeColor() === 'transparent') {
      setStrokeColor('#29483A');
    } else if (val === 0) {
      setStrokeColor('transparent');
    }
    fabricService.updateStyles();
  };

  const typeMeta = () => {
    switch (effectiveType()) {
      case 'rect':
        return { label: 'Kotak', icon: Square, color: 'text-theme-cyan' };
      case 'circle':
        return { label: 'Lingkaran', icon: CircleIcon, color: 'text-theme-cyan' };
      case 'polygon':
        return { label: 'Polygon', icon: Triangle, color: 'text-theme-cyan' };
      case 'line':
      case 'arrow':
        return { label: 'Garis', icon: MoveRight, color: 'text-theme-cyan' };
      case 'text':
        return { label: 'Teks', icon: Type, color: 'text-amber-300' };
      case 'brush':
        return { label: 'Kuas', icon: Pencil, color: 'text-theme-cyan' };
      case 'group':
      case 'activeSelection':
        return { label: 'Grup', icon: Layers, color: 'text-theme-rose' };
      case 'eraser':
        return { label: 'Hapus', icon: Eraser, color: 'text-theme-rose' };
      default:
        return { label: 'Pilih', icon: MousePointer, color: 'text-theme-text/60' };
    }
  };

  return (
    <div ref={containerRef} class="flex items-center gap-1.5 flex-nowrap text-xs font-mono">
      {/* 1. Dynamic Type Badge */}
      <div class="flex items-center gap-1 px-2 py-1 bg-theme-surface border border-theme-border/70 text-xs shrink-0">
        {(() => {
          const Meta = typeMeta();
          const Icon = Meta.icon;
          return (
            <>
              <Icon size={13} class={Meta.color} />
              <span class="font-bold text-theme-text/80">{Meta.label}</span>
            </>
          );
        })()}
      </div>

      {/* ── Case: Eraser ── */}
      <Show when={effectiveType() === 'eraser'}>
        <span class="text-xs text-theme-rose/80 italic px-1">Klik objek di kanvas untuk menghapus</span>
      </Show>

      {/* ── Case: Select Idle ── */}
      <Show when={effectiveType() === 'select'}>
        <span class="text-xs text-theme-text/40 italic px-1 hidden md:inline">Pilih objek untuk inspeksi</span>
      </Show>

      {/* ── Case: Multi-Selection (Boolean Split & Union) ── */}
      <Show when={effectiveType() === 'group' || effectiveType() === 'activeSelection'}>
        <div class="flex items-center gap-1 bg-theme-surface p-0.5 border border-theme-cyan/50 shrink-0">
          <button
            type="button"
            onClick={() => fabricService.splitIntoFragments()}
            title="Pisahkan semua perpotongan objek menjadi pecahan objek mandiri"
            class="px-2 py-1 bg-theme-card hover:bg-theme-cyan hover:text-theme-card text-theme-text text-xs font-mono font-bold border border-theme-border flex items-center gap-1.5 transition-all shadow-sm"
          >
            <Split size={12} class="text-theme-cyan" />
            <span>Pisahkan</span>
          </button>
          <button
            type="button"
            onClick={() => fabricService.unionSelected()}
            title="Gabung semua objek terpilih menjadi 1 bentuk poligon utuh"
            class="px-2 py-1 bg-theme-card hover:bg-theme-cyan hover:text-theme-card text-theme-text text-xs font-mono font-bold border border-theme-border flex items-center gap-1.5 transition-all shadow-sm"
          >
            <Layers size={12} class="text-theme-cyan" />
            <span>Gabung (Union)</span>
          </button>
        </div>
      </Show>

      {/* ── Case: Text Controls ── */}
      <Show when={effectiveType() === 'text'}>
        <ColorPickerPopover
          label="Teks"
          color={strokeColor()}
          isOpen={openDropdown() === 'textColor'}
          onToggle={() => toggleDropdown('textColor')}
          onChangeColor={(c) => {
            setStrokeColor(c);
            fabricService.updateStyles();
          }}
        />

        <div class="flex items-center bg-theme-surface border border-theme-border px-1.5 py-0.5" title="Ukuran Font (px)">
          <span class="text-[10px] text-theme-text/50 font-bold pr-1">T</span>
          <input
            type="number"
            min="8"
            max="200"
            value={fontSize()}
            onInput={(e) => {
              const val = parseInt(e.currentTarget.value, 10);
              if (!isNaN(val) && val >= 8) {
                setFontSize(val);
                fabricService.updateStyles();
              }
            }}
            class="w-8 bg-transparent text-xs font-mono font-bold text-amber-300 text-center focus:outline-none"
          />
        </div>

        <div class="flex items-center gap-1 bg-theme-surface border border-theme-border px-1.5 py-0.5">
          <select
            value={fontFamily()}
            onChange={(e) => {
              setFontFamily(e.currentTarget.value);
              fabricService.updateStyles();
            }}
            class="bg-transparent text-xs font-mono text-theme-text border-none focus:outline-none cursor-pointer max-w-[110px] truncate"
          >
            <For each={availableFonts()}>
              {(f) => (
                <option value={f.family} class="bg-theme-card text-theme-text">
                  {f.label}
                </option>
              )}
            </For>
          </select>
          <button
            type="button"
            onClick={() => queryOsLocalFonts()}
            title="Scan Font OS"
            class="p-0.5 text-theme-text/50 hover:text-amber-300"
          >
            <FolderSearch size={12} />
          </button>
        </div>
      </Show>

      {/* ── Case: Shapes, Lines, Brush Controls ── */}
      <Show when={effectiveType() !== 'text' && effectiveType() !== 'eraser' && effectiveType() !== 'select'}>
        <ColorPickerPopover
          label="Stroke"
          color={strokeColor()}
          isOpen={openDropdown() === 'strokeColor'}
          onToggle={() => toggleDropdown('strokeColor')}
          onChangeColor={(c) => {
            setStrokeColor(c);
            if (strokeWidth() === 0 && c !== 'transparent') setStrokeWidth(2);
            fabricService.updateStyles();
          }}
        />

        <Show when={['rect', 'circle', 'polygon', 'group', 'activeSelection'].includes(effectiveType())}>
          <ColorPickerPopover
            label="Fill"
            color={fillColor()}
            isOpen={openDropdown() === 'fillColor'}
            onToggle={() => toggleDropdown('fillColor')}
            onChangeColor={(c) => {
              setFillColor(c);
              fabricService.updateStyles();
            }}
          />
        </Show>

        {/* Stroke Width Input */}
        <div class="flex items-center bg-theme-surface border border-theme-border px-1.5 py-0.5" title="Tebal Garis (px)">
          <span class="text-[10px] text-theme-text/50 font-bold pr-0.5">W:</span>
          <input
            type="number"
            min="0"
            max="100"
            value={strokeWidth()}
            onInput={(e) => {
              const val = parseInt(e.currentTarget.value, 10);
              if (!isNaN(val) && val >= 0) handleStrokeWidthChange(val);
            }}
            class="w-7 bg-transparent text-xs font-mono font-bold text-theme-cyan text-center focus:outline-none"
          />
        </div>

        {/* Line Style Popover */}
        <div class="relative">
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              toggleDropdown('lineStyle');
            }}
            title="Gaya Garis (Solid, Dashed, Dotted)"
            class="flex items-center gap-1 px-2 py-1 bg-theme-surface hover:bg-theme-card border border-theme-border transition-all"
          >
            <span class="text-[11px] font-bold text-theme-cyan">
              {lineStyle() === 'solid' ? '———' : lineStyle() === 'dashed' ? '- - -' : '· · ·'}
            </span>
            <ChevronDown size={11} class="text-theme-text/50" />
          </button>

          <Show when={openDropdown() === 'lineStyle'}>
            <div class="absolute left-0 top-full mt-1.5 bg-theme-card border border-theme-border shadow-2xl p-1 z-[100] flex flex-col gap-0.5 min-w-[100px]">
              {(['solid', 'dashed', 'dotted'] as LineStyle[]).map((st) => (
                <button
                  type="button"
                  onClick={() => {
                    setLineStyle(st);
                    fabricService.updateStyles();
                    setOpenDropdown(null);
                  }}
                  class={`px-2 py-1 text-left text-xs font-mono flex items-center justify-between ${
                    lineStyle() === st ? 'bg-theme-cyan text-theme-card font-bold' : 'hover:bg-theme-surface'
                  }`}
                >
                  <span class="capitalize">{st}</span>
                  <span>{st === 'solid' ? '———' : st === 'dashed' ? '- - -' : '· · ·'}</span>
                </button>
              ))}
            </div>
          </Show>
        </div>

        {/* Route & Arrowhead (For Line / Arrow) */}
        <Show when={effectiveType() === 'line' || effectiveType() === 'arrow'}>
          <div class="relative">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                toggleDropdown('routeStyle');
              }}
              title="Mode Rute Panah"
              class="flex items-center gap-1 px-2 py-1 bg-theme-surface hover:bg-theme-card border border-theme-border transition-all"
            >
              <span class="text-[11px] font-bold text-theme-cyan">
                {connectorRouting() === 'orthogonal' ? '⮑ Siku' : connectorRouting() === 'curved' ? '∿ Lengkung' : '/ Lurus'}
              </span>
              <ChevronDown size={11} class="text-theme-text/50" />
            </button>

            <Show when={openDropdown() === 'routeStyle'}>
              <div class="absolute left-0 top-full mt-1.5 bg-theme-card border border-theme-border shadow-2xl p-1 z-[100] flex flex-col gap-0.5 min-w-[110px]">
                {[
                  { id: 'orthogonal', label: '⮑ Siku (Elbow)' },
                  { id: 'curved', label: '∿ Lengkung (Curved)' },
                  { id: 'straight', label: '/ Lurus (Straight)' },
                ].map((item) => (
                  <button
                    type="button"
                    onClick={() => {
                      setConnectorRouting(item.id as ConnectorRouting);
                      fabricService.updateStyles();
                      setOpenDropdown(null);
                    }}
                    class={`px-2 py-1 text-left text-xs font-mono ${
                      connectorRouting() === item.id ? 'bg-theme-cyan text-theme-card font-bold' : 'hover:bg-theme-surface'
                    }`}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            </Show>
          </div>

          <div class="relative">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                toggleDropdown('arrowHead');
              }}
              title="Mata Panah"
              class="flex items-center gap-1 px-2 py-1 bg-theme-surface hover:bg-theme-card border border-theme-border transition-all"
            >
              <span class="text-[11px] font-bold text-theme-cyan">
                {arrowHead() === 'none' ? '— Polos' : arrowHead() === 'end' ? '→ Tunggal' : '↔ Ganda'}
              </span>
              <ChevronDown size={11} class="text-theme-text/50" />
            </button>

            <Show when={openDropdown() === 'arrowHead'}>
              <div class="absolute left-0 top-full mt-1.5 bg-theme-card border border-theme-border shadow-2xl p-1 z-[100] flex flex-col gap-0.5 min-w-[100px]">
                {[
                  { id: 'none', label: '— Polos' },
                  { id: 'end', label: '→ Tunggal' },
                  { id: 'both', label: '↔ Ganda' },
                ].map((item) => (
                  <button
                    type="button"
                    onClick={() => {
                      setArrowHead(item.id as ArrowHead);
                      fabricService.updateStyles();
                      setOpenDropdown(null);
                    }}
                    class={`px-2 py-1 text-left text-xs font-mono ${
                      arrowHead() === item.id ? 'bg-theme-cyan text-theme-card font-bold' : 'hover:bg-theme-surface'
                    }`}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            </Show>
          </div>
        </Show>

        {/* Corner Radius for Rect */}
        <Show when={effectiveType() === 'rect' || effectiveType() === 'group'}>
          <div class="flex items-center bg-theme-surface border border-theme-border px-1.5 py-0.5" title="Sudut Tumpul / Radius (px)">
            <span class="text-[10px] text-theme-text/50 font-bold pr-0.5">R:</span>
            <input
              type="number"
              min="0"
              max="200"
              value={cornerRadius()}
              onInput={(e) => {
                const val = parseInt(e.currentTarget.value, 10);
                if (!isNaN(val) && val >= 0) {
                  setCornerRadius(val);
                  fabricService.updateStyles();
                }
              }}
              class="w-7 bg-transparent text-xs font-mono font-bold text-theme-cyan text-center focus:outline-none"
            />
          </div>
        </Show>

        {/* Border Align for Shapes */}
        <Show when={['rect', 'circle', 'polygon'].includes(effectiveType())}>
          <div class="relative">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                toggleDropdown('strokeAlign');
              }}
              title="Border Alignment (Center, Inner, Outer)"
              class="flex items-center gap-1 px-2 py-1 bg-theme-surface hover:bg-theme-card border border-theme-border transition-all uppercase text-[10px] font-bold"
            >
              <span>{strokeAlign()}</span>
              <ChevronDown size={11} class="text-theme-text/50" />
            </button>

            <Show when={openDropdown() === 'strokeAlign'}>
              <div class="absolute left-0 top-full mt-1.5 bg-theme-card border border-theme-border shadow-2xl p-1 z-[100] flex flex-col gap-0.5 min-w-[90px]">
                {(['center', 'inner', 'outer'] as StrokeAlign[]).map((al) => (
                  <button
                    type="button"
                    onClick={() => {
                      setStrokeAlign(al);
                      fabricService.updateStyles();
                      setOpenDropdown(null);
                    }}
                    class={`px-2 py-1 text-left text-xs font-mono uppercase ${
                      strokeAlign() === al ? 'bg-theme-cyan text-theme-card font-bold' : 'hover:bg-theme-surface'
                    }`}
                  >
                    {al}
                  </button>
                ))}
              </div>
            </Show>
          </div>
        </Show>
      </Show>
    </div>
  );
};
