import {
  Canvas,
  Rect,
  Circle,
  Line,
  IText,
  Path,
  Group,
} from 'fabric';
import {
  McpDrawingAction,
  McpDrawingPlan,
  McpDrawingStep,
} from '../types';
import {
  strokeColor,
  fillColor,
  strokeWidth,
  fontSize,
  fontFamily,
  activeAiPlan,
  setActiveAiPlan,
  isAiDrawing,
  setIsAiDrawing,
  currentAiStepIndex,
  setCurrentAiStepIndex,
  aiSpeedMultiplier,
} from '../canvasStore';
import { createArrowObject } from './drawing';

export class McpEngine {
  private static aiAbortController: AbortController | null = null;

  public static async executeAction(
    canvas: Canvas | null,
    action: McpDrawingAction,
    onStateChange?: () => void
  ): Promise<void> {
    if (!canvas) return;

    switch (action.type) {
      case 'clear': {
        canvas.clear();
        canvas.backgroundColor = '';
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'set_background': {
        canvas.backgroundColor = action.color;
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'draw_rect': {
        const rect = new Rect({
          left: action.x,
          top: action.y,
          width: action.width,
          height: action.height,
          stroke: action.stroke || strokeColor(),
          fill: action.fill || (fillColor() === 'transparent' ? '' : fillColor()),
          strokeWidth: action.strokeWidth || strokeWidth(),
          strokeUniform: true,
          rx: action.rx || 6,
          ry: action.ry || 6,
          selectable: true,
        });

        if (action.label) {
          const text = new IText(action.label, {
            left: action.x + action.width / 2,
            top: action.y + action.height / 2,
            fontSize: 14,
            fill: action.stroke || '#FFFFFF',
            originX: 'center',
            originY: 'center',
            textAlign: 'center',
          });
          const group = new Group([rect, text], { selectable: true });
          canvas.add(group);
        } else {
          canvas.add(rect);
        }
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'draw_circle': {
        const circle = new Circle({
          left: action.x,
          top: action.y,
          radius: action.radius,
          stroke: action.stroke || strokeColor(),
          fill: action.fill || (fillColor() === 'transparent' ? '' : fillColor()),
          strokeWidth: action.strokeWidth || strokeWidth(),
          strokeUniform: true,
          originX: 'center',
          originY: 'center',
          selectable: true,
        });

        if (action.label) {
          const text = new IText(action.label, {
            left: action.x,
            top: action.y,
            fontSize: 14,
            fill: action.stroke || '#FFFFFF',
            originX: 'center',
            originY: 'center',
            textAlign: 'center',
          });
          const group = new Group([circle, text], { selectable: true });
          canvas.add(group);
        } else {
          canvas.add(circle);
        }
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'draw_line': {
        const line = new Line([action.x1, action.y1, action.x2, action.y2], {
          stroke: action.stroke || strokeColor(),
          strokeWidth: action.strokeWidth || strokeWidth(),
          strokeUniform: true,
          selectable: true,
        });
        canvas.add(line);
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'draw_arrow': {
        const arrow = createArrowObject(
          action.x1,
          action.y1,
          action.x2,
          action.y2,
          action.stroke || strokeColor(),
          action.strokeWidth || strokeWidth(),
          action.label
        );
        canvas.add(arrow);
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'draw_text': {
        const text = new IText(action.text || '', {
          left: action.x,
          top: action.y,
          fontSize: action.fontSize || 16,
          fill: action.fill || strokeColor(),
          fontFamily: action.fontFamily || fontFamily(),
          selectable: true,
        });
        canvas.add(text);
        canvas.renderAll();
        onStateChange?.();
        break;
      }

      case 'draw_path': {
        const path = new Path(action.path, {
          stroke: action.stroke || strokeColor(),
          fill: action.fill || '',
          strokeWidth: action.strokeWidth || strokeWidth(),
          strokeUniform: true,
          selectable: true,
        });
        canvas.add(path);
        canvas.renderAll();
        onStateChange?.();
        break;
      }
    }
  }

  public static async executePlanAnimated(
    canvas: Canvas | null,
    plan: McpDrawingPlan,
    onStateChange?: () => void
  ): Promise<void> {
    if (!canvas) return;

    this.stopAiDrawing();
    this.aiAbortController = new AbortController();
    const signal = this.aiAbortController.signal;

    setActiveAiPlan(plan);
    setIsAiDrawing(true);
    setCurrentAiStepIndex(0);

    try {
      for (let i = 0; i < plan.steps.length; i++) {
        if (signal.aborted) break;

        setCurrentAiStepIndex(i);
        const step = plan.steps[i];
        step.status = 'drawing';

        await this.executeAction(canvas, step.action, onStateChange);

        const delay = Math.max(50, (step.delayMs || 400) / aiSpeedMultiplier());
        await new Promise((resolve) => setTimeout(resolve, delay));

        step.status = 'completed';
      }
    } catch (err) {
      console.error('MCP AI Animation interrupted or failed:', err);
    } finally {
      this.aiAbortController = null;
      setIsAiDrawing(false);
      onStateChange?.();
    }
  }

  public static stopAiDrawing() {
    if (this.aiAbortController) {
      this.aiAbortController.abort();
      this.aiAbortController = null;
    }
    setIsAiDrawing(false);
  }
}
