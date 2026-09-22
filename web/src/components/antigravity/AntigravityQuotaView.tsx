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
      if (diffMs <= 0) return 'Telah di-reset';

      const diffMins = Math.floor(diffMs / 60000);
      const diffHours = Math.floor(diffMins / 60);
      const remainingMins = diffMins % 60;
      const diffDays = Math.floor(diffHours / 24);

      if (diffDays > 0) {
        const remainingHours = diffHours % 24;
        return `Reset dlm ${diffDays}h ${remainingHours}j`;
      }
      if (diffHours > 0) {
        return `Reset dlm ${diffHours}j ${remainingMins}m`;
      }
      return `Reset dlm ${diffMins}m`;
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
        bar: 'bg-theme-rose shadow-[0_0_8px_rgba(176,138,74,0.35)]',
        badge: 'bg-theme-rose/15 text-theme-rose border border-theme-rose/40',
        text: 'text-theme-rose',
      };
    }
    return {
      bar: 'bg-[#A84B4B] shadow-[0_0_8px_rgba(168,75,75,0.35)]',
      badge: 'bg-[#A84B4B]/15 text-[#A84B4B] border border-[#A84B4B]/40',
      text: 'text-[#A84B4B]',
    };
  };

  return (
    <div class="px-4 pb-4 pt-3 border-t border-theme-border/60 bg-theme-bg/60 space-y-3 animate-fade-in">
      {/* Panel Header */}
      <div class="flex items-center justify-between gap-2 flex-wrap pb-1">
        <div class="flex items-center gap-2">
          <Gauge size={15} class="text-theme-cyan" />
          <span class="text-xs font-bold text-theme-text">Status Kuota Antigravity</span>
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
          title="Perbarui data kuota terkini"
        >
          <RefreshCw size={12} class={props.isLoading ? 'animate-spin text-theme-cyan' : ''} />
          <span>{props.isLoading ? 'Memuat...' : 'Refresh Kuota'}</span>
        </button>
      </div>

      {/* Loading Indicator */}
      <Show when={props.isLoading && !props.quotaData}>
        <div class="p-6 text-center bg-theme-card/50 border border-theme-border flex items-center justify-center gap-3">
          <RefreshCw size={16} class="animate-spin text-theme-cyan" />
          <span class="text-xs text-theme-text/60">Mengambil data kuota dari Google Cloud Code API...</span>
        </div>
      </Show>

      {/* Error State */}
      <Show when={!props.isLoading && props.quotaData?.status === 'error' && (!props.quotaData?.models || props.quotaData?.models.length === 0)}>
        <div class="p-3.5 bg-rose-500/10 border border-rose-500/30 text-xs text-rose-400 flex items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <AlertTriangle size={16} class="shrink-0" />
            <span>{props.quotaData?.error || 'Gagal membaca kuota akun.'}</span>
          </div>
          <button
            onClick={() => props.onRefresh(props.accountId)}
            class="px-2.5 py-1 bg-rose-500/20 hover:bg-rose-500/30 text-rose-300 text-xs font-bold shrink-0 transition-colors"
          >
            Coba Lagi
          </button>
        </div>
      </Show>

      {/* Quota Data Content */}
      <Show when={props.quotaData && (props.quotaData.weekly?.length > 0 || props.quotaData.models?.length > 0)}>
        {/* Weekly Quota Cards */}
        <Show when={props.quotaData?.weekly && props.quotaData.weekly.length > 0}>
          <div class="space-y-1.5">
            <div class="text-xs font-bold text-theme-text/60 uppercase tracking-wider flex items-center gap-1.5">
              <Clock size={12} class="text-amber-400" />
              <span>Kuota Mingguan (Weekly Cap)</span>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
              <For each={props.quotaData!.weekly}>
                {(item) => {
                  const colors = getQuotaColor(item.remaining_percentage);
                  return (
                    <div class="p-3 bg-theme-card border border-theme-border space-y-2 shadow-sm">
                      <div class="flex items-center justify-between gap-2">
                        <span class="text-xs font-bold text-theme-text truncate">{item.display_name}</span>
                        <span class={`text-xs font-mono font-bold px-2 py-0.5 border ${colors.badge}`}>
                          {item.remaining_percentage}% Tersedia
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
            <div class="text-xs font-bold text-theme-text/60 uppercase tracking-wider flex items-center gap-1.5">
              <Zap size={12} class="text-theme-cyan" />
              <span>Kuota Model (Jendela 5 Jam)</span>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
              <For each={props.quotaData!.models}>
                {(item) => {
                  const colors = getQuotaColor(item.remaining_percentage);
                  const fullModelId = () => (item.model_id.startsWith('ag/') ? item.model_id : `ag/${item.model_id}`);
                  const isCopied = () => props.copiedModelId === fullModelId();

                  return (
                    <div class="p-3 bg-theme-card/90 border border-theme-border/80 space-y-2 hover:border-theme-cyan/40 transition-all flex flex-col justify-between shadow-sm group">
                      <div class="space-y-1.5">
                        <div class="flex items-center justify-between gap-1.5">
                          <span class="text-xs font-bold text-theme-text truncate select-text" title={item.display_name}>
                            {item.display_name}
                          </span>
                          <span class={`text-xs font-mono font-bold px-1.5 py-0.5 border shrink-0 ${colors.badge}`}>
                            {item.remaining_percentage}%
                          </span>
                        </div>

                        {/* Klik untuk Copy Model ID */}
                        <div
                          onClick={() => props.onCopyModel(fullModelId(), `Model ID ${fullModelId()}`)}
                          class="flex items-center justify-between gap-1.5 px-2 py-1 bg-theme-bg/80 border border-theme-border/60 hover:border-theme-cyan/50 cursor-pointer transition-colors"
                          title="Klik untuk menyalin Model ID dengan trafik ag/*"
                        >
                          <code class="text-xs font-mono text-theme-cyan truncate font-semibold select-all">
                            {fullModelId()}
                          </code>
                          <button
                            type="button"
                            class="text-theme-text/40 group-hover:text-theme-cyan transition-colors shrink-0"
                          >
                            <Show when={isCopied()} fallback={<Copy size={12} />}>
                              <Check size={12} class="text-emerald-400" />
                            </Show>
                          </button>
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
          </div>
        </Show>
      </Show>

      {/* Empty Quota Info */}
      <Show when={props.quotaData && props.quotaData.status !== 'error' && (!props.quotaData.weekly || props.quotaData.weekly.length === 0) && (!props.quotaData.models || props.quotaData.models.length === 0)}>
        <div class="p-4 text-center bg-theme-card/50 border border-theme-border text-xs text-theme-text/50">
          Tidak ada informasi kuota aktif untuk akun ini. Pastikan Google Cloud Project telah terhubung.
        </div>
      </Show>
    </div>
  );
};
