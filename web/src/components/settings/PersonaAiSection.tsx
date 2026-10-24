import { Component, For, createEffect, createSignal } from 'solid-js';
import { Brain, UserCheck, Cpu, RefreshCw } from 'lucide-solid';
import { antigravityService, codexService, CodexModelItem } from '../../services';
import { SearchableSelect } from '../SearchableSelect';

interface PersonaAiSectionProps {
  assistantName: string;
  setAssistantName: (val: string) => void;
  userCallsign: string;
  setUserCallsign: (val: string) => void;
  assistantGender: string;
  setAssistantGender: (val: string) => void;
  personalityRole: string;
  setPersonalityRole: (val: string) => void;
  llmProvider: string;
  setLlmProvider: (val: string) => void;
  llmBaseUrl: string;
  setLlmBaseUrl: (val: string) => void;
  llmModel: string;
  setLlmModel: (val: string) => void;
  llmApiKey: string;
  setLlmApiKey: (val: string) => void;
  ttsVoiceName?: string;
  setTtsVoiceName?: (val: string) => void;
  isActive?: boolean;
}

export const PersonaAiSection: Component<PersonaAiSectionProps> = (props) => {
  let genderSelectRef: HTMLSelectElement | undefined;
  let roleSelectRef: HTMLSelectElement | undefined;
  let modelSelectRef: HTMLSelectElement | undefined;

  const [availableModels, setAvailableModels] = createSignal<Array<{ id: string; label: string }>>([]);
  const [isRefreshingModels, setIsRefreshingModels] = createSignal(false);

  const refreshDynamicModels = async () => {
    setIsRefreshingModels(true);
    try {
      const [dynamicAg, codexModels] = await Promise.all([
        antigravityService.getDynamicModels().catch(() => [] as Array<{ model_id: string; display_name: string }>),
        codexService.getModels().catch(() => [] as CodexModelItem[])
      ]);

      const list: Array<{ id: string; label: string }> = [];

      if (dynamicAg && dynamicAg.length > 0) {
        dynamicAg.forEach((m: { model_id: string }) => {
          const fullId = `ag/${m.model_id}`;
          list.push({
            id: fullId,
            label: fullId,
          });
        });
      }

      if (codexModels && codexModels.length > 0) {
        codexModels.forEach((m: CodexModelItem) => {
          list.push({
            id: m.id,
            label: m.id,
          });
        });
      }

      const current = props.llmModel;
      if (current && !list.some((m) => m.id === current)) {
        list.unshift({ id: current, label: `${current} (Current)` });
      }

      setAvailableModels(list);
    } catch {
      // fallback to presets
    } finally {
      setIsRefreshingModels(false);
    }
  };

  createEffect(() => {
    // Automatically invoked when the tab opens or isActive becomes true
    if (props.isActive) {
      refreshDynamicModels();
    }
  });

  createEffect(() => {
    if (genderSelectRef && props.assistantGender) {
      genderSelectRef.value = props.assistantGender;
    }
  });

  createEffect(() => {
    if (roleSelectRef && props.personalityRole) {
      roleSelectRef.value = props.personalityRole;
    }
  });

  createEffect(() => {
    const current = props.llmModel;
    const models = availableModels();
    if (!current) return;

    if (models.length > 0 && !models.some((m) => m.id === current)) {
      setAvailableModels((prev) => [{ id: current, label: `${current} (Current)` }, ...prev]);
    } else if (models.length === 0) {
      setAvailableModels([{ id: current, label: current }]);
    }

    queueMicrotask(() => {
      if (modelSelectRef && current) {
        modelSelectRef.value = current;
      }
    });
  });

  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 1. Persona & Identity */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-theme-cyan flex items-center gap-2">
          <UserCheck size={18} />
          1. Persona & Assistant Identity
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Assistant Name</label>
            <input
              type="text"
              value={props.assistantName}
              onInput={(e) => props.setAssistantName(e.currentTarget.value)}
              placeholder="Laras"
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">User Callsign</label>
            <input
              type="text"
              value={props.userCallsign}
              onInput={(e) => props.setUserCallsign(e.currentTarget.value)}
              placeholder="Mas"
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Assistant Gender</label>
            <select
              ref={genderSelectRef}
              value={props.assistantGender || 'female'}
              onChange={(e) => {
                const g = e.currentTarget.value;
                props.setAssistantGender(g);
                if (g === 'male' && props.ttsVoiceName?.startsWith('F')) {
                  props.setTtsVoiceName?.('M1');
                } else if (g === 'female' && props.ttsVoiceName?.startsWith('M')) {
                  props.setTtsVoiceName?.('F4');
                }
              }}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="female">Female</option>
              <option value="male">Male</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Personality Role</label>
            <select
              ref={roleSelectRef}
              value={props.personalityRole || 'partner'}
              onChange={(e) => props.setPersonalityRole(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="partner">{props.assistantGender === 'male' ? 'Partner (Warm Husbando)' : 'Partner (Sweet Waifu)'}</option>
              <option value="assistant">{props.assistantGender === 'male' ? 'Assistant (Professional Butler)' : 'Assistant (Smart Secretary)'}</option>
              <option value="friend">{props.assistantGender === 'male' ? 'Friend (Chill Buddy)' : 'Friend (Bestie)'}</option>
              <option value="tsundere">Tsundere (Anime Style)</option>
              <option value="maid">{props.assistantGender === 'male' ? 'Butler (Polite)' : 'Maid (Polite)'}</option>
            </select>
          </div>
        </div>
      </div>

      {/* 2. AI Brain (LLM Provider) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <h3 class="text-theme-rose flex items-center gap-2">
            <Brain size={18} />
            2. AI Brain (LLM Provider)
          </h3>
          <span class="text-xs text-theme-text/50 font-mono">
            {props.llmProvider === 'local' ? 'Ragasukma Local Engine' : 'Custom AI Assistant'}
          </span>
        </div>

        {/* Provider Selection (2 Clean Cards: Local vs Custom) */}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => {
              props.setLlmProvider('local');
            }}
            class={`p-3.5 border text-left transition-all ${
              props.llmProvider === 'local'
                ? 'bg-theme-cyan/10 border-theme-cyan text-theme-text shadow-sm'
                : 'bg-theme-card border-theme-border text-theme-text/70 hover:border-theme-text/30'
            }`}
          >
            <div class="font-bold text-xs flex items-center gap-1.5 text-theme-cyan">
              <Cpu size={14} />
              <span>Local (Ragasukma Engine)</span>
            </div>
            <div class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              System automatically routes requests (powered by Antigravity & OpenAI Codex pools). You can still select your preferred AI model.
            </div>
          </button>

          <button
            type="button"
            onClick={() => {
              props.setLlmProvider('custom');
            }}
            class={`p-3.5 border text-left transition-all ${
              props.llmProvider === 'custom'
                ? 'bg-theme-rose/10 border-theme-rose text-theme-text shadow-sm'
                : 'bg-theme-card border-theme-border text-theme-text/70 hover:border-theme-text/30'
            }`}
          >
            <div class="font-bold text-xs flex items-center gap-1.5 text-theme-rose">
              <Brain size={14} />
              <span>Custom / External LLM</span>
            </div>
            <div class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              Connect to your own local runtime (Ollama, LM Studio, LocalAI) or external APIs (Groq, OpenRouter, OpenAI).
            </div>
          </button>
        </div>

        {/* Display Konfigurasi Sesuai Provider */}
        {props.llmProvider === 'local' ? (
          <div class="p-4 bg-theme-bg/60 border border-theme-border/80 space-y-3">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80 flex items-center justify-between">
                <span class="flex items-center gap-1.5 font-bold">
                  <Cpu size={14} class="text-theme-cyan" />
                  Select AI Assistant Model (Local Engine)
                </span>
                <span class="text-xs text-theme-cyan font-mono bg-theme-cyan/10 px-2 py-0.5 border border-theme-cyan/20 font-bold">
                  Automatic Internal Routing
                </span>
              </label>
              <div class="flex items-center gap-2">
                <div class="flex-1 min-w-0">
                  <SearchableSelect
                    value={props.llmModel}
                    onChange={(val) => props.setLlmModel(val)}
                    options={availableModels().map((m) => ({
                      value: m.id,
                      label: m.label,
                      badge: m.id.startsWith('ag/')
                        ? 'Antigravity'
                        : m.id.startsWith('cx/')
                        ? 'Codex'
                        : undefined,
                    }))}
                    placeholder="Select or search Assistant Model..."
                    searchPlaceholder="Filter AI model..."
                    buttonClass="w-full flex items-center justify-between px-3.5 py-2.5 bg-theme-card border border-theme-border text-sm font-mono text-theme-text font-medium hover:border-theme-cyan transition-colors"
                    dropdownClass="absolute left-0 mt-1.5 w-full bg-theme-surface border border-theme-border shadow-2xl z-50 overflow-hidden flex flex-col max-h-72"
                  />
                </div>

                <button
                  type="button"
                  onClick={() => refreshDynamicModels()}
                  disabled={isRefreshingModels()}
                  title="Refresh available models"
                  class="px-3 py-2.5 bg-theme-surface hover:bg-theme-card border border-theme-border text-theme-cyan hover:text-theme-cyan/80 transition-colors flex items-center justify-center shrink-0 disabled:opacity-50"
                >
                  <RefreshCw size={15} class={isRefreshingModels() ? 'animate-spin' : ''} />
                </button>
              </div>
            </div>
          </div>
        ) : (
          <div class="space-y-3">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">LLM Base URL (Assistant Server)</label>
                <input
                  type="text"
                  value={props.llmBaseUrl}
                  onInput={(e) => props.setLlmBaseUrl(e.currentTarget.value)}
                  placeholder="http://localhost:11434/v1 or https://api.openai.com/v1"
                  class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>

              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">Model Name</label>
                <input
                  type="text"
                  value={props.llmModel}
                  onInput={(e) => props.setLlmModel(e.currentTarget.value)}
                  placeholder="qwen2.5:7b, llama3.2, mistral, gpt-4o, dll."
                  class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">LLM API Key (Optional for Local Server)</label>
              <input
                type="password"
                value={props.llmApiKey}
                onInput={(e) => props.setLlmApiKey(e.currentTarget.value)}
                placeholder="Leave blank if local Ollama/LM Studio has no auth"
                class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
