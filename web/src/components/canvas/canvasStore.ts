import { createSignal } from 'solid-js';
import { CanvasTool, CanvasStyleConfig, CanvasLayerItem, McpDrawingPlan, McpDrawingStep, StrokeAlign, ArrowHead, LineStyle } from './types';

// Default Styles
export const DEFAULT_STYLE: CanvasStyleConfig = {
  strokeColor: '#7BDFF2', // theme-cyan
  fillColor: 'transparent',
  strokeWidth: 3,
  strokeAlign: 'center',
  cornerRadius: 8,
  fontSize: 18,
  opacity: 1,
};

// Preset Palette Colors matching the App's Theme & Diagrams
export const STROKE_PALETTE = [
  { label: 'Tanpa Border (None)', hex: 'transparent' },
  { label: 'Cyan Glow', hex: '#7BDFF2' },
  { label: 'Neon Rose', hex: '#F28482' },
  { label: 'Amber Sun', hex: '#F6BD60' },
  { label: 'Mint Soft', hex: '#84DCC6' },
  { label: 'Lavender', hex: '#B8C0FF' },
  { label: 'Pure White', hex: '#FFFFFF' },
  { label: 'Charcoal', hex: '#1E293B' },
];

export const COLOR_PALETTE = [
  { label: 'Cyan Glow', hex: '#7BDFF2' },
  { label: 'Neon Rose', hex: '#F28482' },
  { label: 'Amber Sun', hex: '#F6BD60' },
  { label: 'Mint Soft', hex: '#84DCC6' },
  { label: 'Lavender', hex: '#B8C0FF' },
  { label: 'Pure White', hex: '#FFFFFF' },
  { label: 'Charcoal', hex: '#1E293B' },
];

export const FILL_PALETTE = [
  { label: 'None', hex: 'transparent' },
  { label: 'Cyan Tint', hex: 'rgba(123, 223, 242, 0.15)' },
  { label: 'Rose Tint', hex: 'rgba(242, 132, 130, 0.15)' },
  { label: 'Amber Tint', hex: 'rgba(246, 189, 96, 0.15)' },
  { label: 'Surface', hex: '#161B22' },
  { label: 'White', hex: '#FFFFFF' },
];

// Grid Alignment Size (matching CSS 24px dot-matrix)
export const GRID_SIZE = 24;

export interface FontOption {
  label: string;
  family: string;
  category: string;
}

export const SYSTEM_FONTS: FontOption[] = [
  { label: 'Carter One (App Heading)', family: "'Carter One', cursive, sans-serif", category: 'Heading' },
  { label: 'Inter / System Sans', family: "Inter, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif", category: 'Sans' },
  { label: 'JetBrains / Mono Code', family: "'JetBrains Mono', 'Fira Code', Consolas, monospace", category: 'Mono' },
  { label: 'Comic / Handwriting', family: "'Comic Sans MS', 'Caveat', 'Comic Neue', cursive", category: 'Hand' },
  { label: 'Georgia / Classic Serif', family: "Georgia, 'Times New Roman', Cambria, serif", category: 'Serif' },
  { label: 'Impact / Bold Display', family: "Impact, 'Arial Black', sans-serif", category: 'Display' },
  { label: 'Trebuchet / Clean Sans', family: "'Trebuchet MS', Helvetica, sans-serif", category: 'Sans' },
];

