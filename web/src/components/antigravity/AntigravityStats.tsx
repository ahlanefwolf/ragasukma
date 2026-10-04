import { Component } from 'solid-js';
import { Layers, CheckCircle2, AlertTriangle } from 'lucide-solid';

interface AntigravityStatsProps {
  totalAccounts: number;
  activeCount: number;
  rateLimitedCount: number;
}

export const AntigravityStats: Component<AntigravityStatsProps> = (props) => {
  return (
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
      <div class="p-4 bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 bg-theme-bg text-theme-cyan border border-theme-border">
          <Layers size={20} />
        </div>
        <div>
          <div class="text-xs text-theme-text/60 font-semibold">Connected Accounts</div>
          <div class="text-lg font-bold font-mono text-theme-text">{props.totalAccounts}</div>
        </div>
      </div>

      <div class="p-4 bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
          <CheckCircle2 size={20} />
        </div>
        <div>
          <div class="text-xs text-theme-text/60 font-semibold">Active Ready Accounts</div>
          <div class="text-lg font-bold font-mono text-theme-cyan">{props.activeCount}</div>
        </div>
      </div>

      <div class="p-4 bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 bg-theme-rose/15 text-theme-rose border border-theme-rose/30">
          <AlertTriangle size={20} />
        </div>
        <div>
          <div class="text-xs text-theme-text/60 font-semibold">Cooldown (429)</div>
          <div class="text-lg font-bold font-mono text-theme-rose">{props.rateLimitedCount}</div>
        </div>
      </div>
    </div>
  );
};
