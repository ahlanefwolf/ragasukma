import { Canvas } from 'fabric';
import { snapToGrid, GRID_SIZE } from '../../../canvasStore';
import { updateConnectedLines } from '../../connector';
import { syncShapeClipPath } from '../../drawing';
import { LayersEngine } from '../../layers';
import { HistoryManager } from '../history';

export function bindObjectTransforms(
  canvas: Canvas,
  history: HistoryManager,
  isMouseDown: () => boolean
) {
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
}
