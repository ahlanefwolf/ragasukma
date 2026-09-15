import { Component, createSignal, onMount } from 'solid-js';
import { 
  Play, 
  Trash2, 
  Plus, 
  Terminal 
} from 'lucide-solid';
import { api, SkillItem } from '../api';

interface SkillsTabProps {
  showToast: (msg: string) => void;
}

export const SkillsTab: Component<SkillsTabProps> = (props) => {
  const [skills, setSkills] = createSignal<SkillItem[]>([]);
  const [newName, setNewName] = createSignal('');
  const [newDesc, setNewDesc] = createSignal('');
  const [newCmd, setNewCmd] = createSignal('');
  const [loading, setLoading] = createSignal(true);

  const loadSkills = async () => {
    setLoading(true);
    const data = await api.getSkills();
    setSkills(data);
    setLoading(false);
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

  const handleAddSkill = async (e: Event) => {
    e.preventDefault();
    if (!newName().trim() || !newCmd().trim()) {
      props.showToast('Mohon isi Nama Skill dan Script Command!');
      return;
    }

    const newSkill: SkillItem = {
      name: newName().trim(),
      description: newDesc().trim() || 'Custom user skill command',
      command: newCmd().trim()
    };

    const ok = await api.saveSkill(newSkill);
    if (ok) {
      setNewName('');
      setNewDesc('');
      setNewCmd('');
      props.showToast(`Skill "${newSkill.name}" berhasil disimpan ke skill-command.md!`);
      await loadSkills();
    } else {
      props.showToast(`Gagal menyimpan skill "${newSkill.name}"`);
    }
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#232D36] via-[#1D252E] to-[#161D24] p-4 sm:p-6 border border-[#2F3B46] flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2">
            <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30">
              Automation & Execution
            </span>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono bg-[#161D24] text-[#7BDFF2] border border-[#2F3B46]">
              skill-command.md
            </span>
          </div>
          <h2 class="text-2xl sm:text-3xl font-bangers tracking-wide text-[#EFF7F6] mt-2">
            SKILL COMMANDS REGISTRY
          </h2>
          <p class="text-xs sm:text-sm text-[#EFF7F6]/70 mt-1 max-w-xl">
            Daftarkan script command terminal atau aplikasi yang bisa dipanggil secara otomatis oleh AI Assistant saat kamu berbicara.
          </p>
        </div>
      </div>

      {/* Skills List Container */}
      <div class="space-y-3">
        <h3 class="text-base sm:text-lg font-bangers text-[#7BDFF2] flex items-center gap-2">
          <Terminal size={18} />
          DAFTAR SKILL TERDAFTAR ({skills().length})
        </h3>

        {loading() ? (
          <div class="p-8 text-center text-xs text-[#EFF7F6]/50">Memuat skill-command.md...</div>
        ) : skills().length === 0 ? (
          <div class="p-8 text-center text-xs text-[#EFF7F6]/50 bg-[#1D252E] rounded-2xl border border-[#2F3B46]">
            Belum ada skill yang terdaftar di skill-command.md. Tambahkan lewat formulir di bawah.
          </div>
        ) : (
          <div class="space-y-3">
            {skills().map((skill) => (
              <div class="p-4 rounded-xl bg-[#1D252E] border border-[#2F3B46] hover:border-[#7BDFF2]/50 transition-all flex flex-col sm:flex-row sm:items-center justify-between gap-3 sm:gap-4">
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
                    class="px-3.5 py-1.5 rounded-lg text-xs font-bold bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] flex items-center gap-1.5 transition-all shadow-sm anime-shadow-green"
                  >
                    <Play size={13} class="fill-current" />
                    <span>Run</span>
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
            ))}
          </div>
        )}
      </div>

      {/* Add New Skill Form */}
      <form onSubmit={handleAddSkill} class="p-4 sm:p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        <h3 class="text-base sm:text-lg font-bangers text-[#F2B5D4] flex items-center gap-2">
          <Plus size={18} />
          TAMBAH SKILL COMMAND BARU
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Nama Skill</label>
            <input
              type="text"
              value={newName()}
              onInput={(e) => setNewName(e.currentTarget.value)}
              placeholder="Contoh: Buka Kalkulator"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-all"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#EFF7F6]/80">Deskripsi untuk AI</label>
            <input
              type="text"
              value={newDesc()}
              onInput={(e) => setNewDesc(e.currentTarget.value)}
              placeholder="Contoh: Membuka aplikasi kalkulator sistem"
              class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2 text-sm text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-all"
            />
          </div>
        </div>

        <div class="space-y-1">
          <label class="text-xs font-semibold text-[#EFF7F6]/80">Script Shell / Command Prompt</label>
          <input
            type="text"
            value={newCmd()}
            onInput={(e) => setNewCmd(e.currentTarget.value)}
            placeholder="Contoh: xdg-open https://google.com (Linux) atau calc.exe (Windows)"
            class="w-full bg-[#232D36] border border-[#2F3B46] rounded-xl px-3.5 py-2 text-sm font-mono text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-all"
          />
        </div>

        <div class="flex justify-end pt-1">
          <button
            type="submit"
            class="w-full sm:w-auto px-5 py-2.5 rounded-xl font-bold text-sm bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] shadow-md anime-shadow-green flex items-center justify-center gap-2 transition-all font-semibold"
          >
            <Plus size={16} />
            <span>Simpan Skill Baru</span>
          </button>
        </div>
      </form>
    </div>
  );
};
