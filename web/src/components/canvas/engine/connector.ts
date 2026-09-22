import { Canvas, Circle, FabricObject, Line } from 'fabric';
import { ConnectorRouting } from '../types';

export interface ConnectionPort {
  id: 'top' | 'right' | 'bottom' | 'left';
  x: number;
  y: number;
}

export interface ConnectorRouteData {
  points: Array<{ x: number; y: number }>;
  lastAngle: number;
  firstAngle: number;
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

  // Hanya port sisi perimeter (Top, Right, Bottom, Left).
  // Port center ditiadakan agar connector tidak pernah menusuk/masuk ke dalam bodi objek.
  return [
    { id: 'top', x: left + width / 2, y: top },
    { id: 'right', x: left + width, y: top + height / 2 },
    { id: 'bottom', x: left + width / 2, y: top + height },
    { id: 'left', x: left, y: top + height / 2 },
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

import { PortOverlayEngine } from './portOverlay';

// Indicator Visual Manager (Forwarder ke PortOverlayEngine agar backward compatible dan bebas polusi objek canvas)
export function clearConnectionIndicators(canvas: Canvas) {
  PortOverlayEngine.clearIndicators(canvas);
}

export function showConnectionIndicators(
  canvas: Canvas,
  activePort?: ConnectionPort | null,
  targetObj?: FabricObject | null
) {
  PortOverlayEngine.showIndicators(canvas, activePort, targetObj);
}

/**
 * Returns the normal direction vector [dx, dy] leaving from a port.
 */
export function getPortDirection(portId?: string): { dx: number; dy: number } {
  switch (portId) {
    case 'top':
      return { dx: 0, dy: -1 };
    case 'bottom':
      return { dx: 0, dy: 1 };
    case 'left':
      return { dx: -1, dy: 0 };
    case 'right':
      return { dx: 1, dy: 0 };
    default:
      return { dx: 0, dy: 0 };
  }
}

interface BoundingRect {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

function getObjectObstacleBox(obj?: FabricObject, pad = 16): BoundingRect | null {
  if (!obj || typeof obj.getBoundingRect !== 'function') return null;
  const bound = obj.getBoundingRect();
  return {
    left: bound.left - pad,
    right: bound.left + bound.width + pad,
    top: bound.top - pad,
    bottom: bound.top + bound.height + pad,
  };
}

/**
 * Calculates smart orthogonal waypoints between two points with port directions.
 * Produces clean 90-degree elbow route strictly avoiding entering shapes' bodies.
 */
export function calculateOrthogonalRoute(
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  fromPortId?: string,
  toPortId?: string,
  offset = 24,
  fromObj?: FabricObject,
  toObj?: FabricObject
): Array<{ x: number; y: number }> {
  const fromDir = getPortDirection(fromPortId);
  const toDir = getPortDirection(toPortId);

  const fromBox = getObjectObstacleBox(fromObj, 16);
  const toBox = getObjectObstacleBox(toObj, 16);

  // If no ports are specified (free point to free point), make standard elbow
  if (fromDir.dx === 0 && fromDir.dy === 0 && toDir.dx === 0 && toDir.dy === 0) {
    const dx = Math.abs(endX - startX);
    const dy = Math.abs(endY - startY);
    if (dx >= dy) {
      const midX = (startX + endX) / 2;
      return [
        { x: startX, y: startY },
        { x: midX, y: startY },
        { x: midX, y: endY },
        { x: endX, y: endY },
      ];
    } else {
      const midY = (startY + endY) / 2;
      return [
        { x: startX, y: startY },
        { x: startX, y: midY },
        { x: endX, y: midY },
        { x: endX, y: endY },
      ];
    }
  }

  // Initial stub points extending out of port
  const pStartStub = {
    x: startX + fromDir.dx * offset,
    y: startY + fromDir.dy * offset,
  };
  const pEndStub = {
    x: endX + toDir.dx * offset,
    y: endY + toDir.dy * offset,
  };

  const points: Array<{ x: number; y: number }> = [{ x: startX, y: startY }];
  if (fromDir.dx !== 0 || fromDir.dy !== 0) {
    points.push(pStartStub);
  }

  // Determine middle path connecting pStartStub to pEndStub with obstacle avoidance
  const isHorizontalFrom = fromDir.dx !== 0;
  const isHorizontalTo = toDir.dx !== 0;

  if (isHorizontalFrom && isHorizontalTo) {
    // Both horizontal (e.g. Right to Left, Right to Right, Left to Left)
    const isFacing = (fromDir.dx > 0 && pStartStub.x < pEndStub.x) || (fromDir.dx < 0 && pStartStub.x > pEndStub.x);
    if (isFacing) {
      const midX = (pStartStub.x + pEndStub.x) / 2;
      points.push({ x: midX, y: pStartStub.y });
      points.push({ x: midX, y: pEndStub.y });
    } else {
      // Loop around (target berada di belakang port keluar)
      // Jalur wajib memutar keluar tanpa memotong bodi objek!
      const detourX = fromDir.dx > 0
        ? Math.max(pStartStub.x, pEndStub.x, fromBox ? fromBox.right : pStartStub.x)
        : Math.min(pStartStub.x, pEndStub.x, fromBox ? fromBox.left : pStartStub.x);

      // Pilih jalur bypass Y (atas atau bawah objek) agar tidak menabrak bodi objek
      let detourY = pStartStub.y;
      if (fromBox && pStartStub.y >= fromBox.top && pStartStub.y <= fromBox.bottom) {
        const goTop = Math.abs(pStartStub.y - fromBox.top) < Math.abs(pStartStub.y - fromBox.bottom);
        detourY = goTop ? fromBox.top : fromBox.bottom;
      }

      points.push({ x: detourX, y: pStartStub.y });
      if (detourY !== pStartStub.y) {
        points.push({ x: detourX, y: detourY });
      }
      points.push({ x: detourX, y: pEndStub.y });
    }
  } else if (!isHorizontalFrom && !isHorizontalTo) {
    // Both vertical (e.g. Bottom to Top, Bottom to Bottom, Top to Top)
    const isFacing = (fromDir.dy > 0 && pStartStub.y < pEndStub.y) || (fromDir.dy < 0 && pStartStub.y > pEndStub.y);
    if (isFacing) {
      const midY = (pStartStub.y + pEndStub.y) / 2;
      points.push({ x: pStartStub.x, y: midY });
      points.push({ x: pEndStub.x, y: midY });
    } else {
      // Loop around vertical
      const detourY = fromDir.dy > 0
        ? Math.max(pStartStub.y, pEndStub.y, fromBox ? fromBox.bottom : pStartStub.y)
        : Math.min(pStartStub.y, pEndStub.y, fromBox ? fromBox.top : pStartStub.y);

      let detourX = pStartStub.x;
      if (fromBox && pStartStub.x >= fromBox.left && pStartStub.x <= fromBox.right) {
        const goLeft = Math.abs(pStartStub.x - fromBox.left) < Math.abs(pStartStub.x - fromBox.right);
        detourX = goLeft ? fromBox.left : fromBox.right;
      }

      points.push({ x: pStartStub.x, y: detourY });
      if (detourX !== pStartStub.x) {
        points.push({ x: detourX, y: detourY });
      }
      points.push({ x: pEndStub.x, y: detourY });
    }
  } else if (isHorizontalFrom && !isHorizontalTo) {
    // Horizontal start to Vertical end (Right/Left to Top/Bottom)
    // Cek apakah pEndStub.x memotong bodi startBox jika ditarik langsung
    if (fromBox && pEndStub.x >= fromBox.left && pEndStub.x <= fromBox.right) {
      const detourX = fromDir.dx > 0 ? fromBox.right : fromBox.left;
      points.push({ x: detourX, y: pStartStub.y });
      const detourY = toDir.dy > 0 ? Math.max(pEndStub.y, toBox ? toBox.bottom : pEndStub.y) : Math.min(pEndStub.y, toBox ? toBox.top : pEndStub.y);
      points.push({ x: detourX, y: detourY });
      points.push({ x: pEndStub.x, y: detourY });
    } else {
      points.push({ x: pEndStub.x, y: pStartStub.y });
    }
  } else {
    // Vertical start to Horizontal end (Top/Bottom to Right/Left)
    if (fromBox && pEndStub.y >= fromBox.top && pEndStub.y <= fromBox.bottom) {
      const detourY = fromDir.dy > 0 ? fromBox.bottom : fromBox.top;
      points.push({ x: pStartStub.x, y: detourY });
      const detourX = toDir.dx > 0 ? Math.max(pEndStub.x, toBox ? toBox.right : pEndStub.x) : Math.min(pEndStub.x, toBox ? toBox.left : pEndStub.x);
      points.push({ x: detourX, y: detourY });
      points.push({ x: detourX, y: pEndStub.y });
    } else {
      points.push({ x: pStartStub.x, y: pEndStub.y });
    }
  }

  if (toDir.dx !== 0 || toDir.dy !== 0) {
    points.push(pEndStub);
  }
  points.push({ x: endX, y: endY });

  // Clean redundant collinear points
  const simplified: Array<{ x: number; y: number }> = [points[0]];
  for (let i = 1; i < points.length; i++) {
    const prev = simplified[simplified.length - 1];
    const curr = points[i];
    // Skip duplicate point
    if (Math.abs(prev.x - curr.x) < 0.5 && Math.abs(prev.y - curr.y) < 0.5) {
      continue;
    }
    // Check collinearity with next point if exists
    if (i < points.length - 1) {
      const next = points[i + 1];
      const isCollinearX = Math.abs(prev.x - curr.x) < 0.5 && Math.abs(curr.x - next.x) < 0.5;
      const isCollinearY = Math.abs(prev.y - curr.y) < 0.5 && Math.abs(curr.y - next.y) < 0.5;
      if (isCollinearX || isCollinearY) {
        continue;
      }
    }
    simplified.push(curr);
  }

  return simplified;
}

/**
 * Calculates smooth curved waypoints between two ports or points (bezier style).
 * Produces clean and elegant natural arc curves.
 */
export function calculateCurvedRoute(
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  fromPortId?: string,
  toPortId?: string
): Array<{ x: number; y: number }> {
  const fromDir = getPortDirection(fromPortId);
  const toDir = getPortDirection(toPortId);

  const dx = endX - startX;
  const dy = endY - startY;
  const dist = Math.hypot(dx, dy);

  // Jika tanpa arah port spesifik (misal titik bebas), berikan lengkungan busur alami
  if (fromDir.dx === 0 && fromDir.dy === 0 && toDir.dx === 0 && toDir.dy === 0) {
    if (dist < 1) {
      return [{ x: startX, y: startY }, { x: endX, y: endY }];
    }
    // Normal vector tegak lurus
    const nx = -dy / dist;
    const ny = dx / dist;
    const arcHeight = Math.max(25, Math.min(120, dist * 0.28));

    const cp1 = {
      x: startX + dx * 0.25 + nx * arcHeight,
      y: startY + dy * 0.25 + ny * arcHeight,
    };
    const cp2 = {
      x: startX + dx * 0.75 + nx * arcHeight,
      y: startY + dy * 0.75 + ny * arcHeight,
    };

    const points: Array<{ x: number; y: number }> = [];
    const steps = 32;
    for (let i = 0; i <= steps; i++) {
      const t = i / steps;
      const inv = 1 - t;
      const x =
        inv * inv * inv * startX +
        3 * inv * inv * t * cp1.x +
        3 * inv * t * t * cp2.x +
        t * t * t * endX;
      const y =
        inv * inv * inv * startY +
        3 * inv * inv * t * cp1.y +
        3 * inv * t * t * cp2.y +
        t * t * t * endY;
      points.push({ x, y });
    }
    return points;
  }

  // Jika dengan port arah keluar/masuk
  const curvature = Math.max(30, Math.min(180, dist * 0.45));
  const cp1 = {
    x: startX + (fromDir.dx !== 0 || fromDir.dy !== 0 ? fromDir.dx * curvature : dx * 0.35),
    y: startY + (fromDir.dx !== 0 || fromDir.dy !== 0 ? fromDir.dy * curvature : dy * 0.35),
  };
  const cp2 = {
    x: endX + (toDir.dx !== 0 || toDir.dy !== 0 ? toDir.dx * curvature : -dx * 0.35),
    y: endY + (toDir.dx !== 0 || toDir.dy !== 0 ? toDir.dy * curvature : -dy * 0.35),
  };

  // Sample bezier curve to 32 smooth linear waypoints
  const points: Array<{ x: number; y: number }> = [];
  const steps = 32;
  for (let i = 0; i <= steps; i++) {
    const t = i / steps;
    const inv = 1 - t;
    const x =
      inv * inv * inv * startX +
      3 * inv * inv * t * cp1.x +
      3 * inv * t * t * cp2.x +
      t * t * t * endX;
    const y =
      inv * inv * inv * startY +
      3 * inv * inv * t * cp1.y +
      3 * inv * t * t * cp2.y +
      t * t * t * endY;
    points.push({ x, y });
  }

  return points;
}

/**
 * Computes waypoints and arrow angles based on the routing style.
 */
export function computeConnectorPoints(
  startX: number,
  startY: number,
  endX: number,
  endY: number,
  routing: ConnectorRouting = 'orthogonal',
  fromPortId?: string,
  toPortId?: string,
  fromObj?: FabricObject,
  toObj?: FabricObject
): ConnectorRouteData {
  let points: Array<{ x: number; y: number }> = [];

  if (routing === 'straight') {
    points = [
      { x: startX, y: startY },
      { x: endX, y: endY },
    ];
  } else if (routing === 'curved') {
    points = calculateCurvedRoute(startX, startY, endX, endY, fromPortId, toPortId);
  } else {
    // 'orthogonal' dengan obstacle avoidance
    points = calculateOrthogonalRoute(startX, startY, endX, endY, fromPortId, toPortId, 24, fromObj, toObj);
  }

  const firstSegment = {
    dx: points[1].x - points[0].x,
    dy: points[1].y - points[0].y,
  };
  const lastSegment = {
    dx: points[points.length - 1].x - points[points.length - 2].x,
    dy: points[points.length - 1].y - points[points.length - 2].y,
  };

  return {
    points,
    firstAngle: Math.atan2(firstSegment.dy, firstSegment.dx),
    lastAngle: Math.atan2(lastSegment.dy, lastSegment.dx),
  };
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
      const routing: ConnectorRouting = conn.routing || 'orthogonal';
      const route = computeConnectorPoints(
        startX,
        startY,
        endX,
        endY,
        routing,
        conn.fromPortId,
        conn.toPortId,
        conn.fromObj,
        conn.toObj
      );

      conn.computedPoints = route.points;
      conn.lastAngle = route.lastAngle;
      conn.firstAngle = route.firstAngle;

      if (obj instanceof Line || obj.type === 'line') {
        obj.set({
          x1: startX,
          y1: startY,
          x2: endX,
          y2: endY,
        });
        obj.dirty = true;
        obj.setCoords();
      }
    }
  });

  canvas.requestRenderAll();
}
