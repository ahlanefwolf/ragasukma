import { Component } from 'solid-js';
import { 
  Menu as MenuIcon, 
  Zap, 
  Sliders, 
  Settings as SettingsIcon,
  Bot
} from 'lucide-solid';
import { BrandLogo } from './BrandLogo';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  overlayConnected: boolean;
  currentState: string;
}

export const Sidebar: Component<SidebarProps> = (props) => {
  const menuItems = [
    { id: 'menu', label: 'Main Menu', icon: MenuIcon, badge: props.currentState.toUpperCase() },
    { id: 'skills', label: 'Skill Commands', icon: Zap },
    { id: 'tuner', label: 'Pose Tuner', icon: Sliders },
    { id: 'settings', label: 'Pengaturan', icon: SettingsIcon },
  ];

  return (
    <aside class="w-64 bg-[#1D252E] border-r border-[#2F3B46] flex flex-col justify-between p-5 select-none shrink-0">
      <div>
        {/* Brand Header */}
        <div class="flex items-center gap-3 mb-8 px-2">
          <div class="w-11 h-11 rounded-2xl overflow-hidden shadow-lg anime-shadow-green shrink-0 flex items-center justify-center">
            <BrandLogo size={44} />
          </div>
          <div>
            <h1 class="text-2xl font-bangers tracking-wider text-[#EFF7F6] leading-none">
              RAGASUKMA
            </h1>
            <p class="text-xs text-[#7BDFF2] font-semibold tracking-wide">
              NATIVE AI DESKTOP
            </p>
          </div>
        </div>

        {/* Live Overlay Status Pill */}
        <div class="mb-6 p-3 rounded-xl bg-[#232D36] border border-[#2F3B46] flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span 
              class={`w-2.5 h-2.5 rounded-full ${
                props.overlayConnected 
                  ? 'bg-[#7BDFF2] shadow-[0_0_8px_#7BDFF2] animate-pulse' 
                  : 'bg-[#F2B5D4]'
              }`} 
            />
            <span class="text-xs font-semibold text-[#EFF7F6]">
              {props.overlayConnected ? '3D Native Aktif' : 'Offline'}
            </span>
          </div>
          <span class="text-[10px] uppercase font-bold px-2 py-0.5 rounded bg-[#161D24] text-[#7BDFF2] border border-[#2F3B46]">
            WGPU 9:16
          </span>
        </div>

        {/* 4 Main Tabs Navigation */}
        <nav class="space-y-2">
          {menuItems.map((item) => {
            const Icon = item.icon;
            const isActive = () => props.activeTab === item.id;

            return (
              <button
                onClick={() => props.setActiveTab(item.id)}
                class={`w-full flex items-center justify-between px-3.5 py-3 rounded-xl text-sm font-medium transition-all duration-200 ${
                  isActive()
                    ? 'bg-[#7BDFF2] text-[#232D36] shadow-md anime-shadow-green font-semibold'
                    : 'text-[#EFF7F6]/75 hover:bg-[#232D36] hover:text-[#EFF7F6]'
                }`}
              >
                <div class="flex items-center gap-3">
                  <Icon size={19} class={isActive() ? 'stroke-[2.5]' : 'stroke-[2]'} />
                  <span class="tracking-wide">{item.label}</span>
                </div>
                {item.badge && (
                  <span class={`text-[10px] px-2 py-0.5 rounded-full font-bold font-mono uppercase ${
                    isActive() ? 'bg-[#232D36]/20 text-[#232D36]' : 'bg-[#F2B5D4]/20 text-[#F2B5D4] border border-[#F2B5D4]/30'
                  }`}>
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Footer System Info */}
      <div class="pt-4 border-t border-[#2F3B46]/60 text-xs text-[#EFF7F6]/50 px-2 space-y-1.5">
        <div class="flex items-center justify-between">
          <span>Engine Status</span>
          <span class="text-[#7BDFF2] font-semibold">Vulkan / WGPU Native</span>
        </div>
        <div class="flex items-center justify-between text-[11px]">
          <span>Dashboard Port</span>
          <span class="font-mono text-[#EFF7F6]/80">4242</span>
        </div>
      </div>
    </aside>
  );
};
