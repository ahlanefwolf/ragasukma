import { Rect } from 'fabric';
import { cornerRadius, snapToGrid, GRID_SIZE } from '../../canvasStore';
import { snapCoord } from '../drawing';
import { getBaseStrokeProps } from './baseProps';
import { Point2D, ShapeCreator } from './types';

export const RectCreator: ShapeCreator<Rect> = {
  create(origin: Point2D): Rect {
    const base = getBaseStrokeProps();
    const r = cornerRadius();

    return new Rect({
      left: origin.x,
      top: origin.y,
      originX: 'left',
      originY: 'top',
      width: 0,
      height: 0,
      rx: r,
      ry: r,
      selectable: true,
      ...base,
    } as any);
  },

  update(rect: Rect, origin: Point2D, pointer: Point2D) {
    const currentX = snapCoord(pointer.x);
    const currentY = snapCoord(pointer.y);

    const left = Math.min(origin.x, currentX);
    const top = Math.min(origin.y, currentY);
    const rawW = Math.abs(currentX - origin.x);
    const rawH = Math.abs(currentY - origin.y);

    rect.set({
      left,
      top,
      width: snapToGrid() ? Math.max(GRID_SIZE, Math.round(rawW / GRID_SIZE) * GRID_SIZE) : rawW,
      height: snapToGrid() ? Math.max(GRID_SIZE, Math.round(rawH / GRID_SIZE) * GRID_SIZE) : rawH,
    });
  },

  isTiny(rect: Rect): boolean {
    return (rect.width ?? 0) < 3 && (rect.height ?? 0) < 3;
  },
};
