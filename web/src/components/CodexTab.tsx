import { Component, createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { 
  Terminal, 
  RefreshCw, 
  ShieldCheck, 
  LogIn
} from 'lucide-solid';
import { api, CodexAccount, CodexQuotaResponse } from '../api';
import { CodexStats } from './codex/CodexStats';
import { CodexAccountCard } from './codex/CodexAccountCard';
import { showConfirm } from './ConfirmModal';

interface CodexTabProps {
  showToast: (msg: string) => void;
}

export const CodexTab: Component<CodexTabProps> = (props) => {
  const [accounts, setAccounts] = createSignal<CodexAccount[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [actionLoading, setActionLoading] = createSignal<string | null>(null);
  const [isOAuthLoading, setIsOAuthLoading] = createSignal(false);

  // Quota State
  const [accountQuotas, setAccountQuotas] = createSignal<Record<string, CodexQuotaResponse>>({});
  const [loadingQuota, setLoadingQuota] = createSignal<Record<string, boolean>>({});
  const [expandedQuota, setExpandedQuota] = createSignal<Record<string, boolean>>({});

  // Form State
  const [copiedModelId, setCopiedModelId] = createSignal<string | null>(null);

  const copyToClipboard = (text: string, label = 'Text') => {
    navigator.clipboard.writeText(text);
    setCopiedModelId(text);
    props.showToast(`${label} copied to clipboard!`);
    setTimeout(() => {
      if (copiedModelId() === text) setCopiedModelId(null);
    }, 2000);
  };

  const handleFetchQuota = async (id: string) => {
    setLoadingQuota(prev => ({ ...prev, [id]: true }));
    try {
      const res = await api.getCodexAccountQuota(id);
      setAccountQuotas(prev => ({ ...prev, [id]: res }));
      if (res.status === 'error' && res.error) {
        props.showToast(`Quota: ${res.error}`);
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Failed to load quota');
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
      const list = await api.getCodexAccounts();
      setAccounts(Array.isArray(list) ? list : []);
    } catch {
      setAccounts([]);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadAccounts();

    const triggerRefresh = (email?: string) => {
      props.showToast(`OpenAI Codex OAuth login successful! ${email ? `(${email}) ` : ''}Account added to pool.`);
      loadAccounts(true);
    };

    // Realtime push notification from server via WebSocket/SSE event stream
    const unsubscribeActions = api.subscribeActions((act) => {
      if (act.action === 'codex:oauth_success') {
        triggerRefresh(act.payload);
      }
    });

    const handleMessage = (e: MessageEvent) => {
      if (e.data?.type === 'CODEX_OAUTH_SUCCESS') {
        triggerRefresh();
      }
    };

    window.addEventListener('message', handleMessage);

    onCleanup(() => {
      unsubscribeActions();
      window.removeEventListener('message', handleMessage);
    });
  });

  const handleOAuthLogin = async () => {
    setIsOAuthLoading(true);
    try {
      const authUrl = await api.getCodexAuthUrl();
      if (!authUrl) {
        props.showToast('Failed to generate OpenAI OAuth authorization URL');
        return;
      }

      const width = 600;
      const height = 700;
      const left = window.screenX + (window.outerWidth - width) / 2;
      const top = window.screenY + (window.outerHeight - height) / 2;

      const popup = window.open(
        authUrl,
        'CodexOAuth',
        `width=${width},height=${height},left=${left},top=${top},status=no,menubar=no,toolbar=no`
      );

      if (!popup || popup.closed || typeof popup.closed === 'undefined') {
        window.location.href = authUrl;
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Failed to initiate OAuth login');
    } finally {
      setIsOAuthLoading(false);
    }
  };

  const handleAddAccountSubmit = async (refreshToken: string, email?: string, accountId?: string): Promise<boolean> => {
    try {
      const res = await api.addCodexAccount(refreshToken, email, accountId);
      if (res.status === 'success') {
        props.showToast('OpenAI Codex account added successfully!');
        await loadAccounts();
        return true;
      } else {
        props.showToast(res.error || 'Failed to add account');
        return false;
      }
    } catch (err: any) {
      props.showToast(err?.message || 'System error occurred');
      return false;
    }
  };

  const handleDelete = async (id: string) => {
    const ok = await showConfirm({
      title: 'Hapus Akun Codex',
      message: 'Apakah Anda yakin ingin menghapus akun ini dari pool OpenAI Codex? Token yang tersimpan akan dihapus.',
      confirmText: 'Hapus Akun',
      cancelText: 'Batal',
      isDanger: true,
    });
    if (!ok) return;
    setActionLoading(id);
    try {
      const res = await api.deleteCodexAccount(id);
      if (res.status === 'success') {
        props.showToast('Account deleted successfully');
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Failed to delete account');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'An error occurred');
    } finally {
      setActionLoading(null);
    }
  };

  const handleRefresh = async (id: string) => {
    setActionLoading(id);
    try {
      const res = await api.refreshCodexAccount(id);
      if (res.status === 'success') {
        props.showToast('Token refreshed successfully!');
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Token refresh failed');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'Failed to connect to server');
    } finally {
      setActionLoading(null);
    }
  };

  const handleToggle = async (id: string) => {
    setActionLoading(id);
    try {
      const res = await api.toggleCodexAccount(id);
      if (res.status === 'success') {
        props.showToast(`Account status updated (${res.is_active ? 'Active' : 'Disabled'})`);
        await loadAccounts();
      } else {
        props.showToast(res.error || 'Failed to update status');
      }
    } catch (err: any) {
      props.showToast(err?.message || 'An error occurred');
    } finally {
      setActionLoading(null);
    }
  };

  const activeCount = () => accounts().filter(a => a.is_active && !a.is_rate_limited).length;
  const rateLimitedCount = () => accounts().filter(a => a.is_rate_limited).length;

  return (
    <div class="space-y-6 animate-fade-in text-theme-text select-text">
      {/* Header Banner */}
      <div class="bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-5 sm:p-6 border border-theme-border flex flex-col md:flex-row items-start md:items-center justify-between gap-4 shadow-sm">
        <div class="flex items-start gap-4">
          <div class="w-14 h-14 bg-theme-cyan/15 border border-theme-cyan/40 flex items-center justify-center text-theme-cyan shrink-0 shadow-sm mt-0.5">
            <Terminal size={28} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="font-heading tracking-tight text-theme-text">
                OpenAI Codex Pool
              </h2>
              <span class="text-xs font-mono px-2 py-0.5 bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 font-bold uppercase">
                Multi-OAuth
              </span>
            </div>
            <p class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              Connect OpenAI Codex OAuth accounts with 1-click, auto failover & round-robin load balancing.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2.5 w-full md:w-auto shrink-0 flex-nowrap">
          <button
            onClick={() => loadAccounts()}
            class="p-2.5 bg-theme-bg border border-theme-border text-theme-text hover:text-theme-cyan hover:border-theme-cyan/50 transition-all flex items-center justify-center shrink-0"
            title="Reload Accounts"
          >
            <RefreshCw size={18} class={loading() ? 'animate-spin text-theme-cyan' : ''} />
          </button>

          {/* 1-Click OpenAI OAuth Button */}
          <button
            onClick={handleOAuthLogin}
            disabled={isOAuthLoading()}
            class="whitespace-nowrap shrink-0 flex items-center justify-center gap-2 px-4 py-2.5 bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold text-xs shadow-md transition-all active:scale-95"
          >
            <Show when={isOAuthLoading()} fallback={<LogIn size={16} />}>
              <RefreshCw size={16} class="animate-spin" />
            </Show>
            <span>OpenAI Login (OAuth)</span>
          </button>
        </div>
      </div>

      {/* Stats Cards */}
      <CodexStats
        totalAccounts={accounts().length}
        activeCount={activeCount()}
        rateLimitedCount={rateLimitedCount()}
      />

      {/* Account List Section */}
      <div class="space-y-3">
        <div class="flex items-center justify-between px-1">
          <h3 class="text-theme-text flex items-center gap-2">
            <ShieldCheck size={16} class="text-theme-cyan" />
            OpenAI Codex Accounts Pool
          </h3>
          <span class="text-xs text-theme-text/40 font-mono">
            Automatic Round-Robin Rotation
          </span>
        </div>

        <Show
          when={!loading()}
          fallback={
            <div class="p-12 text-center text-xs text-theme-text/50 bg-theme-card border border-theme-border animate-pulse">
              Loading OpenAI Codex accounts...
            </div>
          }
        >
          <Show
            when={accounts().length > 0}
            fallback={
              <div class="p-12 text-center bg-theme-card border border-theme-border/60 space-y-4">
                <div class="w-14 h-14 mx-auto bg-theme-cyan/15 border border-theme-cyan/30 flex items-center justify-center text-theme-cyan">
                  <Terminal size={28} />
                </div>
                <div class="space-y-1">
                  <div class="text-sm font-bold text-theme-text">No OpenAI Codex Accounts Connected</div>
                  <p class="text-xs text-theme-text/50 max-w-sm mx-auto">
                    Click <b>OpenAI Login (OAuth)</b> to link an OpenAI Codex account in 1 click.
                  </p>
                </div>
                <button
                  onClick={handleOAuthLogin}
                  disabled={isOAuthLoading()}
                  class="px-5 py-2.5 bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold text-xs shadow-md transition-all inline-flex items-center gap-2"
                >
                  <LogIn size={16} />
                  <span>Login with OpenAI Now</span>
                </button>
              </div>
            }
          >
            <div class="grid grid-cols-1 gap-3">
              <For each={accounts()}>
                {(acc) => (
                  <CodexAccountCard
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
            <span class="p-1 bg-theme-cyan/10 text-theme-cyan font-mono text-xs font-extrabold uppercase tracking-wider border border-theme-cyan/30">Codex API</span>
            <span>Local Endpoint (OpenAI-Compatible with cx/ prefix)</span>
          </div>
          <span class="text-xs font-mono px-2 py-0.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
            Port 42128 Ready
          </span>
        </div>

        <p class="leading-relaxed text-theme-text/60">
          Use this endpoint in any AI tool (Cursor, Zed Editor, Cline, Continue, Aider, Claude Code, Python OpenAI SDK, etc.):
        </p>

        <div class="p-3 bg-theme-bg/80 border border-theme-border font-mono text-xs flex items-center justify-between gap-2 overflow-x-auto">
          <div class="text-theme-cyan select-all">
            Base URL: <span class="text-theme-text font-bold">http://127.0.0.1:42128/v1</span>
          </div>
          <button
            onClick={() => copyToClipboard('http://127.0.0.1:42128/v1', 'Base URL')}
            class="px-2.5 py-1 bg-theme-card border border-theme-border text-theme-text/70 hover:text-theme-cyan hover:border-theme-cyan text-xs font-sans font-semibold shrink-0 transition-all"
          >
            Copy URL
          </button>
        </div>

        <div class="pt-1 text-xs text-theme-text/50 flex flex-wrap items-center gap-2">
          <span>Endpoints:</span>
          <code class="px-1.5 py-0.5 bg-theme-bg border border-theme-border/60 text-theme-text/70">/v1/chat/completions</code>
          <code class="px-1.5 py-0.5 bg-theme-bg border border-theme-border/60 text-theme-text/70">/v1/models</code>
          <span class="text-theme-border">•</span>
          <span>API Key:</span>
          <span class="text-theme-text/70 font-mono">Any string (not required)</span>
        </div>
      </div>
    </div>
  );
};
