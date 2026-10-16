import { Component, createSignal, createEffect, onMount, onCleanup, Show } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { MenuTab } from './components/MenuTab';
import { RecipesTab } from './components/RecipesTab';
import { HistoryTab } from './components/HistoryTab';

import { SkillsTab } from './components/SkillsTab';
import { MusicTab } from './components/MusicTab';
import { TunerTab } from './components/tuner/TunerTab';
import { LogsTab } from './components/LogsTab';
import { SettingsTab } from './components/SettingsTab';
import { AntigravityTab } from './components/AntigravityTab';
import { CodexTab } from './components/CodexTab';
import { RemoteTab } from './components/RemoteTab';
import { MobileClientView } from './components/MobileClientView';
import { BottomPlayer } from './components/BottomPlayer';
import { ModelLoadingOverlay } from './components/ModelLoadingOverlay';
import { ModelDownloadStatus } from './services/eventsService';
import { initGlobalAudio, killGlobalAudio, currentTrack, isPlaying, isPaused } from './playerStore';
import { Menu as MenuIcon } from 'lucide-solid';
import { api, killCurrentSession } from './api';
import { VoiceSessionManager } from './services/voiceSession';

// Token unik per instance tab, created when module pertama kali loaded
const MY_TOKEN = 'sess_' + Math.random().toString(36).substring(2, 11) + Date.now().toString(36);
const LS_KEY = 'ragasukma_active_tab';
const BC_NAME = 'ragasukma_session';

const TAB_HASH_ALIASES: Record<string, string> = {
  '': 'menu',
  '#': 'menu',
  '#menu': 'menu',
  '#recipes': 'recipes',
  '#recipe': 'recipes',
  '#history': 'history',
  '#chat-history': 'history',
  '#skills': 'skills',
  '#music': 'music',
  '#tuner': 'tuner',
  '#logs': 'logs',
  '#settings': 'settings',
  '#remote': 'remote',
  '#provider-antigravity': 'provider-antigravity',
  '#antigravity': 'provider-antigravity',
  '#provider-codex': 'provider-codex',
  '#codex': 'provider-codex',
};

