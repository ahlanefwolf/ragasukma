import { Canvas } from 'fabric';
import { snapToGrid, GRID_SIZE } from '../../canvasStore';

export function snapCoord(val: number): number {
  if (!snapToGrid()) return val;
  return Math.round(val / GRID_SIZE) * GRID_SIZE;
}

export function getPointerCoords(canvas: Canvas | null, e: any, o?: any): { x: number; y: number } {
  if (o && o.scenePoint) {
    return { x: o.scenePoint.x, y: o.scenePoint.y };
  }
  if (canvas) {
    if (typeof (canvas as any).getScenePoint === 'function') {
      const p = (canvas as any).getScenePoint(e);
      return { x: p.x, y: p.y };
    }
    if (typeof (canvas as any).getPointer === 'function') {
      const p = (canvas as any).getPointer(e);
      return { x: p.x, y: p.y };
    }
  }
  return { x: 0, y: 0 };
}
