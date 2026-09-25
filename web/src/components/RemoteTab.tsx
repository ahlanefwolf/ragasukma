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
import { api } from '../api';

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

  let pollTimer: any = null;

  const fetchStatus = async () => {
    const res = await api.getTunnelStatus();
    setTunnelStatus(res.status);
    if (res.url) setTunnelUrl(res.url);
    if (res.auth_key) setAuthKey(res.auth_key);
    return res;
  };

  onMount(() => {
    fetchStatus();
  });

  onCleanup(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  const handleStartTunnel = async () => {
    setLoading(true);
    props.showToast('Memulai cloudflared tunnel...');
    const res = await api.startTunnel();
    setTunnelStatus(res.status);
    if (res.auth_key) setAuthKey(res.auth_key);

    if (res.error) {
      setLoading(false);
      props.showToast(`Error: ${res.error}`);
      return;
    }

    // Polling status setiap 1 detik sampai URL muncul
    let attempts = 0;
    if (pollTimer) clearInterval(pollTimer);
    pollTimer = setInterval(async () => {
      attempts++;
      const st = await fetchStatus();
      if (st.status === 'running' && st.url) {
        clearInterval(pollTimer);
        pollTimer = null;
        setLoading(false);
        props.showToast('Cloudflared tunnel berhasil aktif!');
      } else if (attempts > 40 || st.status === 'stopped') {
        clearInterval(pollTimer);
        pollTimer = null;
        setLoading(false);
        if (!st.url) {
          props.showToast(st.status === 'stopped' ? 'Cloudflared berhenti atau gagal terkoneksi.' : 'Tunnel berjalan di background, cek log console.');
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
    props.showToast('Cloudflared tunnel dihentikan.');
  };

  const getFullRemoteUrl = () => {
    if (!tunnelUrl()) return '';
    return `${tunnelUrl()}/?key=${encodeURIComponent(authKey())}#remote`;
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
      props.showToast('Link remote berhasil disalin!');
      setTimeout(() => setCopied(false), 2000);
    });
  };

  return (
    <div class="space-y-6 animate-in fade-in duration-200">
      {/* Header Info */}
      <div class="p-6 bg-theme-surface border border-theme-border flex flex-col md:flex-row md:items-center justify-between gap-4 shadow-sm">
        <div class="space-y-1">
          <div class="flex items-center gap-2.5">
            <Radio class="text-theme-cyan" size={24} />
            <h2 class="text-xl font-heading tracking-tight text-theme-text font-bold">
              Mobile Remote & Cloudflared Tunnel
            </h2>
          </div>
          <p class="text-xs text-theme-text/70 leading-relaxed max-w-2xl">
            Kendalikan Ragasukma dari smartphone Anda tanpa perlu satu jaringan WiFi (port forwarding bypass via Cloudflare Tunnel).
            Tampilan HP dioptimasi khusus untuk smartphone: tombol Push-to-Talk (Web Speech API) & Live Console Logs.
          </p>
        </div>

        <div class="flex items-center gap-3 shrink-0">
          <button
            onClick={fetchStatus}
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
                disabled={loading()}
                class="px-5 py-2.5 bg-theme-cyan text-theme-card font-bold text-sm border border-theme-cyan hover:opacity-90 transition-all flex items-center gap-2 shadow-md anime-shadow-green disabled:opacity-50"
              >
                <Radio size={16} />
                <span>{loading() ? 'Memulai...' : 'Nyalakan Tunnel'}</span>
              </button>
            }
          >
            <button
              onClick={handleStopTunnel}
              disabled={loading()}
              class="px-5 py-2.5 bg-theme-rose text-white font-bold text-sm border border-theme-rose hover:opacity-90 transition-all flex items-center gap-2 shadow-md disabled:opacity-50"
            >
              <Square size={16} />
              <span>{loading() ? 'Menghentikan...' : 'Matikan Tunnel'}</span>
            </button>
          </Show>
        </div>
      </div>

      {/* Cloudflared Status Banner */}
      <div class="p-5 bg-theme-card border border-theme-border space-y-4">
        <div class="flex items-center justify-between border-b border-theme-border/60 pb-3">
          <div class="flex items-center gap-2.5">
            <span
              class={`w-3 h-3 rounded-full ${
                tunnelStatus() === 'running'
                  ? 'bg-theme-cyan shadow-[0_0_10px_var(--cyan)] animate-pulse'
                  : tunnelStatus() === 'starting'
                  ? 'bg-yellow-400 animate-bounce'
                  : 'bg-theme-text/30'
              }`}
            />
            <span class="text-sm font-bold text-theme-text uppercase tracking-wider">
              Status Tunnel: {tunnelStatus()}
            </span>
          </div>

          <div class="flex items-center gap-2 text-xs font-mono text-theme-text/60">
            <ShieldCheck size={14} class="text-theme-cyan" />
            <span>Key Auth: {authKey() ? `${authKey().slice(0, 8)}...` : 'Belum aktif'}</span>
          </div>
        </div>

        <Show when={tunnelStatus() === 'running' && tunnelUrl()}>
          <div class="space-y-4 pt-1">
            <div class="text-xs text-theme-text/80 font-semibold flex items-center gap-1.5">
              <Smartphone size={16} class="text-theme-cyan" />
              <span>Scan QR code ini dengan kamera smartphone Anda:</span>
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
                  <span class="text-[10px] text-gray-700 font-mono font-bold mt-1 uppercase tracking-wider">
                    Scan via Kamera HP
                  </span>
                </div>
              </Show>

              <div class="space-y-4 flex-1 w-full">
                <div class="text-xs text-theme-text/70 leading-relaxed">
                  Arahkan kamera smartphone ke kode QR di samping untuk langsung membuka kontrol mobile Ragasukma. 
                  Tidak perlu memasukkan URL atau kode otentikasi secara manual.
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
                    <span>{copied() ? 'Tersalin' : 'Salin Link'}</span>
                  </button>
                  <a
                    href={getFullRemoteUrl()}
                    target="_blank"
                    rel="noreferrer"
                    class="p-2.5 bg-theme-card border border-theme-border hover:border-theme-cyan text-theme-text transition-colors shrink-0"
                    title="Buka Tab Mobile di Browser"
                  >
                    <ExternalLink size={16} />
                  </a>
                </div>

                <div class="p-3 bg-theme-card/60 border border-theme-border/50 text-xs text-theme-text/80 space-y-1.5">
                  <div class="font-bold flex items-center gap-1.5 text-theme-cyan">
                    <Sparkles size={14} />
                    <span>Fitur Mobile Client:</span>
                  </div>
                  <ul class="list-disc list-inside space-y-1 text-theme-text/70">
                    <li>Push-to-Talk via Web Speech API bawaan smartphone (hemat kuota & latensi 0s).</li>
                    <li>Live Console Logs untuk memantau aktivitas & feedback Ragasukma langsung dari HP.</li>
                    <li>Otomatis bypass single-tab lock sehingga tab desktop tetap berjalan normal.</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </Show>

        <Show when={tunnelStatus() === 'stopped'}>
          <div class="p-6 bg-theme-surface/50 border border-dashed border-theme-border text-center space-y-3">
            <Radio size={36} class="mx-auto text-theme-text/30" />
            <div class="space-y-1">
              <p class="text-sm font-bold text-theme-text">Cloudflared Tunnel Belum Aktif</p>
              <p class="text-xs text-theme-text/60 max-w-md mx-auto">
                Klik tombol <strong>"Nyalakan Tunnel"</strong> di atas untuk membuat tautan HTTPS publik yang aman dan QR Code untuk smartphone Anda.
              </p>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};
