import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { MenuTab } from './components/MenuTab';
import { SkillsTab } from './components/SkillsTab';
import { TunerTab } from './components/TunerTab';
import { LogsTab } from './components/LogsTab';
import { SettingsTab } from './components/SettingsTab';
import { Sparkles, Menu as MenuIcon } from 'lucide-solid';
import { api } from './api';

export const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal('menu');
  const [currentState, setCurrentState] = createSignal('idle');
  const [overlayConnected, setOverlayConnected] = createSignal(true);
  const [toastMessage, setToastMessage] = createSignal('');
  const [toastVisible, setToastVisible] = createSignal(false);
  const [mobileMenuOpen, setMobileMenuOpen] = createSignal(false);
  const [isDuplicateTab, setIsDuplicateTab] = createSignal(false);

  // ID Unik sesi tab/browser ini (gunakan sessionStorage agar setiap tab baru memiliki token unik dan tidak bisa dibuka double)
  let sessionToken = sessionStorage.getItem('ragasukma_session_token');
  if (!sessionToken) {
    sessionToken = 'sess_' + Math.random().toString(36).substring(2, 11) + Date.now().toString(36);
    sessionStorage.setItem('ragasukma_session_token', sessionToken);
  }

  let heartbeatTimer: number | null = null;

  // Single-Client Mutex Protection (Server-Side Lease: Bekerja di Beda Tab, Beda Browser, & Incognito)
  const checkSessionHeartbeat = async () => {
    if (!sessionToken || document.hidden) return;
    const isActive = await api.sendHeartbeat(sessionToken);
    setIsDuplicateTab(!isActive);
  };

  const handleTakeover = async () => {
    if (!sessionToken) return;
    const ok = await api.takeoverSession(sessionToken);
    if (ok) {
      setIsDuplicateTab(false);
      showToast('Sesi kontrol berhasil diambil alih');
    }
  };

  onMount(() => {
    // Inisialisasi pengecekan sesi ke backend
    checkSessionHeartbeat();
    heartbeatTimer = window.setInterval(checkSessionHeartbeat, 10000);

    // Real-time EventSource (SSE) stream dari backend - 0 polling, instant event sync & instant session takeover detection
    const unsubscribe = api.subscribeState(
      (st) => {
        if (st) {
          setCurrentState(st.state);
          // Jika backend melaporkan active_session_token berbeda dari tab ini, tab lama langsung mengunci seketika!
          if (st.active_session_token && sessionToken) {
            setIsDuplicateTab(st.active_session_token !== sessionToken);
          }
        }
      },
      (connected) => {
        setOverlayConnected(connected);
      }
    );

    onCleanup(() => {
      unsubscribe();
      if (heartbeatTimer) clearInterval(heartbeatTimer);
    });
  });

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setToastVisible(true);
    setTimeout(() => {
      setToastVisible(false);
    }, 2500);
  };

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-[#161D24] text-[#EFF7F6] font-open">
      {/* Responsive Sidebar (Desktop Static + Mobile Drawer) */}
      <Sidebar 
        activeTab={activeTab()} 
        setActiveTab={setActiveTab} 
        overlayConnected={overlayConnected()} 
        currentState={currentState()}
        isMobileOpen={mobileMenuOpen()}
        onCloseMobile={() => setMobileMenuOpen(false)}
      />

      {/* Main Content Area */}
      <main class="flex-1 flex flex-col h-full overflow-hidden bg-[#1D252E]">
        {/* Top Responsive Navbar */}
        <header class="h-16 px-4 sm:px-6 md:px-8 border-b border-[#2F3B46] flex items-center justify-between shrink-0 bg-[#161D24]/90 backdrop-blur-md">
          <div class="flex items-center gap-2.5 sm:gap-3">
            {/* Hamburger Button for Mobile */}
            <button
              onClick={() => setMobileMenuOpen(true)}
              class="p-2 rounded-xl bg-[#232D36] text-[#EFF7F6] border border-[#2F3B46] hover:border-[#7BDFF2]/50 md:hidden transition-colors"
              aria-label="Buka Menu"
            >
              <MenuIcon size={20} />
            </button>

            {/* Current State Pill */}
            <span class="text-xs font-mono font-bold px-2.5 py-1 rounded-full bg-[#F2B5D4]/20 text-[#F2B5D4] border border-[#F2B5D4]/30 uppercase tracking-wide">
              STATE: {currentState()}
            </span>
            <span class="hidden sm:inline text-xs text-[#EFF7F6]/40">•</span>
            <span class="hidden sm:flex text-xs text-[#7BDFF2] font-semibold items-center gap-1.5">
              <Sparkles size={14} />
              WGPU MToon 1:1 Three-VRM
            </span>
          </div>

          <div class="flex items-center gap-2 sm:gap-3">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-[#232D36] border border-[#2F3B46] text-xs font-semibold text-[#EFF7F6]">
              <span class="w-2 h-2 rounded-full bg-[#7BDFF2] shadow-[0_0_6px_#7BDFF2] animate-pulse" />
              <span class="hidden xs:inline sm:inline">Overlay 9:16</span>
            </div>
          </div>
        </header>

        {/* Scrollable View Content for 5 Main Tabs */}
        <div class="flex-1 overflow-y-auto p-4 sm:p-6 md:p-8 max-w-5xl w-full mx-auto">
          {activeTab() === 'menu' && (
            <MenuTab 
              currentState={currentState()} 
              setCurrentState={setCurrentState} 
              showToast={showToast} 
            />
          )}
          {activeTab() === 'skills' && <SkillsTab showToast={showToast} />}
          {activeTab() === 'tuner' && <TunerTab showToast={showToast} />}
          {activeTab() === 'logs' && <LogsTab showToast={showToast} />}
          {activeTab() === 'settings' && <SettingsTab showToast={showToast} />}
        </div>
      </main>

      {/* Duplicate Tab/Browser Warning Modal (Server-Side Single Client Protection) */}
      {overlayConnected() && isDuplicateTab() && (
        <div class="fixed inset-0 z-[110] bg-[#161D24]/95 backdrop-blur-md flex items-center justify-center p-4">
          <div class="max-w-md w-full p-6 rounded-2xl bg-[#232D36] border border-[#FFD166]/40 shadow-2xl text-center space-y-4 animate-in fade-in zoom-in-95 duration-200">
            <div class="w-14 h-14 mx-auto rounded-2xl bg-[#FFD166]/20 text-[#FFD166] border border-[#FFD166]/30 flex items-center justify-center text-2xl font-bold">
              ⚡
            </div>
            <div>
              <h3 class="text-lg font-bold text-[#EFF7F6]">Dashboard Sedang Aktif di Sesi Lain</h3>
              <p class="text-xs text-[#EFF7F6]/60 mt-1">
                Dashboard Ragasukma sedang aktif di tab/browser lain. Untuk menjaga sinkronisasi real-time dan mencegah konflik data, hanya 1 sesi kontrol yang diizinkan aktif.
              </p>
            </div>
            <div class="flex items-center justify-center gap-3 pt-2">
              <button
                onClick={handleTakeover}
                class="px-5 py-2.5 rounded-xl bg-[#7BDFF2] hover:bg-[#7BDFF2]/90 text-[#161D24] text-xs font-bold transition-all shadow-lg hover:shadow-[#7BDFF2]/30"
              >
                Gunakan Tab/Browser Ini (Ambil Alih)
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Disconnected / Closed App Backdrop Modal */}
      {!overlayConnected() && (
        <div class="fixed inset-0 z-[100] bg-[#161D24]/90 backdrop-blur-md flex items-center justify-center p-4">
          <div class="max-w-md w-full p-6 rounded-2xl bg-[#232D36] border border-[#FF7B7B]/40 shadow-2xl text-center space-y-4 animate-in fade-in zoom-in-95 duration-200">
            <div class="w-14 h-14 mx-auto rounded-2xl bg-[#FF7B7B]/20 text-[#FF7B7B] border border-[#FF7B7B]/30 flex items-center justify-center text-2xl font-bold">
              ✕
            </div>
            <div>
              <h3 class="text-lg font-bold text-[#EFF7F6]">Aplikasi Ragasukma Telah Ditutup</h3>
              <p class="text-xs text-[#EFF7F6]/60 mt-1">
                Overlay avatar dan engine native telah dihentikan. Anda dapat menutup tab browser ini atau jalankan kembali aplikasi Ragasukma.
              </p>
            </div>
            <div class="flex items-center justify-center gap-3 pt-2">
              <button
                onClick={() => window.location.reload()}
                class="px-4 py-2 rounded-xl bg-[#7BDFF2]/20 hover:bg-[#7BDFF2]/30 text-[#7BDFF2] border border-[#7BDFF2]/40 text-xs font-semibold transition-all"
              >
                Coba Hubungkan Ulang
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Animated Toast Notification */}
      <div 
        class={`fixed bottom-6 right-4 sm:right-8 px-4 sm:px-5 py-3 rounded-xl bg-[#232D36] border border-[#7BDFF2] text-xs sm:text-sm font-semibold text-[#EFF7F6] shadow-2xl transition-all duration-300 pointer-events-none z-50 flex items-center gap-2.5 max-w-[90vw] ${
          toastVisible() ? 'opacity-100 translate-y-0 shadow-lg anime-shadow-green' : 'opacity-0 translate-y-4'
        }`}
      >
        <span class="w-2 h-2 rounded-full bg-[#7BDFF2] shrink-0" />
        <span class="truncate">{toastMessage()}</span>
      </div>
    </div>
  );
};

export default App;
