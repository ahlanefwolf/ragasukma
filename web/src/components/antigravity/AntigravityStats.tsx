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
      <div class="p-4 rounded-2xl bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-theme-bg text-theme-cyan border border-theme-border">
          <Layers size={20} />
        </div>
        <div>
          <div class="text-[11px] text-theme-text/60 font-semibold">Total Akun Terhubung</div>
          <div class="text-lg font-bold font-mono text-theme-text">{props.totalAccounts}</div>
        </div>
      </div>

      <div class="p-4 rounded-2xl bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
          <CheckCircle2 size={20} />
        </div>
        <div>
          <div class="text-[11px] text-theme-text/60 font-semibold">Akun Siap Pakai</div>
          <div class="text-lg font-bold font-mono text-emerald-400">{props.activeCount}</div>
        </div>
      </div>

      <div class="p-4 rounded-2xl bg-theme-card border border-theme-border flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20">
          <AlertTriangle size={20} />
        </div>
        <div>
          <div class="text-[11px] text-theme-text/60 font-semibold">Cooldown (429)</div>
          <div class="text-lg font-bold font-mono text-amber-400">{props.rateLimitedCount}</div>
        </div>
      </div>
    </div>
  );
};
