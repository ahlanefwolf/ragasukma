import { Component, For, Show } from 'solid-js';
import {
  Square,
  Circle as CircleIcon,
  Triangle,
  Minus,
  MoveRight,
  Type,
  Pencil,
  Eraser,
  MousePointer,
  Layers,
  Pipette,
  FolderSearch,
  Scissors,
  Split,
  Layers as MultiLayers,
} from 'lucide-solid';
import {
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
  COLOR_PALETTE,
  STROKE_PALETTE,
  FILL_PALETTE,
  arrowHead,
  setArrowHead,
  lineStyle,
  setLineStyle,
} from './canvasStore';
import { ArrowHead, LineStyle, StrokeAlign } from './types';
import { fabricService } from './fabricService';

interface InspectorToolbarProps {
  effectiveType: string;
}

export const InspectorToolbar: Component<InspectorToolbarProps> = (props) => {
  const handleStrokeColorChange = (hex: string) => {
    setStrokeColor(hex);
    if (hex === 'transparent') {
      setStrokeWidth(0);
    } else if (strokeWidth() === 0) {
      setStrokeWidth(2);
    }
    fabricService.updateStyles();
  };

  const handleFillColorChange = (hex: string) => {
    setFillColor(hex);
    fabricService.updateStyles();
  };

  const handleStrokeWidthChange = (val: number) => {
    setStrokeWidth(val);
    if (val > 0 && strokeColor() === 'transparent') {
      setStrokeColor('#7BDFF2');
    } else if (val === 0) {
      setStrokeColor('transparent');
    }
    fabricService.updateStyles();
  };
  const handleStrokeAlignChange = (align: StrokeAlign) => {
    setStrokeAlign(align);
    fabricService.updateStyles();
  };

  const handleArrowHeadChange = (head: ArrowHead) => {
    setArrowHead(head);
    fabricService.updateStyles();
  };

  const handleLineStyleChange = (style: LineStyle) => {
    setLineStyle(style);
    fabricService.updateStyles();
  };

  const handleCornerRadiusChange = (val: number) => {
    setCornerRadius(val);
    fabricService.updateStyles();
  };

  const handleFontSizeChange = (val: number) => {
    setFontSize(val);
    fabricService.updateStyles();
  };

  const handleFontFamilyChange = (font: string) => {
    setFontFamily(font);
    fabricService.updateStyles();
  };

  const handleSplitFragments = () => {
    fabricService.splitIntoFragments();
  };

  const handleUnionSelected = () => {
    fabricService.unionSelected();
  };

  const typeMeta = () => {
    switch (props.effectiveType) {
      case 'rect':
        return { label: 'Kotak (Rect)', icon: Square, color: 'text-theme-cyan' };
      case 'circle':
        return { label: 'Lingkaran', icon: CircleIcon, color: 'text-theme-cyan' };
      case 'polygon':
        return { label: 'Polygon / Segitiga', icon: Triangle, color: 'text-theme-cyan' };
      case 'line':
        return { label: 'Garis / Panah', icon: MoveRight, color: 'text-theme-cyan' };
      case 'text':
        return { label: 'Teks & Label', icon: Type, color: 'text-amber-300' };
      case 'brush':
        return { label: 'Kuas Bebas', icon: Pencil, color: 'text-theme-cyan' };
      case 'group':
        return { label: 'Multi Seleksi', icon: Layers, color: 'text-theme-rose' };
      case 'eraser':
        return { label: 'Penghapus', icon: Eraser, color: 'text-theme-rose' };
      default:
        return { label: 'Pilih Objek', icon: MousePointer, color: 'text-theme-text/60' };
    }
  };

  return (
    <div class="flex items-center gap-2.5 flex-wrap">
      {/* Dynamic Type Badge */}
      <div class="flex items-center gap-1.5 px-2 py-1 bg-theme-surface border border-theme-border/60 text-xs font-mono">
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

      {/* ── Case 1: Eraser Mode ── */}
      <Show when={props.effectiveType === 'eraser'}>
        <span class="text-xs text-theme-rose/80 font-mono italic">
          Klik objek di kanvas untuk menghapus
        </span>
      </Show>

      {/* ── Case 2: Select Mode with Nothing Selected ── */}
      <Show when={props.effectiveType === 'select'}>
        <span class="text-xs text-theme-text/50 font-mono italic">
          Klik objek di kanvas untuk mengubah properti
        </span>
      </Show>

      {/* ── Case 2.5: Multi-Selection (Shape Builder & Auto-Segment) ── */}
      <Show when={props.effectiveType === 'activeSelection' || props.effectiveType === 'group'}>
        <div class="flex items-center gap-1.5 bg-theme-surface p-1 border border-theme-cyan/50">
          <div class="flex items-center gap-1 px-1.5 py-0.5 bg-theme-cyan/15 text-theme-cyan text-xs font-mono font-bold">
            <MultiLayers size={12} />
            <span>Multi Objek</span>
          </div>

          <button
            type="button"
            onClick={handleSplitFragments}
            title="Pisahkan semua perpotongan objek menjadi pecahan objek mandiri"
            class="px-2.5 py-1 bg-theme-card hover:bg-theme-cyan hover:text-theme-card text-theme-text text-xs font-mono font-bold border border-theme-border flex items-center gap-1.5 transition-all shadow-sm"
          >
            <Split size={13} class="text-theme-cyan" />
            <span>Pisahkan Semua (Pecah Objek)</span>
          </button>

          <button
            type="button"
            onClick={handleUnionSelected}
            title="Gabung semua objek terpilih menjadi 1 bentuk poligon utuh"
            class="px-2.5 py-1 bg-theme-card hover:bg-theme-cyan hover:text-theme-card text-theme-text text-xs font-mono font-bold border border-theme-border flex items-center gap-1.5 transition-all shadow-sm"
          >
            <Layers size={13} class="text-theme-cyan" />
            <span>Gabung (Union)</span>
          </button>
        </div>
      </Show>

      {/* ── Case 3: Text Controls (Color, Font Size, Font Family) ── */}
      <Show when={props.effectiveType === 'text'}>
        {/* Font Color */}
        <div class="flex items-center gap-1.5">
          <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase">Warna:</span>
          <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60">
            <For each={COLOR_PALETTE}>
              {(c) => (
                <button
                  type="button"
                  onClick={() => handleStrokeColorChange(c.hex)}
                  title={c.label}
                  class={`w-5 h-5 rounded-none border transition-all ${
                    strokeColor().toLowerCase() === c.hex.toLowerCase()
                      ? 'scale-110 border-white shadow-[0_0_6px_#fff]'
                      : 'border-transparent hover:scale-105'
                  }`}
                  style={{ 'background-color': c.hex }}
                />
              )}
            </For>
            <label
              title="Pilih Warna Teks (Color Picker)"
              class="relative w-5 h-5 flex items-center justify-center cursor-pointer border border-theme-border hover:border-theme-cyan bg-theme-card transition-all overflow-hidden"
              style={{ 'background-color': strokeColor() }}
            >
              <input
                type="color"
                value={strokeColor().startsWith('#') && strokeColor().length === 7 ? strokeColor() : '#7BDFF2'}
                onInput={(e) => handleStrokeColorChange(e.currentTarget.value)}
                class="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
              />
              <Pipette size={11} class="text-white mix-blend-difference pointer-events-none" />
            </label>
          </div>
        </div>

        {/* Font Size Number Input & Range Slider */}
        <div class="flex items-center gap-1.5 bg-theme-surface p-1 border border-theme-border/60" title="Ukuran Huruf / Font Size (Pixel)">
          <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase pl-1">Size:</span>
          <div class="flex items-center bg-theme-card border border-theme-border/80 px-1 py-0.5">
            <input
              type="number"
              min="8"
              max="200"
              value={fontSize()}
              onInput={(e) => {
                const val = parseInt(e.currentTarget.value, 10);
                if (!isNaN(val) && val >= 8) {
                  handleFontSizeChange(val);
                }
              }}
              class="w-10 bg-transparent text-xs font-mono font-bold text-amber-300 text-center focus:outline-none"
            />
            <span class="text-[10px] font-mono text-theme-text/40 pr-0.5">px</span>
          </div>
          <input
            type="range"
            min="10"
            max="96"
            value={fontSize()}
            onInput={(e) => handleFontSizeChange(Number(e.currentTarget.value))}
            class="w-14 accent-amber-400 h-1.5 bg-theme-bg cursor-pointer"
          />
        </div>

        {/* Font Family Selector */}
        <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60">
          <Type size={13} class="text-amber-300 ml-1" />
          <select
            value={fontFamily()}
            onChange={(e) => handleFontFamilyChange(e.currentTarget.value)}
            title="Pilih Font Teks (Font OS / Web)"
            class="bg-transparent text-xs font-mono text-theme-text border-none focus:outline-none cursor-pointer max-w-[130px] truncate"
          >
            <For each={availableFonts()}>
              {(f) => (
                <option value={f.family} class="bg-theme-card text-theme-text">
                  {f.label} ({f.category})
                </option>
              )}
            </For>
          </select>
          <button
            type="button"
            onClick={() => queryOsLocalFonts()}
            title="Pindai Font Lokal dari Sistem Operasi"
            class="p-1 text-theme-text/60 hover:text-amber-300 hover:bg-theme-card transition-all"
          >
            <FolderSearch size={13} />
          </button>
        </div>
      </Show>

      {/* ── Case 4: Shapes & Lines Controls (Rect, Circle, Line, Brush, Group) ── */}
      <Show when={props.effectiveType !== 'text' && props.effectiveType !== 'eraser' && props.effectiveType !== 'select'}>
        {/* Stroke / Line Color */}
        <div class="flex items-center gap-1.5">
          <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase">
            {props.effectiveType === 'brush' ? 'Kuas:' : 'Line:'}
          </span>
          <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60">
            <For each={STROKE_PALETTE}>
              {(c) => (
                <button
                  type="button"
                  onClick={() => handleStrokeColorChange(c.hex)}
                  title={c.label}
                  class={`w-5 h-5 rounded-none border transition-all relative ${
                    strokeColor().toLowerCase() === c.hex.toLowerCase()
                      ? 'scale-110 border-white shadow-[0_0_6px_#fff]'
                      : 'border-transparent hover:scale-105'
                  }`}
                  style={{ 'background-color': c.hex === 'transparent' ? 'transparent' : c.hex }}
                >
                  <Show when={c.hex === 'transparent'}>
                    <span class="text-[9px] font-bold text-theme-rose absolute inset-0 flex items-center justify-center">
                      /
                    </span>
                  </Show>
                </button>
              )}
            </For>
            <label
              title="Pilih Warna Garis (Color Picker)"
              class="relative w-5 h-5 flex items-center justify-center cursor-pointer border border-theme-border hover:border-theme-cyan bg-theme-card transition-all overflow-hidden"
              style={{ 'background-color': strokeColor() === 'transparent' ? 'transparent' : strokeColor() }}
            >
              <input
                type="color"
                value={strokeColor().startsWith('#') && strokeColor().length === 7 ? strokeColor() : '#7BDFF2'}
                onInput={(e) => handleStrokeColorChange(e.currentTarget.value)}
                class="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
              />
              <Pipette size={11} class="text-white mix-blend-difference pointer-events-none" />
            </label>
          </div>
        </div>

        {/* Fill Color (Only for Rect, Circle, Polygon, Group, ActiveSelection) */}
        <Show when={props.effectiveType === 'rect' || props.effectiveType === 'circle' || props.effectiveType === 'polygon' || props.effectiveType === 'group' || props.effectiveType === 'activeSelection'}>
          <div class="hidden sm:flex items-center gap-1.5">
            <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase">Fill:</span>
            <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60">
              <For each={FILL_PALETTE}>
                {(f) => (
                  <button
                    type="button"
                    onClick={() => handleFillColorChange(f.hex)}
                    title={f.label}
                    class={`w-5 h-5 rounded-none border transition-all relative ${
                      fillColor().toLowerCase() === f.hex.toLowerCase()
                        ? 'scale-110 border-theme-cyan shadow-[0_0_6px_#7BDFF2]'
                        : 'border-theme-border/40 hover:scale-105'
                    }`}
                    style={{
                      'background-color': f.hex === 'transparent' ? 'transparent' : f.hex,
                    }}
                  >
                    <Show when={f.hex === 'transparent'}>
                      <span class="text-[9px] font-bold text-theme-rose absolute inset-0 flex items-center justify-center">
                        /
                      </span>
                    </Show>
                  </button>
                )}
              </For>
              <label
                title="Pilih Warna Isi Kustom (Color Picker)"
                class="relative w-5 h-5 flex items-center justify-center cursor-pointer border border-theme-border hover:border-theme-cyan bg-theme-card transition-all overflow-hidden"
                style={{
                  'background-color': fillColor() === 'transparent' ? 'transparent' : fillColor(),
                }}
              >
                <input
                  type="color"
                  value={fillColor().startsWith('#') && fillColor().length === 7 ? fillColor() : '#F6BD60'}
                  onInput={(e) => handleFillColorChange(e.currentTarget.value)}
                  class="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
                />
                <Pipette size={11} class="text-white mix-blend-difference pointer-events-none" />
              </label>
            </div>
          </div>
        </Show>

        {/* Stroke Width / Border Size Input */}
        <div class="flex items-center gap-1.5 bg-theme-surface p-1 border border-theme-border/60" title="Ketebalan Garis / Border (Pixel, 0 = Tanpa Border)">
          <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase pl-1">
            {props.effectiveType === 'brush' ? 'Tebal:' : 'Size:'}
          </span>
          <div class="flex items-center bg-theme-card border border-theme-border/80 px-1 py-0.5">
            <input
              type="number"
              min="0"
              max="100"
              value={strokeWidth()}
              onInput={(e) => {
                const val = parseInt(e.currentTarget.value, 10);
                if (!isNaN(val) && val >= 0) {
                  handleStrokeWidthChange(val);
                }
              }}
              class="w-10 bg-transparent text-xs font-mono font-bold text-theme-cyan text-center focus:outline-none"
            />
            <span class="text-[10px] font-mono text-theme-text/40 pr-0.5">px</span>
          </div>
          <input
            type="range"
            min="0"
            max="48"
            value={strokeWidth()}
            onInput={(e) => handleStrokeWidthChange(Number(e.currentTarget.value))}
            class="w-14 accent-theme-cyan h-1.5 bg-theme-bg cursor-pointer"
          />
        </div>

        {/* Arrowhead Selector (Only for Line & Arrow) */}
        <Show when={props.effectiveType === 'line' || props.effectiveType === 'arrow'}>
          <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60" title="Ujung Panah / Arrow Head">
            <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase px-1">Arrow:</span>
            <div class="flex items-center bg-theme-card border border-theme-border/80 p-0.5 gap-0.5">
              <button
                type="button"
                onClick={() => handleArrowHeadChange('none')}
                title="Garis Polos (Tanpa Panah)"
                class={`px-2 py-0.5 text-[11px] font-mono font-bold transition-all ${
                  arrowHead() === 'none'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                —
              </button>
              <button
                type="button"
                onClick={() => handleArrowHeadChange('end')}
                title="Panah Tunggal (Ujung Akhir)"
                class={`px-2 py-0.5 text-[11px] font-mono font-bold transition-all ${
                  arrowHead() === 'end'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                →
              </button>
              <button
                type="button"
                onClick={() => handleArrowHeadChange('both')}
                title="Panah Ganda (Kedua Ujung)"
                class={`px-2 py-0.5 text-[11px] font-mono font-bold transition-all ${
                  arrowHead() === 'both'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                ↔
              </button>
            </div>
          </div>

          {/* Line Style Selector (Solid, Dashed, Dotted) */}
          <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60" title="Gaya Garis (Solid, Putus-putus, Titik-titik)">
            <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase px-1">Style:</span>
            <div class="flex items-center bg-theme-card border border-theme-border/80 p-0.5 gap-0.5">
              <button
                type="button"
                onClick={() => handleLineStyleChange('solid')}
                title="Garis Solid"
                class={`px-2 py-0.5 text-[10px] font-mono font-bold transition-all ${
                  lineStyle() === 'solid'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                ———
              </button>
              <button
                type="button"
                onClick={() => handleLineStyleChange('dashed')}
                title="Garis Putus-putus (Dashed)"
                class={`px-2 py-0.5 text-[10px] font-mono font-bold transition-all ${
                  lineStyle() === 'dashed'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                - - -
              </button>
              <button
                type="button"
                onClick={() => handleLineStyleChange('dotted')}
                title="Garis Titik-titik (Dotted)"
                class={`px-2 py-0.5 text-[10px] font-mono font-bold transition-all ${
                  lineStyle() === 'dotted'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                · · ·
              </button>
            </div>
          </div>
        </Show>

        {/* Border Alignment (Center, Inner, Outer) for Shapes (Rect, Circle, Polygon, Group, ActiveSelection) */}
        <Show when={props.effectiveType === 'rect' || props.effectiveType === 'circle' || props.effectiveType === 'polygon' || props.effectiveType === 'group' || props.effectiveType === 'activeSelection'}>
          <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60" title="Posisi Penyelarasan Border (Center / Inner / Outer)">
            <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase px-1">Align:</span>
            <div class="flex items-center bg-theme-card border border-theme-border/80 p-0.5 gap-0.5">
              <button
                type="button"
                onClick={() => handleStrokeAlignChange('center')}
                title="Border Center (Tengah Garis Objek)"
                class={`px-2 py-0.5 text-[10px] font-mono font-bold uppercase transition-all ${
                  strokeAlign() === 'center'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                Center
              </button>
              <button
                type="button"
                onClick={() => handleStrokeAlignChange('inner')}
                title="Border Inner (Di Dalam Objek)"
                class={`px-2 py-0.5 text-[10px] font-mono font-bold uppercase transition-all ${
                  strokeAlign() === 'inner'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                Inner
              </button>
              <button
                type="button"
                onClick={() => handleStrokeAlignChange('outer')}
                title="Border Outer (Di Luar Objek)"
                class={`px-2 py-0.5 text-[10px] font-mono font-bold uppercase transition-all ${
                  strokeAlign() === 'outer'
                    ? 'bg-theme-cyan text-theme-bg shadow-sm font-black'
                    : 'text-theme-text/60 hover:text-theme-cyan hover:bg-theme-surface'
                }`}
              >
                Outer
              </button>
            </div>
          </div>
        </Show>

        {/* Corner Radius (Only for Rect & Group) */}
        <Show when={props.effectiveType === 'rect' || props.effectiveType === 'group'}>
          <div class="flex items-center gap-1.5 bg-theme-surface p-1 border border-theme-border/60" title="Sudut Tumpul / Radius (Pixel)">
            <span class="text-[11px] font-mono font-bold text-theme-text/60 uppercase pl-1">Radius:</span>
            <div class="flex items-center bg-theme-card border border-theme-border/80 px-1 py-0.5">
              <input
                type="number"
                min="0"
                max="200"
                value={cornerRadius()}
                onInput={(e) => {
                  const val = parseInt(e.currentTarget.value, 10);
                  if (!isNaN(val) && val >= 0) {
                    handleCornerRadiusChange(val);
                  }
                }}
                class="w-10 bg-transparent text-xs font-mono font-bold text-theme-cyan text-center focus:outline-none"
              />
              <span class="text-[10px] font-mono text-theme-text/40 pr-0.5">px</span>
            </div>
            <input
              type="range"
              min="0"
              max="64"
              step="1"
              value={cornerRadius()}
              onInput={(e) => handleCornerRadiusChange(Number(e.currentTarget.value))}
              class="w-14 accent-theme-cyan h-1.5 bg-theme-bg cursor-pointer"
            />
          </div>
        </Show>
      </Show>
    </div>
  );
};
