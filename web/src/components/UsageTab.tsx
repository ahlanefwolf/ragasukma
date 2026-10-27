import { Component, createSignal, onMount, For, Show } from 'solid-js';
import {
  BarChart3,
  RefreshCw,
  Trash2,
  Zap,
  RotateCcw,
  Clock,
  Cpu,
  Layers,
  ArrowDownLeft,
  ArrowUpRight,
  ShieldCheck,
  AlertCircle,
  ChevronLeft,
  ChevronRight,
  Calendar
} from 'lucide-solid';
import { api, LlmUsageRecord, LlmUsageSummary } from '../api';

const getTodayDateString = () => {
  const d = new Date();
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
};

interface UsageTabProps {
  showToast: (msg: string) => void;
}

export const UsageTab: Component<UsageTabProps> = (props) => {
  const [summary, setSummary] = createSignal<LlmUsageSummary>({
    total_requests: 0,
    total_input_tokens: 0,
    total_cached_tokens: 0,
    total_output_tokens: 0,
    total_tokens: 0,
    cache_hit_rate_pct: 0,
    estimated_saved_tokens: 0,
  });
  const [records, setRecords] = createSignal<LlmUsageRecord[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [selectedProvider, setSelectedProvider] = createSignal<string>('all');
  const [selectedDate, setSelectedDate] = createSignal<string>(getTodayDateString());
  const [showClearConfirm, setShowClearConfirm] = createSignal(false);

  // Pagination Signals
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(25);
  const [total, setTotal] = createSignal(0);
  const [totalPages, setTotalPages] = createSignal(1);

  const fetchUsageData = async (targetPage = page(), provider = selectedProvider(), date = selectedDate()) => {
    setLoading(true);
    try {
      const [sumData, paginatedData] = await Promise.all([
        api.getUsageSummary(provider, date),
        api.getUsageRecords(provider, date, targetPage, limit()),
      ]);
      setSummary(sumData);
      setRecords(paginatedData.items || []);
      setTotal(paginatedData.total || 0);
      setPage(paginatedData.page || 1);
      setTotalPages(paginatedData.total_pages || 1);
    } catch {
      props.showToast('Failed to load usage data');
    } finally {
      setLoading(false);
    }
  };

  const handleProviderChange = (provider: string) => {
    setSelectedProvider(provider);
    setPage(1);
    fetchUsageData(1, provider, selectedDate());
  };

  const handleDateChange = (date: string) => {
    setSelectedDate(date);
    setPage(1);
    fetchUsageData(1, selectedProvider(), date);
  };

  const handlePrevPage = () => {
    if (page() > 1 && !loading()) {
      const prev = page() - 1;
      setPage(prev);
      fetchUsageData(prev);
    }
  };

  const handleNextPage = () => {
    if (page() < totalPages() && !loading()) {
      const next = page() + 1;
      setPage(next);
      fetchUsageData(next);
    }
  };

  const handleClear = async () => {
    setShowClearConfirm(false);
    try {
      const res = await api.clearUsage();
      if (res.success) {
        props.showToast('Usage history cleared successfully');
        setPage(1);
        fetchUsageData(1);
      } else {
        props.showToast('Failed to clear usage history');
      }
    } catch {
      props.showToast('Error clearing usage history');
    }
  };

  const formatNumber = (num: number) => {
    return (num || 0).toLocaleString();
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  };

  onMount(() => {
    fetchUsageData(1);
  });

  return (
    <div class="space-y-6 animate-fade-in pb-12">
      {/* Header section */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 p-5 bg-theme-surface border border-theme-border shadow-sm">
        <div class="flex items-center gap-3">
          <div class="p-2.5 bg-theme-card border border-theme-border text-theme-cyan">
            <BarChart3 size={24} />
          </div>
          <div>
            <h1 class="text-xl font-heading font-bold text-theme-text tracking-wide uppercase">
              Token Usage & Metrics
            </h1>
            <p class="text-xs text-theme-text/60 font-mono mt-0.5">
              Real-time audit log of LLM inferences, cached tokens & prompt volume
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onClick={() => fetchUsageData(page())}
            disabled={loading()}
            class="px-3.5 py-2 text-xs font-semibold bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-2 disabled:opacity-50"
            title="Refresh Metrics"
          >
            <RefreshCw size={14} class={loading() ? 'animate-spin text-theme-cyan' : ''} />
            <span>Refresh</span>
          </button>

          <button
            onClick={() => setShowClearConfirm(true)}
            class="px-3.5 py-2 text-xs font-semibold bg-theme-rose/10 border border-theme-rose/30 text-theme-rose hover:bg-theme-rose/20 transition-all flex items-center gap-2"
            title="Clear Log History"
          >
            <Trash2 size={14} />
            <span>Clear Log</span>
          </button>
        </div>
      </div>

      {/* Confirmation Modal */}
      <Show when={showClearConfirm()}>
        <div class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div class="bg-theme-surface border border-theme-rose/50 p-6 max-w-sm w-full space-y-4 shadow-2xl animate-fade-in">
            <div class="flex items-center gap-3 text-theme-rose">
              <AlertCircle size={22} />
              <h3 class="font-heading font-bold text-sm tracking-wide uppercase">
                Clear Usage Records?
              </h3>
            </div>
            <p class="text-xs text-theme-text/70 leading-relaxed font-mono">
              Are you sure you want to delete all stored LLM token usage records? This action cannot be undone.
            </p>
            <div class="flex justify-end gap-2 pt-2 border-t border-theme-border">
              <button
                onClick={() => setShowClearConfirm(false)}
                class="px-3 py-1.5 text-xs font-semibold bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text"
              >
                Cancel
              </button>
              <button
                onClick={handleClear}
                class="px-3 py-1.5 text-xs bg-theme-rose text-white hover:bg-theme-rose/90 font-bold"
              >
                Yes, Clear All
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Hero Stats Grid */}
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        {/* Total Requests */}
        <div class="p-4 bg-theme-card border border-theme-border relative overflow-hidden group hover:border-theme-cyan/50 transition-colors">
          <div class="flex items-center justify-between text-theme-text/60 mb-2">
            <span class="text-xs font-mono font-semibold uppercase tracking-wider">Total Requests</span>
            <Zap size={16} class="text-gi-status-warning group-hover:scale-110 transition-transform" />
          </div>
          <div class="text-2xl font-mono font-bold text-theme-text">
            {formatNumber(summary().total_requests)}
          </div>
          <div class="text-xs text-theme-text/50 font-mono mt-1">
            Inference sessions executed
          </div>
        </div>

        {/* Total Input Tokens */}
        <div class="p-4 bg-theme-card border border-theme-border relative overflow-hidden group hover:border-theme-cyan/50 transition-colors">
          <div class="flex items-center justify-between text-theme-text/60 mb-2">
            <span class="text-xs font-mono font-semibold uppercase tracking-wider">Total Input Tokens</span>
            <ArrowDownLeft size={16} class="text-gi-status-info group-hover:scale-110 transition-transform" />
          </div>
          <div class="text-2xl font-mono font-bold text-theme-text">
            {formatNumber(summary().total_input_tokens)}
          </div>
          <div class="text-xs text-theme-text/50 font-mono mt-1">
            Prompt & context payload
          </div>
        </div>

        {/* Cached Tokens */}
        <div class="p-4 bg-theme-card border border-theme-border relative overflow-hidden group hover:border-gi-status-success/50 transition-colors">
          <div class="flex items-center justify-between text-theme-text/60 mb-2">
            <span class="text-xs font-mono font-semibold uppercase tracking-wider">Cached Tokens</span>
            <RotateCcw size={16} class="text-gi-status-success group-hover:scale-110 transition-transform" />
          </div>
          <div class="flex items-baseline gap-2">
            <span class="text-2xl font-mono font-bold text-gi-status-success">
              {formatNumber(summary().total_cached_tokens)}
            </span>
            <Show when={summary().cache_hit_rate_pct > 0}>
              <span class="text-2xs font-mono px-1.5 py-0.5 bg-gi-status-success/20 text-gi-status-success border border-gi-status-success/30 font-bold">
                {summary().cache_hit_rate_pct.toFixed(1)}% HIT
              </span>
            </Show>
          </div>
          <div class="text-xs text-gi-status-success/70 font-mono mt-1">
            Zero-latency KV cache reads
          </div>
        </div>

        {/* Output Tokens */}
        <div class="p-4 bg-theme-card border border-theme-border relative overflow-hidden group hover:border-theme-cyan/50 transition-colors">
          <div class="flex items-center justify-between text-theme-text/60 mb-2">
            <span class="text-xs font-mono font-semibold uppercase tracking-wider">Output Tokens</span>
            <ArrowUpRight size={16} class="text-theme-cyan group-hover:scale-110 transition-transform" />
          </div>
          <div class="text-2xl font-mono font-bold text-theme-cyan">
            {formatNumber(summary().total_output_tokens)}
          </div>
          <div class="text-xs text-theme-text/50 font-mono mt-1">
            Assistant generated words
          </div>
        </div>
      </div>

      {/* Filters: Provider & Date */}
      <div class="flex flex-wrap items-center justify-between gap-3 border-b border-theme-border pb-3">
        {/* Provider Filter Tabs */}
        <div class="flex flex-wrap items-center gap-2">
          <button
            onClick={() => handleProviderChange('all')}
            class={`px-3 py-1.5 text-xs font-semibold tracking-wide border transition-all ${
              selectedProvider() === 'all'
                ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-text'
            }`}
          >
            All Providers
          </button>

          <button
            onClick={() => handleProviderChange('antigravity')}
            class={`px-3 py-1.5 text-xs font-semibold tracking-wide border transition-all flex items-center gap-1.5 ${
              selectedProvider() === 'antigravity'
                ? 'bg-theme-rose text-white border-theme-rose font-bold shadow-sm'
                : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-text'
            }`}
          >
            <span>Antigravity</span>
            <span class="text-2xs font-mono opacity-80">(ag)</span>
          </button>

          <button
            onClick={() => handleProviderChange('codex')}
            class={`px-3 py-1.5 text-xs font-semibold tracking-wide border transition-all flex items-center gap-1.5 ${
              selectedProvider() === 'codex'
                ? 'bg-gi-status-success text-white border-gi-status-success font-bold shadow-sm'
                : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-text'
            }`}
          >
            <span>OpenAI Codex</span>
            <span class="text-2xs font-mono opacity-80">(cx)</span>
          </button>
        </div>

        {/* Date Filter Controls */}
        <div class="flex items-center gap-2">
          <div class="flex items-center gap-2 px-3 py-1.5 bg-theme-card border border-theme-border text-xs text-theme-text font-mono">
            <Calendar size={14} class="text-theme-cyan shrink-0" />
            <input
              type="date"
              value={selectedDate() === 'all' ? '' : selectedDate()}
              onInput={(e) => handleDateChange(e.currentTarget.value || 'all')}
              class="bg-transparent text-theme-text focus:outline-none cursor-pointer text-xs font-mono [color-scheme:dark]"
            />
          </div>

          <button
            onClick={() => handleDateChange(getTodayDateString())}
            class={`px-2.5 py-1.5 text-xs font-semibold border transition-all ${
              selectedDate() === getTodayDateString()
                ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-text'
            }`}
            title="Filter today"
          >
            Today
          </button>

          <button
            onClick={() => handleDateChange('all')}
            class={`px-2.5 py-1.5 text-xs font-semibold border transition-all ${
              selectedDate() === 'all'
                ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                : 'bg-theme-card text-theme-text/70 border-theme-border hover:text-theme-text'
            }`}
            title="Show all dates"
          >
            All Dates
          </button>
        </div>
      </div>

      {/* Records Table Card */}
      <div class="bg-theme-card border border-theme-border shadow-sm overflow-hidden">
        <div class="p-4 border-b border-theme-border flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold font-heading uppercase tracking-wide text-theme-text">
              Inference Audit Trail
            </span>
            <Show when={selectedDate() !== 'all'}>
              <span class="text-2xs font-mono px-2 py-0.5 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/30">
                {selectedDate() === getTodayDateString() ? 'Today' : selectedDate()}
              </span>
            </Show>
          </div>
          <span class="text-xs font-mono text-theme-text/50">
            {total() > 0
              ? `Showing ${(page() - 1) * limit() + 1}–${Math.min(page() * limit(), total())} of ${formatNumber(total())} records`
              : '0 records'}
          </span>
        </div>

        <Show
          when={records().length > 0}
          fallback={
            <div class="p-12 text-center text-theme-text/50 font-mono text-xs">
              <BarChart3 size={36} class="mx-auto mb-3 opacity-30" />
              No token usage records found for the selected filter.
            </div>
          }
        >
          <div class="overflow-x-auto">
            <table class="w-full text-left text-xs font-mono border-collapse">
              <thead>
                <tr class="bg-theme-card border-b border-theme-border text-theme-text/60">
                  <th class="py-2.5 px-4 font-semibold">Time</th>
                  <th class="py-2.5 px-4 font-semibold">Provider</th>
                  <th class="py-2.5 px-4 font-semibold">Model</th>
                  <th class="py-2.5 px-4 font-semibold text-right">Input</th>
                  <th class="py-2.5 px-4 font-semibold text-right">Cached</th>
                  <th class="py-2.5 px-4 font-semibold text-right">Output</th>
                  <th class="py-2.5 px-4 font-semibold text-right">Total</th>
                  <th class="py-2.5 px-4 font-semibold text-right">Latency</th>
                  <th class="py-2.5 px-4 font-semibold text-center">Status</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-theme-border/50">
                <For each={records()}>
                  {(rec) => {
                    const isAg = () => rec.provider.includes('antigravity');
                    const isCx = () => rec.provider.includes('codex');
                    const hasCache = () => rec.cached_tokens > 0;

                    return (
                      <tr class="hover:bg-theme-card/40 transition-colors">
                        {/* Time */}
                        <td class="py-3 px-4 whitespace-nowrap text-theme-text/80">
                          <div class="flex flex-col">
                            <span class="font-bold text-theme-text">{rec.time_str}</span>
                            <span class="text-2xs text-theme-text/50">{rec.date_str}</span>
                          </div>
                        </td>

                        {/* Provider */}
                        <td class="py-3 px-4 whitespace-nowrap">
                          <span
                            class={`px-2 py-0.5 text-2xs font-bold uppercase border ${
                              isAg()
                                ? 'bg-theme-rose/15 text-theme-rose border-theme-rose/30'
                                : isCx()
                                ? 'bg-gi-status-success/15 text-gi-status-success border-gi-status-success/30'
                                : 'bg-theme-cyan/15 text-theme-cyan border-theme-cyan/30'
                            }`}
                          >
                            {rec.provider}
                          </span>
                        </td>

                        {/* Model */}
                        <td class="py-3 px-4 whitespace-nowrap text-theme-text font-semibold">
                          <span class="max-w-[180px] truncate block" title={rec.model}>
                            {rec.model}
                          </span>
                        </td>

                        {/* Input Tokens */}
                        <td class="py-3 px-4 text-right whitespace-nowrap text-theme-text/90">
                          {formatNumber(rec.input_tokens)}
                        </td>

                        {/* Cached Tokens */}
                        <td class="py-3 px-4 text-right whitespace-nowrap">
                          <Show
                            when={hasCache()}
                            fallback={<span class="text-theme-text/60">0</span>}
                          >
                            <span class="text-gi-status-success font-bold bg-gi-status-success/10 px-1.5 py-0.5 border border-gi-status-success/30">
                              ↻{formatNumber(rec.cached_tokens)}
                            </span>
                          </Show>
                        </td>

                        {/* Output Tokens */}
                        <td class="py-3 px-4 text-right whitespace-nowrap text-theme-cyan font-semibold">
                          {formatNumber(rec.output_tokens)}
                        </td>

                        {/* Total Tokens */}
                        <td class="py-3 px-4 text-right whitespace-nowrap font-bold text-theme-text">
                          {formatNumber(rec.total_tokens)}
                        </td>

                        {/* Latency */}
                        <td class="py-3 px-4 text-right whitespace-nowrap text-theme-text/60">
                          {formatDuration(rec.duration_ms)}
                        </td>

                        {/* Status */}
                        <td class="py-3 px-4 text-center whitespace-nowrap">
                          <span
                            class={`px-1.5 py-0.5 text-2xs uppercase font-bold ${
                              rec.status === 'success'
                                ? 'bg-gi-status-success/20 text-gi-status-success'
                                : 'bg-theme-rose/20 text-theme-rose'
                            }`}
                          >
                            {rec.status}
                          </span>
                        </td>
                      </tr>
                    );
                  }}
                </For>
              </tbody>
            </table>
          </div>

          {/* Pagination Navigation Footer */}
          <Show when={totalPages() > 1}>
            <div class="p-3 border-t border-theme-border bg-theme-card flex items-center justify-between">
              <span class="text-xs font-mono text-theme-text/60">
                Page <span class="text-theme-text font-bold">{page()}</span> of{' '}
                <span class="text-theme-text font-bold">{totalPages()}</span>
              </span>

              <div class="flex items-center gap-2">
                <button
                  onClick={handlePrevPage}
                  disabled={page() <= 1 || loading()}
                  class="px-3 py-1.5 text-xs font-semibold bg-theme-surface border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <ChevronLeft size={14} />
                  <span>Previous</span>
                </button>

                <button
                  onClick={handleNextPage}
                  disabled={page() >= totalPages() || loading()}
                  class="px-3 py-1.5 text-xs font-semibold bg-theme-surface border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <span>Next</span>
                  <ChevronRight size={14} />
                </button>
              </div>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
};