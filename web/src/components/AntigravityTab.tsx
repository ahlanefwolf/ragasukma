import { Component, createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { 
  Rocket, 
  Plus, 
  Trash2, 
  RefreshCw, 
  CheckCircle2, 
  AlertTriangle, 
  Power, 
  ShieldCheck, 
  HelpCircle,
  KeyRound,
  Mail,
  FolderGit2,
  Sparkles,
  Layers,
  X,
  ExternalLink,
  LogIn,
  Gauge,
  ChevronDown,
  ChevronUp,
  Clock,
  Zap
} from 'lucide-solid';
import { api, AntigravityAccount, AntigravityQuotaResponse } from '../api';

interface AntigravityTabProps {
  showToast: (msg: string) => void;
}

export const AntigravityTab: Component<AntigravityTabProps> = (props) => {
  const [accounts, setAccounts] = createSignal<AntigravityAccount[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [actionLoading, setActionLoading] = createSignal<string | null>(null);
  const [isOAuthLoading, setIsOAuthLoading] = createSignal(false);

  // Quota State
  const [accountQuotas, setAccountQuotas] = createSignal<Record<string, AntigravityQuotaResponse>>({});
  const [loadingQuota, setLoadingQuota] = createSignal<Record<string, boolean>>({});
  const [expandedQuota, setExpandedQuota] = createSignal<Record<string, boolean>>({});

  // Form State
  const [showAddModal, setShowAddModal] = createSignal(false);
  const [refreshToken, setRefreshToken] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [projectId, setProjectId] = createSignal('');
  const [isSubmitting, setIsSubmitting] = createSignal(false);

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
        bar: 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.5)]',
        badge: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30',
        text: 'text-emerald-400',
      };
    }
    if (pct >= 20) {
      return {
        bar: 'bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.5)]',
        badge: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
        text: 'text-amber-400',
      };
    }
    return {
      bar: 'bg-rose-400 shadow-[0_0_8px_rgba(244,63,94,0.5)]',
      badge: 'bg-rose-500/20 text-rose-400 border-rose-500/30',
      text: 'text-rose-400',
    };
  };

  const handleFetchQuota = async (id: string) => {
    setLoadingQuota(prev => ({ ...prev, [id]: true }));
    try {
      const res = await api.getAntigravityAccountQuota(id);
      setAccountQuotas(prev => ({ ...prev, [id]: res }));
      if (res.status === 'error' && res.error) {
        props.showToast(`Kuota: ${res.error}`);
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Gagal memuat kuota');
    } finally {
      setLoadingQuota(prev => ({ ...prev, [id]: false }));
    }
  };

  const toggleQuota = (id: string) => {
    const nextState = !expandedQuota()[id];
    setExpandedQuota(prev => ({ ...prev, [id]: nextState }));
    if (nextState && !accountQuotas()[id] && !loadingQuota()[id]) {
      handleFetchQuota(id);
    }
  };

  const loadAccounts = async (silent = false) => {
    if (!silent) setLoading(true);
    try {
      const list = await api.getAntigravityAccounts();
      setAccounts(Array.isArray(list) ? list : []);
    } catch {
      setAccounts([]);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadAccounts();

    // Listen for OAuth callback success message from popup window
    const handleMessage = (e: MessageEvent) => {
      if (e.data?.type === 'ANTIGRAVITY_OAUTH_SUCCESS') {
        props.showToast('Login Google OAuth berhasil! Akun telah ditambahkan ke pool.');
        loadAccounts(true);
      }
    };

    window.addEventListener('message', handleMessage);
    onCleanup(() => window.removeEventListener('message', handleMessage));
  });

  const handleOAuthLogin = async () => {
    setIsOAuthLoading(true);
    try {
      const authUrl = await api.getAntigravityAuthUrl();
      if (!authUrl) {
        props.showToast('Gagal membuat URL otorisasi Google OAuth');
        return;
      }

      // Buka popup window OAuth Google
      const width = 600;
      const height = 700;
      const left = window.screenX + (window.outerWidth - width) / 2;
      const top = window.screenY + (window.outerHeight - height) / 2;

      const popup = window.open(
        authUrl,
        'AntigravityOAuth',
        `width=${width},height=${height},left=${left},top=${top},status=no,menubar=no,toolbar=no`
      );

      if (!popup || popup.closed || typeof popup.closed === 'undefined') {
        // Fallback jika popup terblokir: arahkan tab langsung
        window.location.href = authUrl;
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Gagal memulai login OAuth');
    } finally {
      setIsOAuthLoading(false);
    }
  };

  const handleAddAccount = async (e: Event) => {
    e.preventDefault();
    if (!refreshToken().trim()) {
      props.showToast('Refresh token wajib diisi!');
      return;
    }

    setIsSubmitting(true);
    try {
      const res = await api.addAntigravityAccount(
        refreshToken().trim(),
        email().trim() || undefined,
        projectId().trim() || undefined
      );

      if (res.status === 'success') {
        props.showToast('Akun Antigravity berhasil ditambahkan!');
        setRefreshToken('');
        setEmail('');
        setProjectId('');
        setShowAddModal(false);
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Gagal menambahkan akun');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Terjadi kesalahan sistem');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Yakin ingin menghapus akun ini dari pool Antigravity?')) return;
    setActionLoading(id);
    try {
      const res = await api.deleteAntigravityAccount(id);
      if (res.status === 'success') {
        props.showToast('Akun berhasil dihapus');
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Gagal menghapus akun');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Terjadi kesalahan');
    } finally {
      setActionLoading(null);
    }
  };

  const handleRefresh = async (id: string) => {
    setActionLoading(id);
    try {
      const res = await api.refreshAntigravityAccount(id);
      if (res.status === 'success') {
        props.showToast('Token & Project ID berhasil di-refresh!');
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Refresh token gagal');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Gagal terhubung ke server');
    } finally {
      setActionLoading(null);
    }
  };

  const handleToggle = async (id: string) => {
    setActionLoading(id);
    try {
      const res = await api.toggleAntigravityAccount(id);
      if (res.status === 'success') {
        props.showToast(`Status akun diperbarui (${res.is_active ? 'Aktif' : 'Nonaktif'})`);
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Gagal memperbarui status');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Terjadi kesalahan');
    } finally {
      setActionLoading(null);
    }
  };

  const activeCount = () => accounts().filter(a => a.is_active && !a.is_rate_limited).length;
  const rateLimitedCount = () => accounts().filter(a => a.is_rate_limited).length;

  return (
    <div class="space-y-6 animate-fade-in select-none">
      {/* Header Banner */}
      <div class="p-6 rounded-3xl bg-theme-card border border-theme-border flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-sm">
        <div class="flex items-center gap-4">
          <div class="w-14 h-14 rounded-2xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-center text-amber-400 shrink-0 shadow-lg shadow-amber-500/5">
            <Rocket size={28} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xl font-carter tracking-wide text-theme-text">
                ANTIGRAVITY POOL
              </h2>
              <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-400 border border-amber-500/30 font-bold uppercase">
                Google Multi-OAuth
              </span>
            </div>
            <p class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              Koneksikan akun Google OAuth Antigravity dengan 1-klik, auto failover & round-robin load balancing.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2.5 w-full md:w-auto flex-wrap sm:flex-nowrap">
          <button
            onClick={() => loadAccounts()}
            class="p-2.5 rounded-xl bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan hover:border-theme-cyan/50 transition-all flex items-center justify-center"
            title="Muat Ulang"
          >
            <RefreshCw size={18} class={loading() ? 'animate-spin text-theme-cyan' : ''} />
          </button>

          {/* 1-Click Google OAuth Button */}
          <button
            onClick={handleOAuthLogin}
            disabled={isOAuthLoading()}
            class="flex-1 sm:flex-initial flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-theme-cyan text-theme-card font-bold text-xs shadow-md anime-shadow-green hover:opacity-95 active:scale-95 transition-all"
          >
            <Show when={isOAuthLoading()} fallback={<LogIn size={16} />}>
              <RefreshCw size={16} class="animate-spin" />
            </Show>
            <span>Login Google (OAuth)</span>
          </button>

          <button
            onClick={() => setShowAddModal(true)}
            class="p-2.5 rounded-xl bg-theme-bg border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/50 text-xs font-semibold transition-all flex items-center gap-1.5"
            title="Input Token Manual"
          >
            <Plus size={16} />
            <span class="hidden sm:inline">Manual</span>
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div class="p-4 rounded-2xl bg-theme-card border border-theme-border flex items-center gap-3">
          <div class="p-2.5 rounded-xl bg-theme-bg text-theme-cyan border border-theme-border">
            <Layers size={20} />
          </div>
          <div>
            <div class="text-[11px] text-theme-text/60 font-semibold">Total Akun Terhubung</div>
            <div class="text-lg font-bold font-mono text-theme-text">{accounts().length}</div>
          </div>
        </div>

        <div class="p-4 rounded-2xl bg-theme-card border border-theme-border flex items-center gap-3">
          <div class="p-2.5 rounded-xl bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            <CheckCircle2 size={20} />
          </div>
          <div>
            <div class="text-[11px] text-theme-text/60 font-semibold">Akun Siap Pakai</div>
            <div class="text-lg font-bold font-mono text-emerald-400">{activeCount()}</div>
          </div>
        </div>

        <div class="p-4 rounded-2xl bg-theme-card border border-theme-border flex items-center gap-3">
          <div class="p-2.5 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20">
            <AlertTriangle size={20} />
          </div>
          <div>
            <div class="text-[11px] text-theme-text/60 font-semibold">Cooldown (429)</div>
            <div class="text-lg font-bold font-mono text-amber-400">{rateLimitedCount()}</div>
          </div>
        </div>
      </div>

      {/* Account List Section */}
      <div class="space-y-3">
        <div class="flex items-center justify-between px-1">
          <h3 class="text-sm font-bold text-theme-text flex items-center gap-2">
            <ShieldCheck size={16} class="text-theme-cyan" />
            Daftar Akun Antigravity
          </h3>
          <span class="text-xs text-theme-text/40 font-mono">
            Rotasi Otomatis Round-Robin
          </span>
        </div>

        <Show
          when={!loading()}
          fallback={
            <div class="p-12 text-center text-xs text-theme-text/50 rounded-2xl bg-theme-card border border-theme-border animate-pulse">
              Memuat daftar akun Antigravity...
            </div>
          }
        >
          <Show
            when={accounts().length > 0}
            fallback={
              <div class="p-12 text-center rounded-2xl bg-theme-card border border-theme-border/60 space-y-4">
                <div class="w-14 h-14 mx-auto rounded-2xl bg-amber-500/10 border border-amber-500/20 flex items-center justify-center text-amber-400">
                  <Rocket size={28} />
                </div>
                <div class="space-y-1">
                  <div class="text-sm font-bold text-theme-text">Belum Ada Akun Antigravity Terhubung</div>
                  <p class="text-xs text-theme-text/50 max-w-sm mx-auto">
                    Klik tombol <b>Login Google (OAuth)</b> untuk menghubungkan akun Google Antigravity dalam 1 kali klik.
                  </p>
                </div>
                <button
                  onClick={handleOAuthLogin}
                  disabled={isOAuthLoading()}
                  class="px-5 py-2.5 rounded-xl bg-theme-cyan text-theme-card font-bold text-xs shadow-md anime-shadow-green hover:opacity-95 active:scale-95 transition-all inline-flex items-center gap-2"
                >
                  <LogIn size={16} />
                  <span>Login dengan Google Sekarang</span>
                </button>
              </div>
            }
          >
            <div class="grid grid-cols-1 gap-3">
              <For each={accounts()}>
                {(acc) => {
                  const isBusy = () => actionLoading() === acc.id;
                  const isQuotaLoading = () => !!loadingQuota()[acc.id];
                  const isExpanded = () => !!expandedQuota()[acc.id];
                  const quotaData = () => accountQuotas()[acc.id];

                  return (
                    <div class={`rounded-2xl bg-theme-card border transition-all duration-200 overflow-hidden ${
                      acc.is_active 
                        ? acc.is_rate_limited 
                          ? 'border-amber-500/40 bg-amber-500/5' 
                          : 'border-theme-border hover:border-theme-cyan/50' 
                        : 'border-theme-border/40 opacity-60 bg-theme-bg/40'
                    }`}>
                      {/* Account Main Row */}
                      <div class="p-4 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                        <div class="space-y-1.5 flex-1 min-w-0">
                          <div class="flex items-center gap-2.5 flex-wrap">
                            <span class="font-mono text-xs font-bold text-theme-text">
                              {acc.email || acc.id}
                            </span>

                            {/* Status Badge */}
                            <Show when={acc.is_active}>
                              <Show 
                                when={acc.is_rate_limited}
                                fallback={
                                  <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 flex items-center gap-1">
                                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
                                    Siap / Aktif
                                  </span>
                                }
                              >
                                <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-amber-500/20 text-amber-400 border border-amber-500/30 flex items-center gap-1">
                                  <span class="w-1.5 h-1.5 rounded-full bg-amber-400" />
                                  Cooldown ({acc.rate_limited_seconds_left}s)
                                </span>
                              </Show>
                            </Show>

                            <Show when={!acc.is_active}>
                              <span class="text-[10px] font-bold px-2 py-0.5 rounded-full bg-theme-text/10 text-theme-text/50 border border-theme-border">
                                Dinonaktifkan
                              </span>
                            </Show>

                            {/* Plan badge */}
                            <Show when={quotaData()?.plan}>
                              <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/30 font-bold">
                                {quotaData()?.plan}
                              </span>
                            </Show>
                          </div>

                          <div class="flex items-center gap-3 text-xs text-theme-text/60 flex-wrap">
                            <span class="flex items-center gap-1 font-mono text-[11px]">
                              <FolderGit2 size={13} class="text-theme-cyan shrink-0" />
                              Project: {acc.project_id || 'cloudaicompanion-default'}
                            </span>
                            <span class="text-theme-border">•</span>
                            <span class="font-mono text-[11px] text-theme-text/40">
                              ID: {acc.id}
                            </span>
                          </div>
                        </div>

                        {/* Action Buttons */}
                        <div class="flex items-center gap-2 self-end sm:self-center shrink-0">
                          {/* Tombol Cek Kuota */}
                          <button
                            onClick={() => toggleQuota(acc.id)}
                            class={`px-3 py-2 rounded-xl border text-xs font-semibold transition-all flex items-center gap-1.5 ${
                              isExpanded()
                                ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                                : 'bg-theme-bg border-theme-border text-theme-text/80 hover:text-theme-cyan hover:border-theme-cyan/50'
                            }`}
                            title="Lihat Quota Akun"
                          >
                            <Show when={isQuotaLoading()} fallback={<Gauge size={15} />}>
                              <RefreshCw size={15} class="animate-spin text-theme-cyan" />
                            </Show>
                            <span>Kuota</span>
                            <Show when={isExpanded()} fallback={<ChevronDown size={13} />}>
                              <ChevronUp size={13} />
                            </Show>
                          </button>

                          <button
                            onClick={() => handleToggle(acc.id)}
                            disabled={isBusy()}
                            class={`p-2 rounded-xl border text-xs font-semibold transition-all ${
                              acc.is_active
                                ? 'bg-theme-bg border-theme-border text-theme-text/70 hover:text-amber-400 hover:border-amber-400/50'
                                : 'bg-emerald-500/10 border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/20'
                            }`}
                            title={acc.is_active ? 'Nonaktifkan Akun' : 'Aktifkan Akun'}
                          >
                            <Power size={16} />
                          </button>

                          <button
                            onClick={() => handleRefresh(acc.id)}
                            disabled={isBusy()}
                            class="p-2 rounded-xl bg-theme-bg border border-theme-border text-theme-text/70 hover:text-theme-cyan hover:border-theme-cyan/50 transition-all"
                            title="Refresh Token & Test Onboarding"
                          >
                            <RefreshCw size={16} class={isBusy() ? 'animate-spin text-theme-cyan' : ''} />
                          </button>

                          <button
                            onClick={() => handleDelete(acc.id)}
                            disabled={isBusy()}
                            class="p-2 rounded-xl bg-theme-bg border border-theme-border text-theme-text/70 hover:text-theme-rose hover:border-theme-rose/50 transition-all"
                            title="Hapus Akun"
                          >
                            <Trash2 size={16} />
                          </button>
                        </div>
                      </div>

                      {/* Quota Section Underneath the Account */}
                      <Show when={isExpanded()}>
                        <div class="px-4 pb-4 pt-3 border-t border-theme-border/60 bg-theme-bg/60 space-y-3 animate-fade-in">
                          {/* Panel Header */}
                          <div class="flex items-center justify-between gap-2 flex-wrap pb-1">
                            <div class="flex items-center gap-2">
                              <Gauge size={15} class="text-theme-cyan" />
                              <span class="text-xs font-bold text-theme-text">Status Kuota Antigravity</span>
                              <Show when={quotaData()?.plan}>
                                <span class="text-[10px] font-mono px-2 py-0.5 rounded-full bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20 font-bold">
                                  {quotaData()?.plan}
                                </span>
                              </Show>
                            </div>

                            <button
                              onClick={() => handleFetchQuota(acc.id)}
                              disabled={isQuotaLoading()}
                              class="px-2.5 py-1 rounded-lg bg-theme-card border border-theme-border text-theme-text/70 hover:text-theme-cyan hover:border-theme-cyan text-[11px] font-medium transition-all flex items-center gap-1.5"
                              title="Perbarui data kuota terkini"
                            >
                              <RefreshCw size={12} class={isQuotaLoading() ? 'animate-spin text-theme-cyan' : ''} />
                              <span>{isQuotaLoading() ? 'Memuat...' : 'Refresh Kuota'}</span>
                            </button>
                          </div>

                          {/* Loading Indicator */}
                          <Show when={isQuotaLoading() && !quotaData()}>
                            <div class="p-6 text-center rounded-xl bg-theme-card/50 border border-theme-border flex items-center justify-center gap-3">
                              <RefreshCw size={16} class="animate-spin text-theme-cyan" />
                              <span class="text-xs text-theme-text/60">Mengambil data kuota dari Google Cloud Code API...</span>
                            </div>
                          </Show>

                          {/* Error State */}
                          <Show when={!isQuotaLoading() && quotaData()?.status === 'error' && (!quotaData()?.models || quotaData()?.models.length === 0)}>
                            <div class="p-3.5 rounded-xl bg-rose-500/10 border border-rose-500/30 text-xs text-rose-400 flex items-center justify-between gap-3">
                              <div class="flex items-center gap-2">
                                <AlertTriangle size={16} class="shrink-0" />
                                <span>{quotaData()?.error || 'Gagal membaca kuota akun.'}</span>
                              </div>
                              <button
                                onClick={() => handleFetchQuota(acc.id)}
                                class="px-2.5 py-1 rounded-lg bg-rose-500/20 hover:bg-rose-500/30 text-rose-300 text-[11px] font-bold shrink-0 transition-colors"
                              >
                                Coba Lagi
                              </button>
                            </div>
                          </Show>

                          {/* Quota Data Content */}
                          <Show when={quotaData() && (quotaData()!.weekly?.length > 0 || quotaData()!.models?.length > 0)}>
                            {/* 1. Weekly Quota Cards */}
                            <Show when={quotaData()?.weekly && quotaData()!.weekly.length > 0}>
                              <div class="space-y-1.5">
                                <div class="text-[10px] font-bold text-theme-text/60 uppercase tracking-wider flex items-center gap-1.5">
                                  <Sparkles size={12} class="text-amber-400" />
                                  <span>Kuota Mingguan (Weekly Cap)</span>
                                </div>
                                <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                                  <For each={quotaData()!.weekly}>
                                    {(item) => {
                                      const colors = getQuotaColor(item.remaining_percentage);
                                      return (
                                        <div class="p-3 rounded-xl bg-theme-card border border-theme-border space-y-2 shadow-sm">
                                          <div class="flex items-center justify-between gap-2">
                                            <span class="text-xs font-bold text-theme-text truncate">{item.display_name}</span>
                                            <span class={`text-[10px] font-mono font-bold px-2 py-0.5 rounded-full border ${colors.badge}`}>
                                              {item.remaining_percentage}% Tersedia
                                            </span>
                                          </div>
                                          {/* Bar */}
                                          <div class="w-full h-1.5 rounded-full bg-theme-bg overflow-hidden border border-theme-border/40">
                                            <div
                                              class={`h-full rounded-full transition-all duration-500 ${colors.bar}`}
                                              style={{ width: `${Math.max(2, Math.min(100, item.remaining_percentage))}%` }}
                                            />
                                          </div>
                                          <Show when={item.reset_time}>
                                            <div class="text-[10px] text-theme-text/50 font-mono flex items-center gap-1">
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

                            {/* 2. Per-Model Quota Breakdown */}
                            <Show when={quotaData()?.models && quotaData()!.models.length > 0}>
                              <div class="space-y-1.5 pt-1">
                                <div class="text-[10px] font-bold text-theme-text/60 uppercase tracking-wider flex items-center gap-1.5">
                                  <Zap size={12} class="text-theme-cyan" />
                                  <span>Kuota Model (Jendela 5 Jam)</span>
                                </div>
                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
                                  <For each={quotaData()!.models}>
                                    {(item) => {
                                      const colors = getQuotaColor(item.remaining_percentage);
                                      return (
                                        <div class="p-2.5 rounded-xl bg-theme-card/80 border border-theme-border/80 space-y-2 hover:border-theme-cyan/30 transition-colors">
                                          <div class="flex items-center justify-between gap-1.5">
                                            <span class="text-xs font-semibold text-theme-text truncate" title={item.display_name}>
                                              {item.display_name}
                                            </span>
                                            <span class={`text-[10px] font-mono font-bold px-1.5 py-0.5 rounded-md border shrink-0 ${colors.badge}`}>
                                              {item.remaining_percentage}%
                                            </span>
                                          </div>
                                          {/* Bar */}
                                          <div class="w-full h-1.5 rounded-full bg-theme-bg overflow-hidden border border-theme-border/40">
                                            <div
                                              class={`h-full rounded-full transition-all duration-500 ${colors.bar}`}
                                              style={{ width: `${Math.max(2, Math.min(100, item.remaining_percentage))}%` }}
                                            />
                                          </div>
                                          <Show when={item.reset_time}>
                                            <div class="text-[10px] text-theme-text/50 font-mono flex items-center gap-1 pt-0.5">
                                              <Clock size={10} class="text-theme-cyan shrink-0" />
                                              <span class="truncate">{formatResetTime(item.reset_time)}</span>
                                            </div>
                                          </Show>
                                        </div>
                                      );
                                    }}
                                  </For>
                                </div>
                              </div>
                            </Show>
                          </Show>

                          {/* Empty Quota Info */}
                          <Show when={quotaData() && quotaData()?.status !== 'error' && (!quotaData()?.weekly || quotaData()?.weekly.length === 0) && (!quotaData()?.models || quotaData()?.models.length === 0)}>
                            <div class="p-4 text-center rounded-xl bg-theme-card/50 border border-theme-border text-xs text-theme-text/50">
                              Tidak ada informasi kuota aktif untuk akun ini. Pastikan Google Cloud Project telah terhubung.
                            </div>
                          </Show>
                        </div>
                      </Show>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* OpenAI-Compatible API Endpoint Info Card */}
      <div class="p-5 rounded-2xl bg-theme-card/80 border border-theme-cyan/30 text-xs text-theme-text/80 space-y-3 shadow-lg">
        <div class="flex items-center justify-between">
          <div class="font-bold text-theme-text flex items-center gap-2">
            <span class="p-1 rounded bg-theme-cyan/10 text-theme-cyan font-mono text-[10px] font-extrabold uppercase tracking-wider border border-theme-cyan/30">OpenAI API</span>
            <span>Local Endpoint (OpenAI-Compatible)</span>
          </div>
          <span class="text-[10px] font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
            Port 42128 Ready
          </span>
        </div>

        <p class="leading-relaxed text-theme-text/60">
          Gunakan endpoint ini di aplikasi AI apapun (Zed Editor, Cline, Continue, Cursor, Aider, Claude Code, Python OpenAI SDK, dll) untuk menggunakan model Antigravity:
        </p>

        <div class="p-3 rounded-xl bg-theme-bg/80 border border-theme-border font-mono text-[11px] flex items-center justify-between gap-2 overflow-x-auto">
          <div class="text-theme-cyan select-all">
            Base URL: <span class="text-theme-text font-bold">http://127.0.0.1:42128/v1</span>
          </div>
          <button
            onClick={() => {
              navigator.clipboard.writeText('http://127.0.0.1:42128/v1');
              props.showToast('Base URL disalin ke clipboard!');
            }}
            class="px-2.5 py-1 rounded-lg bg-theme-card border border-theme-border text-theme-text/70 hover:text-theme-cyan hover:border-theme-cyan text-[10px] font-sans font-semibold shrink-0 transition-all"
          >
            Salin URL
          </button>
        </div>

        <div class="pt-1 text-[11px] text-theme-text/50 flex flex-wrap items-center gap-2">
          <span>Endpoint:</span>
          <code class="px-1.5 py-0.5 rounded bg-theme-bg border border-theme-border/60 text-theme-text/70">/v1/chat/completions</code>
          <code class="px-1.5 py-0.5 rounded bg-theme-bg border border-theme-border/60 text-theme-text/70">/v1/models</code>
          <span class="text-theme-border">•</span>
          <span>API Key:</span>
          <span class="text-theme-text/70 font-mono">Bebas (isi apa saja / sembarang)</span>
        </div>
      </div>

      {/* Info Card */}
      <div class="p-5 rounded-2xl bg-theme-card/60 border border-theme-border/60 text-xs text-theme-text/70 space-y-2">
        <div class="font-bold text-theme-text flex items-center gap-2">
          <HelpCircle size={15} class="text-theme-cyan" />
          Model yang Didukung Antigravity
        </div>
        <p class="leading-relaxed text-theme-text/60">
          Kamu dapat mengatur model LLM di tab <b>Pengaturan</b> atau di client API ke model berikut:
        </p>
        <div class="flex flex-wrap gap-2 pt-1 font-mono text-[11px]">
          <span class="px-2 py-1 rounded-lg bg-theme-bg border border-theme-border text-theme-cyan">ag/gemini-3.8-flash-high</span>
          <span class="px-2 py-1 rounded-lg bg-theme-bg border border-theme-border text-theme-cyan">ag/gemini-3.7-flash-high</span>
          <span class="px-2 py-1 rounded-lg bg-theme-bg border border-theme-border text-theme-cyan">ag/gemini-3.5-flash-high</span>
          <span class="px-2 py-1 rounded-lg bg-theme-bg border border-theme-border text-theme-text/70">ag/claude-sonnet-4-6</span>
          <span class="px-2 py-1 rounded-lg bg-theme-bg border border-theme-border text-theme-text/70">ag/claude-opus-4-6-thinking</span>
        </div>
      </div>

      {/* Modal Tambah Akun Manual */}
      <Show when={showAddModal()}>
        <div class="fixed inset-0 z-[100] bg-black/70 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in">
          <div class="max-w-md w-full p-6 rounded-3xl bg-theme-surface border border-theme-border shadow-2xl space-y-5 animate-in fade-in zoom-in-95 duration-200">
            <div class="flex items-center justify-between pb-3 border-b border-theme-border">
              <div class="flex items-center gap-2.5">
                <div class="p-2 rounded-xl bg-amber-500/10 text-amber-400 border border-amber-500/20">
                  <Rocket size={18} />
                </div>
                <div>
                  <h3 class="text-base font-bold text-theme-text">Input Token Antigravity Manual</h3>
                  <p class="text-[11px] text-theme-text/50">Google OAuth Refresh Token</p>
                </div>
              </div>
              <button
                onClick={() => setShowAddModal(false)}
                class="p-1.5 rounded-lg text-theme-text/50 hover:text-theme-text hover:bg-theme-card transition-colors"
              >
                <X size={18} />
              </button>
            </div>

            <form onSubmit={handleAddAccount} class="space-y-4 text-xs">
              <div class="space-y-1.5">
                <label class="font-bold text-theme-text flex items-center gap-1.5">
                  <KeyRound size={13} class="text-theme-cyan" />
                  Refresh Token Google OAuth <span class="text-theme-rose">*</span>
                </label>
                <textarea
                  value={refreshToken()}
                  onInput={(e) => setRefreshToken(e.currentTarget.value)}
                  placeholder="Contoh: 1//04..."
                  rows={3}
                  required
                  class="w-full px-3.5 py-2.5 rounded-xl bg-theme-bg border border-theme-border focus:border-theme-cyan outline-none text-theme-text font-mono text-xs transition-colors"
                />
              </div>

              <div class="space-y-1.5">
                <label class="font-bold text-theme-text flex items-center gap-1.5">
                  <Mail size={13} class="text-theme-cyan" />
                  Email Akun (Opsional)
                </label>
                <input
                  type="email"
                  value={email()}
                  onInput={(e) => setEmail(e.currentTarget.value)}
                  placeholder="user@gmail.com"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-theme-bg border border-theme-border focus:border-theme-cyan outline-none text-theme-text text-xs transition-colors"
                />
              </div>

              <div class="space-y-1.5">
                <label class="font-bold text-theme-text flex items-center gap-1.5">
                  <FolderGit2 size={13} class="text-theme-cyan" />
                  Project ID Google Cloud (Opsional)
                </label>
                <input
                  type="text"
                  value={projectId()}
                  onInput={(e) => setProjectId(e.currentTarget.value)}
                  placeholder="Biarkan kosong untuk auto-discovery"
                  class="w-full px-3.5 py-2.5 rounded-xl bg-theme-bg border border-theme-border focus:border-theme-cyan outline-none text-theme-text font-mono text-xs transition-colors"
                />
              </div>

              <div class="flex items-center justify-end gap-2.5 pt-2">
                <button
                  type="button"
                  onClick={() => setShowAddModal(false)}
                  class="px-4 py-2.5 rounded-xl bg-theme-card hover:bg-theme-border text-theme-text/80 font-bold transition-all"
                >
                  Batal
                </button>
                <button
                  type="submit"
                  disabled={isSubmitting()}
                  class="px-5 py-2.5 rounded-xl bg-theme-cyan text-theme-card font-bold shadow-md anime-shadow-green hover:opacity-95 active:scale-95 transition-all flex items-center gap-2"
                >
                  <Show when={isSubmitting()} fallback={<Plus size={16} />}>
                    <RefreshCw size={16} class="animate-spin" />
                  </Show>
                  <span>{isSubmitting() ? 'Menyimpan...' : 'Simpan Token'}</span>
                </button>
              </div>
            </form>
          </div>
        </div>
      </Show>
    </div>
  );
};
