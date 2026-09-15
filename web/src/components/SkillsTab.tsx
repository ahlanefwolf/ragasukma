import { Component, createSignal, onMount, For, Show } from 'solid-js';
import { 
  Play, 
  Trash2, 
  Plus, 
  Terminal,
  Edit2,
  Check,
  X,
  Sparkles,
  Info,
  Layers,
  Search,
  Globe,
  Sliders
} from 'lucide-solid';
import { api, SkillItem, SystemAppItem } from '../api';

interface SkillsTabProps {
  showToast: (msg: string) => void;
}

export const SkillsTab: Component<SkillsTabProps> = (props) => {
  const [skills, setSkills] = createSignal<SkillItem[]>([]);
  const [loading, setLoading] = createSignal(true);

  // System Apps State (Untuk preview informasi saja)
  const [showAppsInfo, setShowAppsInfo] = createSignal(false);
  const [systemApps, setSystemApps] = createSignal<SystemAppItem[]>([]);
  const [loadingApps, setLoadingApps] = createSignal(false);
  const [appSearch, setAppSearch] = createSignal('');

  // Form Tambah Skill Kustom Manual
  const [newName, setNewName] = createSignal('');
  const [newDesc, setNewDesc] = createSignal('');
  const [newCmd, setNewCmd] = createSignal('');

  // Inline Edit State
  const [editingOriginalName, setEditingOriginalName] = createSignal<string | null>(null);
  const [editName, setEditName] = createSignal('');
  const [editDesc, setEditDesc] = createSignal('');
  const [editCmd, setEditCmd] = createSignal('');

  const loadSkills = async () => {
    setLoading(true);
    const data = await api.getSkills();
    setSkills(data);
    setLoading(false);
  };

  const loadSystemApps = async () => {
    if (systemApps().length > 0) return;
    setLoadingApps(true);
    const apps = await api.getSystemApps();
    setSystemApps(apps);
    setLoadingApps(false);
  };

  onMount(() => {
    loadSkills();
  });

  const handleRunSkill = async (skill: SkillItem) => {
    props.showToast(`Menjalankan "${skill.name}"...`);
    const ok = await api.runSkill(skill.command);
    if (ok) {
      props.showToast(`Berhasil menjalankan "${skill.name}"`);
    } else {
      props.showToast(`Gagal mengeksekusi "${skill.name}"`);
    }
  };

  const handleDeleteSkill = async (name: string) => {
    if (confirm(`Hapus skill "${name}" dari registry?`)) {
      const ok = await api.deleteSkill(name);
      if (ok) {
        setSkills(skills().filter(s => s.name !== name));
        props.showToast(`Skill "${name}" berhasil dihapus`);
      } else {
        props.showToast(`Gagal menghapus skill "${name}"`);
      }
    }
  };

  const handleStartEdit = (skill: SkillItem) => {
    setEditingOriginalName(skill.name);
    setEditName(skill.name);
    setEditDesc(skill.description);
    setEditCmd(skill.command);
  };

  const handleCancelEdit = () => {
    setEditingOriginalName(null);
    setEditName('');
    setEditDesc('');
    setEditCmd('');
  };

  const handleSaveEdit = async () => {
    if (!editName().trim() || !editCmd().trim()) {
      props.showToast('Nama Skill dan Command tidak boleh kosong!');
      return;
    }

    const origName = editingOriginalName();
    const updatedSkill: SkillItem = {
      name: editName().trim(),
      description: editDesc().trim() || 'Custom user skill command',
      command: editCmd().trim()
    };

    // Update list di state
    const currentList = skills();
    const index = currentList.findIndex(s => s.name === origName);
    if (index !== -1) {
      const newList = [...currentList];
      newList[index] = updatedSkill;
      const ok = await api.saveAllSkills(newList);
      if (ok) {
        setSkills(newList);
        setEditingOriginalName(null);
        props.showToast(`Skill "${updatedSkill.name}" berhasil diperbarui!`);
      } else {
        props.showToast(`Gagal memperbarui skill "${updatedSkill.name}"`);
      }
    }
  };

  const handleAddSkill = async (e: Event) => {
    e.preventDefault();
    if (!newName().trim() || !newCmd().trim()) {
      props.showToast('Mohon isi Nama Skill dan Script Command!');
      return;
    }

    const newSkill: SkillItem = {
      name: newName().trim(),
      description: newDesc().trim() || 'Custom user automation script',
      command: newCmd().trim()
    };

    const ok = await api.saveSkill(newSkill);
    if (ok) {
      setNewName('');
      setNewDesc('');
      setNewCmd('');
      props.showToast(`Skill "${newSkill.name}" berhasil disimpan!`);
      await loadSkills();
    } else {
      props.showToast(`Gagal menyimpan skill "${newSkill.name}"`);
    }
  };

  const filteredApps = () => {
    const q = appSearch().trim().toLowerCase();
    if (!q) return systemApps();
    return systemApps().filter(app => 
      app.name.toLowerCase().includes(q) || 
      app.comment.toLowerCase().includes(q) ||
      app.exec.toLowerCase().includes(q)
    );
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#232D36] via-[#1D252E] to-[#161D24] p-4 sm:p-6 border border-[#2F3B46] flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30">
              Smart Automation
            </span>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono bg-[#161D24] text-[#7BDFF2] border border-[#2F3B46]">
              skill-command.md
            </span>
          </div>
          <h2 class="text-2xl sm:text-3xl font-bangers tracking-wide text-[#EFF7F6] mt-2">
            SKILL & AUTOMATION REGISTRY
          </h2>
          <p class="text-xs sm:text-sm text-[#EFF7F6]/70 mt-1 max-w-xl">
            Aplikasi sistem, kontrol audio, pencarian internet, dan pembukaan website sudah aktif otomatis. Di sini kamu bisa menambahkan script otomasi kustom tambahan.
          </p>
        </div>
      </div>

      {/* Built-in Features Smart Info Bar */}
      <div class="p-4 rounded-xl bg-[#1D252E]/70 border border-[#7BDFF2]/20 space-y-3">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div class="flex items-center gap-2 text-xs text-[#7BDFF2] font-semibold">
            <Sparkles size={16} class="text-[#F2B5D4]" />
            <span>Kemampuan Bawaan AI (Selalu Aktif Otomatis):</span>
          </div>
          <button
            onClick={() => {
              const next = !showAppsInfo();
              setShowAppsInfo(next);
              if (next) loadSystemApps();
            }}
            class="text-[11px] px-2.5 py-1 rounded bg-[#232D36] hover:bg-[#2F3B46] text-[#EFF7F6]/80 hover:text-[#7BDFF2] border border-[#2F3B46] transition-all flex items-center gap-1.5 self-start sm:self-auto"
          >
            <Layers size={13} />
            <span>{showAppsInfo() ? 'Tutup Daftar Aplikasi OS' : 'Lihat Aplikasi OS yang Dikenali'}</span>
          </button>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 text-xs text-[#EFF7F6]/80">
          <div class="flex items-center gap-2 p-2 rounded-lg bg-[#161D24]/60 border border-[#2F3B46]/50">
            <Sliders size={14} class="text-[#7BDFF2] shrink-0" />
            <span><strong>Sistem:</strong> Volume, Mute, Screenshot, Kunci Layar</span>
          </div>
          <div class="flex items-center gap-2 p-2 rounded-lg bg-[#161D24]/60 border border-[#2F3B46]/50">
            <Globe size={14} class="text-[#F2B5D4] shrink-0" />
            <span><strong>Web:</strong> Buka YouTube, Cari Google, dll</span>
          </div>
          <div class="flex items-center gap-2 p-2 rounded-lg bg-[#161D24]/60 border border-[#2F3B46]/50">
            <Layers size={14} class="text-[#7BDFF2] shrink-0" />
            <span><strong>Aplikasi:</strong> Cukup ucapkan <em>"Buka [Nama App]"</em></span>
          </div>
        </div>

        {/* Dropdown Viewer Aplikasi OS */}
        <Show when={showAppsInfo()}>
          <div class="mt-3 pt-3 border-t border-[#2F3B46] space-y-2">
            <div class="flex items-center justify-between gap-2">
              <span class="text-xs text-[#EFF7F6]/60">
                AI mengenali {systemApps().length} aplikasi yang terpasang di sistemmu:
              </span>
              <div class="relative w-48">
                <Search size={12} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-[#EFF7F6]/40" />
                <input
                  type="text"
                  placeholder="Cari aplikasi..."
                  value={appSearch()}
                  onInput={(e) => setAppSearch(e.currentTarget.value)}
                  class="w-full pl-7 pr-2 py-1 bg-[#161D24] border border-[#2F3B46] rounded-lg text-xs text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
                />
              </div>
            </div>

            {loadingApps() ? (
              <div class="p-4 text-center text-xs text-[#EFF7F6]/50">Memindai aplikasi sistem...</div>
            ) : (
              <div class="max-h-48 overflow-y-auto pr-1 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-1.5">
                <For each={filteredApps()}>
                  {(app) => (
                    <div class="p-2 rounded-lg bg-[#161D24] border border-[#2F3B46] text-xs">
                      <div class="font-bold text-[#EFF7F6] truncate">{app.name}</div>
                      <div class="text-[10px] text-[#EFF7F6]/50 font-mono truncate">{app.exec}</div>
                    </div>
                  )}
                </For>
              </div>
            )}
          </div>
        </Show>
      </div>

      {/* Skills List Container */}
      <div class="space-y-3">
        <h3 class="text-base sm:text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
          <Terminal size={18} />
          SKILL / OTOMASI KUSTOM ({skills().length})
        </h3>

        {loading() ? (
          <div class="p-8 text-center text-xs text-[#EFF7F6]/50">Memuat skill-command.md...</div>
        ) : skills().length === 0 ? (
          <div class="p-8 text-center text-xs text-[#EFF7F6]/50 bg-[#1D252E] rounded-2xl border border-[#2F3B46]">
            Belum ada script kustom tambahan. Perintah sistem bawaan dan website tetap bisa langsung kamu gunakan via suara.
          </div>
        ) : (
          <div class="space-y-3">
            <For each={skills()}>
              {(skill) => {
                const isEditing = () => editingOriginalName() === skill.name;

                return (
                  <div class="p-4 rounded-xl bg-[#1D252E] border border-[#2F3B46] hover:border-[#7BDFF2]/50 transition-all flex flex-col gap-3">
                    <Show
                      when={isEditing()}
                      fallback={
                        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4">
                          <div class="space-y-1 min-w-0 flex-1">
                            <div class="flex items-center gap-2">
                              <span class="font-bold text-sm text-[#EFF7F6]">{skill.name}</span>
                            </div>
                            <p class="text-xs text-[#EFF7F6]/60">{skill.description}</p>
                            <div class="flex items-center gap-1.5 pt-1 overflow-x-auto max-w-full">
                              <span class="text-xs font-mono font-bold text-[#F2B5D4] shrink-0">$</span>
                              <code class="text-[11px] sm:text-xs font-mono px-2 py-0.5 rounded bg-[#161D24] text-[#7BDFF2] border border-[#2F3B46] truncate max-w-full">
                                {skill.command}
                              </code>
                            </div>
                          </div>

                          {/* Action Buttons */}
                          <div class="flex items-center gap-2 shrink-0 self-end sm:self-auto">
                            <button
                              onClick={() => handleRunSkill(skill)}
                              class="px-3 py-1.5 rounded-lg text-xs font-bold bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] flex items-center gap-1.5 transition-all shadow-sm"
                              title="Test Eksekusi Script"
                            >
                              <Play size={13} class="fill-current" />
                              <span>Test</span>
                            </button>
                            <button
                              onClick={() => handleStartEdit(skill)}
                              class="p-1.5 rounded-lg text-xs font-semibold bg-[#232D36] hover:bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30 transition-all"
                              title="Edit Skill"
                            >
                              <Edit2 size={15} />
                            </button>
                            <button
                              onClick={() => handleDeleteSkill(skill.name)}
                              class="p-1.5 rounded-lg text-xs font-semibold bg-[#232D36] hover:bg-[#FF7B7B]/20 text-[#FF7B7B] border border-[#FF7B7B]/30 transition-all"
                              title="Hapus Skill"
                            >
                              <Trash2 size={15} />
                            </button>
                          </div>
                        </div>
                      }
                    >
                      {/* Inline Edit Form */}
                      <div class="space-y-3 p-2 bg-[#161D24] rounded-lg border border-[#7BDFF2]/40">
                        <div class="text-xs font-bold text-[#7BDFF2] flex items-center gap-1.5">
                          <Edit2 size={14} />
                          <span>Edit Skill: {skill.name}</span>
                        </div>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                          <div>
                            <label class="text-[11px] text-[#EFF7F6]/70">Nama Skill</label>
                            <input
                              type="text"
                              value={editName()}
                              onInput={(e) => setEditName(e.currentTarget.value)}
                              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-lg px-2.5 py-1.5 text-xs text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
                            />
                          </div>
                          <div>
                            <label class="text-[11px] text-[#EFF7F6]/70">Deskripsi untuk AI</label>
                            <input
                              type="text"
                              value={editDesc()}
                              onInput={(e) => setEditDesc(e.currentTarget.value)}
                              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-lg px-2.5 py-1.5 text-xs text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
                            />
                          </div>
                        </div>
                        <div>
                          <label class="text-[11px] text-[#EFF7F6]/70">Command Script</label>
                          <input
                            type="text"
                            value={editCmd()}
                            onInput={(e) => setEditCmd(e.currentTarget.value)}
                            class="w-full bg-[#232D36] border border-[#2F3B46] rounded-lg px-2.5 py-1.5 text-xs font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2]"
                          />
                        </div>
                        <div class="flex justify-end gap-2 pt-1">
                          <button
                            onClick={handleCancelEdit}
                            class="px-3 py-1 rounded-lg text-xs bg-[#232D36] text-[#EFF7F6]/70 hover:text-[#EFF7F6] flex items-center gap-1"
                          >
                            <X size={13} />
                            <span>Batal</span>
                          </button>
                          <button
                            onClick={handleSaveEdit}
                            class="px-3 py-1 rounded-lg text-xs font-bold bg-[#7BDFF2] text-[#232D36] hover:bg-[#68cbdd] flex items-center gap-1"
                          >
                            <Check size={13} />
                            <span>Simpan Perubahan</span>
                          </button>
                        </div>
                      </div>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>
        )}
      </div>

      {/* Add New Custom Automation Form */}
      <form onSubmit={handleAddSkill} class="p-4 sm:p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-base sm:text-lg font-bangers text-[#F2B5D4] flex items-center gap-2">
          <Plus size={18} />
          TAMBAH SCRIPT OTOMASI KUSTOM
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Nama Perintah / Skill</label>
            <input
              type="text"
              value={newName()}
              onInput={(e) => setNewName(e.currentTarget.value)}
              placeholder="Contoh: Backup File Proyek"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-all"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Deskripsi untuk AI</label>
            <input
              type="text"
              value={newDesc()}
              onInput={(e) => setNewDesc(e.currentTarget.value)}
              placeholder="Contoh: Menjalankan script sinkronisasi file penting"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-all"
            />
          </div>
        </div>

        <div class="space-y-1">
          <label class="text-xs font-semibold text-[#EFF7F6]/80">Script Shell / Command Terminal</label>
          <input
            type="text"
            value={newCmd()}
            onInput={(e) => setNewCmd(e.currentTarget.value)}
            placeholder="Contoh: rsync -av ~/Documents /mnt/backup/ atau ./script.sh"
            class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-all"
          />
        </div>

        <div class="flex justify-end pt-1">
          <button
            type="submit"
            class="w-full sm:w-auto px-5 py-2.5 rounded-xl font-bold text-sm bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] shadow-md anime-shadow-green flex items-center justify-center gap-2 transition-all font-semibold"
          >
            <Plus size={16} />
            <span>Simpan Script Baru</span>
          </button>
        </div>
      </form>
    </div>
  );
};
