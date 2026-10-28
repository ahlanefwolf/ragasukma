import { Component, Show } from 'solid-js';
import { 
  Building2, 
  Gauge, 
  Power, 
  RefreshCw, 
  Trash2, 
  ChevronDown, 
  ChevronUp 
} from 'lucide-solid';
import { ClaudeAccount, ClaudeQuotaResponse } from '../../api';
import { ClaudeQuotaView } from './ClaudeQuotaView';

interface ClaudeAccountCardProps {
  account: ClaudeAccount;
  actionLoading: string | null;
  quotaData?: ClaudeQuotaResponse;
  loadingQuota: boolean;
  isExpanded: boolean;
  copiedModelId: string | null;
  onToggleQuota: (id: string) => void;
  onFetchQuota: (id: string) => void;
  onToggleAccount: (id: string) => void;
  onRefreshAccount: (id: string) => void;
  onDeleteAccount: (id: string) => void;
  onCopyModel: (id: string, label?: string) => void;
}

export const ClaudeAccountCard: Component<ClaudeAccountCardProps> = (props) => {
  const isBusy = () => props.actionLoading === props.account.id;

  return (
    <div class={`bg-theme-card border transition-all duration-200 overflow-hidden ${
      props.account.is_active 
        ? props.account.is_rate_limited 
          ? 'border-theme-rose/50 bg-theme-rose/10' 
          : 'border-theme-border hover:border-[#D97757]/50' 
        : 'border-theme-border/40 opacity-60 bg-theme-bg/40'
    }`}>
      {/* Account Main Row */}
      <div class="p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div class="space-y-1.5 flex-1 min-w-0">
          <div class="flex items-center gap-2.5 flex-wrap">
            <span class="font-mono text-xs font-bold text-theme-text select-text">
              {props.account.email || props.account.id}
            </span>

            {/* Status Badge */}
            <Show when={props.account.is_active}>
              <Show 
                when={props.account.is_rate_limited}
                fallback={
                  <span class="text-xs font-bold px-2 py-0.5 bg-[#D97757]/15 text-[#D97757] border border-[#D97757]/40 flex items-center gap-1">
                    <span class="w-1.5 h-1.5 bg-[#D97757] animate-pulse" />
                    Ready / Active
                  </span>
                }
              >
                <span class="text-xs font-bold px-2 py-0.5 bg-theme-rose/20 text-theme-rose border border-theme-rose/40 flex items-center gap-1">
                  <span class="w-1.5 h-1.5 bg-theme-rose" />
                  Cooldown ({props.account.rate_limited_seconds_left}s)
                </span>
              </Show>
            </Show>

            <Show when={!props.account.is_active}>
              <span class="text-xs font-bold px-2 py-0.5 bg-theme-text/10 text-theme-text/50 border border-theme-border">
                Disabled
              </span>
            </Show>

            {/* Plan badge */}
            <Show when={props.account.plan_type || props.quotaData?.plan}>
              <span class="text-xs font-mono px-2 py-0.5 bg-[#D97757]/10 text-[#D97757] border border-[#D97757]/30 font-bold uppercase">
                {props.account.plan_type || props.quotaData?.plan}
              </span>
            </Show>
          </div>

          <div class="flex items-center gap-3 text-xs text-theme-text/60 flex-wrap">
            <Show when={props.account.org_id}>
              <span class="flex items-center gap-1 font-mono text-xs select-text">
                <Building2 size={13} class="text-[#D97757] shrink-0" />
                Org ID: {props.account.org_id}
              </span>
              <span class="text-theme-border">•</span>
            </Show>
            <span class="font-mono text-xs text-theme-text/70 select-text">
              ID: {props.account.id}
            </span>
          </div>
        </div>

        {/* Action Buttons */}
        <div class="flex items-center gap-2 self-end sm:self-center shrink-0">
          {/* Quota Button */}
          <button
            onClick={() => props.onToggleQuota(props.account.id)}
            class={`px-3 py-2 border text-xs font-semibold transition-all flex items-center gap-1.5 ${
              props.isExpanded
                ? 'bg-[#D97757] text-white border-[#D97757] font-bold shadow-sm'
                : 'bg-theme-bg border-theme-border text-theme-text/80 hover:text-[#D97757] hover:border-[#D97757]/50'
            }`}
            title="View Usage Quota"
          >
            <Show when={props.loadingQuota} fallback={<Gauge size={15} />}>
              <RefreshCw size={15} class="animate-spin text-[#D97757]" />
            </Show>
            <span>Quota</span>
            <Show when={props.isExpanded} fallback={<ChevronDown size={13} />}>
              <ChevronUp size={13} />
            </Show>
          </button>

          <button
            onClick={() => props.onToggleAccount(props.account.id)}
            disabled={isBusy()}
            class={`p-2 border text-xs font-semibold transition-all ${
              props.account.is_active
                ? 'bg-theme-bg border-theme-border text-theme-text/70 hover:text-theme-rose hover:border-theme-rose/50'
                : 'bg-[#D97757]/15 border border-[#D97757]/30 text-[#D97757] hover:bg-[#D97757]/25'
            }`}
            title={props.account.is_active ? 'Disable Account' : 'Enable Account'}
          >
            <Power size={16} />
          </button>

          <button
            onClick={() => props.onRefreshAccount(props.account.id)}
            disabled={isBusy()}
            class="p-2 bg-theme-bg border border-theme-border text-theme-text/70 hover:text-[#D97757] hover:border-[#D97757]/50 transition-all"
            title="Refresh Token"
          >
            <RefreshCw size={16} class={isBusy() ? 'animate-spin text-[#D97757]' : ''} />
          </button>

          <button
            onClick={() => props.onDeleteAccount(props.account.id)}
            disabled={isBusy()}
            class="p-2 bg-theme-bg border border-theme-border text-theme-text/70 hover:text-theme-rose hover:border-theme-rose/50 transition-all"
            title="Delete Account"
          >
            <Trash2 size={16} />
          </button>
        </div>
      </div>

      {/* Quota Accordion */}
      <Show when={props.isExpanded}>
        <ClaudeQuotaView
          accountId={props.account.id}
          quotaData={props.quotaData}
          isLoading={props.loadingQuota}
          copiedModelId={props.copiedModelId}
          onRefresh={props.onFetchQuota}
          onCopyModel={props.onCopyModel}
        />
      </Show>
    </div>
  );
};
