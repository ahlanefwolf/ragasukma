import {
  Canvas,
  Rect,
  IText,
  Line,
  PencilBrush,
  FabricObject,
} from 'fabric';
import { CanvasTool } from '../../types';
import {
  activeTool,
  strokeColor,
  fillColor,
  strokeWidth,
  strokeAlign,
  fontSize,
  fontFamily,
  cornerRadius,
  arrowHead,
  lineStyle,
  connectorRouting,
} from '../../canvasStore';
import { computeConnectorPoints } from '../connector';
import { applyShapeStrokeAlignment } from '../drawing';
import { LayersEngine } from '../layers';
import { HistoryManager } from './history';
import { ClipboardManager } from './clipboard';
import { ViewportManager } from './viewport';
import { EventsManager } from './events';

export * from './history';
export * from './clipboard';
export * from './viewport';
export * from './events';

export class FabricCore {
  private canvas: Canvas | null = null;
  private history = new HistoryManager();
  private clipboard = new ClipboardManager();
  private viewport = new ViewportManager();
  private events = new EventsManager();

  public getCanvas(): Canvas | null {
    return this.canvas;
  }

  public init(canvasEl: HTMLCanvasElement, width: number, height: number) {
    if (this.canvas) {
      this.destroy();
    }

    // Ensure all objects maintain pure pixel stroke widths when scaled
    FabricObject.ownDefaults.strokeUniform = true;
    // Matikan cache raster untuk garis/panah agar tidak terpotong saat zoom in / zoom out
    // dan pastikan fill default garis adalah string kosong (transparan), BUKAN hitam bawaan Fabric
    (Line as any).ownDefaults = {
      ...(Line as any).ownDefaults,
      objectCaching: false,
      fill: '',
    };

    this.canvas = new Canvas(canvasEl, {
      width,
      height,
      backgroundColor: '',
      selection: true,
      selectionFullyContained: false, // Sentuhan seleksi parsial tetap memilih objek
      preserveObjectStacking: true,
    });

    this.events.bindEvents(this.canvas, this.viewport, this.history);
    this.updateTool(activeTool());
    this.saveState();
  }

  public resize(width: number, height: number) {
    if (!this.canvas) return;
    this.canvas.setDimensions({ width, height });
    this.canvas.renderAll();
  }

  public destroy() {
    if (this.canvas) {
      this.canvas.off();
      this.canvas.dispose();
      this.canvas = null;
    }
    this.history.reset();
  }

  public setSpacePressed(pressed: boolean) {
    this.viewport.isSpacePressed = pressed;
    if (!this.canvas) return;
    if (pressed) {
      this.canvas.defaultCursor = 'grab';
      this.canvas.isDrawingMode = false;
    } else {
      this.updateTool(activeTool());
    }
  }

  public updateTool(tool: CanvasTool) {
    if (!this.canvas) return;

    if (tool !== 'line') {
      this.events.resetChain();
    }

    this.canvas.isDrawingMode = tool === 'brush' && !this.viewport.isSpacePressed;
    this.canvas.selection = tool === 'select' && !this.viewport.isSpacePressed;
    this.canvas.defaultCursor = tool === 'select' ? 'default' : 'crosshair';

    this.canvas.forEachObject((obj: any) => {
      if (obj.isConnectionIndicator) return;
      obj.selectable = tool === 'select';
      obj.evented = tool === 'select' || tool === 'eraser';
    });

    if (tool === 'brush' && !this.viewport.isSpacePressed) {
      if (!this.canvas.freeDrawingBrush) {
        this.canvas.freeDrawingBrush = new PencilBrush(this.canvas);
      }
      this.canvas.freeDrawingBrush.color = strokeColor();
      this.canvas.freeDrawingBrush.width = strokeWidth();
    }

    this.canvas.renderAll();
  }

