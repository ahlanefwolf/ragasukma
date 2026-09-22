import {
  Canvas,
  Rect,
  Circle,
  Line,
  IText,
  FabricImage,
  Group,
  Path,
  ActiveSelection,
  FabricObject,
} from 'fabric';
import { CanvasLayerItem } from '../types';
import { setLayersList, setActiveTool, setSelectedObjectType } from '../canvasStore';

export class LayersEngine {
  public static syncLayers(canvas: Canvas | null) {
    if (!canvas) {
      setLayersList([]);
      return;
    }

    const objects = canvas.getObjects();
    const activeObjects = canvas.getActiveObjects();
    const activeIds = new Set(activeObjects.map((o: any) => o.id));

    const layers: CanvasLayerItem[] = [];

    for (let i = objects.length - 1; i >= 0; i--) {
      const obj = objects[i] as any;
      if (obj.__isHatchPreview) continue;
      if (!obj.id) {
        obj.id = 'obj_' + Math.random().toString(36).substring(2, 9);
      }

      let type = 'shape';
      let defaultName = 'Bentuk (Shape)';
      let fillColor = typeof obj.fill === 'string' ? obj.fill : 'transparent';
      let strokeColor = typeof obj.stroke === 'string' ? obj.stroke : '#29483A';
      let details = '';

      if (obj instanceof Rect || obj.type === 'rect') {
        type = 'rect';
        defaultName = obj.customName || 'Kotak (Rect)';
        details = `${Math.round(obj.width * (obj.scaleX || 1))} x ${Math.round(obj.height * (obj.scaleY || 1))}px`;
      } else if (obj instanceof Circle || obj.type === 'circle') {
        type = 'circle';
        defaultName = obj.customName || 'Lingkaran (Circle)';
        details = `r = ${Math.round(obj.radius * (obj.scaleX || 1))}px`;
      } else if (obj instanceof Line || obj.type === 'line') {
        type = 'line';
        defaultName = obj.customName || 'Garis (Line)';
        fillColor = 'transparent';
      } else if (obj instanceof IText || obj.type === 'i-text' || obj.type === 'text') {
        type = 'text';
        const txt = obj.text || '';
        defaultName = obj.customName || (txt.length > 0 ? `Teks: "${txt.slice(0, 16)}${txt.length > 16 ? '...' : ''}"` : 'Teks');
        fillColor = typeof obj.fill === 'string' ? obj.fill : '#FFFFFF';
        strokeColor = 'transparent';
        details = `${obj.fontSize || 16}px`;
      } else if (obj instanceof FabricImage || obj.type === 'image') {
        type = 'image';
        defaultName = obj.customName || 'Gambar (Image)';
        details = `${Math.round((obj.width || 0) * (obj.scaleX || 1))} x ${Math.round((obj.height || 0) * (obj.scaleY || 1))}px`;
      } else if (obj instanceof Group || obj.type === 'group') {
        type = 'group';
        const childCount = typeof obj.getObjects === 'function' ? obj.getObjects().length : 0;
        defaultName = obj.customName || `Grup (${childCount} Objek)`;
        details = `${childCount} objek tergabung`;
      } else if (obj instanceof Path || obj.type === 'path') {
        type = 'path';
        defaultName = obj.customName || 'Kuas Bebas (Brush)';
      }

      layers.push({
        id: obj.id,
        name: defaultName,
        type,
        visible: obj.visible !== false,
        locked: !!obj.lockMovementX,
        selected: activeIds.has(obj.id),
        fillColor,
        strokeColor,
        details,
      });
    }

    setLayersList(layers);
  }

  public static selectLayer(
    canvas: Canvas | null,
    id: string,
    options: { ctrlKey?: boolean; shiftKey?: boolean; metaKey?: boolean } = {}
  ) {
    if (!canvas) return;
    const objects = canvas.getObjects();
    const targetObj = objects.find((o: any) => o.id === id);
    if (!targetObj || targetObj.visible === false || targetObj.lockMovementX) return;

    setActiveTool('select');

    const isMulti = options.ctrlKey || options.metaKey;
    const isRange = options.shiftKey;

    if (isMulti) {
      const activeObjs = canvas.getActiveObjects();
      const exists = activeObjs.includes(targetObj);
      let newSelection: FabricObject[];

      if (exists) {
        newSelection = activeObjs.filter((o) => o !== targetObj);
      } else {
        newSelection = [...activeObjs, targetObj];
      }

      if (newSelection.length === 0) {
        canvas.discardActiveObject();
      } else if (newSelection.length === 1) {
        canvas.setActiveObject(newSelection[0]);
      } else {
        const sel = new ActiveSelection(newSelection, { canvas });
        canvas.setActiveObject(sel);
      }
    } else if (isRange) {
      const activeObjs = canvas.getActiveObjects();
      const lastSelected = activeObjs[activeObjs.length - 1] || targetObj;
      const idx1 = objects.indexOf(lastSelected);
      const idx2 = objects.indexOf(targetObj);
      if (idx1 !== -1 && idx2 !== -1) {
        const start = Math.min(idx1, idx2);
        const end = Math.max(idx1, idx2);
        const rangeObjs = objects
          .slice(start, end + 1)
          .filter((o: any) => o.visible !== false && !o.lockMovementX);
        if (rangeObjs.length > 1) {
          const sel = new ActiveSelection(rangeObjs, { canvas });
          canvas.setActiveObject(sel);
        } else if (rangeObjs.length === 1) {
          canvas.setActiveObject(rangeObjs[0]);
        }
      }
    } else {
      canvas.setActiveObject(targetObj);
    }

    canvas.renderAll();
    this.syncLayers(canvas);
  }

