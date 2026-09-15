use glam::{EulerRot, Quat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AssistantVisualState {
    #[default]
    Idle,
    Listening,
    Speaking,
    Thinking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseKeyframes {
    pub pose_a: HashMap<String, [f32; 3]>,
    pub pose_b: HashMap<String, [f32; 3]>,
    #[serde(default = "default_speed")]
    pub speed: f32,
}

fn default_speed() -> f32 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PosesConfig {
    #[serde(flatten)]
    pub states: HashMap<String, PoseKeyframes>,
}

#[derive(Debug, Clone)]
pub struct PoseManager {
    pub config: PosesConfig,
    pub current_state: AssistantVisualState,
    pub elapsed_time: f32,
    pub target_quats: HashMap<String, Quat>,
    pub current_quats: HashMap<String, Quat>,
}

impl Default for PoseManager {
    fn default() -> Self {
        let mut manager = Self {
            config: PosesConfig::default(),
            current_state: AssistantVisualState::Idle,
            elapsed_time: 0.0,
            target_quats: HashMap::new(),
            current_quats: HashMap::new(),
        };

        let path = crate::server::find_file_in_candidates("poses.json");
        if path.exists() {
            manager.load_from_file(&path);
        }

        manager
    }
}

impl PoseManager {
    pub fn load_from_file(&mut self, path: &Path) {
        if let Ok(mut file) = File::open(path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(parsed) = serde_json::from_str::<PosesConfig>(&contents) {
                    println!("🎭 [PoseManager] Berhasil memuat {} state dari {:?}", parsed.states.len(), path);
                    self.config = parsed;
                }
            }
        }
    }

    /// Update pose 1:1 persis Three.js / VRM standard
    pub fn update_poses(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;

        let state_key = match self.current_state {
            AssistantVisualState::Idle => "idle",
            AssistantVisualState::Listening => "listening",
            AssistantVisualState::Speaking => "speaking",
            AssistantVisualState::Thinking => "thinking",
        };

        let mut active_speed = 1.0f32;

        if let Some(state_data) = self.config.states.get(state_key) {
            active_speed = state_data.speed.max(0.1);
            // Sinusoidal t (0.0 -> 1.0) siklus pose A ⇄ B persis mengikuti speed dari poses.json
            let t = ((self.elapsed_time * active_speed).sin() + 1.0) / 2.0;

            let pose_a = &state_data.pose_a;
            let pose_b = &state_data.pose_b;

            let mut all_bones = std::collections::HashSet::new();
            for k in pose_a.keys() { all_bones.insert(k.clone()); }
            for k in pose_b.keys() { all_bones.insert(k.clone()); }

            // Engine Natural Breathing: Ritme nafas hidup di dada, punggung, pundak, dan mikro-gerak kepala
            let breath_rate = 1.35; // ~13 tarikan nafas per menit (ritme nafas santai)
            let breath_sine = (self.elapsed_time * breath_rate).sin();
            let breath_chest = breath_sine * 0.012;
            let breath_spine = breath_sine * 0.006;
            let breath_shoulder = breath_sine * 0.006;

            for bone_name in &all_bones {
                let rot_a = pose_a.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);
                let rot_b = pose_b.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);

                let breath_offset = if bone_name == "chest" {
                    breath_chest
                } else if bone_name == "spine" {
                    breath_spine
                } else if bone_name == "leftShoulder" || bone_name == "rightShoulder" {
                    breath_shoulder
                } else if bone_name == "head" && self.current_state == AssistantVisualState::Idle {
                    -breath_sine * 0.004
                } else {
                    0.0
                };

                let qa = Quat::from_euler(EulerRot::XYZ, rot_a[0] + breath_offset, rot_a[1], rot_a[2]);
                let qb = Quat::from_euler(EulerRot::XYZ, rot_b[0] + breath_offset, rot_b[1], rot_b[2]);
                let target_q = qa.slerp(qb, t);

                self.target_quats.insert(bone_name.clone(), target_q);
            }
        }

        // Transisi antar pose / state dinamis: Mengikuti proporsi kecepatan `speed` dari poses.json
        let transition_rate = (active_speed * 3.2).clamp(1.5, 7.0);
        let lerp_speed = (1.0 - (-delta_time * transition_rate).exp()).clamp(0.0, 1.0);
        for (bone, target_q) in &self.target_quats {
            let current_q = self.current_quats.entry(bone.clone()).or_insert(*target_q);
            *current_q = current_q.slerp(*target_q, lerp_speed);
        }
    }

    pub fn apply_tuning_pose(&mut self, bones: &HashMap<String, [f32; 3]>) {
        for (bone_name, rot) in bones {
            let q = Quat::from_euler(EulerRot::XYZ, rot[0], rot[1], rot[2]);
            self.target_quats.insert(bone_name.clone(), q);
            self.current_quats.insert(bone_name.clone(), q);
        }
    }

    pub fn get_bone_quat(&self, bone_name: &str) -> Option<Quat> {
        self.current_quats.get(bone_name).copied()
    }
}
