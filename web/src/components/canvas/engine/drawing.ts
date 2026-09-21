import {
  FabricObject,
  Line,
  IText,
  Group,
  Canvas,
  Point,
  Control,
  util,
  controlsUtils,
} from 'fabric';
import { snapToGrid, GRID_SIZE } from '../canvasStore';
import { StrokeAlign } from '../types';

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

// ── Professional Line & Arrow Endpoint Controls ──
export function createLineControls() {
  return {
    p1: new Control({
      x: 0,
      y: 0,
      positionHandler(dim, finalMatrix, fabricItem) {
        const points = (fabricItem as Line).calcLinePoints();
        return util.transformPoint(new Point(points.x1, points.y1), fabricItem.calcTransformMatrix());
      },
      actionHandler(eventData, transform, x, y) {
        const line = transform.target as Line;
        const mat = line.calcTransformMatrix();
        const p2Local = new Point(line.calcLinePoints().x2, line.calcLinePoints().y2);
        const p2Scene = util.transformPoint(p2Local, mat);

        let targetX = snapCoord(x);
        let targetY = snapCoord(y);

        if (eventData?.shiftKey) {
          const dx = targetX - p2Scene.x;
          const dy = targetY - p2Scene.y;
          const dist = Math.hypot(dx, dy);
          const rawAngle = Math.atan2(dy, dx);
          const snapAngle = Math.round(rawAngle / (Math.PI / 4)) * (Math.PI / 4);
          targetX = p2Scene.x + dist * Math.cos(snapAngle);
          targetY = p2Scene.y + dist * Math.sin(snapAngle);
        }

        line.set({
          x1: targetX,
          y1: targetY,
          x2: p2Scene.x,
          y2: p2Scene.y,
          left: (targetX + p2Scene.x) / 2,
          top: (targetY + p2Scene.y) / 2,
          scaleX: 1,
          scaleY: 1,
          angle: 0,
        });
        line.setCoords();
        return true;
      },
      cursorStyleHandler: () => 'crosshair',
      actionName: 'modifyLineStart',
      render: controlsUtils.renderCircleControl,
    }),
    p2: new Control({
      x: 0,
      y: 0,
      positionHandler(dim, finalMatrix, fabricItem) {
        const points = (fabricItem as Line).calcLinePoints();
        return util.transformPoint(new Point(points.x2, points.y2), fabricItem.calcTransformMatrix());
      },
      actionHandler(eventData, transform, x, y) {
        const line = transform.target as Line;
        const mat = line.calcTransformMatrix();
        const p1Local = new Point(line.calcLinePoints().x1, line.calcLinePoints().y1);
        const p1Scene = util.transformPoint(p1Local, mat);

        let targetX = snapCoord(x);
        let targetY = snapCoord(y);

        if (eventData?.shiftKey) {
          const dx = targetX - p1Scene.x;
          const dy = targetY - p1Scene.y;
          const dist = Math.hypot(dx, dy);
          const rawAngle = Math.atan2(dy, dx);
          const snapAngle = Math.round(rawAngle / (Math.PI / 4)) * (Math.PI / 4);
          targetX = p1Scene.x + dist * Math.cos(snapAngle);
          targetY = p1Scene.y + dist * Math.sin(snapAngle);
        }

        line.set({
          x1: p1Scene.x,
          y1: p1Scene.y,
          x2: targetX,
          y2: targetY,
          left: (p1Scene.x + targetX) / 2,
          top: (p1Scene.y + targetY) / 2,
          scaleX: 1,
          scaleY: 1,
          angle: 0,
        });
        line.setCoords();
        return true;
      },
      cursorStyleHandler: () => 'crosshair',
      actionName: 'modifyLineEnd',
      render: controlsUtils.renderCircleControl,
    }),
  };
}

export function setupLineControls(line: Line) {
  if (!line) return;
  line.controls = createLineControls();
  line.hasBorders = false;
  line.padding = 6;
}

