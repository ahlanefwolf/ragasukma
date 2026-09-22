import {
  Line,
  Point,
  Control,
  util,
} from 'fabric';
import { snapCoord } from './helpers';
import { computeConnectorPoints, findClosestConnectionPort } from '../connector';
import {
  renderZoomAdaptiveCircle,
  getLineEndpointPosition,
  getLineEndpointScenePosition,
} from './endpointControls';

// ── Professional Line & Arrow Endpoint Controls ──
export function createLineControls() {
  return {
    p1: new Control({
      x: 0,
      y: 0,
      positionHandler(dim, finalMatrix, fabricItem) {
        return getLineEndpointPosition(fabricItem as Line, 'p1');
      },
      actionHandler(eventData, transform, x, y) {
        const line = transform.target as Line;
        const canvas = line.canvas;
        const p2Scene = getLineEndpointScenePosition(line, 'p2');

        let targetX = snapCoord(x);
        let targetY = snapCoord(y);

        const isConnector = !!((line as any).isArrow || (line as any).isConnectorLine || (line as any).connectorData);

        // Magnetic port snap hanya untuk Connector
        let startPortId: string | undefined = (line as any).connectorData?.fromPortId;
        let startObj: any = (line as any).connectorData?.fromObj;
        if (isConnector && canvas) {
          const snap = findClosestConnectionPort(canvas, { x: targetX, y: targetY }, (line as any).connectorData?.toObj, 24);
          if (snap) {
            targetX = snap.port.x;
            targetY = snap.port.y;
            startPortId = snap.port.id;
            startObj = snap.obj;
          } else {
            startPortId = undefined;
            startObj = undefined;
          }
        }

        if (eventData?.shiftKey) {
          const dx = targetX - p2Scene.x;
          const dy = targetY - p2Scene.y;
          const dist = Math.hypot(dx, dy);
          const rawAngle = Math.atan2(dy, dx);
          const snapAngle = Math.round(rawAngle / (Math.PI / 4)) * (Math.PI / 4);
          targetX = p2Scene.x + dist * Math.cos(snapAngle);
          targetY = p2Scene.y + dist * Math.sin(snapAngle);
        }

        line.set({
          x1: targetX,
          y1: targetY,
          x2: p2Scene.x,
          y2: p2Scene.y,
          left: (targetX + p2Scene.x) / 2,
          top: (targetY + p2Scene.y) / 2,
          scaleX: 1,
          scaleY: 1,
          angle: 0,
        });

        if (isConnector) {
          // Redrawing rute (orthogonal / curved / straight) dengan obstacle avoidance saat handle connector dipindah
          const connData = (line as any).connectorData;
          const routing = connData?.routing || 'orthogonal';
          const toObj = connData?.toObj;
          const route = computeConnectorPoints(
            targetX,
            targetY,
            p2Scene.x,
            p2Scene.y,
            routing,
            startPortId,
            connData?.toPortId,
            startObj,
            toObj
          );

          (line as any).connectorData = {
            ...connData,
            fromObj: startObj,
            fromPortId: startPortId,
            startX: targetX,
            startY: targetY,
            endX: p2Scene.x,
            endY: p2Scene.y,
            routing,
            computedPoints: route.points,
            lastAngle: route.lastAngle,
            firstAngle: route.firstAngle,
          };
        }

        line.dirty = true;
        line.setCoords();
        return true;
      },
      cursorStyleHandler: () => 'crosshair',
      actionName: 'modifyLineStart',
      render: renderZoomAdaptiveCircle,
    }),
    p2: new Control({
      x: 0,
      y: 0,
      positionHandler(dim, finalMatrix, fabricItem) {
        return getLineEndpointPosition(fabricItem as Line, 'p2');
      },
      actionHandler(eventData, transform, x, y) {
        const line = transform.target as Line;
        const canvas = line.canvas;
        const p1Scene = getLineEndpointScenePosition(line, 'p1');

        let targetX = snapCoord(x);
        let targetY = snapCoord(y);

        const isConnector = !!((line as any).isArrow || (line as any).isConnectorLine || (line as any).connectorData);

        // Magnetic port snap hanya untuk Connector
        let endPortId: string | undefined = (line as any).connectorData?.toPortId;
        let endObj: any = (line as any).connectorData?.toObj;
        if (isConnector && canvas) {
          const snap = findClosestConnectionPort(canvas, { x: targetX, y: targetY }, (line as any).connectorData?.fromObj, 24);
          if (snap) {
            targetX = snap.port.x;
            targetY = snap.port.y;
            endPortId = snap.port.id;
            endObj = snap.obj;
          } else {
            endPortId = undefined;
            endObj = undefined;
          }
        }

        if (eventData?.shiftKey) {
          const dx = targetX - p1Scene.x;
          const dy = targetY - p1Scene.y;
          const dist = Math.hypot(dx, dy);
          const rawAngle = Math.atan2(dy, dx);
          const snapAngle = Math.round(rawAngle / (Math.PI / 4)) * (Math.PI / 4);
          targetX = p1Scene.x + dist * Math.cos(snapAngle);
          targetY = p1Scene.y + dist * Math.sin(snapAngle);
        }

        line.set({
          x1: p1Scene.x,
          y1: p1Scene.y,
          x2: targetX,
          y2: targetY,
          left: (p1Scene.x + targetX) / 2,
          top: (p1Scene.y + targetY) / 2,
          scaleX: 1,
          scaleY: 1,
          angle: 0,
        });

        if (isConnector) {
          // Redrawing rute (orthogonal / curved / straight) dengan obstacle avoidance saat handle connector dipindah
          const connData = (line as any).connectorData;
          const routing = connData?.routing || 'orthogonal';
          const fromObj = connData?.fromObj;
          const route = computeConnectorPoints(
            p1Scene.x,
            p1Scene.y,
            targetX,
            targetY,
            routing,
            connData?.fromPortId,
            endPortId,
            fromObj,
            endObj
          );

          (line as any).connectorData = {
            ...connData,
            toObj: endObj,
            toPortId: endPortId,
            startX: p1Scene.x,
            startY: p1Scene.y,
            endX: targetX,
            endY: targetY,
            routing,
            computedPoints: route.points,
            lastAngle: route.lastAngle,
            firstAngle: route.firstAngle,
          };
        }

        line.dirty = true;
        line.setCoords();
        return true;
      },
      cursorStyleHandler: () => 'crosshair',
      actionName: 'modifyLineEnd',
      render: renderZoomAdaptiveCircle,
    }),
  };
}

export function setupLineControls(line: Line) {
  if (!line) return;
  line.controls = createLineControls();
  line.hasBorders = false;
  line.padding = 6;
}
