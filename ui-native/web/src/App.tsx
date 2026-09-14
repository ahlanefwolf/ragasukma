import { Component, createSignal } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { MenuTab } from './components/MenuTab';
import { SkillsTab } from './components/SkillsTab';
import { TunerTab } from './components/TunerTab';
import { SettingsTab } from './components/SettingsTab';
import { Sparkles, Bot } from 'lucide-solid';

export const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal('menu');
  const [currentState, setCurrentState] = createSignal('idle');
  const [overlayConnected, setOverlayConnected] = createSignal(true);
  const [toastMessage, setToastMessage] = createSignal('');
  const [toastVisible, setToastVisible] = createSignal(false);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setToastVisible(true);
    setTimeout(() => {
      setToastVisible(false);
    }, 2500);
  };

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-[#161D24] text-[#EFF7F6] font-open">
      {/* Left Sidebar */}
      <Sidebar 
        activeTab={activeTab()} 
        setActiveTab={setActiveTab} 
        overlayConnected={overlayConnected()} 
        currentState={currentState()}
      />

      {/* Main Content Area */}
      <main class="flex-1 flex flex-col h-full overflow-hidden bg-[#1D252E]">
        {/* Top Navbar */}
        <header class="h-16 px-8 border-b border-[#2F3B46] flex items-center justify-between shrink-0 bg-[#161D24]/90 backdrop-blur-md">
          <div class="flex items-center gap-3">
            <span class="text-xs font-mono font-bold px-2.5 py-1 rounded-full bg-[#F2B5D4]/20 text-[#F2B5D4] border border-[#F2B5D4]/30 uppercase">
              STATE: {currentState()}
            </span>
            <span class="text-xs text-[#EFF7F6]/40">•</span>
            <span class="text-xs text-[#7BDFF2] font-semibold flex items-center gap-1.5">
              <Sparkles size={14} />
              WGPU MToon 1:1 Three-VRM
            </span>
          </div>

          <div class="flex items-center gap-3">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-[#232D36] border border-[#2F3B46] text-xs font-semibold text-[#EFF7F6]">
              <span class="w-2 h-2 rounded-full bg-[#7BDFF2] shadow-[0_0_6px_#7BDFF2] animate-pulse" />
              <span>Overlay Desktop 9:16</span>
            </div>
          </div>
        </header>

        {/* Scrollable View Content for 4 Main Tabs */}
        <div class="flex-1 overflow-y-auto p-8 max-w-5xl w-full mx-auto">
          {activeTab() === 'menu' && (
            <MenuTab 
              currentState={currentState()} 
              setCurrentState={setCurrentState} 
              showToast={showToast} 
            />
          )}
          {activeTab() === 'skills' && <SkillsTab showToast={showToast} />}
          {activeTab() === 'tuner' && <TunerTab showToast={showToast} />}
          {activeTab() === 'settings' && <SettingsTab showToast={showToast} />}
        </div>
      </main>

      {/* Animated Toast Notification */}
      <div 
        class={`fixed bottom-6 right-8 px-5 py-3 rounded-xl bg-[#232D36] border border-[#7BDFF2] text-sm font-semibold text-[#EFF7F6] shadow-2xl transition-all duration-300 pointer-events-none z-50 flex items-center gap-2.5 ${
          toastVisible() ? 'opacity-100 translate-y-0 shadow-lg anime-shadow-green' : 'opacity-0 translate-y-4'
        }`}
      >
        <span class="w-2 h-2 rounded-full bg-[#7BDFF2]" />
        <span>{toastMessage()}</span>
      </div>
    </div>
  );
};

export default App;
