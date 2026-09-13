import { Component, createSignal } from 'solid-js';
import { 
  Zap, 
  Play, 
  Trash2, 
  Plus, 
  Terminal, 
  FileCode,
  Check
} from 'lucide-solid';

interface SkillItem {
  id: string;
  name: string;
  description: string;
  command: string;
}

interface SkillsTabProps {
  showToast: (msg: string) => void;
}

export const SkillsTab: Component<SkillsTabProps> = (props) => {
  const [skills, setSkills] = createSignal<SkillItem[]>([
    {
      id: '1',
      name: 'Buka Browser Web',
      description: 'Membuka Google Chrome / browser default sistem',
      command: 'start https://google.com'
    },
    {
      id: '2',
      name: 'Buka VSCode / Zed',
      description: 'Membuka editor coding di direktori project',
      command: 'zed . || code .'
    },
    {
      id: '3',
      name: 'Buka Spotify Music',
      description: 'Meluncurkan aplikasi pemutar musik Spotify',
      command: 'start spotify:'
    },
    {
      id: '4',
      name: 'Buka File Explorer',
      description: 'Membuka folder project Ragasukma',
      command: 'explorer .'
    }
  ]);

  const [newName, setNewName] = createSignal('');
  const [newDesc, setNewDesc] = createSignal('');
  const [newCmd, setNewCmd] = createSignal('');

  const handleRunSkill = (skill: SkillItem) => {
    props.showToast(`Perintah "${skill.name}" dijalankan: ${skill.command}`);
  };

  const handleDeleteSkill = (id: string, name: string) => {
    setSkills(skills().filter(s => s.id !== id));
    props.showToast(`Skill "${name}" berhasil dihapus`);
  };

  const handleAddSkill = (e: Event) => {
    e.preventDefault();
    if (!newName().trim() || !newCmd().trim()) {
      props.showToast('Mohon isi Nama Skill dan Script Command!');
      return;
    }

    const newSkill: SkillItem = {
      id: Date.now().toString(),
      name: newName().trim(),
      description: newDesc().trim() || 'Custom user skill command',
      command: newCmd().trim()
    };

    setSkills([...skills(), newSkill]);
    setNewName('');
    setNewDesc('');
    setNewCmd('');
    props.showToast(`Skill "${newSkill.name}" berhasil ditambahkan!`);
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48] flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2">
            <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#91AE6E]/20 text-[#91AE6E] border border-[#91AE6E]/30">
              Automation & Execution
            </span>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono bg-[#181820] text-[#91AE6E] border border-[#3B3B48]">
              skill-command.md
            </span>
          </div>
          <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
            SKILL COMMANDS REGISTRY
          </h2>
          <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-xl">
            Daftarkan script command terminal atau aplikasi yang bisa dipanggil secara otomatis oleh AI Assistant saat kamu berbicara.
          </p>
        </div>
      </div>

      {/* Skills List Container */}
      <div class="space-y-3">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Terminal size={18} />
          DAFTAR SKILL TERDAFTAR ({skills().length})
        </h3>

        <div class="space-y-3">
          {skills().map((skill) => (
            <div class="p-4 rounded-xl bg-[#1E1E24] border border-[#3B3B48] hover:border-[#91AE6E]/50 transition-all flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <div class="space-y-1 min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-bold text-sm text-[#F2F2F2]">{skill.name}</span>
                </div>
                <p class="text-xs text-[#F2F2F2]/60">{skill.description}</p>
                <div class="flex items-center gap-1.5 pt-1">
                  <span class="text-xs font-mono font-bold text-[#D96868]">$</span>
                  <code class="text-xs font-mono px-2 py-0.5 rounded bg-[#181820] text-[#91AE6E] border border-[#3B3B48] truncate max-w-md">
                    {skill.command}
                  </code>
                </div>
              </div>

              {/* Action Buttons */}
              <div class="flex items-center gap-2 shrink-0">
                <button
                  onClick={() => handleRunSkill(skill)}
                  class="px-3.5 py-1.5 rounded-lg text-xs font-bold bg-[#689D4B] hover:bg-[#5b8c40] text-white flex items-center gap-1.5 transition-all shadow-sm anime-shadow-green"
                >
                  <Play size={13} class="fill-current" />
                  <span>Run</span>
                </button>
                <button
                  onClick={() => handleDeleteSkill(skill.id, skill.name)}
                  class="p-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#D96868]/20 text-[#D96868] border border-[#3B3B48] transition-all"
                  title="Hapus Skill"
                >
                  <Trash2 size={15} />
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Add New Skill Form */}
      <form onSubmit={handleAddSkill} class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#D96868] flex items-center gap-2">
          <Plus size={18} />
          TAMBAH SKILL COMMAND BARU
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Nama Skill</label>
            <input
              type="text"
              value={newName()}
              onInput={(e) => setNewName(e.currentTarget.value)}
              placeholder="Contoh: Buka Kalkulator"
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2 text-sm text-[#F2F2F2] outline-none focus:border-[#D96868] transition-all"
            />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Deskripsi untuk AI</label>
            <input
              type="text"
              value={newDesc()}
              onInput={(e) => setNewDesc(e.currentTarget.value)}
              placeholder="Contoh: Membuka aplikasi kalkulator sistem"
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2 text-sm text-[#F2F2F2] outline-none focus:border-[#D96868] transition-all"
            />
          </div>
        </div>

        <div class="space-y-1">
          <label class="text-xs font-semibold text-[#F2F2F2]/80">Script Shell / Command Prompt</label>
          <input
            type="text"
            value={newCmd()}
            onInput={(e) => setNewCmd(e.currentTarget.value)}
            placeholder="Contoh: calc.exe"
            class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3.5 py-2 text-sm font-mono text-[#F2F2F2] outline-none focus:border-[#D96868] transition-all"
          />
        </div>

        <div class="flex justify-end pt-1">
          <button
            type="submit"
            class="px-5 py-2.5 rounded-xl font-bold text-sm bg-[#D96868] hover:bg-[#c45a5a] text-white shadow-md anime-shadow-red flex items-center gap-2 transition-all"
          >
            <Plus size={16} />
            <span>Simpan Skill Baru</span>
          </button>
        </div>
      </form>
    </div>
  );
};
