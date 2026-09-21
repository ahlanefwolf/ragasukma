import { Canvas, FabricObject, IText } from 'fabric';
import { LayersEngine } from '../layers';

export class ClipboardManager {
  private clipboard: FabricObject | null = null;

  public hasClipboard(): boolean {
    return this.clipboard !== null;
  }

  public async copy(canvas: Canvas | null): Promise<boolean> {
    if (!canvas) return false;
    const activeObj = canvas.getActiveObject();
    if (!activeObj) return false;

    this.clipboard = await activeObj.clone();
    return true;
  }

  public async paste(canvas: Canvas | null, onSaveState: () => void): Promise<boolean> {
    if (!canvas || !this.clipboard) return false;

    const clonedObj = await this.clipboard.clone();
    canvas.discardActiveObject();

    clonedObj.set({
      left: (clonedObj.left || 0) + 24,
      top: (clonedObj.top || 0) + 24,
      evented: true,
    });

    const isGroupSelection =
      clonedObj.type === 'activeselection' ||
      clonedObj.type === 'activeSelection' ||
      typeof (clonedObj as any).getObjects === 'function';

    if (isGroupSelection && typeof (clonedObj as any).getObjects === 'function') {
      const subObjects = (clonedObj as any).getObjects();
      subObjects.forEach((sub: FabricObject) => {
        sub.set({
          left: (sub.left || 0) + 24,
          top: (sub.top || 0) + 24,
          evented: true,
        });
        canvas.add(sub);
      });
    } else {
      canvas.add(clonedObj);
      canvas.setActiveObject(clonedObj);
    }

    // Offset coordinates for successive paste operations
    this.clipboard.set({
      left: (this.clipboard.left || 0) + 24,
      top: (this.clipboard.top || 0) + 24,
    });

    canvas.requestRenderAll();
    onSaveState();
    LayersEngine.syncLayers(canvas);
    return true;
  }

  public async duplicate(canvas: Canvas | null, onSaveState: () => void): Promise<boolean> {
    const copied = await this.copy(canvas);
    if (!copied) return false;
    return await this.paste(canvas, onSaveState);
  }

  public deleteSelected(canvas: Canvas | null, onSaveState: () => void) {
    if (!canvas) return;
    const activeObjects = canvas.getActiveObjects();
    if (activeObjects && activeObjects.length > 0) {
      activeObjects.forEach((obj) => {
        canvas.remove(obj);
      });
      canvas.discardActiveObject();
      canvas.renderAll();
      onSaveState();
      LayersEngine.syncLayers(canvas);
    }
  }

  public isEditingText(canvas: Canvas | null): boolean {
    if (!canvas) return false;
    const activeObj = canvas.getActiveObject();
    if (activeObj && activeObj instanceof IText) {
      return activeObj.isEditing || false;
    }
    return false;
  }
}
