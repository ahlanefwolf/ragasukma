import {
  Canvas,
  Rect,
  Circle,
  Line,
  IText,
  Polygon,
  FabricObject,
} from 'fabric';
import {
  activeTool,
  setActiveTool,
  strokeColor,
  fillColor,
  strokeWidth,
  strokeAlign,
  setStrokeAlign,
  fontSize,
  setFontSize,
  fontFamily,
  setFontFamily,
  cornerRadius,
  setSelectedObjectType,
  setStrokeWidth,
  setCornerRadius,
  setStrokeColor,
  setFillColor,
  snapToGrid,
  GRID_SIZE,
  arrowHead,
  setArrowHead,
  lineStyle,
  setLineStyle,
} from '../../canvasStore';
import {
  snapCoord,
  getPointerCoords,
  createArrowPath,
  createArrowObject,
  applyShapeStrokeAlignment,
  syncShapeClipPath,
  setupLineControls,
} from '../drawing';
import {
  findClosestConnectionPort,
  showConnectionIndicators,
  clearConnectionIndicators,
  updateConnectedLines,
  ConnectionPort,
} from '../connector';
import { LayersEngine } from '../layers';
import { ShapeBuilderEngine } from '../shapeBuilder';
import { ViewportManager } from './viewport';
import { HistoryManager } from './history';

export class EventsManager {
  public isMouseDown = false;
  public origX = 0;
  public origY = 0;
  public activeShape: FabricObject | null = null;
  private activeChain: Line[] = [];
  private chainPoints: Array<{ x: number; y: number }> = [];
  private lastArrowObject: FabricObject | null = null;
  private startSnapPort: { port: ConnectionPort; obj: FabricObject } | null = null;
  private endSnapPort: { port: ConnectionPort; obj: FabricObject } | null = null;

  public resetChain() {
    this.activeChain = [];
    this.chainPoints = [];
    this.lastArrowObject = null;
    this.startSnapPort = null;
    this.endSnapPort = null;
  }

