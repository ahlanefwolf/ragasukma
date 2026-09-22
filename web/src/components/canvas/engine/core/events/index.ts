import { Canvas, Line, FabricObject } from 'fabric';
import { ViewportManager } from '../viewport';
import { HistoryManager } from '../history';
import { ConnectionPort } from '../../connector';
import {
  handleMouseDownCreation,
  handleMouseMoveCreation,
  handleMouseUpCreation,
  CreationState,
} from './creationHandlers';
import { bindObjectTransforms } from './objectTransform';
import { handleSelectionSync, handleSelectionCleared } from './selectionSync';

export class EventsManager {
  public isMouseDown = false;
  public origX = 0;
  public origY = 0;
  public activeShape: FabricObject | null = null;
  private activeChain: Line[] = [];
  private chainPoints: Array<{ x: number; y: number }> = [];
  private lastArrowObject: FabricObject | null = null;
  private startSnapPort: { port: ConnectionPort; obj: FabricObject } | null = null;
  private endSnapPort: { port: ConnectionPort; obj: FabricObject } | null = null;

  public resetChain() {
    this.activeChain = [];
    this.chainPoints = [];
    this.lastArrowObject = null;
    this.startSnapPort = null;
    this.endSnapPort = null;
  }

  private getCreationState(): CreationState {
    return {
      isMouseDown: this.isMouseDown,
      origX: this.origX,
      origY: this.origY,
      activeShape: this.activeShape,
      activeChain: this.activeChain,
      chainPoints: this.chainPoints,
      startSnapPort: this.startSnapPort,
      endSnapPort: this.endSnapPort,
    };
  }

  private syncFromCreationState(state: CreationState) {
    this.isMouseDown = state.isMouseDown;
    this.origX = state.origX;
    this.origY = state.origY;
    this.activeShape = state.activeShape;
    this.activeChain = state.activeChain;
    this.chainPoints = state.chainPoints;
    this.startSnapPort = state.startSnapPort;
    this.endSnapPort = state.endSnapPort;
  }

  public bindEvents(
    canvas: Canvas,
    viewport: ViewportManager,
    history: HistoryManager
  ) {
    // ── Mouse Wheel Zooming ──
    canvas.on('mouse:wheel', (opt: any) => {
      viewport.handleWheel(canvas, opt);
      // Re-render canvas controls and overlays agar control circles selalu sinkron dengan zoom
      canvas.requestRenderAll();
    });

    // ── Mouse Down: Creation & Panning ──
    canvas.on('mouse:down', (o: any) => {
      // Abaikan pembuatan jika klik terjadi pada control handle objek yang sedang terseleksi
      const activeObj = canvas.getActiveObject();
      if (activeObj && (activeObj as any).__corner) {
        return;
      }

      const state = this.getCreationState();
      handleMouseDownCreation(
        canvas,
        viewport,
        history,
        state,
        o,
        () => this.resetChain()
      );
      this.syncFromCreationState(state);
    });

    // ── Mouse Move: Interactive Resizing & Panning ──
    canvas.on('mouse:move', (o: any) => {
      const state = this.getCreationState();
      handleMouseMoveCreation(canvas, viewport, state, o);
      this.syncFromCreationState(state);
    });

    // ── Mouse Up: Finalize Creation ──
    canvas.on('mouse:up', () => {
      const state = this.getCreationState();
      handleMouseUpCreation(
        canvas,
        viewport,
        history,
        state,
        () => this.resetChain()
      );
      this.syncFromCreationState(state);
    });

    // ── Freehand Brush (path:created) ──
    canvas.on('path:created', () => {
      history.saveState(canvas);
    });

    // ── Object Snapping & Synchronization (Move, Scale, Rotate, Modified) ──
    bindObjectTransforms(canvas, history, () => this.isMouseDown);

    // ── Selection State Synchronization ──
    const onSelection = (e: any) => {
      handleSelectionSync(canvas, e, () => this.isMouseDown || !!this.activeShape);
    };

    canvas.on('selection:created', onSelection);
    canvas.on('selection:updated', onSelection);
    canvas.on('selection:cleared', () => {
      handleSelectionCleared(canvas);
    });
  }
}

export * from './creationHandlers';
export * from './objectTransform';
export * from './selectionSync';
