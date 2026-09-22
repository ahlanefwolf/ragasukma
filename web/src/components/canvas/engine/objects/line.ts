import { Line, Polygon, FabricObject } from 'fabric';
import { snapCoord, setupLineControls, applyShapeStrokeAlignment } from '../drawing';
import { fillColor, strokeColor, strokeWidth, strokeAlign } from '../../canvasStore';
import { getLineStrokeProps } from './baseProps';
import { FinalizeContext, Point2D, ShapeCreator } from './types';

export const LineCreator: ShapeCreator<Line> & {
  resolveStartPoint(origin: Point2D, state: { chainPoints: Point2D[]; resetChain: () => void }): Point2D;
} = {
  resolveStartPoint(origin: Point2D, state: { chainPoints: Point2D[]; resetChain: () => void }): Point2D {
    if (state.chainPoints.length > 0) {
      const lastPt = state.chainPoints[state.chainPoints.length - 1];
      const distToLast = Math.hypot(origin.x - lastPt.x, origin.y - lastPt.y);
      if (distToLast <= 18) {
        return { x: lastPt.x, y: lastPt.y };
      }
      state.resetChain();
    }
    state.chainPoints.push({ x: origin.x, y: origin.y });
    return origin;
  },

  create(origin: Point2D): Line {
    const props = getLineStrokeProps();
    const line = new Line([origin.x, origin.y, origin.x, origin.y], props as any);
    setupLineControls(line);
    return line;
  },

  update(line: Line, origin: Point2D, pointer: Point2D, options?: { shiftKey?: boolean; e?: any }) {
    let targetX = snapCoord(pointer.x);
    let targetY = snapCoord(pointer.y);

    if (options?.shiftKey) {
      const dx = targetX - origin.x;
      const dy = targetY - origin.y;
      const dist = Math.hypot(dx, dy);
      const rawAngle = Math.atan2(dy, dx);
      const snapAngle = Math.round(rawAngle / (Math.PI / 4)) * (Math.PI / 4);
      targetX = origin.x + dist * Math.cos(snapAngle);
      targetY = origin.y + dist * Math.sin(snapAngle);
    }

    line.set({
      x2: targetX,
      y2: targetY,
    });
    (line as any).connectorData = undefined;
    line.dirty = true;
    line.setCoords();
  },

  finalize(line: Line, origin: Point2D, ctx: FinalizeContext): FabricObject | null {
    const endPt = { x: line.x2 || origin.x, y: line.y2 || origin.y };

    // Jika ujung garis menutup kembali ke titik awal (loop poligon tertutup)
    if (
      ctx.chainPoints &&
      ctx.chainPoints.length >= 3 &&
      Math.hypot(endPt.x - ctx.chainPoints[0].x, endPt.y - ctx.chainPoints[0].y) <= 8
    ) {
      const polygonPoints = [...ctx.chainPoints];
      ctx.activeChain?.forEach((l) => ctx.canvas.remove(l));
      ctx.canvas.remove(line);

      const polygon = new Polygon(polygonPoints, {
        fill: fillColor() === 'transparent' ? '' : fillColor(),
        stroke: strokeColor(),
        strokeWidth: strokeWidth(),
        strokeUniform: true,
        strokeAlign: strokeAlign(),
        selectable: true,
      } as any);

      applyShapeStrokeAlignment(polygon, strokeAlign(), strokeWidth());
      ctx.canvas.add(polygon);
      ctx.canvas.setActiveObject(polygon);
      ctx.resetChain?.();
      return polygon;
    }

    // Line bebas 2-titik biasa
    (line as any).connectorData = undefined;
    (line as any).isConnectorLine = false;
    setupLineControls(line);
    line.set({
      x1: origin.x,
      y1: origin.y,
      x2: endPt.x,
      y2: endPt.y,
      left: (origin.x + endPt.x) / 2,
      top: (origin.y + endPt.y) / 2,
      selectable: true,
    });
    line.dirty = true;
    line.setCoords();
    ctx.canvas.setActiveObject(line);
    ctx.activeChain?.push(line);
    ctx.chainPoints?.push(endPt);
    return line;
  },

  isTiny(line: Line): boolean {
    return Math.hypot((line.x2 ?? 0) - (line.x1 ?? 0), (line.y2 ?? 0) - (line.y1 ?? 0)) < 3;
  },
};
