import { Circle } from 'fabric';
import { snapToGrid, GRID_SIZE } from '../../canvasStore';
import { snapCoord } from '../drawing';
import { getBaseStrokeProps } from './baseProps';
import { Point2D, ShapeCreator } from './types';

export const CircleCreator: ShapeCreator<Circle> = {
  create(origin: Point2D): Circle {
    const base = getBaseStrokeProps();

    return new Circle({
      left: origin.x,
      top: origin.y,
      originX: 'center',
      originY: 'center',
      radius: 0,
      selectable: true,
      ...base,
    } as any);
  },

  update(circle: Circle, origin: Point2D, pointer: Point2D) {
    const currentX = snapCoord(pointer.x);
    const currentY = snapCoord(pointer.y);
    const rawRadius =
      Math.sqrt(
        Math.pow(currentX - origin.x, 2) + Math.pow(currentY - origin.y, 2)
      ) / 2;
    const radius = snapToGrid()
      ? Math.max(GRID_SIZE / 2, Math.round((rawRadius * 2) / GRID_SIZE) * (GRID_SIZE / 2))
      : rawRadius;

    circle.set({
      radius,
      left: (origin.x + currentX) / 2,
      top: (origin.y + currentY) / 2,
    });
  },

  isTiny(circle: Circle): boolean {
    return (circle.radius ?? 0) < 2;
  },
};
