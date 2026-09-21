import { Canvas } from 'fabric';
import { CanvasTool, McpDrawingAction, McpDrawingPlan } from './types';
import {
  FabricCore,
  LayersEngine,
  ExportEngine,
  McpEngine,
  ShapeBuilderEngine,
} from './engine';

/**
 * Unified Facade for the 2D Whiteboard & MCP Canvas Engine.
 * Composes dedicated engines:
 * - FabricCore: Lifecycle, Viewport Zoom/Pan, Event Handling, History Undo/Redo
 * - LayersEngine: Realtime Hierarchy, Stacking Z-Index, Visibility, Lock, Rename
 * - ExportEngine: Multi-format (PNG, JPG, SVG), Scope Crop, 4K Scaling, Image Import
 * - McpEngine: AI Tool-calling Drawing Executor & Animated Step Runner
 */
class FabricCanvasService {
  private core = new FabricCore();

  public get canvas(): Canvas | null {
    return this.core.getCanvas();
  }

  // ── Lifecycle & Viewport ──

  public init(canvasEl: HTMLCanvasElement, width: number, height: number) {
    this.core.init(canvasEl, width, height);
  }

  public resize(width: number, height: number) {
    this.core.resize(width, height);
  }

  public destroy() {
    this.stopAiDrawing();
    this.core.destroy();
  }

  public setSpacePressed(pressed: boolean) {
    this.core.setSpacePressed(pressed);
  }

  public updateTool(tool: CanvasTool) {
    this.core.updateTool(tool);
  }

  public updateStyles() {
    this.core.updateStyles();
  }

  public zoomIn() {
    this.core.zoomIn();
  }

  public zoomOut() {
    this.core.zoomOut();
  }

  public resetZoom() {
    this.core.resetZoom();
  }

  // ── History & Selection ──

  public hasClipboard(): boolean {
    return this.core.hasClipboard();
  }

  public async copy(): Promise<boolean> {
    return this.core.copy();
  }

  public async paste(): Promise<boolean> {
    return this.core.paste();
  }

  public async duplicate(): Promise<boolean> {
    return this.core.duplicate();
  }

  public undo() {
    this.core.undo();
  }

  public redo() {
    this.core.redo();
  }

  public clear() {
    this.core.clear();
  }

  public deleteSelected() {
    this.core.deleteSelected();
  }

  public isEditingText(): boolean {
    return this.core.isEditingText();
  }

  // ── Layers & Object Hierarchy ──

  public syncLayers() {
    LayersEngine.syncLayers(this.canvas);
  }

  public selectLayer(
    id: string,
    options: { ctrlKey?: boolean; shiftKey?: boolean; metaKey?: boolean } = {}
  ) {
    LayersEngine.selectLayer(this.canvas, id, options);
  }

  public groupSelected() {
    LayersEngine.groupSelected(this.canvas, () => this.core.saveState());
  }

  public ungroupSelected() {
    LayersEngine.ungroupSelected(this.canvas, () => this.core.saveState());
  }

  public toggleLayerVisibility(id: string) {
    LayersEngine.toggleLayerVisibility(this.canvas, id, () => this.core.saveState());
  }

  public toggleLayerLock(id: string) {
    LayersEngine.toggleLayerLock(this.canvas, id, () => this.core.saveState());
  }

  public moveLayerUp(id: string) {
    LayersEngine.moveLayerUp(this.canvas, id, () => this.core.saveState());
  }

  public moveLayerDown(id: string) {
    LayersEngine.moveLayerDown(this.canvas, id, () => this.core.saveState());
  }

  public deleteLayer(id: string) {
    LayersEngine.deleteLayer(this.canvas, id, () => this.core.saveState());
  }

  public renameLayer(id: string, newName: string) {
    LayersEngine.renameLayer(this.canvas, id, newName, () => this.core.saveState());
  }

  public reorderLayer(fromLayerIndex: number, toLayerIndex: number) {
    LayersEngine.reorderLayer(this.canvas, fromLayerIndex, toLayerIndex, () => this.core.saveState());
  }

  // ── Shape Builder (Pecah Menjadi Objek Kecil & Union) ──

  public splitIntoFragments(): boolean {
    return ShapeBuilderEngine.splitIntoFragments(this.canvas, () => {
      this.core.saveState();
      this.syncLayers();
    });
  }

  public unionSelected(): boolean {
    return ShapeBuilderEngine.unionSelected(this.canvas, () => {
      this.core.saveState();
      this.syncLayers();
    });
  }

  // ── Image Import & Exports ──

  public async addImageFromDataUrl(dataUrl: string): Promise<boolean> {
    return ExportEngine.addImageFromDataUrl(this.canvas, dataUrl, () => {
      this.core.saveState();
      this.syncLayers();
    });
  }

  public getExportScopeInfo() {
    return ExportEngine.getExportScopeInfo(this.canvas);
  }

  public getCanvasDimensions() {
    return ExportEngine.getCanvasDimensions(this.canvas);
  }

  public exportCanvas(options: {
    format: 'png' | 'jpeg' | 'svg';
    scope?: 'selection' | 'canvas';
    width?: number;
    height?: number;
    multiplier?: number;
    background: 'transparent' | 'white' | 'dark';
    quality?: number;
  }): string {
    return ExportEngine.exportCanvas(this.canvas, options);
  }

  public exportImage(format: 'png' | 'svg' = 'png'): string {
    return ExportEngine.exportImage(this.canvas, format);
  }

  // ── MCP AI Tool Calling Engine ──

  public async executeAction(action: McpDrawingAction): Promise<void> {
    return McpEngine.executeAction(this.canvas, action, () => {
      this.core.saveState();
      this.syncLayers();
    });
  }

  public async executePlanAnimated(plan: McpDrawingPlan): Promise<void> {
    return McpEngine.executePlanAnimated(this.canvas, plan, () => {
      this.core.saveState();
      this.syncLayers();
    });
  }

  public stopAiDrawing() {
    McpEngine.stopAiDrawing();
  }
}

export const fabricService = new FabricCanvasService();
export default fabricService;
