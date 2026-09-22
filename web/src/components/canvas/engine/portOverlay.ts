import { Canvas, FabricObject } from 'fabric';
import { ConnectionPort, getObjectConnectionPorts } from './connector';

/**
 * PortOverlayEngine mengelola lingkaran bantu magnetik di sudut/sisi objek.
 * Lingkaran bantu ini digambar langsung di atas render loop context (overlay)
 * atau dikelola secara terisolasi tanpa pernah masuk ke dalam state history JSON (excludeFromExport: true).
 */
export class PortOverlayEngine {
  private static activePort: ConnectionPort | null = null;
  private static activeTargetObj: FabricObject | null = null;
  private static isBound = false;

  public static bindCanvasOverlay(canvas: Canvas) {
    if ((canvas as any).__portOverlayBound) return;
    (canvas as any).__portOverlayBound = true;

    // Gambar lingkaran bantu di atas canvas secara native via 'after:render'
    // Tanpa pernah membuat objek Fabric baru yang mengotori canvas.getObjects() atau history toJSON()!
    canvas.on('after:render', (opt: any) => {
      if (!this.activeTargetObj && !this.activePort) return;
      const ctx = opt.ctx || canvas.getContext();
      if (!ctx) return;

      const objectsToHighlight: FabricObject[] = this.activeTargetObj
        ? [this.activeTargetObj]
        : canvas.getObjects().filter((o: any) => !o.isConnectionIndicator && o.type !== 'line');

      ctx.save();
      const vpt = canvas.viewportTransform || [1, 0, 0, 1, 0, 0];
      ctx.transform(vpt[0], vpt[1], vpt[2], vpt[3], vpt[4], vpt[5]);

      objectsToHighlight.forEach((obj) => {
        const ports = getObjectConnectionPorts(obj);
        ports.forEach((p) => {
          const isCurrentActive =
            this.activePort &&
            Math.hypot(this.activePort.x - p.x, this.activePort.y - p.y) < 6;

          ctx.beginPath();
          ctx.arc(p.x, p.y, isCurrentActive ? 6 : 4, 0, Math.PI * 2);
          ctx.fillStyle = isCurrentActive ? '#596B45' : 'rgba(89, 107, 69, 0.45)';
          ctx.fill();
          ctx.lineWidth = isCurrentActive ? 2 : 1;
          ctx.strokeStyle = isCurrentActive ? '#FFFFFF' : '#596B45';
          ctx.stroke();
        });
      });

      ctx.restore();
    });
  }

  public static showIndicators(
    canvas: Canvas,
    activePort?: ConnectionPort | null,
    targetObj?: FabricObject | null
  ) {
    this.bindCanvasOverlay(canvas);
    this.activePort = activePort || null;
    this.activeTargetObj = targetObj || null;
    canvas.requestRenderAll();
  }

  public static clearIndicators(canvas: Canvas) {
    if (this.activePort || this.activeTargetObj) {
      this.activePort = null;
      this.activeTargetObj = null;
      canvas.requestRenderAll();
    }
  }
}

import { findClosestConnectionPort } from './connector';

/**
 * Mencari anchor port terdekat dan langsung mengaktifkan indikator visual overlay.
 */
export function snapToPort(
  canvas: Canvas,
  pointer: { x: number; y: number },
  excludeObj?: FabricObject | null,
  maxDistance = 24
): { port: ConnectionPort; obj: FabricObject } | null {
  const snap = findClosestConnectionPort(canvas, pointer, excludeObj, maxDistance);
  if (snap) {
    PortOverlayEngine.showIndicators(canvas, snap.port, snap.obj);
  }
  return snap;
}

export function clearPortIndicators(canvas: Canvas) {
  PortOverlayEngine.clearIndicators(canvas);
}
