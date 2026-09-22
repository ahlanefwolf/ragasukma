import { Canvas, Line, IText, FabricObject } from 'fabric';
import { activeTool, strokeWidth, strokeAlign } from '../../../canvasStore';
import {
  snapCoord,
  getPointerCoords,
  applyShapeStrokeAlignment,
  syncShapeClipPath,
} from '../../drawing';
import { snapToPort, clearPortIndicators } from '../../portOverlay';
import { ConnectionPort } from '../../connector';
import { getShapeCreator } from '../../objects';
import { LineCreator } from '../../objects/line';
import { ViewportManager } from '../viewport';
import { HistoryManager } from '../history';

export interface CreationState {
  isMouseDown: boolean;
  origX: number;
  origY: number;
  activeShape: FabricObject | null;
  activeChain: Line[];
  chainPoints: Array<{ x: number; y: number }>;
  startSnapPort: { port: ConnectionPort; obj: FabricObject } | null;
  endSnapPort: { port: ConnectionPort; obj: FabricObject } | null;
}

export function handleMouseDownCreation(
  canvas: Canvas,
  viewport: ViewportManager,
  history: HistoryManager,
  state: CreationState,
  o: any,
  resetChain: () => void
) {
  if (viewport.startPan(canvas, o.e)) return;

  const tool = activeTool();
  if (tool !== 'line' && tool !== 'arrow') resetChain();
  if (tool === 'select' || tool === 'brush') return;

  if (canvas.getActiveObject()) {
    canvas.discardActiveObject();
    canvas.requestRenderAll();
  }

  const pointer = getPointerCoords(canvas, o.e, o);
  state.isMouseDown = true;
  state.origX = snapCoord(pointer.x);
  state.origY = snapCoord(pointer.y);

  if (tool === 'eraser') {
    if (o.target) {
      canvas.remove(o.target);
      canvas.renderAll();
      history.saveState(canvas);
    }
    return;
  }

  if (tool === 'arrow') {
    const snap = snapToPort(canvas, pointer, null, 24);
    if (snap) {
      state.origX = snap.port.x;
      state.origY = snap.port.y;
    }
    state.startSnapPort = snap;
    resetChain();
  } else if (tool === 'line') {
    state.startSnapPort = null;
    const origin = LineCreator.resolveStartPoint(
      { x: state.origX, y: state.origY },
      { chainPoints: state.chainPoints, resetChain }
    );
    state.origX = origin.x;
    state.origY = origin.y;
  }

  const creator = getShapeCreator(tool);
  if (creator) {
    const shape = creator.create({ x: state.origX, y: state.origY });
    canvas.add(shape);

    if (tool === 'text') {
      canvas.setActiveObject(shape);
      (shape as IText).enterEditing();
      (shape as IText).selectAll();
      state.isMouseDown = false;
      history.saveState(canvas);
    } else {
      state.activeShape = shape;
    }
  }
}

export function handleMouseMoveCreation(
  canvas: Canvas,
  viewport: ViewportManager,
  state: CreationState,
  o: any
) {
  if (viewport.pan(canvas, o.e)) return;
  if (!state.isMouseDown || !state.activeShape) return;

  const pointer = getPointerCoords(canvas, o.e, o);
  const tool = activeTool();

  if (tool === 'arrow') {
    state.endSnapPort = snapToPort(canvas, pointer, state.startSnapPort?.obj, 24);
  }

  const creator = getShapeCreator(tool);
  if (creator) {
    creator.update(
      state.activeShape,
      { x: state.origX, y: state.origY },
      pointer,
      { shiftKey: o.e?.shiftKey, e: o.e }
    );
  }

  canvas.renderAll();
}

export function handleMouseUpCreation(
  canvas: Canvas,
  viewport: ViewportManager,
  history: HistoryManager,
  state: CreationState,
  resetChain: () => void
) {
  if (viewport.endPan(canvas, activeTool() === 'select' ? 'default' : 'crosshair')) return;
  if (!state.isMouseDown) return;

  state.isMouseDown = false;
  clearPortIndicators(canvas);

  if (state.activeShape) {
    const tool = activeTool();
    const creator = getShapeCreator(tool);

    if (tool !== 'line' && tool !== 'arrow') {
      applyShapeStrokeAlignment(state.activeShape, strokeAlign(), strokeWidth());
    }
    syncShapeClipPath(state.activeShape);

    let finalObj: FabricObject | null = state.activeShape;
    if (creator?.finalize) {
      finalObj = creator.finalize(
        state.activeShape,
        { x: state.origX, y: state.origY },
        {
          canvas,
          origX: state.origX,
          origY: state.origY,
          startSnapPort: state.startSnapPort,
          endSnapPort: state.endSnapPort,
          activeChain: state.activeChain,
          chainPoints: state.chainPoints,
          resetChain,
        }
      );
    }

    if (finalObj) {
      if (creator?.isTiny && creator.isTiny(finalObj)) {
        canvas.remove(finalObj);
        state.activeShape = null;
        canvas.renderAll();
        return;
      }
      finalObj.set({ selectable: true });
      finalObj.setCoords();
    }

    state.activeShape = null;
    canvas.renderAll();
    history.saveState(canvas);
  }
}