const getInitialTabFromHash = (): string => {
  try {
    const hash = window.location.hash.toLowerCase();
    if (TAB_HASH_ALIASES[hash]) {
      return TAB_HASH_ALIASES[hash];
    }
    const clean = hash.replace(/^#/, '').split('?')[0];
    if (TAB_HASH_ALIASES[`#${clean}`]) {
      return TAB_HASH_ALIASES[`#${clean}`];
    }
    if (clean) return clean;
  } catch {}
  return 'menu';
};

const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal(getInitialTabFromHash());
  const [currentState, setCurrentState] = createSignal('idle');
  const [isDead, setIsDead] = createSignal(false);
  const [deadReason, setDeadReason] = createSignal<'duplicate' | 'shutdown'>('duplicate');
  const [toastMessage, setToastMessage] = createSignal('');
  const [toastVisible, setToastVisible] = createSignal(false);
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);
  const [modelStatus, setModelStatus] = createSignal<ModelDownloadStatus | null>(null);

  const isModelLocked = () => {
    const s = modelStatus()?.status;
    return s === 'downloading' || s === 'extracting' || s === 'error';
  };

  // Detect whether current tab is mobile remote client (opened via QR scan or key auth)
  const isMobileClient = () => {
    try {
      const search = window.location.search;
      const hash = window.location.hash.toLowerCase();
      const origin = window.location.origin.toLowerCase();
      const hasKey = search.includes('key=') || hash.includes('key=');
      // Only treat as mobile client if possessing auth key AND originating from cloudflare tunnel or mobile hash
      if (hasKey && (origin.includes('trycloudflare.com') || hash.includes('mobile'))) {
        return true;
      }
      if (hash === '#mobile' || hash === '#mobile-client') {
        return true;
      }
      return false;
    } catch {
      return false;
    }
  };

  const getMobileAuthKey = () => {
    try {
      // 1. Check query parameter standar (?key=...)
      const params = new URLSearchParams(window.location.search);
      const qKey = params.get('key');
      if (qKey) return qKey;

      // 2. Check query parameter inside hash if URL is #remote?key=...
      const hash = window.location.hash;
      if (hash.includes('?')) {
        const hashQuery = hash.split('?')[1];
        const hashParams = new URLSearchParams(hashQuery);
        const hKey = hashParams.get('key');
        if (hKey) return hKey;
      }
      return '';
    } catch {
      return '';
    }
  };

  const voiceSession = new VoiceSessionManager({
    isDead: () => isDead(),
    isMobileClient: () => isMobileClient(),
  });

  let activeBc: BroadcastChannel | null = null;
  let unsubStateFn: (() => void) | null = null;
  let unsubActionsFn: (() => void) | null = null;
  let unsubModelStatusFn: (() => void) | null = null;

  // Synchronize desktop browser tab title when playing music
  createEffect(() => {
    if (isMobileClient()) return;

    const track = currentTrack();
    if (track && track.title) {
      if (isPlaying()) {
        document.title = `▶ ${track.title} - Ragasukma`;
      } else if (isPaused()) {
        document.title = `⏸ ${track.title} - Ragasukma`;
      } else {
        document.title = 'Ragasukma - AI Companion & Dashboard';
      }
    } else {
      document.title = 'Ragasukma - AI Companion & Dashboard';
    }
  });

  const killThisTab = (reason: 'duplicate' | 'shutdown' = 'duplicate') => {
    if (isDead()) return;
    setDeadReason(reason);
    setIsDead(true);

    // 1. Stop Chrome STT, TTS, & VAD Barge-in monitor
    voiceSession.destroy();

    // 2. Disconnect SSE stream and cancel listeners
    try {
      unsubStateFn?.();
      unsubActionsFn?.();
      unsubModelStatusFn?.();
      api.forceCloseSSE();
    } catch {}

    // 3. Close session, global audio, and BroadcastChannel
    killCurrentSession();
    killGlobalAudio();
    try { activeBc?.close(); } catch {}

    if (document.pictureInPictureElement) {
      document.exitPictureInPicture().catch(() => {});
    }
  };

  onMount(() => {
    // Smartphone Remote Client mode: bypass single-tab kill
    if (isMobileClient()) {
      console.log('[Session] Remote client detected, skipping single-tab exclusive claim');
      return;
    }

    console.log('[Session] Tab mount, MY_TOKEN =', MY_TOKEN);

    let bc: BroadcastChannel | null = null;
    try {
      bc = new BroadcastChannel(BC_NAME);
      activeBc = bc;
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

    // CLAIM 1X
    if (!isMobileClient()) {
      console.log('[Session] Claiming ownership 1x...');
      try { localStorage.setItem(LS_KEY, MY_TOKEN); } catch {}
      try { bc?.postMessage({ type: 'CLAIM', token: MY_TOKEN }); } catch {}
      api.claimSession(MY_TOKEN, activeTab()).catch(() => {});
    }

    // Check initial model status
    api.getModelStatus().then((st) => {
      setModelStatus(st);
    }).catch(() => {});

    // SSE: Model status listener
    unsubModelStatusFn = api.subscribeModelStatus((st) => {
      setModelStatus(st);
    });

    // Start voice session manager (STT, VAD, Echo Guard)
    voiceSession.init();

    // SSE: UI state update & shutdown / claim detection
    const unsubState = api.subscribeState(
      (st) => {
        if (isDead()) return;
        if (st?.state === 'shutdown') {
          console.log('[Session] Backend signaled SHUTDOWN');
          killThisTab('shutdown');
          return;
        }
        if (st?.state) setCurrentState(st.state);

        voiceSession.updateMuteState(Boolean(st?.voice_muted), Boolean(st?.mic_muted));
        if (typeof st?.is_running === 'boolean') {
          voiceSession.updateAssistantRunning(st.is_running);
        }
      }
    );
    unsubStateFn = unsubState;

    const handleTestVoiceEvent = (e: CustomEvent) => {
      voiceSession.handleTestVoice(!!e.detail?.active);
    };
    window.addEventListener('ragasukma:test_voice' as any, handleTestVoiceEvent);

    const handleConfigUpdated = (e: CustomEvent) => {
      const cfg = e.detail;
      voiceSession.handleConfigUpdated(cfg);
      console.log('[Runtime Config] Updated after save:', cfg);
    };
    window.addEventListener('ragasukma:config_updated' as any, handleConfigUpdated);

    let lastHandledActionId = -1;

    const unsubActions = api.subscribeActions((act) => {
      if (isDead()) return;

      if (act.action === 'session:claimed' && act.payload && act.payload !== MY_TOKEN) {
        if (!isMobileClient()) {
          console.log('[Session] Backend signaled another tab claimed ownership -> killThisTab');
          killThisTab('duplicate');
          return;
        }
      }

      if (act.id !== undefined) {
        if (act.id <= lastHandledActionId) {
          return;
        }
        lastHandledActionId = act.id;
      }

      if (act.action === 'assistant:speak' && act.payload) {
        voiceSession.handleAssistantSpeak(act.payload);
      }
    });
    unsubActionsFn = unsubActions;

    const handleHashChange = () => {
      const tab = getInitialTabFromHash();
      if (tab !== activeTab()) {
        setActiveTab(tab);
      }
    };
    window.addEventListener('hashchange', handleHashChange);

    onCleanup(() => {
      voiceSession.destroy();
      try { bc?.close(); } catch {}
      window.removeEventListener('storage', handleStorage);
      window.removeEventListener('hashchange', handleHashChange);
      window.removeEventListener('ragasukma:config_updated' as any, handleConfigUpdated);
      window.removeEventListener('ragasukma:test_voice' as any, handleTestVoiceEvent);
      unsubState();
      unsubActions();
      unsubModelStatusFn?.();
    });
  });

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setToastVisible(true);
    setTimeout(() => setToastVisible(false), 2500);
  };

  const handleTabChange = (tab: string) => {
    if (tab === activeTab()) return;
    setActiveTab(tab);
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
        when={isMobileClient()}
        fallback={
          <Show
            when={!isDead()}
            fallback={
              /* Tab Deactivated / Shutdown Modal */
              <div class="fixed inset-0 z-[200] bg-theme-bg/95 backdrop-blur-md flex items-center justify-center p-4 select-none">
                <div class="max-w-md w-full p-8 bg-theme-surface border border-theme-rose/50 shadow-2xl text-center space-y-5 animate-in fade-in duration-200">
                  <div class="w-14 h-14 mx-auto bg-theme-rose/20 text-theme-rose border border-theme-rose/40 flex items-center justify-center text-2xl font-bold shadow-lg shadow-theme-rose/10">
                    ✕
                  </div>
                  <div class="space-y-2">
                    <h3 class="text-theme-text">
                      {deadReason() === 'shutdown' ? 'Ragasukma Application Closed' : 'Tab Deactivated'}
                    </h3>
                    <p class="text-sm text-theme-text/70 leading-relaxed">
                      {deadReason() === 'shutdown'
                        ? 'The overlay window and backend server have closed. All audio players, background streams, and browser resources have been completely released.'
                        : 'The dashboard was opened in another browser tab. Audio connections, event streams, and resources in this tab have been disabled.'}
                    </p>
                  </div>
                  <Show when={deadReason() === 'duplicate'}>
                    <button
                      onClick={() => window.location.reload()}
                      class="w-full px-5 py-3 bg-theme-card hover:bg-theme-border text-theme-cyan text-sm font-bold border border-theme-border hover:border-theme-cyan/50 transition-all shadow-sm active:scale-[0.98]"
                    >
                      Use This Tab Instead
                    </button>
                  </Show>
                  <Show when={deadReason() === 'shutdown'}>
                    <div class="text-xs font-mono text-theme-text/50 pt-1">
                      You can safely close this browser tab.
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
                    aria-label="Open Menu"
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
                    <span class="w-2 h-2 bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse" />
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
                    onNavigateTab={handleTabChange}
                  />
                </Show>

                <Show when={activeTab() === 'recipes'}>
                  <RecipesTab showToast={showToast} />
                </Show>

                <Show when={activeTab() === 'history'}>
                  <HistoryTab showToast={showToast} />
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
                <Show when={activeTab() === 'remote'}>
                  <RemoteTab showToast={showToast} />
                </Show>
                <div class={activeTab() === 'settings' ? 'block' : 'hidden'}>
                  <SettingsTab showToast={showToast} isActive={activeTab() === 'settings'} />
                </div>
                <Show when={activeTab() === 'provider-antigravity'}>
                  <AntigravityTab showToast={showToast} />
                </Show>
                <Show when={activeTab() === 'provider-codex'}>
                  <CodexTab showToast={showToast} />
                </Show>
              </div>

              <BottomPlayer
                onOpenMusicTab={() => setActiveTab('music')}
                activeTab={activeTab()}
              />
              <audio ref={(el) => initGlobalAudio(el)} preload="auto" />
            </main>
          </Show>
        }
      >
        {/* DISPLAY KHUSUS SMARTPHONE (ONLY MIC PTT + CONSOLE LOGS REAL-TIME) */}
        <MobileClientView
          authKey={getMobileAuthKey()}
          showToast={showToast}
        />
      </Show>

      {/* Model Download & Extraction Fullscreen Overlay */}
      <Show when={isModelLocked()}>
        <ModelLoadingOverlay
          modelStatus={modelStatus()}
          onRetry={() => {
            api.triggerModelDownload().then((st) => {
              setModelStatus(st);
            }).catch(() => {});
          }}
        />
      </Show>

      {/* Toast */}
      <div
        class={`fixed bottom-6 right-4 sm:right-8 px-5 py-3.5 bg-theme-card border border-theme-cyan text-sm font-semibold text-theme-text shadow-2xl transition-all duration-300 pointer-events-none z-[999] flex items-center gap-3 max-w-[90vw] ${
          toastVisible() ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-4'
        }`}
      >
        <span class="w-2 h-2 bg-theme-cyan shrink-0 shadow-[0_0_6px_var(--cyan)]" />
        <span class="truncate">{toastMessage()}</span>
      </div>
    </div>
  );
};

export default App;
