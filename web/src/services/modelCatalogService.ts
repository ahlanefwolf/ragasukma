import { API_BASE, fetchWithTimeout } from './base';

interface ModelListResponse {
  data?: Array<{ id?: string }>;
}

export const modelCatalogService = {
  async getModels(): Promise<Array<{ id: string; label: string }>> {
    const response = await fetchWithTimeout(`${API_BASE}/v1/models`);
    if (!response.ok) {
      throw new Error(`Failed to load models (${response.status})`);
    }
    const result = (await response.json()) as ModelListResponse;
    return (result.data ?? [])
      .filter((model): model is { id: string } => typeof model.id === 'string' && model.id.length > 0)
      .map(({ id }) => ({ id, label: id }));
  },
};
