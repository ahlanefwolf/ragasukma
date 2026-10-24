import { Component, createSignal, onMount, For, Show } from 'solid-js';
import {
  Workflow,
  Play,
  Trash2,
  Plus,
  RefreshCw,
  Globe,
  ChevronDown,
  ChevronUp,
  X,
  Check,
  Edit2,
  Layers,
  ArrowUp,
  ArrowDown
} from 'lucide-solid';
import { api, AutomationRecipe, RecipeStep } from '../api';
import { showConfirm } from './ConfirmModal';

interface RecipesTabProps {
  showToast: (msg: string) => void;
}

export const RecipesTab: Component<RecipesTabProps> = (props) => {
  const [recipes, setRecipes] = createSignal<AutomationRecipe[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [runningId, setRunningId] = createSignal<string | null>(null);
  const [expandedId, setExpandedId] = createSignal<string | null>(null);

  // Edit / Add Recipe Modal State
  const [isEditorOpen, setIsEditorOpen] = createSignal(false);
  const [editingRecipeId, setEditingRecipeId] = createSignal<string | null>(null);
  const [formName, setFormName] = createSignal('');
  const [formRecipeId, setFormRecipeId] = createSignal('');
  const [formDomain, setFormDomain] = createSignal('');
  const [formInitialUrl, setFormInitialUrl] = createSignal('');
  const [formSteps, setFormSteps] = createSignal<RecipeStep[]>([]);
  const [formRequiredParams, setFormRequiredParams] = createSignal('');

  // Run Recipe with Param Modal
  const [paramModalOpen, setParamModalOpen] = createSignal(false);
  const [targetRecipeToRun, setTargetRecipeToRun] = createSignal<AutomationRecipe | null>(null);
  const [paramInputs, setParamInputs] = createSignal<Record<string, string>>({});

  const loadRecipes = async () => {
    setLoading(true);
    try {
      const data = await api.getRecipes();
      setRecipes(Array.isArray(data) ? data : []);
    } catch {
      setRecipes([]);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadRecipes();
  });

  const handleOpenNewRecipe = () => {
    setEditingRecipeId(null);
    setFormRecipeId('');
    setFormName('');
    setFormDomain('');
    setFormInitialUrl('https://');
    setFormRequiredParams('');
    setFormSteps([
      {
        step: 1,
        action: 'type',
        selector: 'input[name="q"]',
        value: '{{query}}',
        target_text: 'Search Box',
        fallback_selectors: [],
      },
      {
        step: 2,
        action: 'enter',
        selector: 'input[name="q"]',
        target_text: 'Submit Search',
        fallback_selectors: [],
      }
    ]);
    setIsEditorOpen(true);
  };

  const handleEditRecipe = (recipe: AutomationRecipe) => {
    setEditingRecipeId(recipe.recipe_id);
    setFormRecipeId(recipe.recipe_id);
    setFormName(recipe.name);
    setFormDomain(recipe.domain || '');
    setFormInitialUrl(recipe.initial_action?.url || '');
    setFormRequiredParams((recipe.required_params || []).join(', '));
    // Clone steps with sequential step numbering
    const clonedSteps: RecipeStep[] = (recipe.steps || []).map((s, idx) => ({
      step: idx + 1,
      action: s.action || 'click',
      selector: s.selector || '',
      fallback_selectors: s.fallback_selectors ? [...s.fallback_selectors] : [],
      target_text: s.target_text || '',
      value: s.value || '',
    }));
    setFormSteps(clonedSteps);
    setIsEditorOpen(true);
  };

  const handleDeleteRecipe = async (recipe: AutomationRecipe) => {
    const ok = await showConfirm({
      title: 'Delete Recipe',
      message: `Are you sure you want to delete recipe "${recipe.name}"?`,
      confirmText: 'Delete',
      isDanger: true,
    });
    if (!ok) return;

    const res = await api.deleteRecipe(recipe.recipe_id);
    if (res.success) {
      props.showToast(`Recipe "${recipe.name}" deleted`);
      setRecipes(recipes().filter(r => r.recipe_id !== recipe.recipe_id));
    } else {
      props.showToast(`Failed to delete recipe: ${res.error || 'Unknown error'}`);
    }
  };

  const handleTriggerRun = (recipe: AutomationRecipe) => {
    const required = recipe.required_params || [];
    if (required.length > 0) {
      // Open modal to enter parameters
      setTargetRecipeToRun(recipe);
      const initial: Record<string, string> = {};
      required.forEach(p => { initial[p] = ''; });
      setParamInputs(initial);
      setParamModalOpen(true);
    } else {
      executeRun(recipe.recipe_id);
    }
  };

  const executeRun = async (recipeId: string, params?: Record<string, string>) => {
    setRunningId(recipeId);
    props.showToast(`Starting recipe "${recipeId}"...`);
    const res = await api.runRecipe(recipeId, params);
    setRunningId(null);
    if (res.success) {
      props.showToast(res.message || `Recipe "${recipeId}" triggered`);
    } else {
      props.showToast(`Execution error: ${res.error || 'Failed'}`);
    }
  };

  // Step Management in Form
  const addStep = () => {
    const nextNum = formSteps().length + 1;
    setFormSteps([
      ...formSteps(),
      {
        step: nextNum,
        action: 'click',
        selector: '',
        fallback_selectors: [],
        target_text: '',
        value: '',
      }
    ]);
  };

  const removeStep = (index: number) => {
    const updated = formSteps().filter((_, i) => i !== index);
    // Auto-reindex steps sequentially
    updated.forEach((s, idx) => {
      s.step = idx + 1;
    });
    setFormSteps([...updated]);
  };

  const moveStep = (index: number, direction: 'up' | 'down') => {
    const steps = [...formSteps()];
    const targetIdx = direction === 'up' ? index - 1 : index + 1;
    if (targetIdx < 0 || targetIdx >= steps.length) return;
    const temp = steps[index];
    steps[index] = steps[targetIdx];
    steps[targetIdx] = temp;
    // Auto-reindex steps
    steps.forEach((s, idx) => {
      s.step = idx + 1;
    });
    setFormSteps(steps);
  };

  const updateStepField = (index: number, field: keyof RecipeStep, value: any) => {
    const steps = [...formSteps()];
    if (steps[index]) {
      steps[index] = { ...steps[index], [field]: value };
      setFormSteps(steps);
    }
  };

  const generateSlug = (text: string) => {
    return text
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9\s_-]/g, '')
      .replace(/[\s-]+/g, '_')
      .replace(/^_+|_+$/g, '');
  };

  const handleNameInput = (nameVal: string) => {
    setFormName(nameVal);
    // Auto-generate slug dynamically
    setFormRecipeId(generateSlug(nameVal));
  };

  const handleSaveRecipeForm = async () => {
    const rawId = formRecipeId().trim() || generateSlug(formName());
    const cleanId = rawId.toLowerCase().replace(/[^a-z0-9_-]/g, '_');
    if (!cleanId) {
      props.showToast('Recipe ID is required');
      return;
    }
    if (!formName().trim()) {
      props.showToast('Recipe Name is required');
      return;
    }

    const params = formRequiredParams()
      .split(',')
      .map(p => p.trim())
      .filter(Boolean);

    // Auto-reindex steps sequentially before saving & preserve fallback_selectors
    const finalSteps: RecipeStep[] = formSteps().map((s, idx) => ({
      ...s,
      step: idx + 1,
      selector: (s.selector || '').trim(),
      fallback_selectors: Array.isArray(s.fallback_selectors) && s.fallback_selectors.length > 0 ? s.fallback_selectors : undefined,
      target_text: s.target_text?.trim() || undefined,
      value: s.value?.trim() || undefined,
    }));

    const recipePayload: AutomationRecipe = {
      recipe_id: cleanId,
      name: formName().trim(),
      domain: formDomain().trim() || undefined,
      version: 1,
      required_params: params,
      initial_action: {
        action: 'open_new_tab',
        url: formInitialUrl().trim() || 'https://',
      },
      steps: finalSteps,
    };

    const res = await api.saveRecipe(recipePayload, editingRecipeId() || undefined);
    if (res.success) {
      props.showToast(`Recipe "${cleanId}" successfully saved`);
      setIsEditorOpen(false);
      loadRecipes();
    } else {
      props.showToast(`Failed to save: ${res.error || 'Unknown error'}`);
    }
  };

  return (
    <div class="space-y-5 animate-fade-in text-theme-text select-none">
      {/* Top Header Card */}
      <div class="p-5 bg-theme-surface border border-theme-border shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="p-3 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
            <Workflow size={22} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="font-heading tracking-wide">
                Automation Recipes
              </h2>
              <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan font-bold">
                {recipes().length} Saved
              </span>
            </div>
            <p class="text-xs text-theme-text/70 mt-1">
              Autonomous web macros learned once, replayed with 0 token overhead
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 self-end sm:self-center">
          <button
            onClick={loadRecipes}
            disabled={loading()}
            class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card border border-theme-border flex items-center gap-2 transition-all cursor-pointer"
            title="Refresh recipes"
          >
            <RefreshCw size={14} class={loading() ? 'animate-spin text-theme-cyan' : ''} />
            <span class="hidden sm:inline">Refresh</span>
          </button>

          <button
            onClick={handleOpenNewRecipe}
            class="px-4 py-2 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/85 text-theme-card border border-theme-cyan flex items-center gap-2 transition-all shadow-md cursor-pointer anime-shadow-green"
          >
            <Plus size={15} />
            <span>Create Recipe</span>
          </button>
        </div>
      </div>

      {/* Main List */}
      <Show when={!loading()} fallback={
        <div class="p-12 text-center text-theme-text/50 font-mono text-xs border border-theme-border bg-theme-surface">
          Loading recipes...
        </div>
      }>
        <Show when={recipes().length > 0} fallback={
          <div class="p-12 text-center border border-theme-border bg-theme-surface space-y-3">
            <Workflow size={36} class="mx-auto text-theme-text/60" />
            <p class="text-sm font-semibold text-theme-text/70">
              No automation recipes saved yet.
            </p>
            <p class="text-xs text-theme-text/50 max-w-md mx-auto">
              You can create recipes manually, or say <span class="font-mono text-theme-cyan">"Train recipe for YouTube"</span> to let the AI learn autonomously.
            </p>
            <button
              onClick={handleOpenNewRecipe}
              class="mt-2 px-4 py-2 text-xs font-bold bg-theme-card hover:bg-theme-bg border border-theme-cyan/40 text-theme-cyan inline-flex items-center gap-1.5 transition-all cursor-pointer"
            >
              <Plus size={14} />
              <span>Create First Recipe</span>
            </button>
          </div>
        }>
          <div class="grid grid-cols-1 gap-4">
            <For each={recipes()}>
              {(recipe) => {
                const isExpanded = () => expandedId() === recipe.recipe_id;
                const isRunning = () => runningId() === recipe.recipe_id;

                return (
                  <div class="bg-theme-surface border border-theme-border shadow-sm transition-all hover:border-theme-cyan/40">
                    {/* Recipe Item Header */}
                    <div class="p-4 flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-theme-bg/50">
                      <div class="flex items-start sm:items-center gap-3">
                        <div class="p-2 border border-theme-cyan/30 bg-theme-cyan/10 text-theme-cyan shrink-0 mt-0.5 sm:mt-0">
                          <Globe size={18} />
                        </div>
                        <div>
                          <div class="flex items-center gap-2 flex-wrap">
                            <h3 class="text-theme-text font-heading">
                              {recipe.name}
                            </h3>
                            <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan">
                              {recipe.recipe_id}
                            </span>
                            {recipe.domain && (
                              <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-text/60">
                                {recipe.domain}
                              </span>
                            )}
                          </div>
                          <div class="flex items-center gap-2 text-xs text-theme-text/70 mt-1 flex-wrap font-mono">
                            <span>Steps: <strong class="text-theme-text">{recipe.steps?.length || 0}</strong></span>
                            <span>•</span>
                            <span>Initial URL: <strong class="text-theme-cyan truncate max-w-[200px] sm:max-w-xs inline-block align-bottom">{recipe.initial_action?.url}</strong></span>
                            <Show when={(recipe.required_params || []).length > 0}>
                              <span>•</span>
                              <span>Params: <strong class="text-theme-rose">{(recipe.required_params || []).join(', ')}</strong></span>
                            </Show>
                          </div>
                        </div>
                      </div>

                      {/* Action Buttons */}
                      <div class="flex items-center gap-2 self-end sm:self-center">
                        <button
                          onClick={() => handleTriggerRun(recipe)}
                          disabled={isRunning()}
                          class="px-3.5 py-1.5 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/85 text-theme-card border border-theme-cyan flex items-center gap-1.5 transition-all shadow-sm cursor-pointer"
                          title="Run recipe now"
                        >
                          <Play size={13} class={isRunning() ? 'animate-spin' : 'fill-current'} />
                          <span>{isRunning() ? 'Running...' : 'Run'}</span>
                        </button>

                        <button
                          onClick={() => handleEditRecipe(recipe)}
                          class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan transition-all cursor-pointer"
                          title="Edit recipe & steps"
                        >
                          <Edit2 size={14} />
                        </button>

                        <button
                          onClick={() => handleDeleteRecipe(recipe)}
                          class="p-2 bg-theme-card hover:bg-theme-rose/20 border border-theme-border text-theme-text hover:text-theme-rose transition-all cursor-pointer"
                          title="Delete recipe"
                        >
                          <Trash2 size={14} />
                        </button>

                        <button
                          onClick={() => setExpandedId(isExpanded() ? null : recipe.recipe_id)}
                          class="p-2 bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text transition-all cursor-pointer"
                          title={isExpanded() ? 'Collapse steps' : 'Expand steps'}
                        >
                          {isExpanded() ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                        </button>
                      </div>
                    </div>

                    {/* Step Details Drawer */}
                    <Show when={isExpanded()}>
                      <div class="p-4 border-t border-theme-border bg-theme-surface space-y-3">
                        <h4 class="font-mono uppercase tracking-wider text-theme-text/80 flex items-center gap-1.5">
                          <Layers size={14} class="text-theme-cyan" />
                          <span>Step Execution Sequence ({recipe.steps?.length || 0} steps)</span>
                        </h4>

                        <div class="space-y-2">
                          <For each={recipe.steps || []}>
                            {(st) => (
                              <div class="p-2.5 bg-theme-bg border border-theme-border/70 flex items-start gap-3 text-xs font-mono">
                                <span class="w-6 h-6 shrink-0 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 flex items-center justify-center font-bold">
                                  {st.step}
                                </span>
                                <div class="flex-1 space-y-1">
                                  <div class="flex items-center gap-2 flex-wrap">
                                    <span class="px-2 py-0.5 bg-theme-card border border-theme-border uppercase font-bold text-theme-text">
                                      {st.action}
                                    </span>
                                    <span class="text-theme-text/80 truncate">
                                      {st.selector}
                                    </span>
                                    {st.target_text && (
                                      <span class="text-theme-text/50">
                                        ({st.target_text})
                                      </span>
                                    )}
                                  </div>
                                  {st.value && (
                                    <div class="text-xs text-theme-cyan">
                                      Value: <span class="bg-theme-card px-1.5 py-0.5 border border-theme-border">{st.value}</span>
                                    </div>
                                  )}
                                  {st.fallback_selectors && st.fallback_selectors.length > 0 && (
                                    <div class="text-2xs text-theme-text/50 truncate">
                                      Fallbacks: {st.fallback_selectors.join(' • ')}
                                    </div>
                                  )}
                                </div>
                              </div>
                            )}
                          </For>
                        </div>
                      </div>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      </Show>

      {/* Run Modal with Parameters */}
      <Show when={paramModalOpen() && targetRecipeToRun()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-fade-in">
          <div class="bg-theme-surface border border-theme-border w-full max-w-md shadow-2xl p-5 space-y-4">
            <div class="flex items-center justify-between pb-3 border-b border-theme-border">
              <div class="flex items-center gap-2">
                <Play size={16} class="text-theme-cyan fill-current" />
                <h3 class="text-theme-text font-heading">
                  Run: {targetRecipeToRun()!.name}
                </h3>
              </div>
              <button
                onClick={() => setParamModalOpen(false)}
                class="p-1 text-theme-text/60 hover:text-theme-text cursor-pointer"
              >
                <X size={18} />
              </button>
            </div>

            <p class="text-xs text-theme-text/70">
              This recipe requires dynamic parameters before running:
            </p>

            <div class="space-y-3">
              <For each={targetRecipeToRun()!.required_params || []}>
                {(param) => (
                  <div>
                    <label class="block text-xs font-mono uppercase text-theme-text/70 mb-1 font-bold">
                      {param}
                    </label>
                    <input
                      type="text"
                      value={paramInputs()[param] || ''}
                      onInput={(e) => {
                        const next = { ...paramInputs(), [param]: e.currentTarget.value };
                        setParamInputs(next);
                      }}
                      placeholder={`Enter ${param}...`}
                      class="w-full px-3 py-2 bg-theme-bg border border-theme-border text-sm text-theme-text focus:border-theme-cyan focus:outline-none font-mono"
                    />
                  </div>
                )}
              </For>
            </div>

            <div class="flex items-center justify-end gap-2 pt-2 border-t border-theme-border">
              <button
                onClick={() => setParamModalOpen(false)}
                class="px-4 py-2 text-xs font-semibold bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-text cursor-pointer"
              >
                Cancel
              </button>
              <button
                onClick={() => {
                  const target = targetRecipeToRun();
                  if (target) {
                    setParamModalOpen(false);
                    executeRun(target.recipe_id, paramInputs());
                  }
                }}
                class="px-4 py-2 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/85 text-theme-card border border-theme-cyan flex items-center gap-1.5 cursor-pointer anime-shadow-green"
              >
                <Play size={13} class="fill-current" />
                <span>Execute Recipe</span>
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Recipe Editor Modal */}
      <Show when={isEditorOpen()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm p-4 animate-fade-in">
          <div class="bg-theme-surface border border-theme-border w-full max-w-2xl max-h-[90vh] shadow-2xl flex flex-col overflow-hidden">
            {/* Modal Header */}
            <div class="p-4 border-b border-theme-border flex items-center justify-between bg-theme-bg">
              <div class="flex items-center gap-2">
                <Workflow size={18} class="text-theme-cyan" />
                <h3 class="text-theme-text font-heading">
                  {editingRecipeId() ? `Edit Recipe: ${editingRecipeId()}` : 'Create New Automation Recipe'}
                </h3>
              </div>
              <button
                onClick={() => setIsEditorOpen(false)}
                class="p-1.5 text-theme-text/60 hover:text-theme-text hover:bg-theme-border border border-theme-border cursor-pointer transition-all"
              >
                <X size={18} />
              </button>
            </div>

            {/* Modal Body */}
            <div class="p-5 overflow-y-auto space-y-4 flex-1 custom-scrollbar text-xs">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label class="block font-mono uppercase text-theme-text/70 mb-1 font-bold">
                    Recipe Display Name
                  </label>
                  <input
                    type="text"
                    value={formName()}
                    onInput={(e) => handleNameInput(e.currentTarget.value)}
                    placeholder="e.g. YouTube Music Player"
                    class="w-full px-3 py-2 bg-theme-bg border border-theme-border text-theme-text focus:border-theme-cyan focus:outline-none font-sans"
                  />
                </div>

                <div>
                  <div class="flex items-center justify-between mb-1">
                    <label class="block font-mono uppercase text-theme-text/70 font-bold">
                      Recipe ID / Slug
                    </label>
                    <button
                      type="button"
                      onClick={() => setFormRecipeId(generateSlug(formName()))}
                      class="text-2xs font-mono text-theme-cyan hover:underline cursor-pointer"
                      title="Generate slug from name"
                    >
                      Sync from Name
                    </button>
                  </div>
                  <input
                    type="text"
                    value={formRecipeId()}
                    onInput={(e) => setFormRecipeId(e.currentTarget.value)}
                    placeholder="e.g. youtube_music"
                    class="w-full px-3 py-2 bg-theme-bg border border-theme-border text-theme-text focus:border-theme-cyan focus:outline-none font-mono"
                  />
                </div>
              </div>

              <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label class="block font-mono uppercase text-theme-text/70 mb-1 font-bold">
                    Domain
                  </label>
                  <input
                    type="text"
                    value={formDomain()}
                    onInput={(e) => setFormDomain(e.currentTarget.value)}
                    placeholder="e.g. youtube.com"
                    class="w-full px-3 py-2 bg-theme-bg border border-theme-border text-theme-text focus:border-theme-cyan focus:outline-none font-mono"
                  />
                </div>

                <div>
                  <label class="block font-mono uppercase text-theme-text/70 mb-1 font-bold">
                    Initial URL to Open
                  </label>
                  <input
                    type="text"
                    value={formInitialUrl()}
                    onInput={(e) => setFormInitialUrl(e.currentTarget.value)}
                    placeholder="https://www.youtube.com"
                    class="w-full px-3 py-2 bg-theme-bg border border-theme-border text-theme-text focus:border-theme-cyan focus:outline-none font-mono"
                  />
                </div>
              </div>

              <div>
                <label class="block font-mono uppercase text-theme-text/70 mb-1 font-bold">
                  Required Parameters (comma separated, e.g. query, keyword)
                </label>
                <input
                  type="text"
                  value={formRequiredParams()}
                  onInput={(e) => setFormRequiredParams(e.currentTarget.value)}
                  placeholder="e.g. query"
                  class="w-full px-3 py-2 bg-theme-bg border border-theme-border text-theme-text focus:border-theme-cyan focus:outline-none font-mono"
                />
              </div>

              {/* Step Sequence Editor */}
              <div class="pt-2 border-t border-theme-border space-y-3">
                <div class="flex items-center justify-between">
                  <span class="font-mono uppercase font-bold text-theme-cyan flex items-center gap-1.5">
                    <Layers size={14} />
                    <span>Automation Steps ({formSteps().length})</span>
                  </span>
                  <button
                    type="button"
                    onClick={addStep}
                    class="px-2.5 py-1 text-xs bg-theme-card hover:bg-theme-bg border border-theme-cyan/40 text-theme-cyan font-bold flex items-center gap-1 cursor-pointer transition-all"
                  >
                    <Plus size={12} />
                    <span>Add Step</span>
                  </button>
                </div>

                <div class="space-y-3">
                  <For each={formSteps()}>
                    {(stepItem, idx) => (
                      <div class="p-3 bg-theme-bg border border-theme-border space-y-2 relative">
                        <div class="flex items-center justify-between gap-2">
                          <div class="flex items-center gap-2">
                            <span class="w-5 h-5 bg-theme-cyan text-theme-card font-mono font-bold flex items-center justify-center text-xs">
                              {idx() + 1}
                            </span>
                            <span class="font-mono font-semibold text-theme-text/80 text-xs">
                              Step #{idx() + 1}
                            </span>
                          </div>

                          <div class="flex items-center gap-1">
                            <button
                              type="button"
                              onClick={() => moveStep(idx(), 'up')}
                              disabled={idx() === 0}
                              class="p-1 bg-theme-card hover:bg-theme-border border border-theme-border text-theme-text disabled:opacity-30 cursor-pointer"
                              title="Move step up"
                            >
                              <ArrowUp size={12} />
                            </button>
                            <button
                              type="button"
                              onClick={() => moveStep(idx(), 'down')}
                              disabled={idx() === formSteps().length - 1}
                              class="p-1 bg-theme-card hover:bg-theme-border border border-theme-border text-theme-text disabled:opacity-30 cursor-pointer"
                              title="Move step down"
                            >
                              <ArrowDown size={12} />
                            </button>
                            <button
                              type="button"
                              onClick={() => removeStep(idx())}
                              class="p-1 bg-theme-card hover:bg-theme-rose/20 border border-theme-border text-theme-rose cursor-pointer ml-1"
                              title="Remove this step"
                            >
                              <Trash2 size={12} />
                            </button>
                          </div>
                        </div>

                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
                          <div>
                            <label class="block text-2xs font-mono text-theme-text/60 mb-0.5">Action</label>
                            <select
                              value={stepItem.action}
                              onChange={(e) => updateStepField(idx(), 'action', e.currentTarget.value)}
                              class="w-full px-2 py-1.5 bg-theme-card border border-theme-border text-theme-text font-mono text-xs focus:outline-none"
                            >
                              <option value="click">click</option>
                              <option value="type">type</option>
                              <option value="enter">enter</option>
                              <option value="scroll">scroll</option>
                            </select>
                          </div>

                          <div class="sm:col-span-2">
                            <label class="block text-2xs font-mono text-theme-text/60 mb-0.5">CSS Selector / Target</label>
                            <input
                              type="text"
                              value={stepItem.selector}
                              onInput={(e) => updateStepField(idx(), 'selector', e.currentTarget.value)}
                              placeholder="e.g. input#search or button[aria-label='Search']"
                              class="w-full px-2 py-1.5 bg-theme-card border border-theme-border text-theme-text font-mono text-xs focus:outline-none"
                            />
                          </div>
                        </div>

                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                          <div>
                            <label class="block text-2xs font-mono text-theme-text/60 mb-0.5">Value (Template or static)</label>
                            <input
                              type="text"
                              value={stepItem.value || ''}
                              onInput={(e) => updateStepField(idx(), 'value', e.currentTarget.value)}
                              placeholder="e.g. {{query}}"
                              class="w-full px-2 py-1.5 bg-theme-card border border-theme-border text-theme-text font-mono text-xs focus:outline-none"
                            />
                          </div>

                          <div>
                            <label class="block text-2xs font-mono text-theme-text/60 mb-0.5">Target Text / Label</label>
                            <input
                              type="text"
                              value={stepItem.target_text || ''}
                              onInput={(e) => updateStepField(idx(), 'target_text', e.currentTarget.value)}
                              placeholder="e.g. Search button"
                              class="w-full px-2 py-1.5 bg-theme-card border border-theme-border text-theme-text font-mono text-xs focus:outline-none"
                            />
                          </div>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </div>
            </div>

            {/* Modal Footer */}
            <div class="p-4 border-t border-theme-border bg-theme-bg flex items-center justify-end gap-2">
              <button
                type="button"
                onClick={() => setIsEditorOpen(false)}
                class="px-4 py-2 text-xs font-semibold bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border cursor-pointer transition-all"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleSaveRecipeForm}
                class="px-4 py-2 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/85 text-theme-card border border-theme-cyan flex items-center gap-1.5 cursor-pointer anime-shadow-green transition-all"
              >
                <Check size={14} />
                <span>Save Recipe</span>
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};