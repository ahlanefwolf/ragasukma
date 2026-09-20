import { Component, For, Show } from 'solid-js';
import { Sparkles, Copy, Check } from 'lucide-solid';
import { ANTIGRAVITY_PRESET_MODELS } from '../../services';

interface AntigravityModelCatalogProps {
  copiedModelId: string | null;
  onCopyModel: (id: string, label?: string) => void;
}

export const AntigravityModelCatalog: Component<AntigravityModelCatalogProps> = (props) => {
  return (
    <div class="p-6 bg-theme-card border border-theme-border space-y-4 shadow-sm">
      <div class="flex items-center justify-between flex-wrap gap-2 pb-1 border-b border-theme-border/60">
        <div class="flex items-center gap-2.5">
          <div class="p-2 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20">
            <Sparkles size={18} />
          </div>
          <div>
            <h3 class="text-sm font-bold text-theme-text">
              Katalog Model Antigravity (Trafik ag/*)
            </h3>
            <p class="text-xs text-theme-text/60">
              Klik kartu atau tombol salin pada model untuk menyalin ID model siap pakai ke editor / SDK Anda.
            </p>
          </div>
        </div>
        <span class="text-xs font-mono px-2.5 py-1 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/30 font-bold">
          {ANTIGRAVITY_PRESET_MODELS.length} Model Tersedia
        </span>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        <For each={ANTIGRAVITY_PRESET_MODELS}>
          {(model) => {
            const isCopied = () => props.copiedModelId === model.id;
            return (
              <div
                onClick={() => props.onCopyModel(model.id, `Model ID ${model.id}`)}
                class="p-3.5 bg-theme-surface border border-theme-border hover:border-theme-cyan/50 hover:bg-theme-card transition-all cursor-pointer flex flex-col justify-between gap-3 group shadow-sm"
              >
                <div class="space-y-1.5">
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-xs font-bold text-theme-text group-hover:text-theme-cyan transition-colors truncate">
                      {model.name}
                    </span>
                    <Show when={model.tag}>
                      <span class="text-xs font-bold px-1.5 py-0.5 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20 shrink-0 uppercase">
                        {model.tag}
                      </span>
                    </Show>
                  </div>
                  <p class="text-xs text-theme-text/60 line-clamp-2 leading-relaxed">
                    {model.description}
                  </p>
                </div>

                <div class="flex items-center justify-between gap-2 px-2.5 py-1.5 bg-theme-bg/90 border border-theme-border/70 group-hover:border-theme-cyan/40 transition-colors">
                  <code class="text-xs font-mono text-theme-cyan font-bold truncate select-all">
                    {model.id}
                  </code>
                  <span class="text-theme-text/40 group-hover:text-theme-cyan transition-colors shrink-0">
                    <Show when={isCopied()} fallback={<Copy size={13} />}>
                      <Check size={13} class="text-emerald-400" />
                    </Show>
                  </span>
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
};
