import { Canvas, Point } from 'fabric';
import { setZoomLevel } from '../../canvasStore';

export class ViewportManager {
  public isPanning = false;
  public isSpacePressed = false;
  public lastPosX = 0;
  public lastPosY = 0;

  public handleWheel(canvas: Canvas | null, opt: any) {
    if (!canvas) return;
    const e = opt.e;
    const delta = e.deltaY;
    let zoom = canvas.getZoom();
    zoom *= 0.999 ** delta;
    if (zoom > 5) zoom = 5;
    if (zoom < 0.2) zoom = 0.2;

    const point = new Point(e.offsetX, e.offsetY);
    canvas.zoomToPoint(point, zoom);
    setZoomLevel(Math.round(zoom * 100));
    canvas.requestRenderAll();

    e.preventDefault();
    e.stopPropagation();
  }

  public zoomIn(canvas: Canvas | null) {
    if (!canvas) return;
    let zoom = canvas.getZoom() * 1.2;
    if (zoom > 5) zoom = 5;
    const center = new Point(canvas.getWidth() / 2, canvas.getHeight() / 2);
    canvas.zoomToPoint(center, zoom);
    setZoomLevel(Math.round(zoom * 100));
    canvas.requestRenderAll();
  }

  public zoomOut(canvas: Canvas | null) {
    if (!canvas) return;
    let zoom = canvas.getZoom() * 0.8;
    if (zoom < 0.2) zoom = 0.2;
    const center = new Point(canvas.getWidth() / 2, canvas.getHeight() / 2);
    canvas.zoomToPoint(center, zoom);
    setZoomLevel(Math.round(zoom * 100));
    canvas.requestRenderAll();
  }

  public resetZoom(canvas: Canvas | null) {
    if (!canvas) return;
    canvas.setViewportTransform([1, 0, 0, 1, 0, 0]);
    canvas.setZoom(1);
    setZoomLevel(100);
    canvas.requestRenderAll();
  }

  // ── Pan Handlers ──

  public startPan(canvas: Canvas, e: any): boolean {
    if (this.isSpacePressed || (e && e.button === 1)) {
      this.isPanning = true;
      this.lastPosX = e.clientX;
      this.lastPosY = e.clientY;
      canvas.defaultCursor = 'grabbing';
      return true;
    }
    return false;
  }

  public pan(canvas: Canvas, e: any): boolean {
    if (!this.isPanning) return false;
    const vpt = canvas.viewportTransform;
    if (vpt) {
      vpt[4] += e.clientX - this.lastPosX;
      vpt[5] += e.clientY - this.lastPosY;
      canvas.requestRenderAll();
      this.lastPosX = e.clientX;
      this.lastPosY = e.clientY;
    }
    return true;
  }

  public endPan(canvas: Canvas, defaultCursor = 'crosshair'): boolean {
    if (!this.isPanning) return false;
    this.isPanning = false;
    canvas.defaultCursor = this.isSpacePressed ? 'grab' : defaultCursor;
    return true;
  }
}
