import { Component, createSignal } from 'solid-js';
import { Sidebar } from './components/Sidebar';
import { StatusTab } from './components/StatusTab';
import { VrmTab } from './components/VrmTab';
import { VoiceTab } from './components/VoiceTab';
import { PoseTab } from './components/PoseTab';
import { SettingsTab } from './components/SettingsTab';
import { Bot, Sparkles } from 'lucide-solid';

export const App: Component = () => {
  const [activeTab, setActiveTab] = createSignal('status');
  const [currentState, setCurrentState] = createSignal('idle');
  const [overlayConnected, setOverlayConnected] = createSignal(true);

  return (
    <div class="flex h-screen w-screen overflow-hidden bg-[#181820] text-[#F2F2F2]">
      {/* Left Sidebar */}
      <Sidebar 
        activeTab={activeTab()} 
        setActiveTab={setActiveTab} 
        overlayConnected={overlayConnected()} 
      />

      {/* Main Content Area */}
      <main class="flex-1 flex flex-col h-full overflow-hidden bg-[#14141A]">
        {/* Top Navbar */}
        <header class="h-16 px-8 border-b border-[#3B3B48] flex items-center justify-between shrink-0 bg-[#181820]/90 backdrop-blur-md">
          <div class="flex items-center gap-2.5">
            <span class="text-xs font-mono font-bold px-2 py-0.5 rounded bg-[#D96868]/20 text-[#D96868] border border-[#D96868]/30">
              STATE: {currentState().toUpperCase()}
            </span>
            <span class="text-xs text-[#F2F2F2]/40">•</span>
            <span class="text-xs text-[#91AE6E] font-medium flex items-center gap-1">
              <Sparkles size={13} />
              WGPU Cel-Shading MToon 1:1
            </span>
          </div>

          <div class="flex items-center gap-3">
            <div class="flex items-center gap-2 px-3 py-1.5 rounded-xl bg-[#282832] border border-[#3B3B48] text-xs font-medium">
              <span class="w-2 h-2 rounded-full bg-[#689D4B] animate-pulse" />
              <span>DirectX 12 / Vulkan Native</span>
            </div>
          </div>
        </header>

        {/* Scrollable View Content */}
        <div class="flex-1 overflow-y-auto p-8 max-w-5xl w-full mx-auto">
          {activeTab() === 'status' && (
            <StatusTab 
              currentState={currentState()} 
              setCurrentState={setCurrentState} 
            />
          )}
          {activeTab() === 'vrm' && <VrmTab />}
          {activeTab() === 'voice' && <VoiceTab />}
          {activeTab() === 'poses' && <PoseTab />}
          {activeTab() === 'settings' && <SettingsTab />}
        </div>
      </main>
    </div>
  );
};

export default App;
