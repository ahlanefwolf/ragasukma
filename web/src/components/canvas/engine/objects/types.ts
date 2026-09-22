import { Canvas, FabricObject } from 'fabric';

export interface Point2D {
  x: number;
  y: number;
}

export interface FinalizeContext {
  canvas: Canvas;
  origX: number;
  origY: number;
  startSnapPort?: any;
  endSnapPort?: any;
  activeChain?: any[];
  chainPoints?: Point2D[];
  resetChain?: () => void;
}

export interface ShapeCreator<T extends FabricObject = FabricObject> {
  create(origin: Point2D): T;
  update(shape: T, origin: Point2D, pointer: Point2D, options?: { shiftKey?: boolean; e?: any }): void;
  finalize?(shape: T, origin: Point2D, ctx: FinalizeContext): FabricObject | null;
  isTiny?(shape: T): boolean;
}
