import { Component, createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { 
  Play, 
  Trash2, 
  Plus, 
  Terminal,
  Edit2,
  Check,
  X,
  Sparkles,
  Layers,
  Search,
  Globe,
  Sliders,
  Loader,
} from 'lucide-solid';
import { api, SkillItem, InstalledApp } from '../api';

interface SkillsTabProps {
  showToast: (msg: string) => void;
}

// Module-level persistent cache agar perpindahan antar tab instan 0ms tanpa re-loading
const [cachedSkills, setCachedSkills] = createSignal<SkillItem[]>([]);
const [cachedSystemApps, setCachedSystemApps] = createSignal<InstalledApp[]>([]);
let isFirstSkillsMount = true;

export const SkillsTab: Component<SkillsTabProps> = (props) => {
  const [skills, setSkills] = createSignal<SkillItem[]>(cachedSkills());
  const [loading, setLoading] = createSignal(cachedSkills().length === 0 && isFirstSkillsMount);

  // System Apps State (Untuk preview informasi saja)
  const [showAppsInfo, setShowAppsInfo] = createSignal(false);
  const [systemApps, setSystemApps] = createSignal<InstalledApp[]>(cachedSystemApps());
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

  const loadSkills = async (silent = false) => {
    if (!silent) setLoading(true);
    try {
      const data = await api.getSkills();
      const list = Array.isArray(data) ? data : [];
      setSkills(list);
      setCachedSkills(list);
    } catch {
      if (cachedSkills().length === 0) setSkills([]);
    } finally {
      setLoading(false);
    }
  };

  const loadSystemApps = async () => {
    if (systemApps().length > 0) return;
    setLoadingApps(true);
    try {
      const apps = await api.getSystemApps();
      const list = Array.isArray(apps) ? apps : [];
      setSystemApps(list);
      setCachedSystemApps(list);
    } catch {
      setSystemApps([]);
    } finally {
      setLoadingApps(false);
    }
  };

  let isComponentMounted = true;

  onMount(() => {
    isComponentMounted = true;
    if (isFirstSkillsMount) {
      isFirstSkillsMount = false;
      loadSkills(false);
    } else {
      // Re-load silently in background
      loadSkills(true);
    }
  });

  onCleanup(() => {
    isComponentMounted = false;
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

    // Update skill via saveSkill (jika rename, hapus skill lama dan simpan yang baru)
    if (origName && origName !== updatedSkill.name) {
      await api.deleteSkill(origName);
    }
    const ok = await api.saveSkill(updatedSkill);
    if (ok) {
      setEditingOriginalName(null);
      props.showToast(`Skill "${updatedSkill.name}" berhasil diperbarui!`);
      await loadSkills();
    } else {
      props.showToast(`Gagal memperbarui skill "${updatedSkill.name}"`);
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
      (app.comment && app.comment.toLowerCase().includes(q)) ||
      app.exec.toLowerCase().includes(q)
    );
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-5 sm:p-6 border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <span class="px-2.5 py-1 text-xs font-bold uppercase tracking-wider bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
              Smart Automation
            </span>
            <span class="px-2 py-0.5 text-xs font-mono bg-theme-bg text-theme-cyan border border-theme-border">
              skill-command.md
            </span>
          </div>
          <h2 class="text-2xl sm:text-3xl font-heading tracking-wide text-theme-text mt-2">
            SKILL & AUTOMATION REGISTRY
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
            Aplikasi sistem, kontrol audio, pencarian internet, dan pembukaan website sudah aktif otomatis. Di sini kamu bisa menambahkan script otomasi kustom tambahan.
          </p>
        </div>
      </div>

      {/* Browser AI Companion Extension Setup Banner */}
      <div class="p-5 bg-gradient-to-r from-theme-card/90 via-theme-surface to-theme-bg border border-theme-cyan/40 shadow-lg space-y-4">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div class="flex items-start gap-3">
            <div class="p-2.5 bg-theme-cyan/10 border border-theme-cyan/30 text-theme-cyan shrink-0">
              <Globe size={22} class="animate-pulse" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h3 class="text-sm sm:text-base font-bold text-theme-text font-heading tracking-wide">
                  EKSTENSI BROWSER AI (CHROME / BRAVE / EDGE / FIREFOX)
                </h3>
                <span class="px-2 py-0.5 text-xs font-bold uppercase bg-theme-rose/20 text-theme-rose border border-theme-rose/30">
                  DOM Learning & Snapshot
                </span>
              </div>
              <p class="text-xs text-theme-text/70 mt-1">
                Memungkinkan AI mengenali elemen halaman (YouTube, pencarian, tombol, form) dan menyimpannya ke <code class="text-theme-cyan font-mono">assets/active_tab_browser.json</code> secara realtime.
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2 shrink-0">
            <button
              onClick={async () => {
                props.showToast("Membuka folder ekstensi di File Manager...");
                await api.runSkill("internal:open_extension_installer");
              }}
              class="px-4 py-2.5 bg-theme-cyan hover:bg-theme-cyan/90 text-theme-bg font-bold text-xs flex items-center gap-2 shadow-md transition-all cursor-pointer border border-theme-cyan"
            >
              <Sparkles size={14} />
              <span>Buka Folder Ekstensi</span>
            </button>
          </div>
        </div>

        {/* 2 Kolom Pemasangan Cepat Browser */}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5 pt-1">
          {/* Box 1: Chrome / Brave / Edge */}
          <div class="p-4 bg-theme-bg border border-theme-cyan/30 hover:border-theme-cyan/60 transition-all flex flex-col justify-between gap-3">
            <div>
              <div class="flex items-center justify-between">
                <span class="px-2.5 py-0.5 text-xs font-bold uppercase bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
                  Chrome / Brave / Edge
                </span>
                <span class="text-xs text-theme-text/60 font-mono">chrome://extensions</span>
              </div>
              <div class="text-xs text-theme-text/80 mt-2 leading-relaxed">
                1. Klik tombol <strong>Buka chrome://extensions</strong> di bawah.<br />
                2. Aktifkan toggle <strong>Developer mode</strong> (kanan atas).<br />
                3. Klik <strong>Load unpacked</strong> $\rightarrow$ paste folder yang disalin.
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <button
                onClick={() => {
                  navigator.clipboard.writeText("chrome://extensions");
                  window.open("chrome://extensions", "_blank");
                  props.showToast("🌐 URL chrome://extensions disalin! Jika tab tidak terbuka otomatis, tempel di tab baru.");
                }}
                class="py-2.5 px-3 bg-theme-cyan/20 hover:bg-theme-cyan/30 text-theme-cyan border border-theme-cyan/40 font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer"
              >
                <Globe size={14} />
                <span>Buka Extensions</span>
              </button>

              <button
                onClick={() => {
                  navigator.clipboard.writeText("/home/headway/MyWork/project/node/ai-voice/extension");
                  props.showToast("📁 Path folder ekstensi disalin!");
                }}
                class="py-2.5 px-3 bg-theme-cyan hover:bg-theme-cyan/90 text-theme-bg font-bold text-xs flex items-center justify-center gap-1.5 shadow-sm transition-all cursor-pointer border border-theme-cyan"
              >
                <Check size={14} />
                <span>Salin Path Folder</span>
              </button>
            </div>
          </div>

          {/* Box 2: Firefox */}
          <div class="p-4 bg-theme-bg border border-theme-rose/30 hover:border-theme-rose/60 transition-all flex flex-col justify-between gap-3">
            <div>
              <div class="flex items-center justify-between">
                <span class="px-2.5 py-0.5 text-xs font-bold uppercase bg-theme-rose/20 text-theme-rose border border-theme-rose/30">
                  Firefox
                </span>
                <span class="text-xs text-theme-text/60 font-mono">about:debugging</span>
              </div>
              <div class="text-xs text-theme-text/80 mt-2 leading-relaxed">
                1. Klik tombol <strong>Buka about:debugging</strong> di bawah.<br />
                2. Pilih <strong>This Firefox</strong> $\rightarrow$ klik <strong>Load Temporary Add-on</strong>.<br />
                3. Pilih file <code>manifest.json</code> yang disalin.
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
              <button
                onClick={() => {
                  navigator.clipboard.writeText("about:debugging#/runtime/this-firefox");
                  window.open("about:debugging#/runtime/this-firefox", "_blank");
                  props.showToast("🦊 URL about:debugging disalin! Jika tab tidak terbuka otomatis, tempel di tab baru.");
                }}
                class="py-2.5 px-3 bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/40 font-bold text-xs flex items-center justify-center gap-1.5 transition-all cursor-pointer"
              >
                <Globe size={14} />
                <span>Buka Debugging</span>
              </button>

              <button
                onClick={() => {
                  navigator.clipboard.writeText("/home/headway/MyWork/project/node/ai-voice/extension/manifest.json");
                  props.showToast("📁 Path manifest.json Firefox disalin!");
                }}
                class="py-2.5 px-3 bg-theme-rose hover:bg-theme-rose/90 text-white font-bold text-xs flex items-center justify-center gap-1.5 shadow-sm transition-all cursor-pointer border border-theme-rose"
              >
                <Check size={14} />
                <span>Salin Path Manifest</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Built-in Features Smart Info Bar */}
      <div class="p-4 bg-theme-surface border border-theme-cyan/25 space-y-3">
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2">
          <div class="flex items-center gap-2 text-xs text-theme-cyan font-bold tracking-wide">
            <Sparkles size={16} class="text-theme-rose" />
            <span>Kemampuan Bawaan AI (Selalu Aktif Otomatis):</span>
          </div>
          <div class="flex items-center gap-2 self-start sm:self-auto">
            <button
              onClick={() => {
                const next = !showAppsInfo();
                setShowAppsInfo(next);
                if (next) loadSystemApps();
              }}
              class="text-xs px-3 py-1 bg-theme-card hover:bg-theme-border text-theme-text/80 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5 font-semibold"
            >
              <Layers size={13} />
              <span>{showAppsInfo() ? 'Tutup Aplikasi OS' : 'Aplikasi OS'}</span>
            </button>
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-2.5 text-xs text-theme-text/80">
          <div class="flex items-center gap-2 p-2.5 bg-theme-bg border border-theme-border/60">
            <Sliders size={15} class="text-theme-cyan shrink-0" />
            <span><strong>Kontrol OS:</strong> Volume, Mute, Screenshot, Kunci Layar</span>
          </div>
          <div class="flex items-center gap-2 p-2.5 bg-theme-bg border border-theme-border/60">
            <Globe size={15} class="text-theme-rose shrink-0" />
            <span><strong>Web & Tab:</strong> Multi-Tab, Klik Judul/Tombol, Ketik & Scroll</span>
          </div>
          <div class="flex items-center gap-2 p-2.5 bg-theme-bg border border-theme-border/60">
            <Layers size={15} class="text-theme-cyan shrink-0" />
            <span><strong>Aplikasi & Musik:</strong> Buka App & Putar Musik Otomatis</span>
          </div>
        </div>

        {/* Dropdown Viewer Aplikasi OS */}
        <Show when={showAppsInfo()}>
          <div class="mt-3 pt-3 border-t border-theme-border space-y-2">
            <div class="flex items-center justify-between gap-2">
              <span class="text-xs text-theme-text/70">
                AI mengenali {systemApps().length} aplikasi yang terpasang di sistemmu:
              </span>
              <div class="relative w-52">
                <Search size={13} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-theme-text/40" />
                <input
                  type="text"
                  placeholder="Cari aplikasi..."
                  value={appSearch()}
                  onInput={(e) => setAppSearch(e.currentTarget.value)}
                  class="w-full pl-8 pr-2.5 py-1 bg-theme-bg border border-theme-border text-xs text-theme-text outline-none focus:border-theme-cyan"
                />
              </div>
            </div>

            {loadingApps() ? (
              <div class="p-4 text-center text-xs text-theme-text/50">Memindai aplikasi sistem...</div>
            ) : (
              <div class="max-h-48 overflow-y-auto pr-1 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                <For each={filteredApps()}>
                  {(app) => (
                    <div class="p-2.5 bg-theme-bg border border-theme-border text-xs">
                      <div class="font-bold text-theme-text truncate">{app.name}</div>
                      <div class="text-xs text-theme-text/50 font-mono truncate mt-0.5">{app.exec}</div>
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
        <h3 class="text-base sm:text-lg font-heading text-theme-cyan flex items-center gap-2 tracking-wide">
          <Terminal size={18} />
          SKILL / OTOMASI KUSTOM ({skills().length})
        </h3>

        {loading() ? (
          <div class="p-8 flex items-center justify-center gap-2 text-xs text-theme-text/50">
            <Loader size={16} class="animate-spin text-theme-cyan" />
            <span>Memuat skill-command.md...</span>
          </div>
        ) : skills().length === 0 ? (
          <div class="p-8 text-center text-xs text-theme-text/60 bg-theme-surface border border-theme-border">
            Belum ada script kustom tambahan. Perintah sistem bawaan dan website tetap bisa langsung kamu gunakan via suara.
          </div>
        ) : (
          <div class="space-y-3">
            <For each={skills()}>
              {(skill) => {
                const isEditing = () => editingOriginalName() === skill.name;

                return (
                  <div class="p-4 bg-theme-surface border border-theme-border hover:border-theme-cyan/50 transition-all flex flex-col gap-3">
                    <Show
                      when={isEditing()}
                      fallback={
                        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4">
                          <div class="space-y-1.5 min-w-0 flex-1">
                            <div class="flex items-center gap-2">
                              <span class="font-bold text-sm text-theme-text">{skill.name}</span>
                            </div>
                            <p class="text-xs text-theme-text/70">{skill.description}</p>
                            <div class="flex items-center gap-2 pt-1 overflow-x-auto max-w-full">
                              <span class="text-xs font-mono font-bold text-theme-rose shrink-0">$</span>
                              <code class="text-xs font-mono px-2 py-0.5 bg-theme-bg text-theme-cyan border border-theme-border truncate max-w-full">
                                {skill.command}
                              </code>
                            </div>
                          </div>

                          {/* Action Buttons */}
                          <div class="flex items-center gap-2 shrink-0 self-end sm:self-auto">
                            <button
                              onClick={() => handleRunSkill(skill)}
                              class="px-3.5 py-1.5 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card flex items-center gap-1.5 transition-all shadow-sm border border-theme-cyan"
                              title="Test Eksekusi Script"
                            >
                              <Play size={13} class="fill-current" />
                              <span>Test</span>
                            </button>
                            <button
                              onClick={() => handleStartEdit(skill)}
                              class="p-2 text-xs font-semibold bg-theme-card hover:bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 transition-all"
                              title="Edit Skill"
                            >
                              <Edit2 size={15} />
                            </button>
                            <button
                              onClick={() => handleDeleteSkill(skill.name)}
                              class="p-2 text-xs font-semibold bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/30 transition-all"
                              title="Hapus Skill"
                            >
                              <Trash2 size={15} />
                            </button>
                          </div>
                        </div>
                      }
                    >
                      {/* Inline Edit Form */}
                      <div class="space-y-3 p-3 bg-theme-bg border border-theme-cyan/40">
                        <div class="text-xs font-bold text-theme-cyan flex items-center gap-1.5">
                          <Edit2 size={14} />
                          <span>Edit Skill: {skill.name}</span>
                        </div>
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5">
                          <div>
                            <label class="text-xs text-theme-text/70 font-semibold">Nama Skill</label>
                            <input
                              type="text"
                              value={editName()}
                              onInput={(e) => setEditName(e.currentTarget.value)}
                              class="w-full bg-theme-card border border-theme-border px-3 py-1.5 text-xs text-theme-text outline-none focus:border-theme-cyan mt-1"
                            />
                          </div>
                          <div>
                            <label class="text-xs text-theme-text/70 font-semibold">Deskripsi untuk AI</label>
                            <input
                              type="text"
                              value={editDesc()}
                              onInput={(e) => setEditDesc(e.currentTarget.value)}
                              class="w-full bg-theme-card border border-theme-border px-3 py-1.5 text-xs text-theme-text outline-none focus:border-theme-cyan mt-1"
                            />
                          </div>
                        </div>
                        <div>
                          <label class="text-xs text-theme-text/70 font-semibold">Command Script</label>
                          <input
                            type="text"
                            value={editCmd()}
                            onInput={(e) => setEditCmd(e.currentTarget.value)}
                            class="w-full bg-theme-card border border-theme-border px-3 py-1.5 text-xs font-mono text-theme-text outline-none focus:border-theme-cyan mt-1"
                          />
                        </div>
                        <div class="flex justify-end gap-2 pt-1">
                          <button
                            onClick={handleCancelEdit}
                            class="px-3.5 py-1.5 text-xs bg-theme-card text-theme-text/70 hover:text-theme-text flex items-center gap-1 border border-theme-border"
                          >
                            <X size={13} />
                            <span>Batal</span>
                          </button>
                          <button
                            onClick={handleSaveEdit}
                            class="px-4 py-1.5 text-xs font-bold bg-theme-cyan text-theme-card hover:bg-theme-cyan/90 flex items-center gap-1 border border-theme-cyan"
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
      <form onSubmit={handleAddSkill} class="p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-heading text-theme-rose flex items-center gap-2 tracking-wide">
          <Plus size={18} />
          TAMBAH SCRIPT OTOMASI KUSTOM
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
          <div class="space-y-1">
            <label class="text-xs font-semibold text-theme-text/80">Nama Perintah / Skill</label>
            <input
              type="text"
              value={newName()}
              onInput={(e) => setNewName(e.currentTarget.value)}
              placeholder="Contoh: Backup File Proyek"
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2 text-sm text-theme-text outline-none focus:border-theme-cyan transition-all"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-semibold text-theme-text/80">Deskripsi untuk AI</label>
            <input
              type="text"
              value={newDesc()}
              onInput={(e) => setNewDesc(e.currentTarget.value)}
              placeholder="Contoh: Menjalankan script sinkronisasi file penting"
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2 text-sm text-theme-text outline-none focus:border-theme-cyan transition-all"
            />
          </div>
        </div>

        <div class="space-y-1">
          <label class="text-xs font-semibold text-theme-text/80">Script Shell / Command Terminal</label>
          <input
            type="text"
            value={newCmd()}
            onInput={(e) => setNewCmd(e.currentTarget.value)}
            placeholder="Contoh: rsync -av ~/Documents /mnt/backup/ atau ./script.sh"
            class="w-full bg-theme-card border border-theme-border px-3.5 py-2 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-all"
          />
        </div>

        <div class="flex justify-end pt-1">
          <button
            type="submit"
            class="w-full sm:w-auto px-5 py-2.5 text-sm bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card shadow-md anime-shadow-green flex items-center justify-center gap-2 transition-all font-semibold border border-theme-cyan"
          >
            <Plus size={16} />
            <span>Simpan Script Baru</span>
          </button>
        </div>
      </form>
    </div>
  );
};
