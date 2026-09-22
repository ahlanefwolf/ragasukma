import { IText } from 'fabric';
import { fontSize, fontFamily, strokeColor } from '../../canvasStore';
import { Point2D, ShapeCreator } from './types';

export const TextCreator: ShapeCreator<IText> = {
  create(origin: Point2D): IText {
    return new IText('Teks disini...', {
      left: origin.x,
      top: origin.y,
      fontFamily: fontFamily(),
      fill: strokeColor(),
      fontSize: fontSize(),
      selectable: true,
    });
  },

  update() {
    // Teks tidak di-drag saat pembuatan awal melainkan langsung inline editing
  },

  isTiny(): boolean {
    return false;
  },
};
