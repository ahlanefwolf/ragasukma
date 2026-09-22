import { Control, Line, Point, util } from 'fabric';

/**
 * Custom control circle renderer yang ukuran radiusnya konstan di layar (viewport-independent).
 * Tidak membesar saat di-zoom in dan tidak mengecil/hilang saat di-zoom out.
 * Fabric v6 me-render controls di screen coordinate context yang diset sesuai retina scaling.
 * Parameter left dan top adalah posisi titik koordinat kontrol di layar.
 */
export function renderZoomAdaptiveCircle(
  ctx: CanvasRenderingContext2D,
  left: number,
  top: number,
  styleOverride: any,
  fabricObject: any
) {
  const r = 6; // ukuran radius konstan 6px di layar

  ctx.save();
  ctx.beginPath();
  ctx.arc(left, top, r, 0, Math.PI * 2, false);
  ctx.fillStyle = '#FFFFFF';
  ctx.fill();
  ctx.lineWidth = 2;
  ctx.strokeStyle = '#7BDFF2';
  ctx.stroke();
  ctx.restore();
}

/**
 * Helper untuk mendapatkan posisi screen koordinat handle kontrol p1 / p2.
 * Di Fabric v6, positionHandler mengembalikan Point di layar (screen coordinate).
 * Dengan mengikatkan koordinat absolut (x1, y1) atau (x2, y2) melalui getViewportTransform(),
 * lingkaran kontrol p1 dan p2 100% TERKUNCI PERSIS di titik start & end garis,
 * stabil tanpa bergeser 1 pixel pun pada level zoom / pan berapapun!
 */
export function getLineEndpointPosition(
  fabricItem: Line,
  pointKey: 'p1' | 'p2'
): Point {
  const line = fabricItem as any;
  const vpt = fabricItem.getViewportTransform
    ? fabricItem.getViewportTransform()
    : fabricItem.canvas?.viewportTransform || [1, 0, 0, 1, 0, 0];

  const targetPt = pointKey === 'p1'
    ? new Point(line.x1 ?? 0, line.y1 ?? 0)
    : new Point(line.x2 ?? 0, line.y2 ?? 0);

  return targetPt.transform(vpt);
}

/**
 * Helper untuk mendapatkan posisi scene/canvas aktual dari titik start (x1,y1) dan end (x2,y2).
 * Terkunci langsung pada nilai scene x1/y1 atau x2/y2 milik garis.
 */
export function getLineEndpointScenePosition(
  fabricItem: Line,
  pointKey: 'p1' | 'p2'
): Point {
  const line = fabricItem as any;
  if (pointKey === 'p1') {
    return new Point(line.x1 ?? 0, line.y1 ?? 0);
  }
  return new Point(line.x2 ?? 0, line.y2 ?? 0);
}
