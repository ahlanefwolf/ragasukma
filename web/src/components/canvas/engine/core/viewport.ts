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
  }

  public zoomOut(canvas: Canvas | null) {
    if (!canvas) return;
    let zoom = canvas.getZoom() * 0.8;
    if (zoom < 0.2) zoom = 0.2;
    const center = new Point(canvas.getWidth() / 2, canvas.getHeight() / 2);
    canvas.zoomToPoint(center, zoom);
    setZoomLevel(Math.round(zoom * 100));
  }

  public resetZoom(canvas: Canvas | null) {
    if (!canvas) return;
    canvas.setViewportTransform([1, 0, 0, 1, 0, 0]);
    canvas.setZoom(1);
    setZoomLevel(100);
  }
}
