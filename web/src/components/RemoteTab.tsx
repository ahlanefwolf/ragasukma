import { Component, createSignal, onMount, onCleanup, createEffect, Show } from 'solid-js';
import QRCode from 'qrcode';
import { 
  Radio, 
  Square, 
  Copy, 
  Check, 
  Smartphone, 
  ShieldCheck, 
  ExternalLink,
  RefreshCw,
  Sparkles
} from 'lucide-solid';
import { api, CloudflaredInfo } from '../api';
import { CloudflaredStatusCard } from './CloudflaredStatusCard';

interface RemoteTabProps {
  showToast: (msg: string) => void;
}

export const RemoteTab: Component<RemoteTabProps> = (props) => {
  const [tunnelStatus, setTunnelStatus] = createSignal<'stopped' | 'starting' | 'running'>('stopped');
  const [tunnelUrl, setTunnelUrl] = createSignal<string | null>(null);
  const [authKey, setAuthKey] = createSignal<string>('');
  const [loading, setLoading] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const [qrDataUrl, setQrDataUrl] = createSignal<string>('');

  // Status binary cloudflared (mirip yt-dlp in MusicTab)
  const [cfInfo, setCfInfo] = createSignal<CloudflaredInfo | null>(null);
  const [checkingCf, setCheckingCf] = createSignal(true);
  const [installingCf, setInstallingCf] = createSignal(false);
  const [installCfMsg, setInstallCfMsg] = createSignal<string | null>(null);
  const [copiedCmd, setCopiedCmd] = createSignal<string | null>(null);

  let pollTimer: any = null;

  const checkCloudflared = async (silent = false) => {
    if (!silent) setCheckingCf(true);
    try {
      const info = await api.checkCloudflared();
      setCfInfo(info);
    } finally {
      setCheckingCf(false);
    }
  };

  const fetchStatus = async () => {
    const res = await api.getTunnelStatus();
    setTunnelStatus(res.status);
    if (res.url) setTunnelUrl(res.url);
    if (res.auth_key) setAuthKey(res.auth_key);
    if (res.cloudflared) {
      setCfInfo(res.cloudflared);
      setCheckingCf(false);
    }
    return res;
  };

  onMount(() => {
    fetchStatus();
    checkCloudflared(false);
  });

  onCleanup(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  const handleStartTunnel = async () => {
    if (cfInfo() && !cfInfo()!.available) {
      props.showToast('cloudflared is not installed on this system. Please install it first.');
      return;
    }
    setLoading(true);
    props.showToast('Starting cloudflared tunnel...');
    const res = await api.startTunnel();
    setTunnelStatus(res.status);
    if (res.auth_key) setAuthKey(res.auth_key);

    if (res.error) {
      setLoading(false);
      props.showToast(`Error: ${res.error}`);
      return;
    }

    // Poll status every 1s until URL appears
    let attempts = 0;
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(async () => {
      attempts++;
      const st = await fetchStatus();
      if (st.status === 'running' && st.url) {
        clearInterval(pollTimer);
        pollTimer = null;
        setLoading(false);
        props.showToast('Cloudflared tunnel active successfully!');
      } else if (attempts > 40 || st.status === 'stopped') {
        clearInterval(pollTimer);
        pollTimer = null;
        setLoading(false);
        if (!st.url) {
          props.showToast(st.status === 'stopped' ? 'Cloudflared stopped or connection failed.' : 'Tunnel running in background, check console logs.');
        }
      }
    }, 1000);
  };

  const handleStopTunnel = async () => {
    setLoading(true);
    await api.stopTunnel();
    setTunnelStatus('stopped');
    setTunnelUrl(null);
    setLoading(false);
    props.showToast('Cloudflared tunnel stopped.');
  };

  const getFullRemoteUrl = () => {
    if (!tunnelUrl()) return '';
    return `${tunnelUrl()}/?key=${encodeURIComponent(authKey())}#mobile`;
  };

  createEffect(() => {
    const url = getFullRemoteUrl();
    if (url) {
      QRCode.toDataURL(url, {
        width: 220,
        margin: 1,
        color: {
          dark: '#000000',
          light: '#ffffff',
        },
      })
        .then((dataUri) => {
          setQrDataUrl(dataUri);
        })
        .catch((err) => {
          console.error('[QRCode] Error generating QR:', err);
        });
    } else {
      setQrDataUrl('');
    }
  });

  const copyUrl = () => {
    const full = getFullRemoteUrl();
    if (!full) return;
    navigator.clipboard.writeText(full).then(() => {
      setCopied(true);
      props.showToast('Remote link copied to clipboard!');
      setTimeout(() => setCopied(false), 2000);
    });
  };

  const handleInstallCf = async () => {
    setInstallingCf(true);
    setInstallCfMsg(null);
    const res = await api.installCloudflared();
    setInstallingCf(false);
    if (res.success) {
      setInstallCfMsg(res.message);
      if (res.info) setCfInfo(res.info);
      props.showToast('cloudflared installed successfully!');
    } else {
      setInstallCfMsg('Failed: ' + res.message);
      props.showToast('Failed to install cloudflared');
    }
  };

  const handleCopyCmd = (cmd: string) => {
    navigator.clipboard.writeText(cmd).then(() => {
      setCopiedCmd(cmd);
      setTimeout(() => setCopiedCmd(null), 2000);
    });
  };

  return (
    <div class="space-y-6 animate-in fade-in duration-200">
      {/* Header Info */}
      <div class="p-6 bg-theme-surface border border-theme-border flex flex-col md:flex-row md:items-center justify-between gap-4 shadow-sm">
        <div class="space-y-1">
          <div class="flex items-center gap-2.5">
            <Radio class="text-theme-cyan" size={24} />
            <h2 class="font-heading tracking-tight text-theme-text">
              Mobile Remote & Cloudflared Tunnel
            </h2>
          </div>
          <p class="text-xs text-theme-text/70 leading-relaxed max-w-2xl">
            Control Ragasukma directly from your smartphone without needing to be on the same local Wi-Fi (port forwarding bypass via Cloudflare Tunnel).
            Mobile layout optimized specifically for smartphones: Push-to-Talk (Web Speech API) & Live Console Logs.
          </p>
        </div>

        <div class="flex items-center gap-3 shrink-0">
          <button
            onClick={() => {
              fetchStatus();
              checkCloudflared(false);
            }}
            class="p-2.5 bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan/50 transition-colors"
            title="Refresh Status"
          >
            <RefreshCw size={18} />
          </button>

          <Show
            when={tunnelStatus() === 'running'}
            fallback={
              <button
                onClick={handleStartTunnel}
                disabled={loading() || (!checkingCf() && !cfInfo()?.available)}
                class="px-5 py-2.5 bg-theme-cyan text-theme-card font-bold text-sm border border-theme-cyan hover:opacity-90 transition-all flex items-center gap-2 shadow-md anime-shadow-green disabled:opacity-50"
              >
                <Radio size={16} />
                <span>{loading() ? 'Starting...' : 'Start Tunnel'}</span>
              </button>
            }
          >
            <button
              onClick={handleStopTunnel}
              disabled={loading()}
              class="px-5 py-2.5 bg-theme-rose text-white font-bold text-sm border border-theme-rose hover:opacity-90 transition-all flex items-center gap-2 shadow-md disabled:opacity-50"
            >
              <Square size={16} />
              <span>{loading() ? 'Stopping...' : 'Stop Tunnel'}</span>
            </button>
          </Show>
        </div>
      </div>

      {/* Cloudflared Binary Status Card */}
      <CloudflaredStatusCard
        checking={checkingCf}
        info={cfInfo}
        onRefresh={() => checkCloudflared(false)}
        onInstall={handleInstallCf}
        installing={installingCf}
        installMsg={installCfMsg}
        copiedCmd={copiedCmd}
        onCopyCmd={handleCopyCmd}
      />

      {/* Cloudflared Status Banner */}
      <div class="p-5 bg-theme-card border border-theme-border space-y-4">
        <div class="flex items-center justify-between border-b border-theme-border/60 pb-3">
          <div class="flex items-center gap-2.5">
            <span
              class={`w-3 h-3 rounded-full ${
                tunnelStatus() === 'running'
                  ? 'bg-theme-cyan shadow-[0_0_10px_var(--cyan)] animate-pulse'
                  : tunnelStatus() === 'starting'
                  ? 'bg-gi-status-warning animate-bounce'
                  : 'bg-theme-text/30'
              }`}
            />
            <span class="text-sm font-bold text-theme-text uppercase tracking-wider">
              Tunnel Status: {tunnelStatus()}
            </span>
          </div>

          <div class="flex items-center gap-2 text-xs font-mono text-theme-text/60">
            <ShieldCheck size={14} class="text-theme-cyan" />
            <span>Auth Key: {authKey() ? `${authKey().slice(0, 8)}...` : 'Inactive'}</span>
          </div>
        </div>

        <Show when={tunnelStatus() === 'running' && tunnelUrl()}>
          <div class="space-y-4 pt-1">
            <div class="text-xs text-theme-text/80 font-semibold flex items-center gap-1.5">
              <Smartphone size={16} class="text-theme-cyan" />
              <span>Scan this QR code with your smartphone camera:</span>
            </div>

            {/* QR Code Container */}
            <div class="flex flex-col sm:flex-row items-center gap-6 p-4 bg-theme-surface border border-theme-border">
              <Show when={qrDataUrl()}>
                <div class="p-2.5 bg-white rounded-lg shadow-md border-2 border-theme-cyan shrink-0 flex flex-col items-center">
                  <img
                    src={qrDataUrl()}
                    alt="Scan QR Mobile Remote"
                    class="w-48 h-48 block select-none"
                  />
                  <span class="text-2xs text-gi-surface font-mono font-bold mt-1 uppercase tracking-wider">
                    Scan via Phone Camera
                  </span>
                </div>
              </Show>

              <div class="space-y-4 flex-1 w-full">
                <div class="text-xs text-theme-text/70 leading-relaxed">
                  Point your mobile camera at the QR code to open the Ragasukma mobile remote view.
                  No need to manually input URLs or authentication keys.
                </div>

                <div class="flex items-center gap-2">
                  <input
                    type="text"
                    readonly
                    value={getFullRemoteUrl()}
                    class="flex-1 px-3.5 py-2.5 bg-theme-card border border-theme-border text-xs font-mono text-theme-cyan select-all focus:outline-none focus:border-theme-cyan"
                  />
                  <button
                    onClick={copyUrl}
                    class="px-4 py-2.5 bg-theme-card border border-theme-border hover:border-theme-cyan text-theme-text text-xs font-bold flex items-center gap-2 transition-colors shrink-0"
                  >
                    <Show when={copied()} fallback={<Copy size={16} />}>
                      <Check size={16} class="text-theme-cyan" />
                    </Show>
                    <span>{copied() ? 'Copied' : 'Copy Link'}</span>
                  </button>
                  <a
                    href={getFullRemoteUrl()}
                    target="_blank"
                    rel="noreferrer"
                    class="p-2.5 bg-theme-card border border-theme-border hover:border-theme-cyan text-theme-text transition-colors shrink-0"
                    title="Open Mobile Tab in Browser"
                  >
                    <ExternalLink size={16} />
                  </a>
                </div>

                <div class="p-3 bg-theme-card/60 border border-theme-border/50 text-xs text-theme-text/80 space-y-1.5">
                  <div class="font-bold flex items-center gap-1.5 text-theme-cyan">
                    <Sparkles size={14} />
                    <span>Mobile Client Features:</span>
                  </div>
                  <ul class="list-disc list-inside space-y-1 text-theme-text/70">
                    <li>Push-to-Talk via smartphone native Web Speech API (low latency).</li>
                    <li>Live Console Logs to monitor Ragasukma activity directly from mobile.</li>
                    <li>Automatic single-tab bypass allowing desktop and mobile to run simultaneously.</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </Show>

        <Show when={tunnelStatus() === 'stopped'}>
          <div class="p-6 bg-theme-surface/50 border border-dashed border-theme-border text-center space-y-3">
            <Radio size={36} class="mx-auto text-theme-text/60" />
            <div class="space-y-1">
              <p class="text-sm font-bold text-theme-text">Cloudflared Tunnel Inactive</p>
              <p class="text-xs text-theme-text/60 max-w-md mx-auto">
                Click the <strong>"Start Tunnel"</strong> button above to generate a secure public HTTPS link and QR code for your smartphone.
              </p>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};
