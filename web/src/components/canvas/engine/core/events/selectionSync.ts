import { Canvas, Line, ActiveSelection } from 'fabric';
import {
  activeTool,
  setActiveTool,
  setSelectedObjectType,
  setStrokeWidth,
  setStrokeAlign,
  setCornerRadius,
  setStrokeColor,
  setFillColor,
  setFontSize,
  setFontFamily,
  setArrowHead,
  setLineStyle,
  setConnectorRouting,
} from '../../../canvasStore';
import { setupLineControls } from '../../drawing';
import { ShapeBuilderEngine } from '../../shapeBuilder';
import { LayersEngine } from '../../layers';

export function handleSelectionSync(
  canvas: Canvas,
  e: any,
  isDrawingActive: () => boolean
) {
  const activeObjects = canvas.getActiveObjects();

  // Sinkronkan selection state
  const obj = e.selected?.[0] || canvas.getActiveObject();

  if (obj) {
    // Jangan paksa ubah tool jika user sedang aktif menggambar (line/arrow/rect/circle)
    if (activeTool() !== 'select' && activeTool() !== 'brush') {
      if (!isDrawingActive()) {
        setActiveTool('select');
      }
    }

    const isArrowObj = (obj instanceof Line || obj.type === 'line') && (obj.arrowEnd || obj.isArrow);
    const typeStr = activeObjects.length > 1 
      ? 'activeSelection' 
      : isArrowObj ? 'arrow' : (obj.type ? obj.type.toLowerCase() : null);
    setSelectedObjectType(typeStr);

    if (obj instanceof Line || obj.type === 'line') {
      setupLineControls(obj);
      if (obj.arrowStart && obj.arrowEnd) {
        setArrowHead('both');
      } else if (obj.arrowEnd || obj.isArrow) {
        setArrowHead('end');
      } else {
        setArrowHead('none');
      }

      if (obj.strokeDashArray && obj.strokeDashArray.length > 0) {
        if (obj.strokeDashArray[0] <= 4) {
          setLineStyle('dotted');
        } else {
          setLineStyle('dashed');
        }
      } else {
        setLineStyle('solid');
      }

      // Sinkronkan connector routing yang sedang aktif pada garis ini
      if (obj.connectorData?.routing) {
        setConnectorRouting(obj.connectorData.routing);
      } else {
        setConnectorRouting('straight');
      }
    }

    if (activeObjects.length > 1) {
      ShapeBuilderEngine.updateHatchPreview(canvas);
    } else {
      ShapeBuilderEngine.clearHatchPreview(canvas);
    }

    const currentWidth =
      typeof obj.strokeWidth === 'number'
        ? obj.strokeWidth
        : typeof obj.rawStrokeWidth === 'number'
        ? obj.rawStrokeWidth
        : 0;
    if (currentWidth >= 0) {
      setStrokeWidth(currentWidth);
    }
    if (obj.strokeAlign) {
      setStrokeAlign(obj.strokeAlign);
    } else {
      setStrokeAlign('center');
    }
    if (typeof obj.rx === 'number') {
      setCornerRadius(obj.rx);
    }
    if (typeof obj.stroke === 'string' && obj.stroke && obj.stroke !== 'transparent' && currentWidth > 0) {
      setStrokeColor(obj.stroke);
    } else if (!obj.stroke || currentWidth === 0) {
      setStrokeColor('transparent');
    }
    if (typeof obj.fill === 'string') {
      setFillColor(obj.fill || 'transparent');
    }
    if (typeof obj.fontSize === 'number') {
      setFontSize(obj.fontSize);
    }
    if (typeof obj.fontFamily === 'string' && obj.fontFamily) {
      setFontFamily(obj.fontFamily);
    }
  } else {
    setSelectedObjectType(null);
  }
  LayersEngine.syncLayers(canvas);
}

export function handleSelectionCleared(canvas: Canvas) {
  setSelectedObjectType(null);
  ShapeBuilderEngine.clearHatchPreview(canvas);
  LayersEngine.syncLayers(canvas);
}
