export type CanvasTool = 
  | 'select' 
  | 'brush' 
  | 'rectangle' 
  | 'circle' 
  | 'line' 
  | 'arrow' 
  | 'text' 
  | 'eraser';

export type StrokeAlign = 'center' | 'inner' | 'outer';
export type ArrowHead = 'none' | 'end' | 'both';
export type LineStyle = 'solid' | 'dashed' | 'dotted';

export interface CanvasStyleConfig {
  strokeColor: string;
  fillColor: string;
  strokeWidth: number;
  strokeAlign: StrokeAlign;
  cornerRadius: number;
  fontSize: number;
  opacity: number;
}

export interface CanvasLayerItem {
  id: string;
  name: string;
  type: string;
  visible: boolean;
  locked: boolean;
  selected: boolean;
  fillColor?: string;
  strokeColor?: string;
  details?: string;
}

// ── MCP AI Tool Calling Action Schemas ──
// Standardized JSON format that can be produced by LLM tools or backend workers

export interface McpDrawRect {
  type: 'draw_rect';
  x: number;
  y: number;
  width: number;
  height: number;
  stroke?: string;
  fill?: string;
  strokeWidth?: number;
  rx?: number;
  ry?: number;
  label?: string;
}

export interface McpDrawCircle {
  type: 'draw_circle';
  x: number;
  y: number;
  radius: number;
  stroke?: string;
  fill?: string;
  strokeWidth?: number;
  label?: string;
}

export interface McpDrawLine {
  type: 'draw_line';
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  stroke?: string;
  strokeWidth?: number;
}

export interface McpDrawArrow {
  type: 'draw_arrow';
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  stroke?: string;
  strokeWidth?: number;
  label?: string;
}

export interface McpDrawText {
  type: 'draw_text';
  x: number;
  y: number;
  text: string;
  fontSize?: number;
  fill?: string;
  fontFamily?: string;
}

export interface McpDrawPath {
  type: 'draw_path';
  path: string; // SVG path string e.g. "M 10 10 L 20 20 Z"
  stroke?: string;
  fill?: string;
  strokeWidth?: number;
}

export interface McpClearCanvas {
  type: 'clear';
}

export interface McpSetBackground {
  type: 'set_background';
  color: string;
}

export type McpDrawingAction = 
  | McpDrawRect
  | McpDrawCircle
  | McpDrawLine
  | McpDrawArrow
  | McpDrawText
  | McpDrawPath
  | McpClearCanvas
  | McpSetBackground;

export interface McpDrawingStep {
  id: string;
  action: McpDrawingAction;
  description?: string;
  delayMs?: number;
  status: 'pending' | 'drawing' | 'completed' | 'error';
}

export interface McpDrawingPlan {
  title: string;
  steps: McpDrawingStep[];
}
