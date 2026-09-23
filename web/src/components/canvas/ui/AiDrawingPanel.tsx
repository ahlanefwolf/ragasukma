import { Component, For, Show, createSignal } from 'solid-js';
import {
  Play,
  Square as StopIcon,
  Wand2,
  Bot,
  Zap,
  Code2,
  CheckCircle2,
  Clock,
  Loader2,
  FileCode,
} from 'lucide-solid';
import {
  activeAiPlan,
  isAiDrawing,
  currentAiStepIndex,
  aiSpeedMultiplier,
  setAiSpeedMultiplier,
  DEMO_AI_PLANS,
} from '../canvasStore';
import { fabricService } from '../fabricService';
import { McpDrawingPlan } from '../types';

interface AiDrawingPanelProps {
  showToast: (msg: string) => void;
}

export const AiDrawingPanel: Component<AiDrawingPanelProps> = (props) => {
  const [customJson, setCustomJson] = createSignal('');
  const [selectedDemoIndex, setSelectedDemoIndex] = createSignal(0);

  const handleRunDemo = (plan: McpDrawingPlan) => {
    props.showToast(`🤖 AI Menggambar: "${plan.title}"...`);
    fabricService.executePlanAnimated(plan);
  };

  const handleRunCustomJson = () => {
    try {
      const parsed = JSON.parse(customJson());
      if (parsed.steps && Array.isArray(parsed.steps)) {
        props.showToast('🚀 Menjalankan MCP Plan custom...');
        fabricService.executePlanAnimated(parsed);
      } else if (parsed.type) {
        props.showToast('⚡ Menjalankan single MCP action...');
        fabricService.executeAction(parsed);
      } else {
        props.showToast('⚠️ Format JSON tidak valid (harus McpDrawingPlan atau McpDrawingAction)');
      }
    } catch (err: any) {
      props.showToast(`⚠️ JSON Parse Error: ${err.message}`);
    }
  };

  return (
    <div class="bg-theme-card border border-[#B08A4A]/40 p-4 space-y-4 shadow-md">
      {/* Header */}
      <div class="flex items-center justify-between border-b border-theme-border/70 pb-3">
        <div class="flex items-center gap-2">
          <div class="p-1.5 bg-[#B08A4A]/15 text-[#B08A4A] border border-[#B08A4A]/30">
            <Bot size={18} />
          </div>
          <div>
            <h3 class="text-sm font-bold text-theme-text flex items-center gap-2">
              AI MCP Live Drawing Engine
              <span class="text-[10px] font-mono px-2 py-0.5 bg-theme-cyan/20 text-[#29483A] border border-theme-cyan/40 font-bold">
                Pondasi MCP
              </span>
            </h3>
            <p class="text-xs text-theme-text/70">
              Visualisasi langsung saat AI menggambar diagram, flowchart, atau mindmap langkah-demi-langkah.
            </p>
          </div>
        </div>

        {/* Speed Multiplier */}
        <div class="flex items-center gap-1 bg-theme-surface p-1 border border-theme-border/60 text-xs">
          <span class="text-[10px] font-mono px-1 text-theme-text/70 font-bold">SPEED:</span>
          <For each={[1, 2, 4]}>
            {(spd) => (
              <button
                type="button"
                onClick={() => setAiSpeedMultiplier(spd)}
                class={`px-2 py-0.5 font-mono font-bold transition-all ${
                  aiSpeedMultiplier() === spd
                    ? 'bg-[#B08A4A] text-theme-card'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                {spd}x
              </button>
            )}
          </For>
        </div>
      </div>

      {/* Demo Plans Quick Runner */}
      <div class="space-y-2">
        <span class="text-xs font-mono font-bold text-[#876447] uppercase tracking-wider flex items-center gap-1.5">
          <Wand2 size={13} />
          Preset Template Demo AI:
        </span>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
          <For each={DEMO_AI_PLANS}>
            {(plan, idx) => (
              <div class="p-3 bg-theme-surface border border-theme-border hover:border-[#B08A4A]/50 transition-all flex flex-col justify-between gap-2">
                <div>
                  <div class="text-xs font-bold text-theme-text">{plan.title}</div>
                  <div class="text-[11px] font-mono text-theme-text/60 mt-0.5">
                    {plan.steps.length} Langkah Terstruktur
                  </div>
                </div>
                <button
                  type="button"
                  disabled={isAiDrawing()}
                  onClick={() => handleRunDemo(plan)}
                  class="w-full py-1.5 text-xs font-bold bg-[#B08A4A]/15 hover:bg-[#B08A4A] text-[#876447] hover:text-theme-card border border-[#B08A4A]/30 transition-all flex items-center justify-center gap-1.5 disabled:opacity-40"
                >
                  <Play size={13} />
                  <span>Simulasi Live Draw</span>
                </button>
              </div>
            )}
          </For>
        </div>
      </div>

      {/* Active Step-by-Step Progress Display */}
      <Show when={activeAiPlan()}>
        {(plan) => (
          <div class="bg-theme-surface border border-theme-border p-3 space-y-2.5">
            <div class="flex items-center justify-between text-xs">
              <span class="font-bold text-theme-text flex items-center gap-2">
                <FileCode size={14} class="text-theme-cyan" />
                {plan().title}
              </span>
              <Show when={isAiDrawing()}>
                <button
                  type="button"
                  onClick={() => fabricService.stopAiDrawing()}
                  class="px-2 py-0.5 text-xs font-bold bg-[#A84B4B]/15 text-[#A84B4B] hover:bg-[#A84B4B] hover:text-theme-card border border-[#A84B4B]/30 flex items-center gap-1 transition-all"
                >
                  <StopIcon size={12} />
                  <span>Hentikan</span>
                </button>
              </Show>
            </div>

            {/* Steps Timeline */}
            <div class="space-y-1.5 max-h-48 overflow-y-auto pr-1">
              <For each={plan().steps}>
                {(step, idx) => {
                  const isCurrent = () => currentAiStepIndex() === idx();
                  const isPassed = () => currentAiStepIndex() > idx() || (!isAiDrawing() && currentAiStepIndex() >= 0);

                  return (
                    <div
                      class={`p-2 text-xs flex items-center justify-between border transition-all ${
                        isCurrent() && isAiDrawing()
                          ? 'bg-[#B08A4A]/15 border-[#B08A4A] text-[#29483A] shadow-sm font-semibold'
                          : isPassed()
                          ? 'bg-theme-card/60 border-theme-border/40 text-theme-text/60'
                          : 'bg-theme-card border-theme-border text-theme-text/80'
                      }`}
                    >
                      <div class="flex items-center gap-2">
                        <Show
                          when={isCurrent() && isAiDrawing()}
                          fallback={
                            <Show
                              when={isPassed()}
                              fallback={<Clock size={14} class="text-theme-text/40" />}
                            >
                              <CheckCircle2 size={14} class="text-theme-cyan" />
                            </Show>
                          }
                        >
                          <Loader2 size={14} class="text-[#B08A4A] animate-spin" />
                        </Show>
                        <span class="font-mono font-bold text-[11px] text-theme-text/50">
                          #{idx() + 1}
                        </span>
                        <span class="font-medium truncate max-w-xs sm:max-w-md">
                          {step.description || step.action.type}
                        </span>
                      </div>
                      <span class="font-mono text-[10px] text-theme-cyan bg-theme-bg px-1.5 py-0.5 border border-theme-border">
                        {step.action.type}
                      </span>
                    </div>
                  );
                }}
              </For>
            </div>
          </div>
        )}
      </Show>

      {/* Raw MCP JSON Input Tester */}
      <details class="bg-theme-surface border border-theme-border p-3 group">
        <summary class="text-xs font-bold font-mono text-theme-text/80 cursor-pointer flex items-center justify-between select-none">
          <span class="flex items-center gap-1.5">
            <Code2 size={14} class="text-theme-cyan" />
            Input Raw MCP Action / Plan JSON
          </span>
          <span class="text-[10px] text-theme-text/50 group-open:rotate-180 transition-transform">
            ▼
          </span>
        </summary>
        <div class="pt-3 space-y-2">
          <textarea
            value={customJson()}
            onInput={(e) => setCustomJson(e.currentTarget.value)}
            rows={4}
            placeholder='{"type": "draw_rect", "x": 100, "y": 100, "width": 150, "height": 80, "stroke": "#596B45", "label": "Hello AI"}'
            class="w-full bg-theme-bg border border-theme-border p-2.5 text-xs font-mono text-theme-text focus:border-[#B08A4A] focus:outline-none"
          />
          <button
            type="button"
            onClick={handleRunCustomJson}
            class="px-4 py-2 text-xs font-bold bg-[#B08A4A] hover:bg-[#876447] text-theme-card border border-[#B08A4A] flex items-center gap-1.5 transition-all shadow-sm"
          >
            <Zap size={14} />
            <span>Jalankan Instruksi MCP</span>
          </button>
        </div>
      </details>
    </div>
  );
};
