import { Canvas, FabricImage } from 'fabric';

export class ExportEngine {
  public static async addImageFromDataUrl(
    canvas: Canvas | null,
    dataUrl: string,
    onStateChange?: () => void
  ): Promise<boolean> {
    if (!canvas) return false;
    try {
      const img = await FabricImage.fromURL(dataUrl, { crossOrigin: 'anonymous' });
      const canvasWidth = canvas.getWidth() || 800;
      const canvasHeight = canvas.getHeight() || 600;
      const maxW = canvasWidth * 0.75;
      const maxH = canvasHeight * 0.75;
      let scale = 1;

      if (img.width && img.height) {
        if (img.width > maxW || img.height > maxH) {
          scale = Math.min(maxW / img.width, maxH / img.height);
        }
      }

      const scaledW = (img.width || 200) * scale;
      const scaledH = (img.height || 200) * scale;

      img.set({
        left: (canvasWidth - scaledW) / 2,
        top: (canvasHeight - scaledH) / 2,
        scaleX: scale,
        scaleY: scale,
        selectable: true,
      });

      canvas.add(img);
      canvas.setActiveObject(img);
      canvas.renderAll();
      onStateChange?.();
      return true;
    } catch (err) {
      console.error('Failed to load image onto canvas:', err);
      return false;
    }
  }

  public static getExportScopeInfo(canvas: Canvas | null): {
    hasSelection: boolean;
    selectionWidth: number;
    selectionHeight: number;
    canvasWidth: number;
    canvasHeight: number;
  } {
    if (!canvas) {
      return {
        hasSelection: false,
        selectionWidth: 800,
        selectionHeight: 600,
        canvasWidth: 800,
        canvasHeight: 600,
      };
    }

    const canvasWidth = Math.max(1, Math.round(canvas.getWidth()));
    const canvasHeight = Math.max(1, Math.round(canvas.getHeight()));
    const activeObj = canvas.getActiveObject();

    if (activeObj) {
      let left = Infinity;
      let top = Infinity;
      let right = -Infinity;
      let bottom = -Infinity;

      const objectsToCheck = (activeObj as any).getObjects ? (activeObj as any).getObjects() : [activeObj];
      objectsToCheck.forEach((obj: any) => {
        if ((obj.type === 'line' || obj.isArrow) && obj.connectorData?.computedPoints?.length > 1) {
          const strokeW = (obj.strokeWidth || 2) * 2;
          for (const pt of obj.connectorData.computedPoints) {
            if (pt.x - strokeW < left) left = pt.x - strokeW;
            if (pt.x + strokeW > right) right = pt.x + strokeW;
            if (pt.y - strokeW < top) top = pt.y - strokeW;
            if (pt.y + strokeW > bottom) bottom = pt.y + strokeW;
          }
        } else if (typeof obj.getBoundingRect === 'function') {
          const r = obj.getBoundingRect();
          if (r.left < left) left = r.left;
          if (r.top < top) top = r.top;
          if (r.left + r.width > right) right = r.left + r.width;
          if (r.top + r.height > bottom) bottom = r.top + r.height;
        }
      });

      const fallbackRect = activeObj.getBoundingRect();
      if (!isFinite(left) || !isFinite(right) || !isFinite(top) || !isFinite(bottom)) {
        left = fallbackRect.left;
        top = fallbackRect.top;
        right = fallbackRect.left + fallbackRect.width;
        bottom = fallbackRect.top + fallbackRect.height;
      }

      const padding = 8;
      const cropLeft = Math.max(0, left - padding);
      const cropTop = Math.max(0, top - padding);
      const cropWidth = Math.min(canvasWidth - cropLeft, (right - left) + padding * 2);
      const cropHeight = Math.min(canvasHeight - cropTop, (bottom - top) + padding * 2);

      return {
        hasSelection: true,
        selectionWidth: Math.max(1, Math.round(cropWidth)),
        selectionHeight: Math.max(1, Math.round(cropHeight)),
        canvasWidth,
        canvasHeight,
      };
    }

    return {
      hasSelection: false,
      selectionWidth: canvasWidth,
      selectionHeight: canvasHeight,
      canvasWidth,
      canvasHeight,
    };
  }

  public static getCanvasDimensions(canvas: Canvas | null): { width: number; height: number } {
    if (!canvas) return { width: 800, height: 600 };
    return {
      width: Math.round(canvas.getWidth()),
      height: Math.round(canvas.getHeight()),
    };
  }

