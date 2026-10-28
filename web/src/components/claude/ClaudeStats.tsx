import { Component } from 'solid-js';
import { Layers, CheckCircle2, AlertTriangle } from 'lucide-solid';

interface ClaudeStatsProps {
  totalAccounts: number;
  activeCount: number;
  rateLimitedCount: number;
}

export const ClaudeStats: Component<ClaudeStatsProps> = (props) => {
  return (
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
      <div class="p-4 bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 bg-theme-bg text-[#D97757] border border-theme-border">
          <Layers size={20} />
        </div>
        <div>
          <div class="text-xs text-theme-text/60 font-semibold">Claude Accounts</div>
          <div class="text-lg font-bold font-mono text-theme-text">{props.totalAccounts}</div>
        </div>
      </div>

      <div class="p-4 bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 bg-[#D97757]/15 text-[#D97757] border border-[#D97757]/30">
          <CheckCircle2 size={20} />
        </div>
        <div>
          <div class="text-xs text-theme-text/60 font-semibold">Active Ready</div>
          <div class="text-lg font-bold font-mono text-[#D97757]">{props.activeCount}</div>
        </div>
      </div>

      <div class="p-4 bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 bg-theme-rose/15 text-theme-rose border border-theme-rose/30">
          <AlertTriangle size={20} />
        </div>
        <div>
          <div class="text-xs text-theme-text/60 font-semibold">Rate Limited / Cooldown</div>
          <div class="text-lg font-bold font-mono text-theme-rose">{props.rateLimitedCount}</div>
        </div>
      </div>
    </div>
  );
};
