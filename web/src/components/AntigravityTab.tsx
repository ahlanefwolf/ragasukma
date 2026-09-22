import { Component, createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { 
  Rocket, 
  Plus, 
  RefreshCw, 
  ShieldCheck, 
  LogIn
} from 'lucide-solid';
import { api, AntigravityAccount, AntigravityQuotaResponse } from '../api';
import { AntigravityStats } from './antigravity/AntigravityStats';
import { AntigravityAccountCard } from './antigravity/AntigravityAccountCard';
import { AntigravityAddModal } from './antigravity/AntigravityAddModal';
import { AntigravityModelCatalog } from './antigravity/AntigravityModelCatalog';

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
  const [copiedModelId, setCopiedModelId] = createSignal<string | null>(null);

  const copyToClipboard = (text: string, label = 'Teks') => {
    navigator.clipboard.writeText(text);
    setCopiedModelId(text);
    props.showToast(`${label} disalin ke clipboard!`);
    setTimeout(() => {
      if (copiedModelId() === text) setCopiedModelId(null);
    }, 2000);
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
        window.location.href = authUrl;
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Gagal memulai login OAuth');
    } finally {
      setIsOAuthLoading(false);
    }
  };

  const handleAddAccountSubmit = async (refreshToken: string, email?: string, projectId?: string): Promise<boolean> => {
    try {
      const res = await api.addAntigravityAccount(refreshToken, email, projectId);
      if (res.status === 'success') {
        props.showToast('Akun Antigravity berhasil ditambahkan!');
        await loadAccounts();
        return true;
      } else {
        props.showToast(res.error || 'Gagal menambahkan akun');
        return false;
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Terjadi kesalahan sistem');
      return false;
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
    <div class="space-y-6 animate-fade-in text-theme-text select-text">
      {/* Header Banner */}
      <div class="p-6 bg-theme-card border border-theme-border flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-sm">
        <div class="flex items-center gap-4">
          <div class="w-14 h-14 bg-theme-rose/15 border border-theme-rose/40 flex items-center justify-center text-theme-rose shrink-0 shadow-sm">
            <Rocket size={28} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xl font-carter tracking-wide text-theme-text">
                ANTIGRAVITY POOL
              </h2>
              <span class="text-xs font-mono px-2 py-0.5 bg-theme-rose/20 text-theme-rose border border-theme-rose/30 font-bold uppercase">
                Google Multi-OAuth
              </span>
            </div>
            <p class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              Koneksikan akun Google OAuth Antigravity dengan 1-klik, auto failover & round-robin load balancing.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2.5 w-full md:w-auto shrink-0 flex-nowrap">
          <button
            onClick={() => loadAccounts()}
            class="p-2.5 bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan hover:border-theme-cyan/50 transition-all flex items-center justify-center shrink-0"
            title="Muat Ulang"
          >
            <RefreshCw size={18} class={loading() ? 'animate-spin text-theme-cyan' : ''} />
          </button>

          {/* 1-Click Google OAuth Button */}
          <button
            onClick={handleOAuthLogin}
            disabled={isOAuthLoading()}
            class="whitespace-nowrap shrink-0 flex items-center justify-center gap-2 px-4 py-2.5 bg-theme-cyan text-theme-card font-bold text-xs shadow-md anime-shadow-green hover:opacity-95 active:scale-95 transition-all"
          >
            <Show when={isOAuthLoading()} fallback={<LogIn size={16} />}>
              <RefreshCw size={16} class="animate-spin" />
            </Show>
            <span>Login Google (OAuth)</span>
          </button>

          <button
            onClick={() => setShowAddModal(true)}
            class="whitespace-nowrap shrink-0 p-2.5 sm:px-3 sm:py-2.5 bg-theme-bg border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/50 text-xs font-semibold transition-all flex items-center gap-1.5"
            title="Input Token Manual"
          >
            <Plus size={16} />
            <span class="inline">Manual</span>
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <AntigravityStats
        totalAccounts={accounts().length}
        activeCount={activeCount()}
        rateLimitedCount={rateLimitedCount()}
      />

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
            <div class="p-12 text-center text-xs text-theme-text/50 bg-theme-card border border-theme-border animate-pulse">
              Memuat daftar akun Antigravity...
            </div>
          }
        >
          <Show
            when={accounts().length > 0}
            fallback={
              <div class="p-12 text-center bg-theme-card border border-theme-border/60 space-y-4">
                <div class="w-14 h-14 mx-auto bg-theme-rose/15 border border-theme-rose/30 flex items-center justify-center text-theme-rose">
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
                  class="px-5 py-2.5 bg-theme-cyan text-theme-card font-bold text-xs shadow-md anime-shadow-green hover:opacity-95 active:scale-95 transition-all inline-flex items-center gap-2"
                >
                  <LogIn size={16} />
                  <span>Login dengan Google Sekarang</span>
                </button>
              </div>
            }
          >
            <div class="grid grid-cols-1 gap-3">
              <For each={accounts()}>
                {(acc) => (
                  <AntigravityAccountCard
                    account={acc}
                    actionLoading={actionLoading()}
                    quotaData={accountQuotas()[acc.id]}
                    loadingQuota={!!loadingQuota()[acc.id]}
                    isExpanded={!!expandedQuota()[acc.id]}
                    copiedModelId={copiedModelId()}
                    onToggleQuota={toggleQuota}
                    onFetchQuota={handleFetchQuota}
                    onToggleAccount={handleToggle}
                    onRefreshAccount={handleRefresh}
                    onDeleteAccount={handleDelete}
                    onCopyModel={copyToClipboard}
                  />
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* OpenAI-Compatible API Endpoint Info Card */}
      <div class="p-5 bg-theme-card/80 border border-theme-cyan/30 text-xs text-theme-text/80 space-y-3 shadow-lg">
        <div class="flex items-center justify-between">
          <div class="font-bold text-theme-text flex items-center gap-2">
            <span class="p-1 bg-theme-cyan/10 text-theme-cyan font-mono text-xs font-extrabold uppercase tracking-wider border border-theme-cyan/30">OpenAI API</span>
            <span>Local Endpoint (OpenAI-Compatible)</span>
          </div>
          <span class="text-xs font-mono px-2 py-0.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
            Port 42128 Ready
          </span>
        </div>

        <p class="leading-relaxed text-theme-text/60">
          Gunakan endpoint ini di aplikasi AI apapun (Zed Editor, Cline, Continue, Cursor, Aider, Claude Code, Python OpenAI SDK, dll) untuk menggunakan model Antigravity:
        </p>

        <div class="p-3 bg-theme-bg/80 border border-theme-border font-mono text-xs flex items-center justify-between gap-2 overflow-x-auto">
          <div class="text-theme-cyan select-all">
            Base URL: <span class="text-theme-text font-bold">http://127.0.0.1:42128/v1</span>
          </div>
          <button
            onClick={() => copyToClipboard('http://127.0.0.1:42128/v1', 'Base URL')}
            class="px-2.5 py-1 bg-theme-card border border-theme-border text-theme-text/70 hover:text-theme-cyan hover:border-theme-cyan text-xs font-sans font-semibold shrink-0 transition-all"
          >
            Salin URL
          </button>
        </div>

        <div class="pt-1 text-xs text-theme-text/50 flex flex-wrap items-center gap-2">
          <span>Endpoint:</span>
          <code class="px-1.5 py-0.5 bg-theme-bg border border-theme-border/60 text-theme-text/70">/v1/chat/completions</code>
          <code class="px-1.5 py-0.5 bg-theme-bg border border-theme-border/60 text-theme-text/70">/v1/models</code>
          <span class="text-theme-border">•</span>
          <span>API Key:</span>
          <span class="text-theme-text/70 font-mono">Bebas (isi apa saja / sembarang)</span>
        </div>
      </div>

      {/* Model Catalog & Copy Helper */}
      <AntigravityModelCatalog
        copiedModelId={copiedModelId()}
        onCopyModel={copyToClipboard}
      />

      {/* Modal Tambah Akun Manual */}
      <AntigravityAddModal
        isOpen={showAddModal()}
        onClose={() => setShowAddModal(false)}
        onSubmit={handleAddAccountSubmit}
      />
    </div>
  );
};
