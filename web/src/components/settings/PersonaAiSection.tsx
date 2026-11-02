import { Component, createEffect, createSignal } from 'solid-js';
import { Brain, UserCheck, Cpu, RefreshCw } from 'lucide-solid';
import { modelCatalogService } from '../../services';
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
      const models = await modelCatalogService.getModels().catch(() => []);
      const list = [...models];
      const current = props.llmModel;
      if (current && !list.some((model) => model.id === current)) {
        list.unshift({ id: current, label: `${current} (Current)` });
      }
      setAvailableModels(list);
    } finally {
      setIsRefreshingModels(false);
    }
  };

  createEffect(() => {
    if (props.isActive) void refreshDynamicModels();
  });
  createEffect(() => {
    if (genderSelectRef && props.assistantGender) genderSelectRef.value = props.assistantGender;
  });
  createEffect(() => {
    if (roleSelectRef && props.personalityRole) roleSelectRef.value = props.personalityRole;
  });

  const displayModels = () => {
    const list = [...availableModels()];
    const current = props.llmModel;
    if (current && !list.some((model) => model.id === current)) {
      list.unshift({ id: current, label: `${current} (Current)` });
    }
    return list;
  };

  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-theme-cyan flex items-center gap-2"><UserCheck size={18} />1. Persona &amp; Assistant Identity</h3>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
          <div class="space-y-1.5"><label class="text-xs font-semibold text-theme-text/80">Assistant Name</label><input type="text" value={props.assistantName} onInput={(event) => props.setAssistantName(event.currentTarget.value)} placeholder="Laras" class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors" /></div>
          <div class="space-y-1.5"><label class="text-xs font-semibold text-theme-text/80">User Callsign</label><input type="text" value={props.userCallsign} onInput={(event) => props.setUserCallsign(event.currentTarget.value)} placeholder="Mas" class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors" /></div>
          <div class="space-y-1.5"><label class="text-xs font-semibold text-theme-text/80">Assistant Gender</label><select ref={genderSelectRef} value={props.assistantGender || 'female'} onChange={(event) => { const gender = event.currentTarget.value; props.setAssistantGender(gender); if (gender === 'male' && props.ttsVoiceName?.startsWith('F')) props.setTtsVoiceName?.('M1'); else if (gender === 'female' && props.ttsVoiceName?.startsWith('M')) props.setTtsVoiceName?.('F4'); }} class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"><option value="female">Female</option><option value="male">Male</option></select></div>
          <div class="space-y-1.5"><label class="text-xs font-semibold text-theme-text/80">Personality Role</label><select ref={roleSelectRef} value={props.personalityRole || 'partner'} onChange={(event) => props.setPersonalityRole(event.currentTarget.value)} class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"><option value="partner">{props.assistantGender === 'male' ? 'Partner (Warm Husbando)' : 'Partner (Sweet Waifu)'}</option><option value="assistant">{props.assistantGender === 'male' ? 'Assistant (Professional Butler)' : 'Assistant (Smart Secretary)'}</option><option value="friend">{props.assistantGender === 'male' ? 'Friend (Chill Buddy)' : 'Friend (Bestie)'}</option><option value="tsundere">Tsundere (Anime Style)</option><option value="maid">{props.assistantGender === 'male' ? 'Butler (Polite)' : 'Maid (Polite)'}</option></select></div>
        </div>
      </div>
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <div class="flex items-center justify-between flex-wrap gap-2"><h3 class="text-theme-rose flex items-center gap-2"><Brain size={18} />2. AI Brain</h3><span class="text-xs text-theme-text/50 font-mono">Local engine and registered API accounts</span></div>
        <p class="text-xs text-theme-text/60">Select a built-in model or a registered OpenAI-compatible model by its account prefix. Manage API accounts in the Provider LLM tab.</p>
        <div class="p-4 bg-theme-bg/60 border border-theme-border/80 space-y-3">
          <label class="text-xs font-semibold text-theme-text/80 flex items-center justify-between"><span class="flex items-center gap-1.5 font-bold"><Cpu size={14} class="text-theme-cyan" />Select AI Assistant Model</span><span class="text-xs text-theme-cyan font-mono bg-theme-cyan/10 px-2 py-0.5 border border-theme-cyan/20 font-bold">Prefix-based routing</span></label>
          <div class="flex items-center gap-2"><div class="flex-1 min-w-0"><SearchableSelect value={props.llmModel} onChange={(value) => props.setLlmModel(value)} options={displayModels().map((model) => ({ value: model.id, label: model.label, badge: model.id.startsWith('ag/') ? 'Antigravity' : model.id.startsWith('cx/') ? 'Codex' : model.id.includes('/') ? 'API' : undefined }))} placeholder="Select or search Assistant Model..." searchPlaceholder="Filter AI model..." buttonClass="w-full flex items-center justify-between px-3.5 py-2.5 bg-theme-card border border-theme-border text-sm font-mono text-theme-text font-medium hover:border-theme-cyan transition-colors" dropdownClass="absolute left-0 mt-1.5 w-full bg-theme-surface border border-theme-border shadow-2xl z-50 overflow-hidden flex flex-col max-h-72" /></div><button type="button" onClick={() => void refreshDynamicModels()} disabled={isRefreshingModels()} title="Refresh available models" class="px-3 py-2.5 bg-theme-surface hover:bg-theme-card border border-theme-border text-theme-cyan transition-colors flex items-center justify-center shrink-0 disabled:opacity-50"><RefreshCw size={15} class={isRefreshingModels() ? 'animate-spin' : ''} /></button></div>
        </div>
      </div>
    </div>
  );
};
