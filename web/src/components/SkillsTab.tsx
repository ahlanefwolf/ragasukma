import { Component, createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import { 
  Play, 
  Trash2, 
  Plus, 
  Terminal,
  Edit2,
  Check,
  X,
  Zap,
  Loader,
} from 'lucide-solid';
import { api, SkillItem } from '../api';

interface SkillsTabProps {
  showToast: (msg: string) => void;
}

// Module-level persistent cache agar perpindahan antar tab instan 0ms tanpa re-loading
const [cachedSkills, setCachedSkills] = createSignal<SkillItem[]>([]);
let isFirstSkillsMount = true;

export const SkillsTab: Component<SkillsTabProps> = (props) => {
  const [skills, setSkills] = createSignal<SkillItem[]>(cachedSkills());
  const [loading, setLoading] = createSignal(cachedSkills().length === 0 && isFirstSkillsMount);

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
          <h2 class="text-2xl sm:text-3xl font-heading tracking-tight text-theme-text mt-2">
            Skill Custom & Otomasi
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
            Kelola perintah script otomasi kustom, integrasi API, dan alur kerja mandiri buatanmu sendiri di sini.
          </p>
        </div>
      </div>

      {/* Skills List Container */}
      <div class="space-y-3">
        <h3 class="text-base sm:text-lg font-bold text-theme-cyan flex items-center gap-2 tracking-wide">
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
        <h3 class="text-base sm:text-lg font-bold text-theme-rose flex items-center gap-2 tracking-wide">
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
