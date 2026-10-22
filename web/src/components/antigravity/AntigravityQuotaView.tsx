import { Component, For, Show } from 'solid-js';
import { Gauge, RefreshCw, AlertTriangle, Zap, Clock, Copy, Check } from 'lucide-solid';
import { AntigravityQuotaResponse } from '../../api';

interface AntigravityQuotaViewProps {
  accountId: string;
  quotaData?: AntigravityQuotaResponse;
  isLoading: boolean;
  copiedModelId: string | null;
  onRefresh: (id: string) => void;
  onCopyModel: (id: string, label?: string) => void;
}

export const AntigravityQuotaView: Component<AntigravityQuotaViewProps> = (props) => {
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

  const getQuotaColor = (pct: number) => {
    if (pct >= 50) {
      return {
        bar: 'bg-theme-cyan shadow-[0_0_8px_rgba(89,107,69,0.35)]',
        badge: 'bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/40',
        text: 'text-theme-cyan',
      };
    }
    if (pct >= 20) {
      return {
        bar: 'bg-theme-rose shadow-[0_0_8px_rgba(224,159,62,0.35)]',
        badge: 'bg-theme-rose/15 text-theme-rose border border-theme-rose/40',
        text: 'text-theme-rose',
      };
    }
    return {
      bar: 'bg-gi-status-danger shadow-[0_0_8px_rgba(224,83,69,0.35)]',
      badge: 'bg-gi-status-danger/15 text-gi-status-danger border border-gi-status-danger/40',
      text: 'text-gi-status-danger',
    };
  };

  return (
    <div class="px-4 pb-4 pt-3 border-t border-theme-border/60 bg-theme-bg/60 space-y-3 animate-fade-in">
      {/* Panel Header */}
      <div class="flex items-center justify-between gap-2 flex-wrap pb-1">
        <div class="flex items-center gap-2">
          <Gauge size={15} class="text-theme-cyan" />
          <span class="text-xs font-bold text-theme-text">Antigravity Quota Status</span>
          <Show when={props.quotaData?.plan}>
            <span class="text-xs font-mono px-2 py-0.5 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20 font-bold">
              {props.quotaData?.plan}
            </span>
          </Show>
        </div>

        <button
          onClick={() => props.onRefresh(props.accountId)}
          disabled={props.isLoading}
          class="px-2.5 py-1 bg-theme-card border border-theme-border text-theme-text/70 hover:text-theme-cyan hover:border-theme-cyan text-xs font-medium transition-all flex items-center gap-1.5"
          title="Refresh live quota data"
        >
          <RefreshCw size={12} class={props.isLoading ? 'animate-spin text-theme-cyan' : ''} />
          <span>{props.isLoading ? 'Loading...' : 'Refresh Quota'}</span>
        </button>
      </div>

      {/* Loading Indicator */}
      <Show when={props.isLoading && !props.quotaData}>
        <div class="p-6 text-center bg-theme-card/50 border border-theme-border flex items-center justify-center gap-3">
          <RefreshCw size={16} class="animate-spin text-theme-cyan" />
          <span class="text-xs text-theme-text/60">Fetching quota data from Google Cloud Code API...</span>
        </div>
      </Show>

      {/* Error State */}
      <Show when={!props.isLoading && props.quotaData?.status === 'error' && (!props.quotaData?.models || props.quotaData?.models.length === 0)}>
        <div class="p-3.5 bg-gi-status-danger/10 border border-gi-status-danger/30 text-xs text-gi-status-danger flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <AlertTriangle size={16} class="shrink-0" />
            <span>{props.quotaData?.error || 'Failed to read account quota.'}</span>
          </div>
          <button
            onClick={() => props.onRefresh(props.accountId)}
            class="px-2.5 py-1 bg-gi-status-danger/20 hover:bg-gi-status-danger/30 text-gi-status-danger text-xs font-bold shrink-0 transition-colors"
          >
            Retry
          </button>
        </div>
      </Show>

      {/* Quota Data Content */}
      <Show when={props.quotaData && (props.quotaData.weekly?.length > 0 || props.quotaData.models?.length > 0)}>
        {/* Weekly Quota Cards */}
        <Show when={props.quotaData?.weekly && props.quotaData.weekly.length > 0}>
          <div class="space-y-1.5">
            <h3 class="text-theme-text/80 flex items-center gap-1.5">
              <Clock size={14} class="text-gi-status-warning" />
              <span>Weekly Quota (Weekly Cap)</span>
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              <For each={props.quotaData!.weekly}>
                {(item) => {
                  const colors = getQuotaColor(item.remaining_percentage);
                  return (
                    <div class="p-3 bg-theme-card border border-theme-border space-y-2 shadow-sm">
                      <div class="flex items-center justify-between gap-2">
                        <span class="text-xs font-mono font-bold text-theme-cyan truncate">{item.model_id}</span>
                        <span class={`text-xs font-mono font-bold px-2 py-0.5 border ${colors.badge}`}>
                          {item.remaining_percentage}% Available
                        </span>
                      </div>
                      <div class="w-full h-1.5 bg-theme-bg overflow-hidden border border-theme-border/40">
                        <div
                          class={`h-full transition-all duration-500 ${colors.bar}`}
                          style={{ width: `${Math.max(2, Math.min(100, item.remaining_percentage))}%` }}
                        />
                      </div>
                      <Show when={item.reset_time}>
                        <div class="text-xs text-theme-text/50 font-mono flex items-center gap-1">
                          <Clock size={11} class="text-theme-cyan shrink-0" />
                          <span>{formatResetTime(item.reset_time)}</span>
                        </div>
                      </Show>
                    </div>
                  );
                }}
              </For>
            </div>
          </div>
        </Show>

        {/* Per-Model Quota Breakdown */}
        <Show when={props.quotaData?.models && props.quotaData.models.length > 0}>
          <div class="space-y-1.5 pt-1">
            <h3 class="text-theme-text/80 flex items-center gap-1.5">
              <Zap size={14} class="text-theme-cyan" />
              <span>Model Quota (5-Hour Window)</span>
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
              <For each={props.quotaData!.models}>
                {(item) => {
                  const colors = getQuotaColor(item.remaining_percentage);
                  const fullModelId = () => (item.model_id.startsWith('ag/') ? item.model_id : `ag/${item.model_id}`);
                  const isCopied = () => props.copiedModelId === fullModelId();

                  return (
                    <div class="p-3 bg-theme-card/90 border border-theme-border/80 space-y-2 hover:border-theme-cyan/40 transition-all flex flex-col justify-between shadow-sm group">
                      <div class="space-y-1.5">
                        {/* Click to Copy Model ID */}
                        <div
                          onClick={() => props.onCopyModel(fullModelId(), `Model ID ${fullModelId()}`)}
                          class="flex items-center justify-between gap-1.5 px-2 py-1.5 bg-theme-bg/80 border border-theme-border/60 hover:border-theme-cyan/50 cursor-pointer transition-colors"
                          title="Click to copy Model ID"
                        >
                          <code class="text-xs font-mono text-theme-cyan truncate font-bold select-all">
                            {fullModelId()}
                          </code>
                          <div class="flex items-center gap-1.5 shrink-0">
                            <span class={`text-xs font-mono font-bold px-1.5 py-0.5 border ${colors.badge}`}>
                              {item.remaining_percentage}%
                            </span>
                            <button
                              type="button"
                              class="text-theme-text/40 group-hover:text-theme-cyan transition-colors"
                            >
                              <Show when={isCopied()} fallback={<Copy size={12} />}>
                                <Check size={12} class="text-gi-status-success" />
                              </Show>
                            </button>
                          </div>
                        </div>
                      </div>

                      {/* Bar & Reset Info */}
                      <div class="space-y-1.5 pt-1">
                        <div class="w-full h-1.5 bg-theme-bg overflow-hidden border border-theme-border/40">
                          <div
                            class={`h-full transition-all duration-500 ${colors.bar}`}
                            style={{ width: `${Math.max(2, Math.min(100, item.remaining_percentage))}%` }}
                          />
                        </div>
                        <Show when={item.reset_time}>
                          <div class="text-xs text-theme-text/50 font-mono flex items-center gap-1">
                            <Clock size={10} class="text-theme-cyan shrink-0" />
                            <span class="truncate">{formatResetTime(item.reset_time)}</span>
                          </div>
                        </Show>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
            <div class="text-xs text-theme-text/40 italic">
              Prefix model ID with <span class="font-mono text-theme-cyan">ag/</span> when configuring tools if not auto-detected.
            </div>
          </div>
        </Show>
      </Show>

      {/* Empty Quota Info */}
      <Show when={props.quotaData && props.quotaData.status !== 'error' && (!props.quotaData.weekly || props.quotaData.weekly.length === 0) && (!props.quotaData.models || props.quotaData.models.length === 0)}>
        <div class="p-4 text-center bg-theme-card/50 border border-theme-border text-xs text-theme-text/50">
          No active quota information found for this account. Ensure a Google Cloud Project is connected.
        </div>
      </Show>
    </div>
  );
};
