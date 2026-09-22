import {
  FabricObject,
  Line,
  IText,
  Group,
} from 'fabric';
import { setupLineControls } from './controls';

export function renderArrowHead(
  ctx: CanvasRenderingContext2D,
  tipX: number,
  tipY: number,
  angle: number,
  headLength: number,
  arrowSpread: number,
  color: string
) {
  const leftX = tipX - headLength * Math.cos(angle - arrowSpread);
  const leftY = tipY - headLength * Math.sin(angle - arrowSpread);
  const rightX = tipX - headLength * Math.cos(angle + arrowSpread);
  const rightY = tipY - headLength * Math.sin(angle + arrowSpread);
  const notchX = tipX - headLength * 0.8 * Math.cos(angle);
  const notchY = tipY - headLength * 0.8 * Math.sin(angle);

  ctx.save();
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.moveTo(tipX, tipY);
  ctx.lineTo(leftX, leftY);
  ctx.lineTo(notchX, notchY);
  ctx.lineTo(rightX, rightY);
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

const origLineRender = Line.prototype._render;

Line.prototype._render = function (ctx: CanvasRenderingContext2D) {
  const hasArrowEnd = !!((this as any).arrowEnd || (this as any).isArrow);
  const hasArrowStart = !!(this as any).arrowStart;
  const connData = (this as any).connectorData;
  const computedPoints: Array<{ x: number; y: number }> | undefined = connData?.computedPoints;

  // Jika garis memiliki data rute (orthogonal, curved, atau straight)
  if (computedPoints && computedPoints.length >= 2) {
    const strokeW = Math.max(1, this.strokeWidth || 2);
    const headLength = Math.max(12, Math.min(36, strokeW * 3.6));
    const arrowSpread = Math.PI / 6.5;

    // Di Fabric.js v6: calcLinePoints() mengembalikan koordinat relatif terhadap center ({ x1: -w/2, ... }).
    // Gunakan midpoint global dari properti garis x1, y1, x2, y2 asli:
    const lineX1 = (this as any).x1;
    const lineY1 = (this as any).y1;
    const lineX2 = (this as any).x2;
    const lineY2 = (this as any).y2;

    const realMidGlobalX = (lineX1 + lineX2) / 2;
    const realMidGlobalY = (lineY1 + lineY2) / 2;

    const localPoints = computedPoints.map((pt) => ({
      x: pt.x - realMidGlobalX,
      y: pt.y - realMidGlobalY,
    }));

    const firstPt = localPoints[0];
    const lastPt = localPoints[localPoints.length - 1];

    const firstSegment = {
      dx: localPoints[1].x - firstPt.x,
      dy: localPoints[1].y - firstPt.y,
    };
    const lastSegment = {
      dx: lastPt.x - localPoints[localPoints.length - 2].x,
      dy: lastPt.y - localPoints[localPoints.length - 2].y,
    };

    const firstAngle = Math.atan2(firstSegment.dy, firstSegment.dx);
    const lastAngle = Math.atan2(lastSegment.dy, lastSegment.dx);

    const lineStartX = hasArrowStart ? firstPt.x + headLength * 0.75 * Math.cos(firstAngle) : firstPt.x;
    const lineStartY = hasArrowStart ? firstPt.y + headLength * 0.75 * Math.sin(firstAngle) : firstPt.y;
    const lineEndX = hasArrowEnd ? lastPt.x - headLength * 0.75 * Math.cos(lastAngle) : lastPt.x;
    const lineEndY = hasArrowEnd ? lastPt.y - headLength * 0.75 * Math.sin(lastAngle) : lastPt.y;

    ctx.save();
    ctx.beginPath();
    ctx.moveTo(lineStartX, lineStartY);
    for (let i = 1; i < localPoints.length - 1; i++) {
      ctx.lineTo(localPoints[i].x, localPoints[i].y);
    }
    ctx.lineTo(lineEndX, lineEndY);
    ctx.lineWidth = strokeW;
    ctx.lineCap = this.strokeLineCap || 'round';
    ctx.lineJoin = this.strokeLineJoin || 'round';
    if (this.strokeDashArray && this.strokeDashArray.length > 0) {
      ctx.setLineDash(this.strokeDashArray);
    } else {
      ctx.setLineDash([]);
    }
    const strokeColor: string = typeof (this as any).stroke === 'string' && (this as any).stroke ? (this as any).stroke : '#29483A';
    (ctx as any).strokeStyle = strokeColor;
    ctx.stroke();
    ctx.restore();

    if (hasArrowEnd) {
      renderArrowHead(ctx, lastPt.x, lastPt.y, lastAngle, headLength, arrowSpread, strokeColor);
    }

    if (hasArrowStart) {
      renderArrowHead(ctx, firstPt.x, firstPt.y, firstAngle + Math.PI, headLength, arrowSpread, strokeColor);
    }
    return;
  }

  if (!hasArrowEnd && !hasArrowStart) {
    origLineRender.call(this, ctx);
    return;
  }

  const p = this.calcLinePoints();
  const dx = p.x2 - p.x1;
  const dy = p.y2 - p.y1;
  const length = Math.hypot(dx, dy);
  if (length === 0) return;

  const angle = Math.atan2(dy, dx);
  const strokeW = Math.max(1, this.strokeWidth || 2);
  const headLength = Math.max(12, Math.min(36, strokeW * 3.6));
  const arrowSpread = Math.PI / 6.5;

  const lineStartX = hasArrowStart ? p.x1 + headLength * 0.75 * Math.cos(angle) : p.x1;
  const lineStartY = hasArrowStart ? p.y1 + headLength * 0.75 * Math.sin(angle) : p.y1;
  const lineEndX = hasArrowEnd ? p.x2 - headLength * 0.75 * Math.cos(angle) : p.x2;
  const lineEndY = hasArrowEnd ? p.y2 - headLength * 0.75 * Math.sin(angle) : p.y2;

  ctx.save();
  ctx.beginPath();
  ctx.moveTo(lineStartX, lineStartY);
  ctx.lineTo(lineEndX, lineEndY);
  ctx.lineWidth = strokeW;
  ctx.lineCap = this.strokeLineCap || 'round';
  ctx.lineJoin = this.strokeLineJoin || 'round';
  if (this.strokeDashArray && this.strokeDashArray.length > 0) {
    ctx.setLineDash(this.strokeDashArray);
  } else {
    ctx.setLineDash([]);
  }
  const strokeColor: string = typeof (this as any).stroke === 'string' && (this as any).stroke ? (this as any).stroke : '#29483A';
  (ctx as any).strokeStyle = strokeColor;
  ctx.stroke();
  ctx.restore();

  if (hasArrowEnd) {
    renderArrowHead(ctx, p.x2, p.y2, angle, headLength, arrowSpread, strokeColor);
  }

  if (hasArrowStart) {
    renderArrowHead(ctx, p.x1, p.y1, angle + Math.PI, headLength, arrowSpread, strokeColor);
  }
};

export function createArrowObject(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  stroke: string,
  strokeWidth: number,
  label?: string
): FabricObject {
  const line = new Line([x1, y1, x2, y2], {
    stroke: stroke || '#29483A',
    strokeWidth: strokeWidth || 3,
    strokeUniform: true,
    strokeLineCap: 'round',
    strokeLineJoin: 'round',
    selectable: true,
  } as any);
  (line as any).isArrow = true;
  (line as any).arrowEnd = true;
  setupLineControls(line);

  if (label) {
    const midX = (x1 + x2) / 2;
    const midY = (y1 + y2) / 2 - 14;
    const text = new IText(label, {
      left: midX,
      top: midY,
      fontSize: 13,
      fill: stroke,
      originX: 'center',
      originY: 'center',
      backgroundColor: 'rgba(245, 240, 230, 0.9)',
    });
    return new Group([line, text], { selectable: true });
  }

  return line;
}

export function createArrowPath(
  points: Array<{ x: number; y: number }>,
  stroke: string,
  strokeWidth: number,
  label?: string
): FabricObject {
  if (points.length < 2) {
    return createArrowObject(0, 0, 100, 0, stroke, strokeWidth, label);
  }
  const p1 = points[0];
  const p2 = points[points.length - 1];
  return createArrowObject(p1.x, p1.y, p2.x, p2.y, stroke, strokeWidth, label);
}