  public updateStyles() {
    if (!this.canvas) return;
    if (this.canvas.isDrawingMode && this.canvas.freeDrawingBrush) {
      this.canvas.freeDrawingBrush.color = strokeColor();
      this.canvas.freeDrawingBrush.width = strokeWidth();
    }

    const activeObj = this.canvas.getActiveObject();
    if (activeObj) {
      const applyStyleToObj = (obj: any) => {
        if (obj instanceof IText || obj.type === 'i-text' || obj.type === 'text') {
          obj.set({ fill: strokeColor(), fontSize: fontSize(), fontFamily: fontFamily() });
        } else if (obj instanceof Line || obj.type === 'line') {
          const hasBorder = strokeColor() !== 'transparent' && strokeWidth() > 0;
          let dash: number[] | null = null;
          if (lineStyle() === 'dashed') {
            dash = [8, 6];
          } else if (lineStyle() === 'dotted') {
            dash = [3, 4];
          } else {
            dash = null;
          }

          const hasEnd = arrowHead() === 'end' || arrowHead() === 'both';
          const hasStart = arrowHead() === 'both';

          obj.set({
            stroke: hasBorder ? strokeColor() : '#29483A',
            strokeWidth: hasBorder ? strokeWidth() : 2,
            strokeDashArray: dash,
            strokeLineCap: 'round',
            strokeLineJoin: 'round',
          });
          obj.arrowEnd = hasEnd;
          obj.arrowStart = hasStart;
          obj.isArrow = hasEnd || hasStart;

          // Update rute garis / konektor secara reaktif (Lurus, Siku, Lengkung)
          const lineStartX = (obj as any).x1 ?? obj.connectorData?.startX ?? 0;
          const lineStartY = (obj as any).y1 ?? obj.connectorData?.startY ?? 0;
          const lineEndX = (obj as any).x2 ?? obj.connectorData?.endX ?? 100;
          const lineEndY = (obj as any).y2 ?? obj.connectorData?.endY ?? 100;

          const routing = connectorRouting();

          if (!obj.connectorData) {
            obj.connectorData = {
              startX: lineStartX,
              startY: lineStartY,
              endX: lineEndX,
              endY: lineEndY,
              routing,
            };
          }

          obj.connectorData.routing = routing;
          obj.connectorData.startX = lineStartX;
          obj.connectorData.startY = lineStartY;
          obj.connectorData.endX = lineEndX;
          obj.connectorData.endY = lineEndY;

          const route = computeConnectorPoints(
            lineStartX,
            lineStartY,
            lineEndX,
            lineEndY,
            routing,
            obj.connectorData.fromPortId,
            obj.connectorData.toPortId,
            obj.connectorData.fromObj,
            obj.connectorData.toObj
          );
          obj.connectorData.computedPoints = route.points;
          obj.connectorData.lastAngle = route.lastAngle;
          obj.connectorData.firstAngle = route.firstAngle;

          obj.dirty = true;
          obj.setCoords();
        } else {
          const hasBorder = strokeColor() !== 'transparent' && strokeWidth() > 0;
          let dash: number[] | null = null;
          if (lineStyle() === 'dashed') {
            dash = [8, 6];
          } else if (lineStyle() === 'dotted') {
            dash = [3, 4];
          } else {
            dash = null;
          }

          const updateData: any = {
            stroke: hasBorder ? strokeColor() : '',
            strokeWidth: hasBorder ? strokeWidth() : 0,
            strokeDashArray: dash,
            strokeUniform: true,
            fill: fillColor() === 'transparent' ? '' : fillColor(),
          };
          if (obj instanceof Rect || obj.type === 'rect') {
            updateData.rx = cornerRadius();
            updateData.ry = cornerRadius();
          }
          obj.set(updateData);
          applyShapeStrokeAlignment(obj, strokeAlign(), hasBorder ? strokeWidth() : 0);
          obj.dirty = true;
          obj.setCoords();
        }
      };

      if (
        typeof (activeObj as any).getObjects === 'function' &&
        activeObj.type === 'activeSelection'
      ) {
        (activeObj as any).getObjects().forEach((child: any) => applyStyleToObj(child));
        activeObj.setCoords();
      } else {
        applyStyleToObj(activeObj);
      }

      this.canvas.renderAll();
      this.saveState();
      LayersEngine.syncLayers(this.canvas);
    }
  }

  // ── Viewport Panning & Zoom Controls ──

  public zoomIn() {
    this.viewport.zoomIn(this.canvas);
  }

  public zoomOut() {
    this.viewport.zoomOut(this.canvas);
  }

  public resetZoom() {
    this.viewport.resetZoom(this.canvas);
  }

  // ── Clipboard Controls ──

  public hasClipboard(): boolean {
    return this.clipboard.hasClipboard();
  }

  public async copy(): Promise<boolean> {
    return this.clipboard.copy(this.canvas);
  }

  public async paste(): Promise<boolean> {
    return this.clipboard.paste(this.canvas, () => this.saveState());
  }

  public async duplicate(): Promise<boolean> {
    return this.clipboard.duplicate(this.canvas, () => this.saveState());
  }

  public deleteSelected() {
    this.clipboard.deleteSelected(this.canvas, () => this.saveState());
  }

  public isEditingText(): boolean {
    return this.clipboard.isEditingText(this.canvas);
  }

  // ── History & Lifecycle Controls ──

  public saveState() {
    this.history.saveState(this.canvas);
  }

  public undo() {
    this.history.undo(this.canvas);
  }

  public redo() {
    this.history.redo(this.canvas);
  }

  public clear() {
    if (!this.canvas) return;
    this.canvas.clear();
    this.canvas.backgroundColor = '';
    this.canvas.renderAll();
    this.saveState();
    LayersEngine.syncLayers(this.canvas);
  }
}