// Reactive Canvas State
export const [activeTool, setActiveTool] = createSignal<CanvasTool>('brush');
export const [selectedObjectType, setSelectedObjectType] = createSignal<string | null>(null);
export const [snapToGrid, setSnapToGrid] = createSignal<boolean>(true);
export const [strokeColor, setStrokeColor] = createSignal<string>(DEFAULT_STYLE.strokeColor);
export const [fillColor, setFillColor] = createSignal<string>(DEFAULT_STYLE.fillColor);
export const [strokeWidth, setStrokeWidth] = createSignal<number>(DEFAULT_STYLE.strokeWidth);
export const [strokeAlign, setStrokeAlign] = createSignal<StrokeAlign>(DEFAULT_STYLE.strokeAlign);
export const [arrowHead, setArrowHead] = createSignal<ArrowHead>('end');
export const [lineStyle, setLineStyle] = createSignal<LineStyle>('solid');
export const [cornerRadius, setCornerRadius] = createSignal<number>(DEFAULT_STYLE.cornerRadius);
export const [fontSize, setFontSize] = createSignal<number>(DEFAULT_STYLE.fontSize);
export const [fontFamily, setFontFamily] = createSignal<string>(SYSTEM_FONTS[0].family);
export const [availableFonts, setAvailableFonts] = createSignal<FontOption[]>(SYSTEM_FONTS);
export const [zoomLevel, setZoomLevel] = createSignal<number>(100);

export const queryOsLocalFonts = async (): Promise<boolean> => {
  if (typeof (window as any).queryLocalFonts === 'function') {
    try {
      const fonts = await (window as any).queryLocalFonts();
      const uniqueFamilies = Array.from(new Set(fonts.map((f: any) => f.family))) as string[];
      const osList: FontOption[] = uniqueFamilies.slice(0, 100).map((name) => ({
        label: name,
        family: `"${name}", sans-serif`,
        category: 'OS Local',
      }));
      setAvailableFonts([...SYSTEM_FONTS, ...osList]);
      return true;
    } catch (err) {
      console.warn('Local font access permission denied or failed:', err);
    }
  }
  return false;
};

// Undo / Redo signals
export const [canUndo, setCanUndo] = createSignal<boolean>(false);
export const [canRedo, setCanRedo] = createSignal<boolean>(false);

// Layers & Objects Management
export const [layersList, setLayersList] = createSignal<CanvasLayerItem[]>([]);
export const [showLayersPanel, setShowLayersPanel] = createSignal<boolean>(true);

// AI MCP Step-by-Step Drawing State
export const [isFullscreen, setIsFullscreen] = createSignal<boolean>(false);
export const [activeAiPlan, setActiveAiPlan] = createSignal<McpDrawingPlan | null>(null);
export const [isAiDrawing, setIsAiDrawing] = createSignal<boolean>(false);
export const [currentAiStepIndex, setCurrentAiStepIndex] = createSignal<number>(-1);
export const [aiSpeedMultiplier, setAiSpeedMultiplier] = createSignal<number>(1);
export const [showAiSimulator, setShowAiSimulator] = createSignal<boolean>(false);