  public static groupSelected(canvas: Canvas | null, onStateChange?: () => void) {
    if (!canvas) return;
    const activeObj = canvas.getActiveObject();
    if (!activeObj) return;

    // Handle ActiveSelection to fabric.Group
    if (activeObj.type === 'activeSelection' || (activeObj as any)._objects) {
      const objects: FabricObject[] = (activeObj as any).getObjects().slice();
      if (objects.length < 2) return;

      canvas.discardActiveObject();
      objects.forEach((obj) => canvas.remove(obj));

      const group = new Group(objects, {
        subTargetCheck: false,
        selectable: true,
      } as any);
      (group as any).id = 'group_' + Math.random().toString(36).substring(2, 9);
      (group as any).customName = `Grup (${objects.length} Objek)`;

      canvas.add(group);
      canvas.setActiveObject(group);
      canvas.renderAll();
      onStateChange?.();
      this.syncLayers(canvas);
    }
  }

  public static ungroupSelected(canvas: Canvas | null, onStateChange?: () => void) {
    if (!canvas) return;
    const activeObj = canvas.getActiveObject();
    if (!activeObj || !(activeObj instanceof Group || activeObj.type === 'group')) return;

    const group = activeObj as Group;
    const items = group.getObjects().slice();
    if (items.length === 0) return;

    canvas.discardActiveObject();
    canvas.remove(group);

    items.forEach((item) => {
      // Restore matrix / coordinates to canvas root coordinate space
      group.remove(item);
      canvas.add(item);
    });

    const activeSel = new ActiveSelection(items, { canvas });
    canvas.setActiveObject(activeSel);
    canvas.renderAll();
    onStateChange?.();
    this.syncLayers(canvas);
  }

  public static toggleLayerVisibility(canvas: Canvas | null, id: string, onStateChange?: () => void) {
    if (!canvas) return;
    const obj = canvas.getObjects().find((o: any) => o.id === id);
    if (obj) {
      obj.set({ visible: !obj.visible });
      if (!obj.visible && canvas.getActiveObject() === obj) {
        canvas.discardActiveObject();
      }
      canvas.renderAll();
      onStateChange?.();
      this.syncLayers(canvas);
    }
  }

  public static toggleLayerLock(canvas: Canvas | null, id: string, onStateChange?: () => void) {
    if (!canvas) return;
    const obj = canvas.getObjects().find((o: any) => o.id === id);
    if (obj) {
      const isLocked = !obj.lockMovementX;
      obj.set({
        lockMovementX: isLocked,
        lockMovementY: isLocked,
        lockRotation: isLocked,
        lockScalingX: isLocked,
        lockScalingY: isLocked,
        hasControls: !isLocked,
        selectable: !isLocked,
      });
      if (isLocked && canvas.getActiveObject() === obj) {
        canvas.discardActiveObject();
      }
      canvas.renderAll();
      onStateChange?.();
      this.syncLayers(canvas);
    }
  }

  public static moveLayerUp(canvas: Canvas | null, id: string, onStateChange?: () => void) {
    if (!canvas) return;
    const obj = canvas.getObjects().find((o: any) => o.id === id);
    if (obj) {
      if (typeof (canvas as any).bringObjectForward === 'function') {
        (canvas as any).bringObjectForward(obj);
      } else if (typeof (canvas as any).bringForward === 'function') {
        (canvas as any).bringForward(obj);
      }
      canvas.renderAll();
      onStateChange?.();
      this.syncLayers(canvas);
    }
  }

  public static moveLayerDown(canvas: Canvas | null, id: string, onStateChange?: () => void) {
    if (!canvas) return;
    const obj = canvas.getObjects().find((o: any) => o.id === id);
    if (obj) {
      if (typeof (canvas as any).sendObjectBackwards === 'function') {
        (canvas as any).sendObjectBackwards(obj);
      } else if (typeof (canvas as any).sendBackwards === 'function') {
        (canvas as any).sendBackwards(obj);
      }
      canvas.renderAll();
      onStateChange?.();
      this.syncLayers(canvas);
    }
  }

  public static deleteLayer(canvas: Canvas | null, id: string, onStateChange?: () => void) {
    if (!canvas) return;
    const obj = canvas.getObjects().find((o: any) => o.id === id);
    if (obj) {
      canvas.remove(obj);
      canvas.renderAll();
      onStateChange?.();
      this.syncLayers(canvas);
    }
  }

  public static renameLayer(canvas: Canvas | null, id: string, newName: string, onStateChange?: () => void) {
    if (!canvas) return;
    const obj = canvas.getObjects().find((o: any) => o.id === id);
    if (obj) {
      (obj as any).customName = newName.trim() || undefined;
      this.syncLayers(canvas);
      onStateChange?.();
    }
  }

  public static reorderLayer(canvas: Canvas | null, fromLayerIndex: number, toLayerIndex: number, onStateChange?: () => void) {
    if (!canvas) return;
    const objects = canvas.getObjects();
    const total = objects.length;
    if (fromLayerIndex < 0 || fromLayerIndex >= total || toLayerIndex < 0 || toLayerIndex >= total) {
      return;
    }

    const fromCanvasIndex = total - 1 - fromLayerIndex;
    const toCanvasIndex = total - 1 - toLayerIndex;

    const obj = objects[fromCanvasIndex];
    if (!obj) return;

    if (typeof (canvas as any).moveObjectTo === 'function') {
      (canvas as any).moveObjectTo(obj, toCanvasIndex);
    } else {
      objects.splice(fromCanvasIndex, 1);
      objects.splice(toCanvasIndex, 0, obj);
    }

    canvas.renderAll();
    onStateChange?.();
    this.syncLayers(canvas);
  }
}
