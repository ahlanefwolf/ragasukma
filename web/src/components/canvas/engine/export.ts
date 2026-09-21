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
      const rect = activeObj.getBoundingRect();
      return {
        hasSelection: true,
        selectionWidth: Math.max(1, Math.round(rect.width)),
        selectionHeight: Math.max(1, Math.round(rect.height)),
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
        const rect = activeObj.getBoundingRect();
        const targetW = options.width || rect.width;
        const targetH = options.height || rect.height;
        return `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" width="${targetW}" height="${targetH}" viewBox="0 0 ${rect.width} ${rect.height}">
  <g transform="translate(${-rect.left}, ${-rect.top})">
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
      bg = '#10141D';
    } else if (options.background === 'transparent') {
      bg = options.format === 'jpeg' ? '#FFFFFF' : '';
    }

    canvas.backgroundColor = bg;
    canvas.renderAll();

    let dataUrl = '';
    if (hasSelection && activeObj) {
      const rect = activeObj.getBoundingRect();
      const mult = options.width ? options.width / Math.max(1, rect.width) : (options.multiplier || 1);
      dataUrl = canvas.toDataURL({
        left: rect.left,
        top: rect.top,
        width: rect.width,
        height: rect.height,
        format: options.format === 'jpeg' ? 'jpeg' : 'png',
        quality: options.quality || 0.95,
        multiplier: mult,
      });
    } else {
      const cWidth = Math.max(1, canvas.getWidth());
      const mult = options.width ? options.width / cWidth : (options.multiplier || 1);
      dataUrl = canvas.toDataURL({
        format: options.format === 'jpeg' ? 'jpeg' : 'png',
        quality: options.quality || 0.95,
        multiplier: mult,
      });
    }

    canvas.backgroundColor = originalBg;
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
