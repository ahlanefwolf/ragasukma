import { Component, createSignal, onMount, onCleanup, Show } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { MenuTab } from './components/MenuTab';
import { CanvasTab } from './components/canvas/CanvasTab';
import { SkillsTab } from './components/SkillsTab';
import { MusicTab } from './components/MusicTab';
import { TunerTab } from './components/TunerTab';
import { LogsTab } from './components/LogsTab';
import { SettingsTab } from './components/SettingsTab';
import { AntigravityTab } from './components/AntigravityTab';
import { BottomPlayer } from './components/BottomPlayer';
import { initGlobalAudio, killGlobalAudio } from './playerStore';
import { Menu as MenuIcon } from 'lucide-solid';
import { api, killCurrentSession } from './api';

// Token unik per instance tab, dibuat saat module pertama kali dimuat
const MY_TOKEN = 'sess_' + Math.random().toString(36).substring(2, 11) + Date.now().toString(36);
const LS_KEY = 'ragasukma_active_tab';
const BC_NAME = 'ragasukma_session';

const TAB_HASH_ALIASES: Record<string, string> = {
  '': 'menu',
  '#': 'menu',
  '#menu': 'menu',
  '#canvas': 'canvas',
  '#2d': 'canvas',
  '#whiteboard': 'canvas',
  '#draw': 'canvas',
  '#skills': 'skills',
  '#music': 'music',
  '#tuner': 'tuner',
  '#logs': 'logs',
  '#settings': 'settings',
  '#provider-antigravity': 'provider-antigravity',
  '#antigravity': 'provider-antigravity',
};

