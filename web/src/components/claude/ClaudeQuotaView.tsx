import { Component, For, Show, createSignal, onMount } from 'solid-js';
import { Gauge, RefreshCw, AlertTriangle, Clock, Copy, Check, Sparkles } from 'lucide-solid';
import { ClaudeQuotaResponse, ClaudeQuotaWindow } from '../../api';
import { ClaudeModelItem, claudeService } from '../../services/claudeService';

interface ClaudeQuotaViewProps {
  accountId: string;
  quotaData?: ClaudeQuotaResponse;
  isLoading: boolean;
  copiedModelId: string | null;
  onRefresh: (id: string) => void;
  onCopyModel: (id: string, label?: string) => void;
}

export const ClaudeQuotaView: Component<ClaudeQuotaViewProps> = (props) => {
  const [liveModels, setLiveModels] = createSignal<ClaudeModelItem[]>([]);
  const [isLoadingModels, setIsLoadingModels] = createSignal<boolean>(false);

  const fetchLiveModels = async () => {
    setIsLoadingModels(true);
    try {
      const models = await claudeService.getModels();
      setLiveModels(Array.isArray(models) ? models : []);
    } catch {
      setLiveModels([]);
    } finally {
      setIsLoadingModels(false);
    }
  };

  onMount(() => {
    fetchLiveModels();
  });

  const formatResetTime = (isoString?: string): string => {
    if (!isoString) return '';
    try {
      const resetDate = new Date(isoString);
      const now = new Date();
      const diffMs = resetDate.getTime() - now.getTime();
      if (diffMs <= 0) return 'Reset completed';

      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMins / 60);
      const remainingMins = diffMins % 60;
      const diffDays = Math.floor(diffHours / 24);

      if (diffDays > 0) {
        const remainingHours = diffHours % 24;
        return `Resets in ${diffDays}d ${remainingHours}h`;
      }
      if (diffHours > 0) {
        return `Resets in ${diffHours}h ${remainingMins}m`;
      }
      return `Resets in ${diffMins}m`;
    } catch {
      return isoString;
    }
  };

  const getQuotaColor = (remainingPct: number) => {
    if (remainingPct >= 50) {
      return {
        bar: 'bg-[#D97757] shadow-[0_0_8px_rgba(217,119,87,0.35)]',
        badge: 'bg-[#D97757]/15 text-[#D97757] border border-[#D97757]/40',
      };
    }
    if (remainingPct >= 20) {
      return {
        bar: 'bg-theme-rose shadow-[0_0_8px_rgba(224,159,62,0.35)]',
        badge: 'bg-theme-rose/15 text-theme-rose border border-theme-rose/40',
      };
    }
    return {
      bar: 'bg-gi-status-danger shadow-[0_0_8px_rgba(224,83,69,0.35)]',
      badge: 'bg-gi-status-danger/15 text-gi-status-danger border border-gi-status-danger/40',
    };
  };

  const renderQuotaCard = (title: string, windowData?: ClaudeQuotaWindow) => {
    if (!windowData) return null;
    const colors = getQuotaColor(windowData.remaining);
    return (
      <div class="p-3 bg-theme-card border border-theme-border space-y-2 shadow-sm">
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs font-bold text-theme-text truncate">{title}</span>
          <span class={`text-xs font-mono font-bold px-2 py-0.5 border ${colors.badge}`}>
            {Math.round(windowData.remaining)}% Available
          </span>
        </div>
        <div class="w-full h-1.5 bg-theme-bg overflow-hidden border border-theme-border/40">
          <div
            class={`h-full transition-all duration-500 ${colors.bar}`}
            style={{ width: `${Math.max(2, Math.min(100, windowData.remaining))}%` }}
          />
        </div>
        <Show when={windowData.reset_at}>
          <div class="text-xs text-theme-text/50 font-mono flex items-center gap-1">
            <Clock size={11} class="text-[#D97757] shrink-0" />
            <span>{formatResetTime(windowData.reset_at)}</span>
          </div>
        </Show>
      </div>
    );
  };

  return (
    <div class="px-4 pb-4 pt-3 border-t border-theme-border/60 bg-theme-bg/60 space-y-3 animate-fade-in">
      {/* Panel Header */}
      <div class="flex items-center justify-between gap-2 flex-wrap pb-1">
        <div class="flex items-center gap-2">
          <Gauge size={15} class="text-[#D97757]" />
          <span class="text-xs font-bold text-theme-text">Claude OAuth Usage Status</span>
          <Show when={props.quotaData?.reset_credits !== undefined && props.quotaData?.reset_credits !== null}>
            <span class="text-xs font-mono px-2 py-0.5 bg-theme-rose/10 text-theme-rose border border-theme-rose/20 font-bold">
              {props.quotaData?.reset_credits} Reset Grants
            </span>
          </Show>
        </div>

        <button
          onClick={() => props.onRefresh(props.accountId)}
          disabled={props.isLoading}
          class="px-2.5 py-1 bg-theme-card border border-theme-border text-theme-text/70 hover:text-[#D97757] hover:border-[#D97757] text-xs font-medium transition-all flex items-center gap-1.5"
          title="Refresh live quota data"
        >
          <RefreshCw size={12} class={props.isLoading ? 'animate-spin text-[#D97757]' : ''} />
          <span>{props.isLoading ? 'Loading...' : 'Refresh Quota'}</span>
        </button>
      </div>

      {/* Loading Indicator */}
      <Show when={props.isLoading && !props.quotaData}>
        <div class="p-6 text-center bg-theme-card/50 border border-theme-border flex items-center justify-center gap-3">
          <RefreshCw size={16} class="animate-spin text-[#D97757]" />
          <span class="text-xs text-theme-text/60">Fetching usage from Anthropic OAuth API...</span>
        </div>
      </Show>

      {/* Error State */}
      <Show when={!props.isLoading && props.quotaData?.status === 'error'}>
        <div class="p-3.5 bg-gi-status-danger/10 border border-gi-status-danger/30 text-xs text-gi-status-danger flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <AlertTriangle size={16} class="shrink-0" />
            <span>{props.quotaData?.error || 'Failed to read account usage.'}</span>
          </div>
          <button
            onClick={() => props.onRefresh(props.accountId)}
            class="px-2.5 py-1 bg-gi-status-danger/20 hover:bg-gi-status-danger/30 text-gi-status-danger text-xs font-bold shrink-0 transition-colors"
          >
            Retry
          </button>
        </div>
      </Show>

      {/* Quota Windows */}
      <Show when={props.quotaData && (props.quotaData.session_quota || props.quotaData.weekly_quota)}>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
          {renderQuotaCard('Session Window (5h)', props.quotaData?.session_quota)}
          {renderQuotaCard('Weekly Window (7d)', props.quotaData?.weekly_quota)}
        </div>
      </Show>

      {/* Claude Live Models Grid (Dynamically fetched, no hardcoding) */}
      <Show when={liveModels().length > 0}>
        <div class="space-y-1.5 pt-1">
          <div class="flex items-center justify-between">
            <h3 class="text-theme-text/80 flex items-center gap-1.5">
              <Sparkles size={14} class="text-[#D97757]" />
              <span>Available Claude Models (Click to copy ID)</span>
            </h3>
            <button
              onClick={fetchLiveModels}
              disabled={isLoadingModels()}
              class="text-2xs text-theme-text/50 hover:text-[#D97757] flex items-center gap-1 transition-colors"
              title="Refresh models list"
            >
              <RefreshCw size={10} class={isLoadingModels() ? 'animate-spin text-[#D97757]' : ''} />
              <span>Refresh Models</span>
            </button>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
            <For each={liveModels()}>
              {(model) => {
                const isCopied = () => props.copiedModelId === model.id;
                return (
                  <div
                    onClick={() => props.onCopyModel(model.id, `Model ID ${model.id}`)}
                    class="p-2.5 bg-theme-card/90 border border-theme-border/80 hover:border-[#D97757]/50 cursor-pointer transition-all flex items-center justify-between gap-2 shadow-sm group"
                    title={`Click to copy ${model.id} (${model.display_name})`}
                  >
                    <div class="truncate">
                      <code class="text-xs font-mono font-bold text-[#D97757] truncate select-all block">{model.id}</code>
                      <span class="text-2xs text-theme-text/50 truncate block">{model.display_name}</span>
                    </div>
                    <button type="button" class="text-theme-text/70 group-hover:text-[#D97757] transition-colors shrink-0">
                      <Show when={isCopied()} fallback={<Copy size={13} />}>
                        <Check size={13} class="text-gi-status-success" />
                      </Show>
                    </button>
                  </div>
                );
              }}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
};
