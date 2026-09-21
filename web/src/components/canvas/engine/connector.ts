import { Canvas, Circle, FabricObject, Line } from 'fabric';

export interface ConnectionPort {
  id: 'top' | 'right' | 'bottom' | 'left' | 'center';
  x: number;
  y: number;
}

/**
 * Calculates the connection anchor ports (Top, Right, Bottom, Left, Center)
 * for any given shape in global canvas coordinates.
 */
export function getObjectConnectionPorts(obj: FabricObject): ConnectionPort[] {
  if (!obj || typeof obj.getBoundingRect !== 'function') return [];

  const bound = obj.getBoundingRect();
  const left = bound.left;
  const top = bound.top;
  const width = bound.width;
  const height = bound.height;

  return [
    { id: 'top', x: left + width / 2, y: top },
    { id: 'right', x: left + width, y: top + height / 2 },
    { id: 'bottom', x: left + width / 2, y: top + height },
    { id: 'left', x: left, y: top + height / 2 },
    { id: 'center', x: left + width / 2, y: top + height / 2 },
  ];
}

/**
 * Finds the closest connection port to the pointer within snap distance.
 */
export function findClosestConnectionPort(
  canvas: Canvas,
  pointer: { x: number; y: number },
  excludeObj?: FabricObject | null,
  maxDistance = 24
): { port: ConnectionPort; obj: FabricObject } | null {
  let closest: { port: ConnectionPort; obj: FabricObject; dist: number } | null = null;

  canvas.forEachObject((obj: any) => {
    // Exclude indicators, lines, arrows, or current drawing object
    if (
      obj === excludeObj ||
      obj.isConnectionIndicator ||
      obj.type === 'line' ||
      obj.isConnectorLine ||
      obj.type === 'activeSelection'
    ) {
      return;
    }

    const ports = getObjectConnectionPorts(obj);
    for (const port of ports) {
      const dist = Math.hypot(pointer.x - port.x, pointer.y - port.y);
      if (dist <= maxDistance) {
        if (!closest || dist < closest.dist) {
          closest = { port, obj, dist };
        }
      }
    }
  });

  const result = closest as { port: ConnectionPort; obj: FabricObject; dist: number } | null;
  return result ? { port: result.port, obj: result.obj } : null;
}

// Indicator Visual Manager
let activeIndicators: FabricObject[] = [];

export function clearConnectionIndicators(canvas: Canvas) {
  if (activeIndicators.length > 0) {
    activeIndicators.forEach((ind) => canvas.remove(ind));
    activeIndicators = [];
    canvas.requestRenderAll();
  }
}

export function showConnectionIndicators(
  canvas: Canvas,
  activePort?: ConnectionPort | null,
  targetObj?: FabricObject | null
) {
  clearConnectionIndicators(canvas);

  const objectsToHighlight: FabricObject[] = targetObj
    ? [targetObj]
    : canvas.getObjects().filter((o: any) => !o.isConnectionIndicator && o.type !== 'line');

  objectsToHighlight.forEach((obj) => {
    const ports = getObjectConnectionPorts(obj);
    ports.forEach((p) => {
      const isCurrentActive =
        activePort &&
        Math.hypot(activePort.x - p.x, activePort.y - p.y) < 4;

      const dot = new Circle({
        left: p.x,
        top: p.y,
        originX: 'center',
        originY: 'center',
        radius: isCurrentActive ? 6 : 4,
        fill: isCurrentActive ? '#7BDFF2' : 'rgba(123, 223, 242, 0.4)',
        stroke: isCurrentActive ? '#FFFFFF' : '#7BDFF2',
        strokeWidth: isCurrentActive ? 2 : 1,
        selectable: false,
        evented: false,
        isConnectionIndicator: true,
      } as any);

      canvas.add(dot);
      activeIndicators.push(dot);
    });
  });

  canvas.requestRenderAll();
}

/**
 * Updates all connected lines & arrows dynamically when an object is moved.
 */
export function updateConnectedLines(canvas: Canvas, movedObj: FabricObject) {
  canvas.forEachObject((obj: any) => {
    if (!obj.connectorData) return;

    const conn = obj.connectorData;
    let needsUpdate = false;

    let startX = conn.startX;
    let startY = conn.startY;
    let endX = conn.endX;
    let endY = conn.endY;

    if (conn.fromObj === movedObj && conn.fromPortId) {
      const ports = getObjectConnectionPorts(movedObj);
      const port = ports.find((p) => p.id === conn.fromPortId) || ports[0];
      if (port) {
        startX = port.x;
        startY = port.y;
        conn.startX = startX;
        conn.startY = startY;
        needsUpdate = true;
      }
    }

    if (conn.toObj === movedObj && conn.toPortId) {
      const ports = getObjectConnectionPorts(movedObj);
      const port = ports.find((p) => p.id === conn.toPortId) || ports[0];
      if (port) {
        endX = port.x;
        endY = port.y;
        conn.endX = endX;
        conn.endY = endY;
        needsUpdate = true;
      }
    }

    if (needsUpdate) {
      if (obj instanceof Line || obj.type === 'line') {
        obj.set({
          x1: startX,
          y1: startY,
          x2: endX,
          y2: endY,
        });
        obj.setCoords();
      }
    }
  });

  canvas.requestRenderAll();
}
