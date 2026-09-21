import { Canvas } from 'fabric';
import { setCanUndo, setCanRedo } from '../../canvasStore';
import { LayersEngine } from '../layers';

export class HistoryManager {
  private undoStack: string[] = [];
  private redoStack: string[] = [];
  public isProcessingHistory = false;

  public reset() {
    this.undoStack = [];
    this.redoStack = [];
    this.isProcessingHistory = false;
    setCanUndo(false);
    setCanRedo(false);
  }

  public saveState(canvas: Canvas | null) {
    if (!canvas || this.isProcessingHistory) return;
    const json = JSON.stringify(canvas.toJSON());
    this.undoStack.push(json);
    if (this.undoStack.length > 30) {
      this.undoStack.shift();
    }
    this.redoStack = [];
    setCanUndo(this.undoStack.length > 1);
    setCanRedo(false);
  }

  public undo(canvas: Canvas | null) {
    if (!canvas || this.undoStack.length <= 1) return;

    this.isProcessingHistory = true;
    const current = this.undoStack.pop();
    if (current) {
      this.redoStack.push(current);
    }

    const previous = this.undoStack[this.undoStack.length - 1];
    if (previous) {
      canvas.loadFromJSON(previous).then(() => {
        canvas.renderAll();
        this.isProcessingHistory = false;
        setCanUndo(this.undoStack.length > 1);
        setCanRedo(this.redoStack.length > 0);
        LayersEngine.syncLayers(canvas);
      });
    } else {
      this.isProcessingHistory = false;
    }
  }

  public redo(canvas: Canvas | null) {
    if (!canvas || this.redoStack.length === 0) return;

    this.isProcessingHistory = true;
    const next = this.redoStack.pop();
    if (next) {
      this.undoStack.push(next);
      canvas.loadFromJSON(next).then(() => {
        canvas.renderAll();
        this.isProcessingHistory = false;
        setCanUndo(this.undoStack.length > 1);
        setCanRedo(this.redoStack.length > 0);
        LayersEngine.syncLayers(canvas);
      });
    } else {
      this.isProcessingHistory = false;
    }
  }
}
