import { FabricObject, Line, Point } from 'fabric';
import { StrokeAlign } from '../../types';

// ── Native Stroke Alignment Engine (Center, Inner, Outer) ──
const origRenderPaintInOrder = (FabricObject.prototype as any)._renderPaintInOrder;
const origGetTransformedDimensions = (FabricObject.prototype as any)._getTransformedDimensions;
const origGetNonTransformedDimensions = (FabricObject.prototype as any)._getNonTransformedDimensions;
const origNeedsItsOwnCache = (FabricObject.prototype as any).needsItsOwnCache;
const origToObject = (FabricObject.prototype as any).toObject;
const origLineToObject = (Line.prototype as any).toObject;
const origLineToSVG = (Line.prototype as any)._toSVG;

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
      'id',
      'name',
      'locked',
      'rx',
      'ry',
      ...propertiesToInclude,
    ]);
  };

  // Line memiliki prototype toObject sendiri di Fabric.js v6 yang meng-override FabricObject
  (Line.prototype as any).toObject = function (propertiesToInclude: string[] = []) {
    return origLineToObject.call(this, [
      'strokeAlign',
      'arrowEnd',
      'arrowStart',
      'isArrow',
      'strokeDashArray',
      'connectorData',
      'connectorType',
      'isConnectorLine',
      'id',
      'name',
      'locked',
      ...propertiesToInclude,
    ]);
  };

  // Pastikan ekspor SVG juga merender waypoints lengkung/siku (bukan garis lurus x1-x2 biasa)
  (Line.prototype as any)._toSVG = function (reviver?: any) {
    const connData = (this as any).connectorData;
    const computedPoints = connData?.computedPoints;
    if (computedPoints && computedPoints.length >= 2) {
      const lineX1 = (this as any).x1;
      const lineY1 = (this as any).y1;
      const lineX2 = (this as any).x2;
      const lineY2 = (this as any).y2;
      const realMidGlobalX = (lineX1 + lineX2) / 2;
      const realMidGlobalY = (lineY1 + lineY2) / 2;
      const localPoints = computedPoints.map((pt: any) => ({
        x: pt.x - realMidGlobalX,
        y: pt.y - realMidGlobalY,
      }));
      let pathData = `M ${localPoints[0].x} ${localPoints[0].y}`;
      for (let i = 1; i < localPoints.length; i++) {
        pathData += ` L ${localPoints[i].x} ${localPoints[i].y}`;
      }

      // Render arrowhead jika ada arrowEnd atau arrowStart
      const hasArrowEnd = !!((this as any).arrowEnd || (this as any).isArrow);
      const hasArrowStart = !!(this as any).arrowStart;
      let arrowsSvg = '';
      const strokeW = Math.max(1, this.strokeWidth || 2);
      const headLength = Math.max(12, Math.min(36, strokeW * 3.6));
      const arrowSpread = Math.PI / 6.5;

      if (hasArrowEnd && localPoints.length >= 2) {
        const lastPt = localPoints[localPoints.length - 1];
        const prevPt = localPoints[localPoints.length - 2];
        const lastAngle = Math.atan2(lastPt.y - prevPt.y, lastPt.x - prevPt.x);
        const leftX = lastPt.x - headLength * Math.cos(lastAngle - arrowSpread);
        const leftY = lastPt.y - headLength * Math.sin(lastAngle - arrowSpread);
        const rightX = lastPt.x - headLength * Math.cos(lastAngle + arrowSpread);
        const rightY = lastPt.y - headLength * Math.sin(lastAngle + arrowSpread);
        arrowsSvg += `<polygon points="${lastPt.x},${lastPt.y} ${leftX},${leftY} ${rightX},${rightY}" fill="${this.stroke || '#596B45'}" />`;
      }

      if (hasArrowStart && localPoints.length >= 2) {
        const firstPt = localPoints[0];
        const nextPt = localPoints[1];
        const firstAngle = Math.atan2(nextPt.y - firstPt.y, nextPt.x - firstPt.x) + Math.PI;
        const leftX = firstPt.x - headLength * Math.cos(firstAngle - arrowSpread);
        const leftY = firstPt.y - headLength * Math.sin(firstAngle - arrowSpread);
        const rightX = firstPt.x - headLength * Math.cos(firstAngle + arrowSpread);
        const rightY = firstPt.y - headLength * Math.sin(firstAngle + arrowSpread);
        arrowsSvg += `<polygon points="${firstPt.x},${firstPt.y} ${leftX},${leftY} ${rightX},${rightY}" fill="${this.stroke || '#596B45'}" />`;
      }

      // Menggunakan COMMON_PARTS bawaan Fabric (yang sudah memuat style="... fill: none; ...")
      return [`<g>`, `<path `, `COMMON_PARTS`, `d="${pathData}" />\n`, arrowsSvg, `</g>\n`];
    }

    // Untuk garis lurus biasa tanpa connector waypoints tapi memiliki arrowhead
    const hasArrowEnd = !!((this as any).arrowEnd || (this as any).isArrow);
    const hasArrowStart = !!(this as any).arrowStart;
    if (hasArrowEnd || hasArrowStart) {
      const p = this.calcLinePoints();
      const dx = p.x2 - p.x1;
      const dy = p.y2 - p.y1;
      const angle = Math.atan2(dy, dx);
      const strokeW = Math.max(1, this.strokeWidth || 2);
      const headLength = Math.max(12, Math.min(36, strokeW * 3.6));
      const arrowSpread = Math.PI / 6.5;

      let arrowsSvg = '';
      if (hasArrowEnd) {
        const leftX = p.x2 - headLength * Math.cos(angle - arrowSpread);
        const leftY = p.y2 - headLength * Math.sin(angle - arrowSpread);
        const rightX = p.x2 - headLength * Math.cos(angle + arrowSpread);
        const rightY = p.y2 - headLength * Math.sin(angle + arrowSpread);
        arrowsSvg += `<polygon points="${p.x2},${p.y2} ${leftX},${leftY} ${rightX},${rightY}" fill="${this.stroke || '#596B45'}" />`;
      }
      if (hasArrowStart) {
        const revAngle = angle + Math.PI;
        const leftX = p.x1 - headLength * Math.cos(revAngle - arrowSpread);
        const leftY = p.y1 - headLength * Math.sin(revAngle - arrowSpread);
        const rightX = p.x1 - headLength * Math.cos(revAngle + arrowSpread);
        const rightY = p.y1 - headLength * Math.sin(revAngle + arrowSpread);
        arrowsSvg += `<polygon points="${p.x1},${p.y1} ${leftX},${leftY} ${rightX},${rightY}" fill="${this.stroke || '#596B45'}" />`;
      }

      const baseSvg = origLineToSVG.call(this, reviver);
      return [`<g>`, ...baseSvg, arrowsSvg, `</g>\n`];
    }

    return origLineToSVG.call(this, reviver);
  };

  // 2. Pastikan outer stroke selalu menggunakan cache canvas pribadi agar destination-out tidak bocor ke canvas utama
  (FabricObject.prototype as any).needsItsOwnCache = function () {
    if (this.type === 'line' || this.isArrow) return false;
    if (this.strokeAlign === 'outer') return true;
    return origNeedsItsOwnCache.call(this);
  };

  // 3. Hitung dimensi geometris & bounding box akurat untuk seleksi dan koordinat transform
  (FabricObject.prototype as any)._getTransformedDimensions = function (options?: any) {
    const align: StrokeAlign = this.strokeAlign || 'center';

    // Untuk Line & Arrow yang memiliki rute siku/lengkung (connectorData),
    // hitung bounding box dari seluruh waypoint (bukan hanya titik p1 dan p2 lurus)
    if ((this.type === 'line' || this.isArrow) && this.connectorData?.computedPoints?.length > 1) {
      const pts: Array<{ x: number; y: number }> = this.connectorData.computedPoints;
      let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
      for (const pt of pts) {
        if (pt.x < minX) minX = pt.x;
        if (pt.x > maxX) maxX = pt.x;
        if (pt.y < minY) minY = pt.y;
        if (pt.y > maxY) maxY = pt.y;
      }
      const pad = (this.strokeWidth || 2) * 4 + 20;
      return new Point(Math.max(20, maxX - minX + pad), Math.max(20, maxY - minY + pad));
    }

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
      this._renderFill(ctx);
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
      ctx.save();
      this.strokeWidth = origWidth * 2;
      try {
        this._renderStroke(ctx);
      } finally {
        this.strokeWidth = origWidth;
      }

      ctx.save();
      ctx.globalCompositeOperation = 'destination-out';
      ctx.fillStyle = '#000000';
      ctx.fill();
      ctx.restore();
      ctx.restore();

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

  if (obj._cacheCanvas) {
    obj._cacheCanvas = null;
    obj._cacheContext = null;
  }
  obj.dirty = true;
  obj.setCoords();
}
