import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { MenuTab } from './components/MenuTab';
import { SkillsTab } from './components/SkillsTab';
import { MusicTab } from './components/MusicTab';
import { TunerTab } from './components/TunerTab';
import { LogsTab } from './components/LogsTab';
import { SettingsTab } from './components/SettingsTab';
import { BottomPlayer } from './components/BottomPlayer';
import { initGlobalAudio, killGlobalAudio } from './playerStore';
import { Sparkles, Menu as MenuIcon } from 'lucide-solid';
import { api, killCurrentSession } from './api';

// Token unik per instance tab, dibuat saat module pertama kali dimuat
const MY_TOKEN = 'sess_' + Math.random().toString(36).substring(2, 11) + Date.now().toString(36);
const LS_KEY = 'ragasukma_active_tab';
const BC_NAME = 'ragasukma_session';

export const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal('menu');
  const [currentState, setCurrentState] = createSignal('idle');
  const [isDead, setIsDead] = createSignal(false);
  const [toastMessage, setToastMessage] = createSignal('');
  const [toastVisible, setToastVisible] = createSignal(false);
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);

  const killThisTab = () => {
    if (isDead()) return;
    setIsDead(true);
    killCurrentSession();
    killGlobalAudio();
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

    // ── 2. LocalStorage polling (fallback cross-browser / incognito) ──
    const checkLS = () => {
      try {
        const active = localStorage.getItem(LS_KEY);
        if (active && active !== MY_TOKEN) {
          console.log('[Session] localStorage poll -> killThisTab, active =', active);
          killThisTab();
        }
      } catch {}
    };
    window.addEventListener('storage', (e) => {
      console.log('[Session] storage event:', e.key, e.newValue);
      if (e.key === LS_KEY && e.newValue && e.newValue !== MY_TOKEN) {
        console.log('[Session] storage event -> killThisTab');
        killThisTab();
      }
    });
    const lsTimer = setInterval(checkLS, 300);

    // ── 3. CLAIM: tulis ke localStorage + broadcast (tab lama langsung mati) ──
    console.log('[Session] Claiming ownership...');
    try { localStorage.setItem(LS_KEY, MY_TOKEN); } catch {}
    try { bc?.postMessage({ type: 'CLAIM', token: MY_TOKEN }); } catch {}
    api.takeoverSession(MY_TOKEN).catch(() => {});
    console.log('[Session] Claim done');

    // ── 4. SSE: untuk update state UI ──
    const unsubSSE = api.subscribeState(
      (st) => {
        if (st?.state) setCurrentState(st.state);
        if (st?.active_session_token && st.active_session_token !== MY_TOKEN) {
          console.log('[Session] SSE token mismatch -> killThisTab');
          killThisTab();
        }
      },
      (_connected) => {}
    );

    onCleanup(() => {
      clearInterval(lsTimer);
      try { bc?.close(); } catch {}
      unsubSSE();
    });
  });

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setToastVisible(true);
    setTimeout(() => setToastVisible(false), 2500);
  };

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-theme-bg text-theme-text font-open">
      <Sidebar
        activeTab={activeTab()}
        setActiveTab={setActiveTab}
        overlayConnected={!isDead()}
        currentState={currentState()}
        isMobileOpen={mobileMenuOpen()}
        onCloseMobile={() => setMobileMenuOpen(false)}
      />

      <main class="flex-1 flex flex-col h-full overflow-hidden bg-theme-surface">
        {/* Top Responsive Navbar */}
        <header class="h-16 px-4 sm:px-6 md:px-8 border-b border-theme-border flex items-center justify-between shrink-0 bg-theme-bg/90 backdrop-blur-md">
          <div class="flex items-center gap-2.5 sm:gap-3">
            <button
              onClick={() => setMobileMenuOpen(true)}
              class="p-2 rounded-xl bg-theme-card text-theme-text border border-theme-border hover:border-theme-cyan/50 md:hidden transition-colors"
              aria-label="Buka Menu"
            >
              <MenuIcon size={20} />
            </button>

            <span class="text-xs font-mono font-bold px-2.5 py-1 rounded-full bg-theme-rose/20 text-theme-rose border border-theme-rose/30 uppercase tracking-wide">
              STATE: {currentState()}
            </span>
            <span class="hidden sm:inline text-xs text-theme-text/40">•</span>
            <span class="hidden sm:flex text-xs text-theme-cyan font-semibold items-center gap-1.5">
              <Sparkles size={14} />
              WGPU MToon 1:1 Three-VRM
            </span>
          </div>

          <div class="flex items-center gap-2 sm:gap-3">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-theme-card border border-theme-border text-xs font-semibold text-theme-text">
              <span class="w-2 h-2 rounded-full bg-theme-cyan shadow-[0_0_6px_#7BDFF2] animate-pulse" />
              <span class="hidden xs:inline sm:inline">Overlay 9:16</span>
            </div>
          </div>
        </header>

        {/* Scrollable View Content */}
        <div class="flex-1 overflow-y-auto p-4 sm:p-6 md:p-8 max-w-5xl w-full mx-auto pb-28">
          {activeTab() === 'menu' && (
            <MenuTab
              currentState={currentState()}
              setCurrentState={setCurrentState}
              showToast={showToast}
            />
          )}
          {activeTab() === 'skills' && <SkillsTab showToast={showToast} />}
          {activeTab() === 'music' && <MusicTab showToast={showToast} />}
          {activeTab() === 'tuner' && <TunerTab showToast={showToast} />}
          {activeTab() === 'logs' && <LogsTab showToast={showToast} />}
          {activeTab() === 'settings' && <SettingsTab showToast={showToast} />}
        </div>

        <BottomPlayer onOpenMusicTab={() => setActiveTab('music')} />
        <audio ref={(el) => initGlobalAudio(el)} preload="auto" />
      </main>

      {/* Modal Tab Mati — overlay penuh, tidak bisa diapa-apain */}
      {isDead() && (
        <div class="fixed inset-0 z-[200] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4"
          onMouseDown={(e) => e.stopPropagation()}
          onKeyDown={(e) => e.stopPropagation()}
        >
          <div class="max-w-sm w-full p-6 rounded-2xl bg-theme-surface border border-theme-rose/50 shadow-2xl text-center space-y-3">
            <div class="w-12 h-12 mx-auto rounded-xl bg-theme-rose/20 text-theme-rose border border-theme-rose/30 flex items-center justify-center text-xl font-bold">
              ✕
            </div>
            <h3 class="text-sm font-semibold font-open normal-case tracking-normal text-theme-text">
              Tab Ini Sudah Tidak Aktif
            </h3>
            <p class="text-xs text-theme-text/50">
              Dashboard dibuka di tab lain. Tab ini sudah dinonaktifkan.
            </p>
          </div>
        </div>
      )}

      {/* Toast */}
      <div
        class={`fixed bottom-6 right-4 sm:right-8 px-4 sm:px-5 py-3 rounded-xl bg-theme-card border border-theme-cyan text-xs sm:text-sm font-semibold text-theme-text shadow-2xl transition-all duration-300 pointer-events-none z-50 flex items-center gap-2.5 max-w-[90vw] ${
          toastVisible() ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
        }`}
      >
        <span class="w-2 h-2 rounded-full bg-theme-cyan shrink-0" />
        <span class="truncate">{toastMessage()}</span>
      </div>
    </div>
  );
};

export default App;