const getInitialTabFromHash = (): string => {
  try {
    const hash = window.location.hash.toLowerCase();
    if (TAB_HASH_ALIASES[hash]) {
      return TAB_HASH_ALIASES[hash];
    }
    const clean = hash.replace(/^#/, '');
    if (clean) return clean;
  } catch {}
  return 'menu';
};

export const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal(getInitialTabFromHash());
  const [currentState, setCurrentState] = createSignal('idle');
  const [isDead, setIsDead] = createSignal(false);
  const [deadReason, setDeadReason] = createSignal<'duplicate' | 'shutdown'>('duplicate');
  const [toastMessage, setToastMessage] = createSignal('');
  const [toastVisible, setToastVisible] = createSignal(false);
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);

  const killThisTab = (reason: 'duplicate' | 'shutdown' = 'duplicate') => {
    if (isDead()) return;
    setDeadReason(reason);
    setIsDead(true);
    killCurrentSession();
    killGlobalAudio();
    if (document.pictureInPictureElement) {
      document.exitPictureInPicture().catch(() => {});
    }
  };

  onMount(() => {
    console.log('[Session] Tab mount, MY_TOKEN =', MY_TOKEN);

    // ── 1. BroadcastChannel: siap dengar SEBELUM kita claim ──
    let bc: BroadcastChannel | null = null;
    try {
      bc = new BroadcastChannel(BC_NAME);
      bc.onmessage = (e) => {
        console.log('[Session] BroadcastChannel msg:', e.data, 'MY_TOKEN =', MY_TOKEN);
        if (e.data?.type === 'CLAIM' && e.data.token !== MY_TOKEN) {
          console.log('[Session] BroadcastChannel -> killThisTab');
          killThisTab();
        }
      };
      console.log('[Session] BroadcastChannel listener ready');
    } catch (err) {
      console.warn('[Session] BroadcastChannel not supported:', err);
    }

    const handleStorage = (e: StorageEvent) => {
      if (e.key === LS_KEY && e.newValue && e.newValue !== MY_TOKEN) {
        console.log('[Session] storage event -> killThisTab');
        killThisTab('duplicate');
      }
    };
    window.addEventListener('storage', handleStorage);

    // ── 3. CLAIM 1X: kirim ke backend + tulis ke localStorage (tab lama langsung mati) ──
    console.log('[Session] Claiming ownership 1x...');
    try { localStorage.setItem(LS_KEY, MY_TOKEN); } catch {}
    try { bc?.postMessage({ type: 'CLAIM', token: MY_TOKEN }); } catch {}
    api.claimSession(MY_TOKEN, activeTab()).catch(() => {});

    // ── 4. SSE: untuk update state UI & deteksi shutdown / token klaim ──
    const unsubState = api.subscribeState(
      (st) => {
        if (st?.state === 'shutdown') {
          console.log('[Session] Backend signaled SHUTDOWN');
          killThisTab('shutdown');
          return;
        }
        if (st?.state) setCurrentState(st.state);
      }
    );

    const unsubActions = api.subscribeActions((act) => {
      if (act.action === 'session:claimed' && act.payload && act.payload !== MY_TOKEN) {
        console.log('[Session] Backend signaled another tab claimed ownership -> killThisTab');
        killThisTab('duplicate');
      }
    });

    // ── 5. Hash change listener (Back/Forward / Manual URL edit) ──
    const handleHashChange = () => {
      const tab = getInitialTabFromHash();
      if (tab !== activeTab()) {
        setActiveTab(tab);
      }
    };
    window.addEventListener('hashchange', handleHashChange);

    onCleanup(() => {
      try { bc?.close(); } catch {}
      window.removeEventListener('storage', handleStorage);
      window.removeEventListener('hashchange', handleHashChange);
      unsubState();
      unsubActions();
    });
  });

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setToastVisible(true);
    setTimeout(() => setToastVisible(false), 2500);
  };

  const handleTabChange = (tab: string) => {
    if (activeTab() === 'menu' && tab !== 'menu') {
      api.closeMenuSSE();
    }
    setActiveTab(tab);
    // Sinkronisasi URL Hash tanpa reload
    try {
      const targetHash = tab === 'menu' ? '' : `#${tab}`;
      if (window.location.hash !== targetHash) {
        window.history.replaceState(null, '', targetHash || window.location.pathname);
      }
    } catch {}
    api.notifyActiveTab(tab);
  };

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-theme-bg text-theme-text font-sans selection:bg-theme-rose selection:text-theme-card">
      <Show
        when={!isDead()}
        fallback={
          /* Modal Tab Mati / Aplikasi Ditutup — DOM ringan, 0 background resource */
          <div class="fixed inset-0 z-[200] bg-theme-bg/95 backdrop-blur-md flex items-center justify-center p-4 select-none">
            <div class="max-w-md w-full p-8 bg-theme-surface border border-theme-rose/50 shadow-2xl text-center space-y-5 animate-in fade-in duration-200">
              <div class="w-14 h-14 mx-auto bg-theme-rose/20 text-theme-rose border border-theme-rose/40 flex items-center justify-center text-2xl font-bold shadow-lg shadow-theme-rose/10">
                ✕
              </div>
              <div class="space-y-2">
                <h3 class="text-lg font-bold text-theme-text font-heading">
                  {deadReason() === 'shutdown' ? 'Aplikasi Ragasukma Ditutup' : 'Tab Dinonaktifkan'}
                </h3>
                <p class="text-sm text-theme-text/70 leading-relaxed">
                  {deadReason() === 'shutdown'
                    ? 'Jendela overlay dan server utama telah ditutup. Seluruh pemutar audio, background stream, dan memori browser telah dibersihkan secara tuntas.'
                    : 'Dashboard telah dibuka di tab browser lain. Semua koneksi audio, event stream, dan resource di tab ini sudah dinonaktifkan.'}
                </p>
              </div>
              <Show when={deadReason() === 'duplicate'}>
                <button
                  onClick={() => window.location.reload()}
                  class="w-full px-5 py-3 bg-theme-card hover:bg-theme-border text-theme-cyan text-sm font-bold border border-theme-border hover:border-theme-cyan/50 transition-all shadow-sm active:scale-[0.98]"
                >
                  Gunakan Tab Ini Kembali
                </button>
              </Show>
              <Show when={deadReason() === 'shutdown'}>
                <div class="text-xs font-mono text-theme-text/50 pt-1">
                  Kamu dapat menutup tab browser ini.
                </div>
              </Show>
            </div>
          </div>
        }
      >
        <Sidebar
          activeTab={activeTab()}
          setActiveTab={handleTabChange}
          overlayConnected={true}
          currentState={currentState()}
          isMobileOpen={mobileMenuOpen()}
          onCloseMobile={() => setMobileMenuOpen(false)}
        />

        <main class="flex-1 flex flex-col h-full overflow-hidden bg-theme-surface">
          {/* Top Header */}
          <header class="h-14 px-4 sm:px-6 md:px-8 border-b border-theme-border flex items-center justify-between shrink-0 bg-theme-bg/90 backdrop-blur-xl select-none">
            <div class="flex items-center gap-3">
              <button
                onClick={() => setMobileMenuOpen(true)}
                class="p-2 bg-theme-card text-theme-text border border-theme-border hover:border-theme-cyan/50 md:hidden transition-colors"
                aria-label="Buka Menu"
              >
                <MenuIcon size={18} />
              </button>

              <div class="flex items-center gap-2">
                <span class="text-xs font-mono font-bold px-2.5 py-1 bg-theme-rose/20 text-theme-rose border border-theme-rose/30 uppercase tracking-wider">
                  STATE: {currentState()}
                </span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="flex items-center gap-2 px-3 py-1 bg-theme-card border border-theme-border text-xs font-semibold text-theme-text">
                <span class="w-2 h-2 bg-theme-cyan shadow-[0_0_8px_#7BDFF2] animate-pulse" />
                <span class="tracking-wide">Avatar Overlay</span>
              </div>
            </div>
          </header>

          {/* Scrollable View Content */}
          <div class="flex-1 overflow-y-auto p-4 sm:p-6 md:p-8 max-w-5xl w-full mx-auto pb-12">
            <Show when={activeTab() === 'menu'}>
              <MenuTab
                currentState={currentState()}
                setCurrentState={setCurrentState}
                showToast={showToast}
              />
            </Show>
            <Show when={activeTab() === 'canvas'}>
              <CanvasTab showToast={showToast} />
            </Show>
            <Show when={activeTab() === 'skills'}>
              <SkillsTab showToast={showToast} />
            </Show>
            <Show when={activeTab() === 'music'}>
              <MusicTab showToast={showToast} />
            </Show>
            <Show when={activeTab() === 'tuner'}>
              <TunerTab showToast={showToast} />
            </Show>
            <Show when={activeTab() === 'logs'}>
              <LogsTab showToast={showToast} />
            </Show>
            <Show when={activeTab() === 'settings'}>
              <SettingsTab showToast={showToast} />
            </Show>
            <Show when={activeTab() === 'provider-antigravity'}>
              <AntigravityTab showToast={showToast} />
            </Show>
          </div>

          <BottomPlayer
            onOpenMusicTab={() => setActiveTab('music')}
            activeTab={activeTab()}
          />
          <audio ref={(el) => initGlobalAudio(el)} preload="auto" />
        </main>
      </Show>

      {/* Toast */}
      <div
        class={`fixed bottom-6 right-4 sm:right-8 px-5 py-3.5 bg-theme-card border border-theme-cyan text-sm font-semibold text-theme-text shadow-2xl transition-all duration-300 pointer-events-none z-50 flex items-center gap-3 max-w-[90vw] ${
          toastVisible() ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
        }`}
      >
        <span class="w-2 h-2 bg-theme-cyan shrink-0 shadow-[0_0_6px_#7BDFF2]" />
        <span class="truncate">{toastMessage()}</span>
      </div>
    </div>
  );
};

export default App;
