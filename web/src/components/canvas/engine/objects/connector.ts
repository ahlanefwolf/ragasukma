import { Line, FabricObject } from 'fabric';
import { snapCoord, setupLineControls } from '../drawing';
import { arrowHead, connectorRouting } from '../../canvasStore';
import { computeConnectorPoints } from '../connector';
import { getLineStrokeProps } from './baseProps';
import { FinalizeContext, Point2D, ShapeCreator } from './types';

export const ConnectorCreator: ShapeCreator<Line> = {
  create(origin: Point2D): Line {
    const props = getLineStrokeProps();
    const line = new Line([origin.x, origin.y, origin.x, origin.y], props as any);

    const head = arrowHead();
    (line as any).isArrow = true;
    (line as any).arrowEnd = head === 'none' ? true : (head === 'end' || head === 'both');
    (line as any).arrowStart = head === 'both';

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

    const route = computeConnectorPoints(
      origin.x,
      origin.y,
      endPt.x,
      endPt.y,
      connectorRouting(),
      ctx.startSnapPort?.port?.id,
      ctx.endSnapPort?.port?.id,
      ctx.startSnapPort?.obj,
      ctx.endSnapPort?.obj
    );

    (line as any).connectorData = {
      fromObj: ctx.startSnapPort?.obj,
      fromPortId: ctx.startSnapPort?.port?.id,
      toObj: ctx.endSnapPort?.obj,
      toPortId: ctx.endSnapPort?.port?.id,
      startX: origin.x,
      startY: origin.y,
      endX: endPt.x,
      endY: endPt.y,
      routing: connectorRouting(),
      computedPoints: route.points,
      lastAngle: route.lastAngle,
      firstAngle: route.firstAngle,
    };
    (line as any).connectorType = 'arrow';
    (line as any).isConnectorLine = true;

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
    return line;
  },

  isTiny(line: Line): boolean {
    return Math.hypot((line.x2 ?? 0) - (line.x1 ?? 0), (line.y2 ?? 0) - (line.y1 ?? 0)) < 3;
  },
};
