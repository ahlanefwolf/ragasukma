import { Component } from 'solid-js';
import { 
  Sparkles, 
  User, 
  Mic, 
  Activity, 
  Settings, 
  ExternalLink,
  Bot
} from 'lucide-solid';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  overlayConnected: boolean;
}

export const Sidebar: Component<SidebarProps> = (props) => {
  const menuItems = [
    { id: 'status', label: 'Avatar Status', icon: Sparkles, badge: 'Live' },
    { id: 'vrm', label: 'Model VRM', icon: User },
    { id: 'voice', label: 'Suara & AI', icon: Mic },
    { id: 'poses', label: 'Pose & Gestur', icon: Activity },
    { id: 'settings', label: 'Pengaturan', icon: Settings },
  ];

  return (
    <aside class="w-64 bg-[#1E1E24] border-r border-[#3B3B48] flex flex-col justify-between p-5 select-none shrink-0">
      <div>
        {/* Brand Header */}
        <div class="flex items-center gap-3 mb-8 px-2">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-[#D96868] to-[#689D4B] flex items-center justify-center text-white shadow-lg anime-shadow-red">
            <Bot size={22} class="stroke-[2.5]" />
          </div>
          <div>
            <h1 class="text-2xl font-bangers tracking-wider text-[#F2F2F2] leading-none">
              RAGASUKMA
            </h1>
            <p class="text-xs text-[#91AE6E] font-medium tracking-wide">
              3D AI AVATAR ENGINE
            </p>
          </div>
        </div>

        {/* Live Overlay Status Pill */}
        <div class="mb-6 p-3 rounded-xl bg-[#282832] border border-[#3B3B48] flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span 
              class={`w-2.5 h-2.5 rounded-full ${
                props.overlayConnected 
                  ? 'bg-[#689D4B] shadow-[0_0_8px_#689D4B] animate-pulse' 
                  : 'bg-[#D96868]'
              }`} 
            />
            <span class="text-xs font-semibold text-[#F2F2F2]">
              {props.overlayConnected ? '3D Native Aktif' : 'Menghubungkan...'}
            </span>
          </div>
          <span class="text-[10px] uppercase font-bold px-2 py-0.5 rounded bg-[#181820] text-[#91AE6E] border border-[#3B3B48]">
            WGPU 9:16
          </span>
        </div>

        {/* Navigation Links */}
        <nav class="space-y-1.5">
          {menuItems.map((item) => {
            const Icon = item.icon;
            const isActive = () => props.activeTab === item.id;

            return (
              <button
                onClick={() => props.setActiveTab(item.id)}
                class={`w-full flex items-center justify-between px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-200 ${
                  isActive()
                    ? 'bg-[#D96868] text-white shadow-md anime-shadow-red font-semibold'
                    : 'text-[#F2F2F2]/70 hover:bg-[#282832] hover:text-[#F2F2F2]'
                }`}
              >
                <div class="flex items-center gap-3">
                  <Icon size={18} class={isActive() ? 'stroke-[2.5]' : 'stroke-[1.75]'} />
                  <span>{item.label}</span>
                </div>
                {item.badge && (
                  <span class={`text-[10px] px-1.5 py-0.5 rounded font-bold uppercase ${
                    isActive() ? 'bg-white/20 text-white' : 'bg-[#689D4B]/20 text-[#91AE6E]'
                  }`}>
                    {item.badge}
                  </span>
                )}
              </button>
            );
          })}
        </nav>
      </div>

      {/* Footer Info */}
      <div class="pt-4 border-t border-[#3B3B48]/60 text-xs text-[#F2F2F2]/50 px-2 space-y-1">
        <div class="flex items-center justify-between">
          <span>Engine Status</span>
          <span class="text-[#91AE6E] font-medium">Ultra-Light</span>
        </div>
        <div class="flex items-center justify-between text-[11px]">
          <span>Port Server</span>
          <span class="font-mono text-[#F2F2F2]/80">4242</span>
        </div>
      </div>
    </aside>
  );
};
