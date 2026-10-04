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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SinglePose {
    #[serde(default)]
    pub bones: HashMap<String, [f32; 3]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseKeyframes {
    #[serde(default = "default_speed")]
    pub speed: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub poses: Vec<SinglePose>,
}

impl PoseKeyframes {
    pub fn resolved_poses(&self) -> Vec<HashMap<String, [f32; 3]>> {
        if !self.poses.is_empty() {
            self.poses.iter().map(|p| p.bones.clone()).collect()
        } else {
            vec![HashMap::new()]
        }
    }
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
    pub last_state: AssistantVisualState,
    pub state_transition_t: f32, // 0.0 -> 1.0 (smooth interpolation on state change)
    pub state_from_quats: HashMap<String, Quat>, // Snapshot joint rotations immediately before state change
    pub elapsed_time: f32,
    pub state_elapsed_time: f32, // Current state active elapsed time (reset to 0 on state change, identical to Three.js)
    pub target_quats: HashMap<String, Quat>,
    pub current_quats: HashMap<String, Quat>,
}

impl Default for PoseManager {
    fn default() -> Self {
        let mut manager = Self {
            config: PosesConfig::default(),
            current_state: AssistantVisualState::Idle,
            last_state: AssistantVisualState::Idle,
            state_transition_t: 1.0,
            state_from_quats: HashMap::new(),
            elapsed_time: 0.0,
            state_elapsed_time: 0.0,
            target_quats: HashMap::new(),
            current_quats: HashMap::new(),
        };

        let path = crate::assets::find_file_in_candidates("poses.json");
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
                    println!(
                        "[PoseManager] Successfully loaded {} states from {:?}",
                        parsed.states.len(),
                        path
                    );
                    self.config = parsed;
                }
            }
        }
    }

    /// Update pose 1:1 persis Three.js / VRM standard with multi-pose dynamic cycle & natural breathing
    pub fn update_poses(&mut self, delta_time: f32) {
        self.elapsed_time += delta_time;
        self.state_elapsed_time += delta_time;

        // Check state transition changes (e.g. Idle -> Listening -> Thinking -> Speaking)
        if self.current_state != self.last_state {
            // Snapshot joint rotations as starting point for smooth slerp transition
            self.state_from_quats = self.current_quats.clone();
            self.state_transition_t = 0.0;
            self.state_elapsed_time = 0.0;
            self.last_state = self.current_state;
        }

        // Run timer interpolasi transisi antar state (~0.45 detik smooth slerp)
        let state_transition_duration = 0.45f32;
        if self.state_transition_t < 1.0 {
            self.state_transition_t =
                (self.state_transition_t + delta_time / state_transition_duration).min(1.0);
        }

        let state_key = match self.current_state {
            AssistantVisualState::Idle => "idle",
            AssistantVisualState::Listening => "listening",
            AssistantVisualState::Speaking => "speaking",
            AssistantVisualState::Thinking => "thinking",
        };

        if let Some(state_data) = self.config.states.get(state_key) {
            let active_speed = state_data.speed.max(0.1);
            let poses = state_data.resolved_poses();
            let count = poses.len();

            // Engine Natural Breathing: Ritme nafas hidup in chest, punggung, pundak, and mikro-gerak head
            let breath_rate = 1.35; // ~13 tarikan nafas per menit (ritme nafas santai)
            let breath_sine = (self.state_elapsed_time * breath_rate).sin();
            let breath_chest = breath_sine * 0.012;
            let breath_spine = breath_sine * 0.006;
            let breath_shoulder = breath_sine * 0.006;

            if count == 0 {
                // No registered poses
            } else if count == 1 {
                // Only 1 pose: diam in pose tersebut with efek nafas alami
                let single = &poses[0];
                for (bone_name, rot) in single {
                    let breath_offset = if bone_name == "chest" {
                        breath_chest
                    } else if bone_name == "spine" {
                        breath_spine
                    } else if bone_name == "leftShoulder" || bone_name == "rightShoulder" {
                        breath_shoulder
                    } else if bone_name == "head"
                        && self.current_state == AssistantVisualState::Idle
                    {
                        -breath_sine * 0.004
                    } else {
                        0.0
                    };

                    let q = Quat::from_euler(EulerRot::XYZ, rot[0] + breath_offset, rot[1], rot[2]);
                    self.target_quats.insert(bone_name.clone(), q);
                }
            } else {
                // Multi-pose (2 pose or more): Catmull-Rom Spline interpolation (C1 continuous)
                // Ensures continuous motion speed across keyframes without abrupt pauses (ease-in/out stall).
                let cycle_step = 2.5 / active_speed;
                let total_cycle_time = count as f32 * cycle_step;
                let loop_time = self.state_elapsed_time % total_cycle_time;
                let cur_idx = (loop_time / cycle_step).floor() as usize % count;
                let next_idx = (cur_idx + 1) % count;
                let prev_idx = if cur_idx == 0 { count - 1 } else { cur_idx - 1 };
                let next2_idx = (cur_idx + 2) % count;
                let t = (loop_time % cycle_step) / cycle_step;

                let p0 = &poses[prev_idx];
                let p1 = &poses[cur_idx];
                let p2 = &poses[next_idx];
                let p3 = &poses[next2_idx];

                let mut all_bones = std::collections::HashSet::new();
                for k in p1.keys() {
                    all_bones.insert(k.clone());
                }
                for k in p2.keys() {
                    all_bones.insert(k.clone());
                }

                // Catmull-Rom spline evaluator: q(t) = 0.5 * ((2*P1) + (-P0 + P2)*t + (2*P0 - 5*P1 + 4*P2 - P3)*t^2 + (-P0 + 3*P1 - 3*P2 + P3)*t^3)
                let catmull_rom = |v0: f32, v1: f32, v2: f32, v3: f32, t: f32| -> f32 {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    0.5 * ((2.0 * v1)
                        + (-v0 + v2) * t
                        + (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2
                        + (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3)
                };

                for bone_name in &all_bones {
                    let r0 = p0.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);
                    let mut r1 = p1.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);
                    let mut r2 = p2.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);
                    let mut r3 = p3.get(bone_name).copied().unwrap_or([0.0, 0.0, 0.0]);

                    // Handle hip yaw angular phase unwrapping so rotation does not reverse on wrap-around
                    if bone_name == "hips" {
                        while r1[1] < r0[1] - std::f32::consts::PI {
                            r1[1] += std::f32::consts::TAU;
                        }
                        while r2[1] < r1[1] - std::f32::consts::PI {
                            r2[1] += std::f32::consts::TAU;
                        }
                        while r3[1] < r2[1] - std::f32::consts::PI {
                            r3[1] += std::f32::consts::TAU;
                        }
                    }

                    let rot_interp = [
                        catmull_rom(r0[0], r1[0], r2[0], r3[0], t),
                        catmull_rom(r0[1], r1[1], r2[1], r3[1], t),
                        catmull_rom(r0[2], r1[2], r2[2], r3[2], t),
                    ];

                    let breath_offset = if bone_name == "chest" {
                        breath_chest
                    } else if bone_name == "spine" {
                        breath_spine
                    } else if bone_name == "leftShoulder" || bone_name == "rightShoulder" {
                        breath_shoulder
                    } else if bone_name == "head"
                        && self.current_state == AssistantVisualState::Idle
                    {
                        -breath_sine * 0.004
                    } else {
                        0.0
                    };

                    let target_q = Quat::from_euler(
                        EulerRot::XYZ,
                        rot_interp[0] + breath_offset,
                        rot_interp[1],
                        rot_interp[2],
                    );

                    self.target_quats.insert(bone_name.clone(), target_q);
                }
            }
        }

        // Apply pose to joints: If transitioning between states, perform S-Curve Slerp from previous pose to new pose
        let blend_t = if self.state_transition_t < 1.0 {
            (1.0 - (self.state_transition_t * std::f32::consts::PI).cos()) / 2.0
        } else {
            1.0
        };

        for (bone, target_q) in &self.target_quats {
            let current_q = self.current_quats.entry(bone.clone()).or_insert(*target_q);
            if blend_t < 1.0 {
                if let Some(from_q) = self.state_from_quats.get(bone) {
                    *current_q = from_q.slerp(*target_q, blend_t);
                } else {
                    *current_q = *target_q;
                }
            } else {
                *current_q = *target_q;
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
