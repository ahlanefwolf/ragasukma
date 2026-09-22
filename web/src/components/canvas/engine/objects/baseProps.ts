import {
  strokeColor,
  fillColor,
  strokeWidth,
  strokeAlign,
  lineStyle,
} from '../../canvasStore';

export function getBaseStrokeProps() {
  const hasBorder = strokeColor() !== 'transparent' && strokeWidth() > 0;
  const actualStroke = hasBorder ? strokeColor() : '';
  const actualWidth = hasBorder ? strokeWidth() : 0;

  let dashArray: number[] | undefined = undefined;
  if (lineStyle() === 'dashed') {
    dashArray = [8, 6];
  } else if (lineStyle() === 'dotted') {
    dashArray = [3, 4];
  }

  return {
    hasBorder,
    stroke: actualStroke,
    strokeWidth: actualWidth,
    strokeDashArray: dashArray,
    strokeUniform: true,
    strokeAlign: strokeAlign(),
    fill: fillColor() === 'transparent' ? '' : fillColor(),
  };
}

export function getLineStrokeProps() {
  let dashArray: number[] | undefined = undefined;
  if (lineStyle() === 'dashed') {
    dashArray = [8, 6];
  } else if (lineStyle() === 'dotted') {
    dashArray = [3, 4];
  }

  return {
    fill: '',
    stroke: strokeColor() === 'transparent' ? '#29483A' : strokeColor(),
    strokeWidth: strokeWidth() || 2,
    strokeDashArray: dashArray,
    strokeUniform: true,
    strokeLineCap: 'round' as const,
    strokeLineJoin: 'round' as const,
    selectable: false,
  };
}
