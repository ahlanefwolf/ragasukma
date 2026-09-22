import { CanvasTool } from '../../types';
import { RectCreator } from './rect';
import { CircleCreator } from './circle';
import { LineCreator } from './line';
import { ConnectorCreator } from './connector';
import { TextCreator } from './text';
import { ShapeCreator } from './types';

export * from './types';
export * from './baseProps';
export * from './rect';
export * from './circle';
export * from './line';
export * from './connector';
export * from './text';

export const shapeRegistry: Partial<Record<CanvasTool, ShapeCreator<any>>> = {
  rectangle: RectCreator,
  circle: CircleCreator,
  line: LineCreator,
  arrow: ConnectorCreator,
  text: TextCreator,
};

export function getShapeCreator(tool: CanvasTool): ShapeCreator<any> | undefined {
  return shapeRegistry[tool];
}