  public bindEvents(
    canvas: Canvas,
    viewport: ViewportManager,
    history: HistoryManager
  ) {
    // ── Mouse Wheel Zooming ──
    canvas.on('mouse:wheel', (opt: any) => {
      viewport.handleWheel(canvas, opt);
    });

    // ── Mouse Down: Creation & Panning ──
    canvas.on('mouse:down', (o: any) => {
      const e = o.e;
      if (viewport.isSpacePressed || (e && e.button === 1)) {
        viewport.isPanning = true;
        viewport.lastPosX = e.clientX;
        viewport.lastPosY = e.clientY;
        canvas.defaultCursor = 'grabbing';
        return;
      }

      const tool = activeTool();
      if (tool !== 'line' && tool !== 'arrow') {
        this.resetChain();
      }
      if (tool === 'select' || tool === 'brush') return;

      const pointer = getPointerCoords(canvas, o.e, o);
      this.isMouseDown = true;
      this.origX = snapCoord(pointer.x);
      this.origY = snapCoord(pointer.y);

      if (tool === 'line' || tool === 'arrow') {
        const snap = findClosestConnectionPort(canvas, pointer, null, 24);
        if (snap) {
          this.origX = snap.port.x;
          this.origY = snap.port.y;
          this.startSnapPort = snap;
          showConnectionIndicators(canvas, snap.port, snap.obj);
        } else {
          this.startSnapPort = null;
        }

        if (this.chainPoints.length > 0) {
          const lastPt = this.chainPoints[this.chainPoints.length - 1];
          const distToLast = Math.hypot(this.origX - lastPt.x, this.origY - lastPt.y);
          if (distToLast <= 18) {
            this.origX = lastPt.x;
            this.origY = lastPt.y;
          } else {
            this.resetChain();
            this.chainPoints = [{ x: this.origX, y: this.origY }];
          }
        } else {
          this.chainPoints = [{ x: this.origX, y: this.origY }];
        }
      }

      if (tool === 'eraser') {
        const target = o.target;
        if (target) {
          canvas.remove(target);
          canvas.renderAll();
          history.saveState(canvas);
        }
        return;
      }

      if (tool === 'rectangle') {
        const r = cornerRadius();
        const hasBorder = strokeColor() !== 'transparent' && strokeWidth() > 0;
        const actualWidth = hasBorder ? strokeWidth() : 0;
        const actualStroke = hasBorder ? strokeColor() : '';
        const rect = new Rect({
          left: this.origX,
          top: this.origY,
          originX: 'left',
          originY: 'top',
          width: 0,
          height: 0,
          fill: fillColor() === 'transparent' ? '' : fillColor(),
          stroke: actualStroke,
          strokeWidth: actualWidth,
          strokeUniform: true,
          strokeAlign: strokeAlign(),
          rx: r,
          ry: r,
          selectable: true,
        } as any);
        canvas.add(rect);
        this.activeShape = rect;
      }

      if (tool === 'circle') {
        const hasBorder = strokeColor() !== 'transparent' && strokeWidth() > 0;
        const actualWidth = hasBorder ? strokeWidth() : 0;
        const actualStroke = hasBorder ? strokeColor() : '';
        const circle = new Circle({
          left: this.origX,
          top: this.origY,
          originX: 'center',
          originY: 'center',
          radius: 0,
          fill: fillColor() === 'transparent' ? '' : fillColor(),
          stroke: actualStroke,
          strokeWidth: actualWidth,
          strokeUniform: true,
          strokeAlign: strokeAlign(),
          selectable: true,
        } as any);
        canvas.add(circle);
        this.activeShape = circle;
      } else if (tool === 'line') {
        const line = new Line([this.origX, this.origY, this.origX, this.origY], {
          stroke: strokeColor() === 'transparent' ? '#7BDFF2' : strokeColor(),
          strokeWidth: strokeWidth() || 2,
          strokeUniform: true,
          strokeLineCap: 'round',
          strokeLineJoin: 'round',
          selectable: false,
        } as any);
        if (lineStyle() === 'dashed') {
          line.strokeDashArray = [8, 6];
        } else if (lineStyle() === 'dotted') {
          line.strokeDashArray = [3, 4];
        }
        setupLineControls(line);
        canvas.add(line);
        this.activeShape = line;
      } else if (tool === 'arrow') {
        const line = new Line([this.origX, this.origY, this.origX, this.origY], {
          stroke: strokeColor() === 'transparent' ? '#7BDFF2' : strokeColor(),
          strokeWidth: strokeWidth() || 2,
          strokeUniform: true,
          strokeLineCap: 'round',
          strokeLineJoin: 'round',
          selectable: false,
        } as any);
        (line as any).isArrow = true;
        (line as any).arrowEnd = arrowHead() === 'none' ? true : (arrowHead() === 'end' || arrowHead() === 'both');
        (line as any).arrowStart = arrowHead() === 'both';
        if (lineStyle() === 'dashed') {
          line.strokeDashArray = [8, 6];
        } else if (lineStyle() === 'dotted') {
          line.strokeDashArray = [3, 4];
        }
        setupLineControls(line);
        canvas.add(line);
        this.activeShape = line;
      } else if (tool === 'text') {
        const itext = new IText('Teks disini...', {
          left: this.origX,
          top: this.origY,
          fontFamily: fontFamily(),
          fill: strokeColor(),
          fontSize: fontSize(),
          selectable: true,
        });
        canvas.add(itext);
        canvas.setActiveObject(itext);
        itext.enterEditing();
        itext.selectAll();
        this.isMouseDown = false;
        history.saveState(canvas);
      }
    });

    // ── Mouse Move: Interactive Resizing & Panning ──
    canvas.on('mouse:move', (o: any) => {
      if (viewport.isPanning) {
        const e = o.e;
        const vpt = canvas.viewportTransform;
        if (vpt) {
          vpt[4] += e.clientX - viewport.lastPosX;
          vpt[5] += e.clientY - viewport.lastPosY;
          canvas.requestRenderAll();
          viewport.lastPosX = e.clientX;
          viewport.lastPosY = e.clientY;
        }
        return;
      }

      if (!this.isMouseDown || !this.activeShape) return;

      const pointer = getPointerCoords(canvas, o.e, o);
      const tool = activeTool();

      if (tool === 'rectangle') {
        const rect = this.activeShape as Rect;
        const currentX = snapCoord(pointer.x);
        const currentY = snapCoord(pointer.y);

        const left = Math.min(this.origX, currentX);
        const top = Math.min(this.origY, currentY);
        const rawW = Math.abs(currentX - this.origX);
        const rawH = Math.abs(currentY - this.origY);

        rect.set({
          left,
          top,
          width: snapToGrid() ? Math.max(GRID_SIZE, Math.round(rawW / GRID_SIZE) * GRID_SIZE) : rawW,
          height: snapToGrid() ? Math.max(GRID_SIZE, Math.round(rawH / GRID_SIZE) * GRID_SIZE) : rawH,
        });
      } else if (tool === 'circle') {
        const circle = this.activeShape as Circle;
        const currentX = snapCoord(pointer.x);
        const currentY = snapCoord(pointer.y);
        const rawRadius =
          Math.sqrt(
            Math.pow(currentX - this.origX, 2) + Math.pow(currentY - this.origY, 2)
          ) / 2;
        const radius = snapToGrid()
          ? Math.max(GRID_SIZE / 2, Math.round((rawRadius * 2) / GRID_SIZE) * (GRID_SIZE / 2))
          : rawRadius;

        circle.set({
          radius,
          left: (this.origX + currentX) / 2,
          top: (this.origY + currentY) / 2,
        });
      } else if (tool === 'line' || tool === 'arrow') {
        const line = this.activeShape as Line;
        let targetX = snapCoord(pointer.x);
        let targetY = snapCoord(pointer.y);

        // Snap to destination anchor port (Draw.io magnetic connection)
        const snapTarget = findClosestConnectionPort(canvas, pointer, this.startSnapPort?.obj, 24);
        if (snapTarget) {
          targetX = snapTarget.port.x;
          targetY = snapTarget.port.y;
          this.endSnapPort = snapTarget;
          showConnectionIndicators(canvas, snapTarget.port, snapTarget.obj);
        } else {
          this.endSnapPort = null;
        }

        // Snap to start point of the chain if close enough (creates a closed loop)
        if (tool === 'line' && this.chainPoints.length >= 3) {
          const startPt = this.chainPoints[0];
          const distToStart = Math.hypot(targetX - startPt.x, targetY - startPt.y);
          if (distToStart <= 20) {
            targetX = startPt.x;
            targetY = startPt.y;
          }
        }

        // Shift key: Snap to 45 degree angle intervals
        if (o.e?.shiftKey) {
          const dx = targetX - this.origX;
          const dy = targetY - this.origY;
          const dist = Math.hypot(dx, dy);
          const rawAngle = Math.atan2(dy, dx);
          const snapAngle = Math.round(rawAngle / (Math.PI / 4)) * (Math.PI / 4);
          targetX = this.origX + dist * Math.cos(snapAngle);
          targetY = this.origY + dist * Math.sin(snapAngle);
        }

        line.set({
          x2: targetX,
          y2: targetY,
        });
        line.setCoords();
      }

      canvas.renderAll();
    });

    // ── Mouse Up: Finalize Creation ──
    canvas.on('mouse:up', () => {
      if (viewport.isPanning) {
        viewport.isPanning = false;
        canvas.defaultCursor = viewport.isSpacePressed
          ? 'grab'
          : activeTool() === 'select'
          ? 'default'
          : 'crosshair';
        return;
      }

      if (!this.isMouseDown) return;
      this.isMouseDown = false;
      clearConnectionIndicators(canvas);

      if (this.activeShape) {
        if (activeTool() !== 'line' && activeTool() !== 'arrow') {
          applyShapeStrokeAlignment(this.activeShape, strokeAlign(), strokeWidth());
        }
        syncShapeClipPath(this.activeShape);

        if (activeTool() === 'line') {
          const line = this.activeShape as Line;
          const endPt = { x: line.x2 || this.origX, y: line.y2 || this.origY };

          // Check if it closes the loop back to the initial start point
          if (
            this.chainPoints.length >= 3 &&
            Math.hypot(endPt.x - this.chainPoints[0].x, endPt.y - this.chainPoints[0].y) <= 8
          ) {
            // Convert multi-line chain into a unified closed polygon
            const polygonPoints = [...this.chainPoints];
            this.activeChain.forEach((l) => canvas.remove(l));
            canvas.remove(line);

            const polygon = new Polygon(polygonPoints, {
              fill: fillColor() === 'transparent' ? '' : fillColor(),
              stroke: strokeColor(),
              strokeWidth: strokeWidth(),
              strokeUniform: true,
              strokeAlign: strokeAlign(),
              selectable: true,
            } as any);

            applyShapeStrokeAlignment(polygon, strokeAlign(), strokeWidth());
            canvas.add(polygon);
            canvas.setActiveObject(polygon);
            this.resetChain();
          } else {
            if (this.startSnapPort || this.endSnapPort) {
              (line as any).connectorData = {
                fromObj: this.startSnapPort?.obj,
                fromPortId: this.startSnapPort?.port.id,
                toObj: this.endSnapPort?.obj,
                toPortId: this.endSnapPort?.port.id,
                startX: this.origX,
                startY: this.origY,
                endX: endPt.x,
                endY: endPt.y,
              };
              (line as any).connectorType = 'line';
              (line as any).isConnectorLine = true;
            }
            setupLineControls(line);
            line.set({ selectable: true });
            line.setCoords();
            canvas.setActiveObject(line);
            this.activeChain.push(line);
            this.chainPoints.push(endPt);
          }
        }

        if (activeTool() === 'arrow') {
          const line = this.activeShape as Line;
          const endPt = { x: line.x2 || this.origX, y: line.y2 || this.origY };

          if (this.startSnapPort || this.endSnapPort) {
            (line as any).connectorData = {
              fromObj: this.startSnapPort?.obj,
              fromPortId: this.startSnapPort?.port.id,
              toObj: this.endSnapPort?.obj,
              toPortId: this.endSnapPort?.port.id,
              startX: this.origX,
              startY: this.origY,
              endX: endPt.x,
              endY: endPt.y,
            };
            (line as any).connectorType = 'arrow';
            (line as any).isConnectorLine = true;
          }

          setupLineControls(line);
          line.set({ selectable: true });
          line.setCoords();
          canvas.setActiveObject(line);
        }

        if (this.activeShape) {
          this.activeShape.set({ selectable: true });
          this.activeShape.setCoords();
        }
        this.activeShape = null;
        canvas.renderAll();
        history.saveState(canvas);
      }
    });

    // ── Object Snapping & Synchronization ──
    canvas.on('object:moving', (e: any) => {
      if (e.target) {
        if (snapToGrid()) {
          e.target.set({
            left: Math.round((e.target.left || 0) / GRID_SIZE) * GRID_SIZE,
            top: Math.round((e.target.top || 0) / GRID_SIZE) * GRID_SIZE,
          });
        }
        updateConnectedLines(canvas, e.target);
      }
    });

    canvas.on('object:scaling', (e: any) => {
      const target = e.target;
      if (target) {
        updateConnectedLines(canvas, target);
      }
    });

    canvas.on('object:rotating', (e: any) => {
      if (e.target) {
        updateConnectedLines(canvas, e.target);
      }
    });

    canvas.on('object:added', () => {
      LayersEngine.syncLayers(canvas);
      if (!history.isProcessingHistory && !this.isMouseDown) {
        history.saveState(canvas);
      }
    });

    canvas.on('object:removed', () => {
      LayersEngine.syncLayers(canvas);
      if (!history.isProcessingHistory) {
        history.saveState(canvas);
      }
    });

    canvas.on('object:modified', (e: any) => {
      const target = e.target;
      if (target) {
        if (snapToGrid()) {
          // Normalisasi dimensi scale ke real width & height tanpa menggeser posisi origin/titik tumpu
          if (target.type === 'rect') {
            const rawW = target.width * (target.scaleX || 1);
            const rawH = target.height * (target.scaleY || 1);
            const snappedW = Math.max(
              GRID_SIZE,
              Math.round(rawW / GRID_SIZE) * GRID_SIZE
            );
            const snappedH = Math.max(
              GRID_SIZE,
              Math.round(rawH / GRID_SIZE) * GRID_SIZE
            );
            target.set({
              width: snappedW,
              height: snappedH,
              scaleX: 1,
              scaleY: 1,
            });
            target.setCoords();
          } else if (target.type === 'circle') {
            const rawDiameter = target.radius * 2 * (target.scaleX || 1);
            const snapD = Math.max(
              GRID_SIZE,
              Math.round(rawDiameter / GRID_SIZE) * GRID_SIZE
            );
            target.set({
              radius: snapD / 2,
              scaleX: 1,
              scaleY: 1,
            });
            target.setCoords();
          }
        }
        syncShapeClipPath(target);
        updateConnectedLines(canvas, target);
      }
      LayersEngine.syncLayers(canvas);
      if (!history.isProcessingHistory) {
        history.saveState(canvas);
      }
    });

    const handleSelectionSync = (e: any) => {
      const activeObjects = canvas.getActiveObjects();
      const obj = e.selected?.[0] || canvas.getActiveObject();
      if (obj) {
        // Otomatis ubah active tool ke 'select' saat objek diklik/dipilih
        if (activeTool() !== 'select' && activeTool() !== 'brush') {
          setActiveTool('select');
        }

        const isArrowObj = (obj instanceof Line || obj.type === 'line') && (obj.arrowEnd || obj.isArrow);
        const typeStr = activeObjects.length > 1 
          ? 'activeSelection' 
          : isArrowObj ? 'arrow' : (obj.type ? obj.type.toLowerCase() : null);
        setSelectedObjectType(typeStr);

        if (obj instanceof Line || obj.type === 'line') {
          setupLineControls(obj);
          if (obj.arrowStart && obj.arrowEnd) {
            setArrowHead('both');
          } else if (obj.arrowEnd || obj.isArrow) {
            setArrowHead('end');
          } else {
            setArrowHead('none');
          }

          if (obj.strokeDashArray && obj.strokeDashArray.length > 0) {
            if (obj.strokeDashArray[0] <= 4) {
              setLineStyle('dotted');
            } else {
              setLineStyle('dashed');
            }
          } else {
            setLineStyle('solid');
          }
        }

        if (activeObjects.length > 1) {
          ShapeBuilderEngine.updateHatchPreview(canvas);
        } else {
          ShapeBuilderEngine.clearHatchPreview(canvas);
        }

        const currentWidth =
          typeof obj.strokeWidth === 'number' ? obj.strokeWidth : (typeof obj.rawStrokeWidth === 'number' ? obj.rawStrokeWidth : 0);
        if (currentWidth >= 0) {
          setStrokeWidth(currentWidth);
        }
        if (obj.strokeAlign) {
          setStrokeAlign(obj.strokeAlign);
        } else {
          setStrokeAlign('center');
        }
        if (typeof obj.rx === 'number') {
          setCornerRadius(obj.rx);
        }
        if (typeof obj.stroke === 'string' && obj.stroke && obj.stroke !== 'transparent' && currentWidth > 0) {
          setStrokeColor(obj.stroke);
        } else if (!obj.stroke || currentWidth === 0) {
          setStrokeColor('transparent');
        }
        if (typeof obj.fill === 'string') {
          setFillColor(obj.fill || 'transparent');
        }
        if (typeof obj.fontSize === 'number') {
          setFontSize(obj.fontSize);
        }
        if (typeof obj.fontFamily === 'string' && obj.fontFamily) {
          setFontFamily(obj.fontFamily);
        }
      } else {
        setSelectedObjectType(null);
      }
      LayersEngine.syncLayers(canvas);
    };

    canvas.on('selection:created', handleSelectionSync);
    canvas.on('selection:updated', handleSelectionSync);
    canvas.on('selection:cleared', () => {
      setSelectedObjectType(null);
      ShapeBuilderEngine.clearHatchPreview(canvas);
      LayersEngine.syncLayers(canvas);
    });
  }
}
