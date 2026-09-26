import { Component, createSignal, onMount, onCleanup, Show } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { MenuTab } from './components/MenuTab';

import { SkillsTab } from './components/SkillsTab';
import { MusicTab } from './components/MusicTab';
import { TunerTab } from './components/TunerTab';
import { LogsTab } from './components/LogsTab';
import { SettingsTab } from './components/SettingsTab';
import { AntigravityTab } from './components/AntigravityTab';
import { RemoteTab } from './components/RemoteTab';
import { MobileClientView } from './components/MobileClientView';
import { BottomPlayer } from './components/BottomPlayer';
import { initGlobalAudio, killGlobalAudio } from './playerStore';
import { Menu as MenuIcon } from 'lucide-solid';
import { api, killCurrentSession } from './api';
import { configService } from './services/configService';
import { API_BASE } from './services/base';

// Token unik per instance tab, dibuat saat module pertama kali dimuat
const MY_TOKEN = 'sess_' + Math.random().toString(36).substring(2, 11) + Date.now().toString(36);
const LS_KEY = 'ragasukma_active_tab';
const BC_NAME = 'ragasukma_session';

const TAB_HASH_ALIASES: Record<string, string> = {
  '': 'menu',
  '#': 'menu',
  '#menu': 'menu',

  '#skills': 'skills',
  '#music': 'music',
  '#tuner': 'tuner',
  '#logs': 'logs',
  '#settings': 'settings',
  '#remote': 'remote',
  '#provider-antigravity': 'provider-antigravity',
  '#antigravity': 'provider-antigravity',
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

export const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal(getInitialTabFromHash());
  const [currentState, setCurrentState] = createSignal('idle');
  const [isDead, setIsDead] = createSignal(false);
  const [deadReason, setDeadReason] = createSignal<'duplicate' | 'shutdown'>('duplicate');
  const [toastMessage, setToastMessage] = createSignal('');
  const [toastVisible, setToastVisible] = createSignal(false);
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);

  // Deteksi apakah tab saat ini adalah Remote Client HP (dibuka lewat scan QR / key auth)
  const isMobileClient = () => {
    try {
      const hash = window.location.hash.toLowerCase();
      const origin = window.location.origin.toLowerCase();
      const search = window.location.search;
      return (
        hash.includes('remote') ||
        origin.includes('trycloudflare.com') ||
        search.includes('key=')
      );
    } catch {
      return false;
    }
  };

  const getMobileAuthKey = () => {
    try {
      // 1. Cek query parameter standar (?key=...)
      const params = new URLSearchParams(window.location.search);
      const qKey = params.get('key');
      if (qKey) return qKey;

      // 2. Cek query parameter di dalam hash jika URL berupa #remote?key=...
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

  let activeRecognition: any = null;
  let activeBc: BroadcastChannel | null = null;
  let unsubStateFn: (() => void) | null = null;
  let unsubActionsFn: (() => void) | null = null;

  const killThisTab = (reason: 'duplicate' | 'shutdown' = 'duplicate') => {
    if (isDead()) return;
    setDeadReason(reason);
    setIsDead(true);

    // 1. Matikan Chrome Speech Synthesis seketika
    if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
      try {
        window.speechSynthesis.cancel();
      } catch {}
    }

    // 2. Putus mikrofon Chrome STT seketika tanpa restart
    if (activeRecognition) {
      try {
        activeRecognition.onend = null;
        activeRecognition.onerror = null;
        activeRecognition.onresult = null;
        activeRecognition.onspeechstart = null;
        activeRecognition.abort();
      } catch {}
      activeRecognition = null;
    }

    // 3. Putus stream SSE & batalkan listener agar tab mati tidak lagi menerima / memproses respon LLM
    try {
      unsubStateFn?.();
      unsubActionsFn?.();
      api.forceCloseSSE();
    } catch {}

    // 4. Tutup session, global audio, dan BroadcastChannel
    killCurrentSession();
    killGlobalAudio();
    try { activeBc?.close(); } catch {}

    if (document.pictureInPictureElement) {
      document.exitPictureInPicture().catch(() => {});
    }
  };

  onMount(() => {
    // Mode Remote Client dari smartphone: bypass single-tab kill agar tidak mematikan tab utama di desktop
    if (isMobileClient()) {
      console.log('[Session] Remote client detected, skipping single-tab exclusive claim');
      return;
    }

    console.log('[Session] Tab mount, MY_TOKEN =', MY_TOKEN);

    // ── 1. BroadcastChannel: siap dengar SEBELUM kita claim ──
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

    // ── 3. CLAIM 1X: kirim ke backend + tulis ke localStorage (tab lama langsung mati) ──
    console.log('[Session] Claiming ownership 1x...');
    try { localStorage.setItem(LS_KEY, MY_TOKEN); } catch {}
    try { bc?.postMessage({ type: 'CLAIM', token: MY_TOKEN }); } catch {}
    api.claimSession(MY_TOKEN, activeTab()).catch(() => {});

    // ── 4. SSE: untuk update state UI & deteksi shutdown / token klaim ──
    const unsubState = api.subscribeState(
      (st) => {
        if (isDead()) return;
        if (st?.state === 'shutdown') {
          console.log('[Session] Backend signaled SHUTDOWN');
          killThisTab('shutdown');
          return;
        }
        if (st?.state) setCurrentState(st.state);

        // Sinkronkan state Mute langsung dari State SSE
        const nextVoiceMuted = Boolean(st?.voice_muted);
        const nextMicMuted = Boolean(st?.mic_muted);

        if (isVoiceMuted !== nextVoiceMuted) {
          isVoiceMuted = nextVoiceMuted;
          if (isVoiceMuted && typeof window !== 'undefined' && 'speechSynthesis' in window) {
            window.speechSynthesis.cancel();
          }
        }

        if (isMicMuted !== nextMicMuted) {
          isMicMuted = nextMicMuted;
          if (isMicMuted) {
            try { recognition?.abort(); } catch {}
          } else if (!isTtsSpeaking && !isDead()) {
            try { recognition?.start(); } catch {}
          }
        }
      }
    );
    unsubStateFn = unsubState;

    // Cache konfigurasi lokal untuk reaktivitas instan tanpa fetch berulang
    let runtimeConfig: any = null;
    configService.getConfig().then((cfg) => {
      runtimeConfig = cfg;
    });

    // 🎙️ Google Chrome Speech Recognition (STT) - Dengar suara dari browser dan kirim transkrip ke backend
    let recognition: any = null;
    let isTtsSpeaking = false;
    let isVoiceMuted = false;
    let isMicMuted = false;
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;

    const applyRecognitionLang = (lang: string) => {
      if (!recognition) return;
      if (recognition.lang !== lang) {
        recognition.lang = lang;
        console.log('[Chrome Web STT] Reaktif ganti bahasa STT ke:', lang);
        try {
          recognition.stop();
        } catch {}
      }
    };



    try {
      if (SpeechRecognition) {
        recognition = new SpeechRecognition();
        activeRecognition = recognition;
        recognition.continuous = true;
        recognition.interimResults = false;

        configService.getConfig().then((cfg) => {
          if (recognition) {
            recognition.lang = cfg?.stt_lang || 'id-ID';
          }
        });

        recognition.onspeechstart = () => {
          // ⚡ Voice Interruption (Barge-in): Begitu pengguna mulai bersuara, potong TTS seketika!
          if (isTtsSpeaking && runtimeConfig?.voice_interruption) {
            console.log('⚡ [Barge-in: onspeechstart] Suara terdeteksi, memotong suara asisten seketika!');
            if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
              window.speechSynthesis.cancel();
            }
            isTtsSpeaking = false;
            fetch(`${API_BASE}/api/tts/state`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
            }).catch(() => {});
          }
        };

        recognition.onresult = (event: any) => {
          if (isMicMuted) return;

          // ⚡ Voice Interruption (Barge-in): Potong TTS seketika saat user bicara
          if (isTtsSpeaking) {
            if (runtimeConfig?.voice_interruption) {
              console.log('⚡ [Barge-in: onresult] Suara user memotong pembicaraan asisten!');
              if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
                window.speechSynthesis.cancel();
              }
              isTtsSpeaking = false;
              fetch(`${API_BASE}/api/tts/state`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
              }).catch(() => {});
            } else {
              // Jika interupsi nonaktif, abaikan suara selama TTS bersuara
              return;
            }
          }

          for (let i = event.resultIndex; i < event.results.length; ++i) {
            if (event.results[i].isFinal) {
              const transcript = event.results[i][0].transcript?.trim();
              if (transcript) {
                // Filter self-echo jika mikrofon menangkap gema suara asisten sendiri
                const spokenClean = (lastSpokenText || '').toLowerCase().replace(/[^a-z0-9]/g, '');
                const transcriptClean = transcript.toLowerCase().replace(/[^a-z0-9]/g, '');
                if (
                  spokenClean.length > 6 &&
                  transcriptClean.length > 5 &&
                  spokenClean.includes(transcriptClean)
                ) {
                  console.warn('[Barge-in] Self-echo suara asisten diabaikan:', transcript);
                  continue;
                }

                console.log('[Chrome Web STT] Sending transcript:', transcript);
                fetch(`${API_BASE}/api/stt/transcript`, {
                  method: 'POST',
                  headers: { 'Content-Type': 'application/json' },
                  body: JSON.stringify({ text: transcript }),
                }).catch((err) => console.error('[Chrome Web STT] Failed to send transcript:', err));
              }
            }
          }
        };

        recognition.onerror = (e: any) => {
          console.warn('[Chrome Web STT] Error:', e.error);
        };

        recognition.onend = () => {
          const shouldListen =
            !isDead() && !isMicMuted && (runtimeConfig?.voice_interruption || !isTtsSpeaking);
          if (shouldListen) {
            try { recognition.start(); } catch {}
          }
        };

        // Mulai listen di background jika tidak muted
        if (!isMicMuted) {
          try { recognition.start(); } catch {}
        }
      }
    } catch (e) {
      console.warn('[Chrome Web STT] SpeechRecognition init failed:', e);
    }

    // Listener custom event untuk reaktivitas instan HANYA saat user klik "Simpan" di SettingsTab (1 Pintu)
    const handleConfigUpdated = (e: CustomEvent) => {
      const cfg = e.detail;
      runtimeConfig = cfg;
      if (cfg?.stt_lang) {
        applyRecognitionLang(cfg.stt_lang);
      }
      console.log('[Runtime Config] Diperbarui 1 pintu setelah Simpan:', cfg);
    };
    window.addEventListener('ragasukma:config_updated' as any, handleConfigUpdated);

    let lastSpokenText = '';
    let lastSpokenTime = 0;
    let lastHandledActionId = -1;

    const unsubActions = api.subscribeActions((act) => {
      if (isDead()) return;

      if (act.action === 'session:claimed' && act.payload && act.payload !== MY_TOKEN) {
        console.log('[Session] Backend signaled another tab claimed ownership -> killThisTab');
        killThisTab('duplicate');
        return;
      }

      // Deduplikasi action SSE via ID unik incremental dari backend
      if (act.id !== undefined) {
        if (act.id <= lastHandledActionId) {
          return;
        }
        lastHandledActionId = act.id;
      }

      // 🔊 Google Chrome Speech Synthesis (TTS) - Mainkan respon suara LLM langsung dari browser
      if (act.action === 'assistant:speak' && act.payload) {
        let textToSpeak = '';
        let emotion = 'neutral';
        try {
          const parsed = typeof act.payload === 'string' ? JSON.parse(act.payload) : act.payload;
          textToSpeak = parsed.text || '';
          emotion = parsed.emotion || 'neutral';
        } catch {
          textToSpeak = typeof act.payload === 'string' ? act.payload : '';
        }

        if (isVoiceMuted) {
          console.log('[TTS] Suara TTS sedang dimute, pembicaraan dilewati.');
          return;
        }

        const now = Date.now();
        // Cegah duplikasi kalimat identik yang ditembak berulang dalam jeda < 1.5 detik
        if (textToSpeak === lastSpokenText && now - lastSpokenTime < 1500) {
          console.warn('[TTS] Duplicate speak action suppressed:', textToSpeak);
          return;
        }
        lastSpokenText = textToSpeak;
        lastSpokenTime = now;

        if (textToSpeak && typeof window !== 'undefined' && 'speechSynthesis' in window) {
          // Matikan antrean suara lama agar tidak pernah tumpuk/dobel
          window.speechSynthesis.cancel();

          // Chrome bug guard: teks panjang (>200 karakter) di Chrome speechSynthesis bisa macet di tengah jalan.
          // Kita pecah per kalimat pendek secara rapi.
          const splitSentences = (txt: string): string[] => {
            const matches = txt.match(/[^.!?]+[.!?]+(\s|$)|[^.!?]+$/g);
            return matches ? matches.map((s) => s.trim()).filter(Boolean) : [txt.trim()];
          };

          const sentences = splitSentences(textToSpeak);
          if (sentences.length === 0) return;

          const cfg = runtimeConfig;
          const voices = window.speechSynthesis.getVoices();
          const matchedVoice = cfg?.tts_voice_name ? voices.find((v) => v.name === cfg.tts_voice_name) : null;

          let currentIndex = 0;
          let keepAliveTimer: any = null;

          const finishAll = () => {
            if (keepAliveTimer) {
              clearInterval(keepAliveTimer);
              keepAliveTimer = null;
            }
            isTtsSpeaking = false;
            fetch(`${API_BASE}/api/tts/state`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ speaking: false }),
            }).catch(() => {});

            // Jeda 350ms hening setelah suara selesai sebelum mic menyala lagi
            setTimeout(() => {
              if (!isDead() && !isTtsSpeaking) {
                try { recognition?.start(); } catch {}
              }
            }, 350);
          };

          const speakNextSentence = () => {
            if (currentIndex >= sentences.length) {
              finishAll();
              return;
            }

            const sentenceText = sentences[currentIndex];
            currentIndex++;

            const utter = new SpeechSynthesisUtterance(sentenceText);
            if (matchedVoice) {
              utter.voice = matchedVoice;
              utter.lang = matchedVoice.lang;
            } else if (cfg?.stt_lang) {
              utter.lang = cfg.stt_lang;
            }

            if (cfg?.tts_speed) utter.rate = cfg.tts_speed;
            if (cfg?.tts_pitch) utter.pitch = cfg.tts_pitch;

            if (currentIndex === 1) {
              // Kalimat pertama dimulai
              isTtsSpeaking = true;
              // Hanya matikan mic jika pengguna menonaktifkan voice interruption
              if (!runtimeConfig?.voice_interruption) {
                try { recognition?.stop(); } catch {}
              }
              fetch(`${API_BASE}/api/tts/state`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ speaking: true, emotion }),
              }).catch(() => {});

              // Chrome Workaround: Chrome speechSynthesis pauses silently after ~15 seconds of speech
              if (keepAliveTimer) clearInterval(keepAliveTimer);
              keepAliveTimer = setInterval(() => {
                if (window.speechSynthesis.speaking) {
                  window.speechSynthesis.pause();
                  window.speechSynthesis.resume();
                } else {
                  clearInterval(keepAliveTimer);
                  keepAliveTimer = null;
                }
              }, 10000);
            }

            utter.onend = () => {
              speakNextSentence();
            };

            utter.onerror = (e: any) => {
              if (e?.error === 'canceled' || e?.error === 'interrupted') {
                finishAll();
                return;
              }
              console.warn('[TTS] Utterance error on chunk:', e);
              speakNextSentence();
            };

            window.speechSynthesis.speak(utter);
          };

          speakNextSentence();
        }
      }
    });
    unsubActionsFn = unsubActions;

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
      try { recognition?.stop(); } catch {}
      try { window.speechSynthesis?.cancel(); } catch {}
      window.removeEventListener('storage', handleStorage);
      window.removeEventListener('hashchange', handleHashChange);
      window.removeEventListener('ragasukma:config_updated' as any, handleConfigUpdated);
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
    if (tab === activeTab()) return;
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
        when={isMobileClient()}
        fallback={
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
                    <h3 class="text-lg font-bold text-theme-text">
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
        }
      >
        {/* TAMPILAN KHUSUS SMARTPHONE (HANYA MIC PTT + CONSOLE LOGS REAL-TIME) */}
        <MobileClientView
          authKey={getMobileAuthKey()}
          showToast={showToast}
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
