import { Canvas, Point } from 'fabric';
import { setCanUndo, setCanRedo } from '../../canvasStore';
import { LayersEngine } from '../layers';
import { setupLineControls } from '../drawing';

/**
 * Standard History Manager berbasis State Pointer.
 * Menyimpan array snapshots dan index pointer `currentIndex`.
 * Pola ini menjamin Undo dan Redo 100% simetris, akurat, dan tidak pernah kehilangan state.
 */
export class HistoryManager {
  private history: string[] = [];
  private currentIndex = -1;
  public isProcessingHistory = false;

  public reset() {
    this.history = [];
    this.currentIndex = -1;
    this.isProcessingHistory = false;
    setCanUndo(false);
    setCanRedo(false);
  }

  public saveState(canvas: Canvas | null) {
    if (!canvas || this.isProcessingHistory) return;
    const json = JSON.stringify(canvas.toJSON());

    // Hindari snapshot duplikat jika tidak ada perubahan
    if (this.currentIndex >= 0 && this.history[this.currentIndex] === json) {
      return;
    }

    // Jika sedang berada di tengah-tengah undo lalu melakukan aksi baru,
    // buang cabang redo masa depan
    if (this.currentIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.currentIndex + 1);
    }

    this.history.push(json);
    if (this.history.length > 30) {
      this.history.shift();
    } else {
      this.currentIndex++;
    }

    setCanUndo(this.currentIndex > 0);
    setCanRedo(this.currentIndex < this.history.length - 1);
  }

  private restoreState(canvas: Canvas, stateJson: string) {
    canvas.loadFromJSON(stateJson).then(() => {
      canvas.getObjects().forEach((o: any) => {
        if (o.type === 'line' || o.isArrow) {
          // Fabric v6 toJSON mengubah x1/y1 menjadi koordinat lokal (-w/2, -h/2).
          // Normalisasikan kembali ke koordinat scene global asli:
          const pts = o.calcLinePoints();
          const p1 = new Point(pts.x1, pts.y1).transform(o.calcTransformMatrix());
          const p2 = new Point(pts.x2, pts.y2).transform(o.calcTransformMatrix());
          o.set({
            x1: p1.x,
            y1: p1.y,
            x2: p2.x,
            y2: p2.y,
            left: (p1.x + p2.x) / 2,
            top: (p1.y + p2.y) / 2,
            scaleX: 1,
            scaleY: 1,
            angle: 0,
          });
          setupLineControls(o);
          o.dirty = true;
          o.setCoords();
        }
      });
      canvas.discardActiveObject();
      canvas.requestRenderAll();
      this.isProcessingHistory = false;
      setCanUndo(this.currentIndex > 0);
      setCanRedo(this.currentIndex < this.history.length - 1);
      LayersEngine.syncLayers(canvas);
    });
  }

  public undo(canvas: Canvas | null) {
    if (!canvas || this.isProcessingHistory || this.currentIndex <= 0) return;

    this.isProcessingHistory = true;
    this.currentIndex--;
    const state = this.history[this.currentIndex];

    if (state) {
      this.restoreState(canvas, state);
    } else {
      this.isProcessingHistory = false;
    }
  }

  public redo(canvas: Canvas | null) {
    if (!canvas || this.isProcessingHistory || this.currentIndex >= this.history.length - 1) return;

    this.isProcessingHistory = true;
    this.currentIndex++;
    const state = this.history[this.currentIndex];

    if (state) {
      this.restoreState(canvas, state);
    } else {
      this.isProcessingHistory = false;
    }
  }
}