// Demo AI MCP Plans for offline demonstration and testing
export const DEMO_AI_PLANS: McpDrawingPlan[] = [
  {
    title: 'Arsitektur Ragasukma (Brain & Overlay)',
    steps: [
      {
        id: 'step-1',
        description: 'Membersihkan canvas & pasang frame sistem',
        action: { type: 'clear' },
        delayMs: 300,
        status: 'pending',
      },
      {
        id: 'step-2',
        description: 'Menggambar Box "Antigravity LLM Brain"',
        action: {
          type: 'draw_rect',
          x: 80,
          y: 60,
          width: 220,
          height: 100,
          stroke: '#7BDFF2',
          fill: 'rgba(123, 223, 242, 0.12)',
          strokeWidth: 3,
          rx: 8,
          ry: 8,
          label: '🧠 LLM Brain\n(Antigravity Engine)',
        },
        delayMs: 600,
        status: 'pending',
      },
      {
        id: 'step-3',
        description: 'Menggambar Box "Native 3D VRM Overlay"',
        action: {
          type: 'draw_rect',
          x: 420,
          y: 60,
          width: 220,
          height: 100,
          stroke: '#F28482',
          fill: 'rgba(242, 132, 130, 0.12)',
          strokeWidth: 3,
          rx: 8,
          ry: 8,
          label: '🎭 VRM Overlay\n(WGPU 1:1 Three.js)',
        },
        delayMs: 600,
        status: 'pending',
      },
      {
        id: 'step-4',
        description: 'Menghubungkan Brain ke Overlay dengan Panah',
        action: {
          type: 'draw_arrow',
          x1: 300,
          y1: 110,
          x2: 420,
          y2: 110,
          stroke: '#F6BD60',
          strokeWidth: 3,
          label: 'LipSync & Poses',
        },
        delayMs: 500,
        status: 'pending',
      },
      {
        id: 'step-5',
        description: 'Menggambar Box "Audio DSP & STT"',
        action: {
          type: 'draw_rect',
          x: 80,
          y: 220,
          width: 220,
          height: 90,
          stroke: '#84DCC6',
          fill: 'rgba(132, 220, 198, 0.12)',
          strokeWidth: 3,
          rx: 8,
          ry: 8,
          label: '🎙️ Audio Pipeline\n(VAD + Google STT)',
        },
        delayMs: 600,
        status: 'pending',
      },
      {
        id: 'step-6',
        description: 'Menghubungkan Audio ke LLM Brain',
        action: {
          type: 'draw_arrow',
          x1: 190,
          y1: 220,
          x2: 190,
          y2: 160,
          stroke: '#84DCC6',
          strokeWidth: 3,
          label: 'Transkripsi',
        },
        delayMs: 500,
        status: 'pending',
      },
      {
        id: 'step-7',
        description: 'Menambahkan Catatan Status Sistem',
        action: {
          type: 'draw_text',
          x: 80,
          y: 350,
          text: '✨ Semua modul terisolasi dalam background worker & zero lock.',
          fontSize: 15,
          fill: '#FFFFFF',
        },
        delayMs: 400,
        status: 'pending',
      },
    ],
  },
  {
    title: 'Flowchart Logika Decision VAD',
    steps: [
      {
        id: 'vad-1',
        description: 'Membersihkan papan',
        action: { type: 'clear' },
        delayMs: 250,
        status: 'pending',
      },
      {
        id: 'vad-2',
        description: 'Start: Raw PCM Input',
        action: {
          type: 'draw_circle',
          x: 180,
          y: 80,
          radius: 50,
          stroke: '#7BDFF2',
          fill: 'rgba(123, 223, 242, 0.15)',
          strokeWidth: 3,
          label: 'Raw Mic\n44.1kHz',
        },
        delayMs: 500,
        status: 'pending',
      },
      {
        id: 'vad-3',
        description: 'Panah ke Filter',
        action: {
          type: 'draw_arrow',
          x1: 180,
          y1: 130,
          x2: 180,
          y2: 200,
          stroke: '#7BDFF2',
          strokeWidth: 2,
        },
        delayMs: 400,
        status: 'pending',
      },
      {
        id: 'vad-4',
        description: 'Bandpass Filter Step',
        action: {
          type: 'draw_rect',
          x: 80,
          y: 200,
          width: 200,
          height: 70,
          stroke: '#F6BD60',
          fill: 'rgba(246, 189, 96, 0.15)',
          strokeWidth: 3,
          rx: 6,
          ry: 6,
          label: 'Bandpass 160-3800Hz\n+ Highpass 100Hz',
        },
        delayMs: 500,
        status: 'pending',
      },
      {
        id: 'vad-5',
        description: 'Panah ke Evaluasi Energi',
        action: {
          type: 'draw_arrow',
          x1: 180,
          y1: 270,
          x2: 180,
          y2: 340,
          stroke: '#F6BD60',
          strokeWidth: 2,
        },
        delayMs: 400,
        status: 'pending',
      },
      {
        id: 'vad-6',
        description: 'Energi Threshold Decision',
        action: {
          type: 'draw_rect',
          x: 70,
          y: 340,
          width: 220,
          height: 70,
          stroke: '#84DCC6',
          fill: 'rgba(132, 220, 198, 0.15)',
          strokeWidth: 3,
          rx: 6,
          ry: 6,
          label: 'Energy > NoiseFloor * 1.8\n(Speech Detected)',
        },
        delayMs: 500,
        status: 'pending',
      },
    ],
  },
];