function renderArrowHead(
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
  if (this.strokeDashArray) {
    ctx.setLineDash(this.strokeDashArray);
  }
  ctx.strokeStyle = this.stroke || '#7BDFF2';
  ctx.stroke();
  ctx.restore();

  if (hasArrowEnd) {
    renderArrowHead(ctx, p.x2, p.y2, angle, headLength, arrowSpread, this.stroke || '#7BDFF2');
  }

  if (hasArrowStart) {
    renderArrowHead(ctx, p.x1, p.y1, angle + Math.PI, headLength, arrowSpread, this.stroke || '#7BDFF2');
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
    stroke: stroke || '#7BDFF2',
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
      backgroundColor: 'rgba(16, 20, 29, 0.85)',
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

// ── Native Stroke Alignment Engine (Center, Inner, Outer) ──
const origRenderPaintInOrder = (FabricObject.prototype as any)._renderPaintInOrder;
const origGetTransformedDimensions = (FabricObject.prototype as any)._getTransformedDimensions;
const origGetNonTransformedDimensions = (FabricObject.prototype as any)._getNonTransformedDimensions;
const origNeedsItsOwnCache = (FabricObject.prototype as any).needsItsOwnCache;
const origToObject = (FabricObject.prototype as any).toObject;

if (!(FabricObject.prototype as any).__strokeAlignPatched) {
  (FabricObject.prototype as any).__strokeAlignPatched = true;

  // 1. Pastikan serialisasi toObject selalu menyertakan strokeAlign & arrow properties untuk History Undo/Redo
  (FabricObject.prototype as any).toObject = function (propertiesToInclude: string[] = []) {
    return origToObject.call(this, [
      'strokeAlign',
      'arrowEnd',
      'arrowStart',
      'isArrow',
      'strokeDashArray',
      'connectorData',
      'connectorType',
      'isConnectorLine',
      ...propertiesToInclude,
    ]);
  };

  // 2. Pastikan outer stroke selalu menggunakan cache canvas pribadi agar destination-out tidak bocor ke canvas utama
  (FabricObject.prototype as any).needsItsOwnCache = function () {
    if (this.strokeAlign === 'outer') return true;
    return origNeedsItsOwnCache.call(this);
  };

  // 3. Hitung dimensi geometris & bounding box akurat untuk seleksi dan koordinat transform
  (FabricObject.prototype as any)._getTransformedDimensions = function (options?: any) {
    const align: StrokeAlign = this.strokeAlign || 'center';
    const hasStroke = !!(
      this.stroke &&
      typeof this.strokeWidth === 'number' &&
      this.stroke !== 'transparent' &&
      this.strokeWidth > 0
    );

    if (
      !hasStroke ||
      align === 'center' ||
      this.type === 'line' ||
      this.type === 'arrow' ||
      this.type === 'text' ||
      this.type === 'i-text'
    ) {
      return origGetTransformedDimensions.call(this, options);
    }

    // Inner: border di dalam bentuk, tidak menambah dimensi luar
    // Outer: border di luar bentuk sebesar strokeWidth di tiap sisi (total 2x strokeWidth)
    const effectiveStrokeWidth = align === 'inner' ? 0 : this.strokeWidth * 2;
    return origGetTransformedDimensions.call(this, {
      ...options,
      strokeWidth: effectiveStrokeWidth,
    });
  };

  (FabricObject.prototype as any)._getNonTransformedDimensions = function () {
    const align: StrokeAlign = this.strokeAlign || 'center';
    const hasStroke = !!(
      this.stroke &&
      typeof this.strokeWidth === 'number' &&
      this.stroke !== 'transparent' &&
      this.strokeWidth > 0
    );

    if (
      !hasStroke ||
      align === 'center' ||
      this.type === 'line' ||
      this.type === 'arrow' ||
      this.type === 'text' ||
      this.type === 'i-text'
    ) {
      return origGetNonTransformedDimensions.call(this);
    }

    const effectiveStrokeWidth = align === 'inner' ? 0 : this.strokeWidth * 2;
    return origGetTransformedDimensions.call(this, {
      strokeWidth: effectiveStrokeWidth,
      scaleX: 1,
      scaleY: 1,
      skewX: 0,
      skewY: 0,
    });
  };

  // 4. Render Engine untuk Center, Inner, dan Outer
  (FabricObject.prototype as any)._renderPaintInOrder = function (ctx: CanvasRenderingContext2D) {
    const align: StrokeAlign = this.strokeAlign || 'center';
    const hasStroke = !!(
      this.stroke &&
      typeof this.strokeWidth === 'number' &&
      this.stroke !== 'transparent' &&
      this.strokeWidth > 0
    );

    if (
      !hasStroke ||
      align === 'center' ||
      this.type === 'line' ||
      this.type === 'arrow' ||
      this.type === 'text' ||
      this.type === 'i-text'
    ) {
      origRenderPaintInOrder.call(this, ctx);
      return;
    }

    const origWidth = this.strokeWidth;

    if (align === 'inner') {
      // 1. Gambar Fill
      this._renderFill(ctx);

      // 2. Gambar Stroke di-clip ke area dalam bentuk
      ctx.save();
      ctx.clip();
      this.strokeWidth = origWidth * 2;
      try {
        this._renderStroke(ctx);
      } finally {
        this.strokeWidth = origWidth;
        ctx.restore();
      }
      return;
    }

    if (align === 'outer') {
      // 1. Gambar Stroke 2x lipat
      ctx.save();
      this.strokeWidth = origWidth * 2;
      try {
        this._renderStroke(ctx);
      } finally {
        this.strokeWidth = origWidth;
      }

      // 2. Hapus bagian dalam stroke agar tidak masuk ke isi bentuk
      ctx.save();
      ctx.globalCompositeOperation = 'destination-out';
      ctx.fillStyle = '#000000';
      ctx.fill();
      ctx.restore();
      ctx.restore();

      // 3. Gambar Fill di atasnya
      this._renderFill(ctx);
      return;
    }

    origRenderPaintInOrder.call(this, ctx);
  };
}

export function syncShapeClipPath(obj: any) {
  if (obj && obj.clipPath) {
    obj.clipPath = undefined;
  }
}

export function applyShapeStrokeAlignment(
  obj: any,
  align: StrokeAlign,
  baseWidth: number
) {
  if (!obj) return;

  const isClosedShape =
    obj.type !== 'line' &&
    obj.type !== 'arrow' &&
    obj.type !== 'text' &&
    obj.type !== 'i-text';

  const effectiveAlign: StrokeAlign = isClosedShape ? (align || 'center') : 'center';
  obj.strokeAlign = effectiveAlign;
  obj.clipPath = undefined;

  if (baseWidth <= 0 || !obj.stroke || obj.stroke === 'transparent') {
    obj.set({
      strokeWidth: 0,
      stroke: '',
      clipPath: undefined,
    });
  } else {
    obj.set({
      strokeWidth: baseWidth,
      clipPath: undefined,
    });
  }

  // Bersihkan cache canvas lama agar Fabric membuat cache baru dengan dimensi alignment yang baru
  if (obj._cacheCanvas) {
    obj._cacheCanvas = null;
    obj._cacheContext = null;
  }
  obj.dirty = true;
  obj.setCoords();
}
