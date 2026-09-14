use glam::{EulerRot, Quat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssistantVisualState {
    Idle,
    Listening,
    Speaking,
    Thinking,
}

impl Default for AssistantVisualState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseKeyframes {
    #[serde(default)]
    pub emotion: Option<String>,
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

        let path = super::server::find_file_in_candidates("poses.json");
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

        if let Some(state_data) = self.config.states.get(state_key) {
            let speed = state_data.speed.max(0.1);
            // Sinusoidal t (0.0 -> 1.0) persis formula Three.js
            let t = ((self.elapsed_time * speed * 2.2).sin() + 1.0) / 2.0;

            let pose_a = &state_data.pose_a;
            let pose_b = &state_data.pose_b;

            let mut all_bones = std::collections::HashSet::new();
            for k in pose_a.keys() { all_bones.insert(k.clone()); }
            for k in pose_b.keys() { all_bones.insert(k.clone()); }

            for bone_name in &all_bones {
                let rot_a = pose_a.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);
                let rot_b = pose_b.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);

                // Ayunan nafas halus pada spine & chest
                let mut breath_offset = 0.0;
                if self.current_state == AssistantVisualState::Idle {
                    if bone_name == "chest" || bone_name == "spine" {
                        breath_offset = (self.elapsed_time * 2.4).sin() * 0.015;
                    } else if bone_name == "leftShoulder" || bone_name == "rightShoulder" {
                        breath_offset = (self.elapsed_time * 2.4).sin() * 0.01;
                    }
                }

                let qa = Quat::from_euler(EulerRot::XYZ, rot_a[0] + breath_offset, rot_a[1], rot_a[2]);
                let qb = Quat::from_euler(EulerRot::XYZ, rot_b[0] + breath_offset, rot_b[1], rot_b[2]);
                let target_q = qa.slerp(qb, t);

                self.target_quats.insert(bone_name.clone(), target_q);
            }

            // Finger natural gestures jika belum ada di keyframe
            let finger_pulse = (self.elapsed_time * 3.5).sin() * 0.05;
            let (curl_l, curl_r) = match self.current_state {
                AssistantVisualState::Speaking => (0.45 + finger_pulse, 0.45 - finger_pulse),
                AssistantVisualState::Listening => (0.35, 0.35),
                AssistantVisualState::Thinking => (0.55, 0.40),
                AssistantVisualState::Idle => (0.28, 0.28),
            };

            self.apply_fingers_curl("left", curl_l, &all_bones);
            self.apply_fingers_curl("right", curl_r, &all_bones);
        }

        // Smooth Exponential Slerp
        let lerp_speed = (1.0 - (-delta_time * 12.0).exp()).clamp(0.0, 1.0);
        for (bone, target_q) in &self.target_quats {
            let current_q = self.current_quats.entry(bone.clone()).or_insert(*target_q);
            *current_q = current_q.slerp(*target_q, lerp_speed);
        }
    }

    fn apply_fingers_curl(&mut self, side: &str, curl_amount: f32, configured: &std::collections::HashSet<String>) {
        let fingers = ["Index", "Middle", "Ring", "Little"];
        let joints = ["Proximal", "Intermediate", "Distal"];

        let thumb_meta = format!("{}ThumbMetacarpal", side);
        let thumb_prox = format!("{}ThumbProximal", side);
        let thumb_dist = format!("{}ThumbDistal", side);

        if !configured.contains(&thumb_meta) {
            self.target_quats.insert(thumb_meta, Quat::from_euler(EulerRot::XYZ, 0.12 * curl_amount, 0.12 * curl_amount, -0.18 * curl_amount));
        }
        if !configured.contains(&thumb_prox) {
            self.target_quats.insert(thumb_prox, Quat::from_euler(EulerRot::XYZ, 0.18 * curl_amount, 0.0, -0.22 * curl_amount));
        }
        if !configured.contains(&thumb_dist) {
            self.target_quats.insert(thumb_dist, Quat::from_euler(EulerRot::XYZ, 0.22 * curl_amount, 0.0, -0.25 * curl_amount));
        }

        for (f_idx, finger) in fingers.iter().enumerate() {
            let f_factor = 1.0 + (f_idx as f32 * 0.1);
            for (j_idx, joint) in joints.iter().enumerate() {
                let bone_name = format!("{}{}{}", side, finger, joint);
                if !configured.contains(&bone_name) {
                    let z_rot = -0.35 * curl_amount * f_factor * if j_idx == 0 { 1.0 } else { 1.25 };
                    let final_z = if side == "left" { z_rot } else { -z_rot };
                    self.target_quats.insert(bone_name, Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, final_z));
                }
            }
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