  public static exportCanvas(
    canvas: Canvas | null,
    options: {
      format: 'png' | 'jpeg' | 'svg';
      scope?: 'selection' | 'canvas';
      width?: number;
      height?: number;
      multiplier?: number;
      background: 'transparent' | 'white' | 'dark';
      quality?: number;
    }
  ): string {
    if (!canvas) return '';

    const scope = options.scope || 'canvas';
    const activeObj = canvas.getActiveObject();
    const hasSelection = scope === 'selection' && !!activeObj;

    if (options.format === 'svg') {
      if (hasSelection && activeObj) {
        let left = Infinity;
        let top = Infinity;
        let right = -Infinity;
        let bottom = -Infinity;

        const objectsToCheck = (activeObj as any).getObjects ? (activeObj as any).getObjects() : [activeObj];
        objectsToCheck.forEach((obj: any) => {
          if ((obj.type === 'line' || obj.isArrow) && obj.connectorData?.computedPoints?.length > 1) {
            const strokeW = (obj.strokeWidth || 2) * 2;
            for (const pt of obj.connectorData.computedPoints) {
              if (pt.x - strokeW < left) left = pt.x - strokeW;
              if (pt.x + strokeW > right) right = pt.x + strokeW;
              if (pt.y - strokeW < top) top = pt.y - strokeW;
              if (pt.y + strokeW > bottom) bottom = pt.y + strokeW;
            }
          } else if (typeof obj.getBoundingRect === 'function') {
            const r = obj.getBoundingRect();
            if (r.left < left) left = r.left;
            if (r.top < top) top = r.top;
            if (r.left + r.width > right) right = r.left + r.width;
            if (r.top + r.height > bottom) bottom = r.top + r.height;
          }
        });

        const fallbackRect = activeObj.getBoundingRect();
        if (!isFinite(left) || !isFinite(right) || !isFinite(top) || !isFinite(bottom)) {
          left = fallbackRect.left;
          top = fallbackRect.top;
          right = fallbackRect.left + fallbackRect.width;
          bottom = fallbackRect.top + fallbackRect.height;
        }

        const padding = 8;
        const cropLeft = Math.max(0, left - padding);
        const cropTop = Math.max(0, top - padding);
        const cropWidth = (right - left) + padding * 2;
        const cropHeight = (bottom - top) + padding * 2;

        const targetW = options.width || cropWidth;
        const targetH = options.height || cropHeight;
        return `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" width="${targetW}" height="${targetH}" viewBox="0 0 ${cropWidth} ${cropHeight}">
  <g transform="translate(${-cropLeft}, ${-cropTop})">
    ${activeObj.toSVG()}
  </g>
</svg>`;
      }
      return canvas.toSVG();
    }

    // Handle background color for bitmap rasterization
    const originalBg = canvas.backgroundColor;
    let bg = '';
    if (options.background === 'white') {
      bg = '#FFFFFF';
    } else if (options.background === 'dark') {
      bg = '#FAF7F2';
    } else if (options.background === 'transparent') {
      bg = options.format === 'jpeg' ? '#FFFFFF' : '';
    }

    // Simpan viewport transform saat ini agar export selalu merender scene kanvas penuh (1:1 tanpa zoom/pan terpotong)
    const originalVpt = canvas.viewportTransform ? [...canvas.viewportTransform] : [1, 0, 0, 1, 0, 0];
    const originalZoom = canvas.getZoom();

    // Reset viewport sementara saat export
    canvas.setViewportTransform([1, 0, 0, 1, 0, 0]);
    canvas.backgroundColor = bg;
    canvas.renderAll();

    let dataUrl = '';
    if (hasSelection && activeObj) {
      // Hitung bounding rect aktual yang memperhitungkan waypoint connectorData
      // jika ada garis/arrow dengan rute siku/melengkung yang melebihi batas 2 objek
      let left = Infinity;
      let top = Infinity;
      let right = -Infinity;
      let bottom = -Infinity;

      const objectsToCheck = (activeObj as any).getObjects ? (activeObj as any).getObjects() : [activeObj];
      objectsToCheck.forEach((obj: any) => {
        if ((obj.type === 'line' || obj.isArrow) && obj.connectorData?.computedPoints?.length > 1) {
          const strokeW = (obj.strokeWidth || 2) * 2;
          for (const pt of obj.connectorData.computedPoints) {
            if (pt.x - strokeW < left) left = pt.x - strokeW;
            if (pt.x + strokeW > right) right = pt.x + strokeW;
            if (pt.y - strokeW < top) top = pt.y - strokeW;
            if (pt.y + strokeW > bottom) bottom = pt.y + strokeW;
          }
        } else if (typeof obj.getBoundingRect === 'function') {
          const r = obj.getBoundingRect();
          if (r.left < left) left = r.left;
          if (r.top < top) top = r.top;
          if (r.left + r.width > right) right = r.left + r.width;
          if (r.top + r.height > bottom) bottom = r.top + r.height;
        }
      });

      const fallbackRect = activeObj.getBoundingRect();
      if (!isFinite(left) || !isFinite(right) || !isFinite(top) || !isFinite(bottom)) {
        left = fallbackRect.left;
        top = fallbackRect.top;
        right = fallbackRect.left + fallbackRect.width;
        bottom = fallbackRect.top + fallbackRect.height;
      }

      // Berikan margin padding kecil (8px) agar stroke garis paling luar tidak terpotong
      const padding = 8;
      const cropLeft = Math.max(0, left - padding);
      const cropTop = Math.max(0, top - padding);
      const cropWidth = Math.min(canvas.getWidth() - cropLeft, (right - left) + padding * 2);
      const cropHeight = Math.min(canvas.getHeight() - cropTop, (bottom - top) + padding * 2);

      const mult = options.width ? options.width / Math.max(1, cropWidth) : (options.multiplier || 1);
      dataUrl = canvas.toDataURL({
        left: cropLeft,
        top: cropTop,
        width: cropWidth,
        height: cropHeight,
        format: options.format === 'jpeg' ? 'jpeg' : 'png',
        quality: options.quality || 0.95,
        multiplier: mult,
      });
    } else {
      const cWidth = Math.max(1, canvas.getWidth());
      const mult = options.width ? options.width / cWidth : (options.multiplier || 1);
      dataUrl = canvas.toDataURL({
        left: 0,
        top: 0,
        width: canvas.getWidth(),
        height: canvas.getHeight(),
        format: options.format === 'jpeg' ? 'jpeg' : 'png',
        quality: options.quality || 0.95,
        multiplier: mult,
      });
    }

    // Kembalikan background dan viewport semula
    canvas.backgroundColor = originalBg;
    canvas.setViewportTransform(originalVpt as any);
    canvas.setZoom(originalZoom);
    canvas.renderAll();

    return dataUrl;
  }

  public static exportImage(canvas: Canvas | null, format: 'png' | 'svg' = 'png'): string {
    return this.exportCanvas(canvas, {
      format: format === 'svg' ? 'svg' : 'png',
      multiplier: 2,
      background: 'transparent',
    });
  }
}
