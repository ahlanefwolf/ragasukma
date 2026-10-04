import { API_BASE, fetchWithTimeout } from './base';

export interface RecipeStep {
  step: number;
  action: 'click' | 'type' | 'enter' | 'scroll' | string;
  selector: string;
  fallback_selectors?: string[];
  target_text?: string;
  value?: string;
}

export interface AutomationRecipe {
  recipe_id: string;
  name: string;
  domain?: string;
  version?: number;
  required_params?: string[];
  initial_action: {
    action: string;
    url: string;
  };
  steps: RecipeStep[];
}

export const recipeService = {
  async getRecipes(): Promise<AutomationRecipe[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/recipes`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get recipes:', e);
      return [];
    }
  },

  async saveRecipe(recipe: AutomationRecipe, previousId?: string): Promise<{ success: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/recipes/save`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...recipe, previous_id: previousId }),
      });
      return await res.json();
    } catch (e: any) {
      return { success: false, error: e?.message || 'Network error' };
    }
  },

  async deleteRecipe(recipeId: string): Promise<{ success: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/recipes/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ recipe_id: recipeId }),
      });
      return await res.json();
    } catch (e: any) {
      return { success: false, error: e?.message || 'Network error' };
    }
  },

  async runRecipe(recipeId: string, params?: Record<string, string>): Promise<{ success: boolean; message?: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/recipes/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ recipe_id: recipeId, params }),
      });
      return await res.json();
    } catch (e: any) {
      return { success: false, error: e?.message || 'Network error' };
    }
  },
};
