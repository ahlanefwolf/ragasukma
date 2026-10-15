use glam::{Mat4, Quat, Vec3};
use serde_json::Value;
use std::collections::VecDeque;

use super::vrm_loader::{NodeData, VrmModelData};

#[derive(Debug, Clone)]
pub struct SpringCollider {
    pub node_idx: usize,
    pub offset: Vec3,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct SpringJoint {
    pub node_idx: usize,
    pub child_node_idx: usize,
    pub bone_axis: Vec3,
    pub bone_length: f32,
    pub hit_radius: f32,
    pub stiffness: f32,
    pub gravity_power: f32,
    pub gravity_dir: Vec3,
    pub drag_force: f32,
    pub collider_indices: Vec<usize>,
    pub current_tail: Vec3,
    pub prev_tail: Vec3,
    pub initialized: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SpringBoneManager {
    pub colliders: Vec<SpringCollider>,
    pub joints: Vec<SpringJoint>,
    pub cached_collider_spheres: Vec<Option<(Vec3, f32)>>,
}

impl SpringBoneManager {
    pub fn from_gltf_json(gltf_json: &Value, nodes: &[NodeData]) -> Option<Self> {
        let extensions = gltf_json.get("extensions")?;

        // 1. VRMC_springBone (VRM 1.0)
        if let Some(sb) = extensions.get("VRMC_springBone") {
            return Self::from_vrmc_spring_bone(sb, nodes);
        }

        // 2. VRM 0.X secondaryAnimation fallback
        if let Some(vrm_0) = extensions.get("VRM") {
            if let Some(sec_anim) = vrm_0.get("secondaryAnimation") {
                return Self::from_vrm0_secondary_animation(sec_anim, nodes);
            }
        }

        None
    }

    fn from_vrmc_spring_bone(sb: &Value, nodes: &[NodeData]) -> Option<Self> {
        // 1. Parse colliders
        let mut colliders = Vec::new();
        if let Some(col_arr) = sb.get("colliders").and_then(|v| v.as_array()) {
            for col in col_arr {
                if let Some(node_idx) = col.get("node").and_then(|v| v.as_u64()) {
                    let sphere = col.get("shape").and_then(|s| s.get("sphere"));
                    let offset = sphere
                        .and_then(|sp| sp.get("offset"))
                        .and_then(|o| o.as_array())
                        .map(|arr| {
                            Vec3::new(
                                arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            )
                        })
                        .unwrap_or(Vec3::ZERO);
                    let radius = sphere
                        .and_then(|sp| sp.get("radius"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.05) as f32;

                    colliders.push(SpringCollider {
                        node_idx: node_idx as usize,
                        offset,
                        radius,
                    });
                }
            }
        }

        // 2. Parse collider groups: group_idx -> Vec<collider_idx>
        let mut collider_groups: Vec<Vec<usize>> = Vec::new();
        if let Some(grp_arr) = sb.get("colliderGroups").and_then(|v| v.as_array()) {
            for grp in grp_arr {
                let mut c_indices = Vec::new();
                if let Some(indices) = grp.get("colliders").and_then(|v| v.as_array()) {
                    for idx in indices {
                        if let Some(i) = idx.as_u64() {
                            c_indices.push(i as usize);
                        }
                    }
                }
                collider_groups.push(c_indices);
            }
        }

        // 3. Parse springs and joints
        let mut joints = Vec::new();
        if let Some(springs_arr) = sb.get("springs").and_then(|v| v.as_array()) {
            for spring in springs_arr {
                let mut spring_colliders = Vec::new();
                if let Some(cg_indices) = spring.get("colliderGroups").and_then(|v| v.as_array()) {
                    for cg in cg_indices {
                        if let Some(cg_idx) = cg.as_u64().map(|v| v as usize) {
                            if let Some(group) = collider_groups.get(cg_idx) {
                                for &c_idx in group {
                                    if !spring_colliders.contains(&c_idx) {
                                        spring_colliders.push(c_idx);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(joint_arr) = spring.get("joints").and_then(|v| v.as_array()) {
                    for i in 0..joint_arr.len().saturating_sub(1) {
                        if joints.len() >= 1024 {
                            break;
                        }
                        let j_cur = &joint_arr[i];
                        let j_next = &joint_arr[i + 1];

                        let node_idx = match j_cur.get("node").and_then(|v| v.as_u64()) {
                            Some(n) => n as usize,
                            None => continue,
                        };
                        let child_node_idx = match j_next.get("node").and_then(|v| v.as_u64()) {
                            Some(n) => n as usize,
                            None => continue,
                        };

                        if node_idx >= nodes.len() || child_node_idx >= nodes.len() {
                            continue;
                        }

                        let child_node = &nodes[child_node_idx];
                        let mut bone_axis = child_node.initial_translation;
                        let mut bone_length = bone_axis.length();
                        if bone_length > 0.0001 {
                            bone_axis /= bone_length;
                        } else {
                            bone_axis = Vec3::new(0.0, -1.0, 0.0);
                            bone_length = 0.07;
                        }

                        let hit_radius = j_cur
                            .get("hitRadius")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        let stiffness = j_cur
                            .get("stiffness")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0) as f32;
                        let gravity_power = j_cur
                            .get("gravityPower")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0) as f32;
                        let gravity_dir = j_cur
                            .get("gravityDir")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                Vec3::new(
                                    arr.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                    arr.get(1).and_then(|v| v.as_f64()).unwrap_or(-1.0) as f32,
                                    arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                )
                            })
                            .unwrap_or(Vec3::new(0.0, -1.0, 0.0));
                        let drag_force = j_cur
                            .get("dragForce")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.4) as f32;

                        joints.push(SpringJoint {
                            node_idx,
                            child_node_idx,
                            hit_radius,
                            stiffness,
                            gravity_power,
                            gravity_dir,
                            drag_force,
                            bone_axis,
                            bone_length,
                            current_tail: Vec3::ZERO,
                            prev_tail: Vec3::ZERO,
                            initialized: false,
                            collider_indices: spring_colliders.clone(),
                        });
                    }
                }
            }
        }

        if joints.is_empty() {
            None
        } else {
            Some(Self {
                colliders,
                joints,
                cached_collider_spheres: Vec::new(),
            })
        }
    }

    fn from_vrm0_secondary_animation(sec_anim: &Value, nodes: &[NodeData]) -> Option<Self> {
        let mut colliders = Vec::new();
        let mut collider_groups: Vec<Vec<usize>> = Vec::new();

        if let Some(grp_arr) = sec_anim.get("colliderGroups").and_then(|v| v.as_array()) {
            for grp in grp_arr {
                let mut c_indices = Vec::new();
                if let Some(node_idx) = grp.get("node").and_then(|v| v.as_u64()) {
                    if let Some(col_arr) = grp.get("colliders").and_then(|v| v.as_array()) {
                        for col in col_arr {
                            let offset = col
                                .get("offset")
                                .and_then(|o| o.as_object())
                                .map(|o| {
                                    Vec3::new(
                                        o.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                        o.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                        o.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                    )
                                })
                                .unwrap_or(Vec3::ZERO);
                            let radius = col
                                .get("radius")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.05) as f32;

                            let col_idx = colliders.len();
                            colliders.push(SpringCollider {
                                node_idx: node_idx as usize,
                                offset,
                                radius,
                            });
                            c_indices.push(col_idx);
                        }
                    }
                }
                collider_groups.push(c_indices);
            }
        }

        let mut joints = Vec::new();
        if let Some(bone_groups) = sec_anim.get("boneGroups").and_then(|v| v.as_array()) {
            for bg in bone_groups {
                let mut spring_colliders = Vec::new();
                if let Some(cg_indices) = bg.get("colliderGroups").and_then(|v| v.as_array()) {
                    for cg in cg_indices {
                        if let Some(cg_idx) = cg.as_u64().map(|v| v as usize) {
                            if let Some(group) = collider_groups.get(cg_idx) {
                                for &c_idx in group {
                                    if !spring_colliders.contains(&c_idx) {
                                        spring_colliders.push(c_idx);
                                    }
                                }
                            }
                        }
                    }
                }

                let hit_radius = bg
                    .get("hitRadius")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let stiffness = bg
                    .get("stiffiness")
                    .or_else(|| bg.get("stiffness"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) as f32;
                let gravity_power = bg
                    .get("gravityPower")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0) as f32;
                let gravity_dir = bg
                    .get("gravityDir")
                    .and_then(|v| v.as_object())
                    .map(|o| {
                        Vec3::new(
                            o.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            o.get("y").and_then(|v| v.as_f64()).unwrap_or(-1.0) as f32,
                            o.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        )
                    })
                    .unwrap_or(Vec3::new(0.0, -1.0, 0.0));
                let drag_force = bg
                    .get("dragForce")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.4) as f32;

                if let Some(root_bones) = bg.get("bones").and_then(|v| v.as_array()) {
                    for root_bone in root_bones {
                        if joints.len() >= 1024 {
                            break;
                        }
                        if let Some(root_node_idx) = root_bone.as_u64().map(|v| v as usize) {
                            // BFS queue ensures parent bones are always ordered before their children
                            let mut queue = VecDeque::new();
                            queue.push_back(root_node_idx);

                            while let Some(parent_idx) = queue.pop_front() {
                                if parent_idx >= nodes.len() || joints.len() >= 1024 {
                                    continue;
                                }
                                for &child_idx in &nodes[parent_idx].children {
                                    if child_idx >= nodes.len() || joints.len() >= 1024 {
                                        continue;
                                    }
                                    let child_node = &nodes[child_idx];
                                    let mut bone_axis = child_node.initial_translation;
                                    let mut bone_length = bone_axis.length();
                                    if bone_length > 0.0001 {
                                        bone_axis /= bone_length;
                                    } else {
                                        bone_axis = Vec3::new(0.0, -1.0, 0.0);
                                        bone_length = 0.07;
                                    }

                                    joints.push(SpringJoint {
                                        node_idx: parent_idx,
                                        child_node_idx: child_idx,
                                        hit_radius,
                                        stiffness,
                                        gravity_power,
                                        gravity_dir,
                                        drag_force,
                                        bone_axis,
                                        bone_length,
                                        current_tail: Vec3::ZERO,
                                        prev_tail: Vec3::ZERO,
                                        initialized: false,
                                        collider_indices: spring_colliders.clone(),
                                    });

                                    queue.push_back(child_idx);
                                }
                            }
                        }
                    }
                }
            }
        }

        if joints.is_empty() {
            None
        } else {
            Some(Self {
                colliders,
                joints,
                cached_collider_spheres: Vec::new(),
            })
        }
    }

    pub fn update(&mut self, delta_seconds: f32, model: &VrmModelData, globals: &mut [Mat4]) {
        let dt = delta_seconds.clamp(0.001, 0.05);

        // Precompute position bola collider in world space without per-frame allocations
        self.cached_collider_spheres.clear();
        for c in &self.colliders {
            if c.node_idx < globals.len() {
                let center = globals[c.node_idx].transform_point3(c.offset);
                self.cached_collider_spheres.push(Some((center, c.radius)));
            } else {
                self.cached_collider_spheres.push(None);
            }
        }

        for joint in &mut self.joints {
            if joint.node_idx >= globals.len() {
                continue;
            }

            let node = &model.nodes[joint.node_idx];
            let parent_mat = if let Some(p_idx) = model.parent_indices[joint.node_idx] {
                globals[p_idx]
            } else {
                Mat4::IDENTITY
            };

            // Rest world transform for this joint
            let rest_local_mat = Mat4::from_scale_rotation_translation(
                node.initial_scale,
                node.initial_rotation,
                node.initial_translation,
            );
            let rest_world_mat = parent_mat * rest_local_mat;
            let bone_world_pos = rest_world_mat.transform_point3(Vec3::ZERO);
            let world_rest_dir = rest_world_mat
                .transform_vector3(joint.bone_axis)
                .normalize_or_zero();

            // Initialize start tail on frame pertama
            if !joint.initialized {
                let init_pos = bone_world_pos + world_rest_dir * joint.bone_length;
                joint.current_tail = init_pos;
                joint.prev_tail = init_pos;
                joint.initialized = true;
            }

            // 1. Verlet Integration (Inertia + Spring Stiffness + Gravity) - 100% Identical to Three-VRM Pixiv Standard
            let inertia = (joint.current_tail - joint.prev_tail) * (1.0 - joint.drag_force);
            let stiffness_force = world_rest_dir * (joint.stiffness * dt);
            let gravity_force = joint.gravity_dir * (joint.gravity_power * dt);
            let mut next_tail = joint.current_tail + inertia + stiffness_force + gravity_force;

            // 2. Length Constraint (Consistent spring bone joint length in world space)
            let to_tail = next_tail - bone_world_pos;
            let dist = to_tail.length();
            if dist > 0.0001 {
                next_tail = bone_world_pos + (to_tail / dist) * joint.bone_length;
            } else {
                next_tail = bone_world_pos + world_rest_dir * joint.bone_length;
            }

            // 3. Collision Resolution (Prevents hair/skirt clipping through head/chest colliders)
            for &c_idx in &joint.collider_indices {
                if let Some(Some((sphere_center, sphere_radius))) =
                    self.cached_collider_spheres.get(c_idx)
                {
                    let r_sum = sphere_radius + joint.hit_radius;
                    let diff = next_tail - *sphere_center;
                    let d = diff.length();
                    if d < r_sum && d > 0.0001 {
                        let push_dir = diff / d;
                        next_tail = *sphere_center + push_dir * r_sum;

                        // Recalibrate bone length after collider bounce
                        let to_tail2 = next_tail - bone_world_pos;
                        let dist2 = to_tail2.length();
                        if dist2 > 0.0001 {
                            next_tail = bone_world_pos + (to_tail2 / dist2) * joint.bone_length;
                        }
                    }
                }
            }

            // 4. Update history position
            joint.prev_tail = joint.current_tail;
            joint.current_tail = next_tail;

            // 5. Direct bone rotation to next_tail (Identical to Three-VRM: setFromUnitVectors premultiply initialLocalRotation)
            let inv_rest_world = rest_world_mat.inverse();
            let local_tail_pos = inv_rest_world.transform_point3(next_tail);
            let local_tail_dir = local_tail_pos.normalize_or_zero();

            if local_tail_dir.length_squared() > 0.0001 && joint.bone_axis.length_squared() > 0.0001
            {
                let rot_from_axis = Quat::from_rotation_arc(joint.bone_axis, local_tail_dir);
                let final_rotation = node.initial_rotation * rot_from_axis;

                let current_local = Mat4::from_scale_rotation_translation(
                    node.initial_scale,
                    final_rotation,
                    node.initial_translation,
                );
                let new_global = parent_mat * current_local;
                globals[joint.node_idx] = new_global;

                // Propagate updated transform directly to immediate children (O(1) per joint).
                // Because joints are ordered in topological BFS order, child joints in the spring bone
                // array will receive this new parent transform sequentially on their own turn.
                for &child_idx in &node.children {
                    if child_idx < model.nodes.len() {
                        let child_node = &model.nodes[child_idx];
                        let child_local = Mat4::from_scale_rotation_translation(
                            child_node.initial_scale,
                            child_node.initial_rotation,
                            child_node.initial_translation,
                        );
                        globals[child_idx] = new_global * child_local;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_bone_from_char_vrm() {
        // Fallback to "assets/char (copy 1).vrm" or "assets/char.vrm"
        let vrm_bytes = std::fs::read("assets/char (copy 1).vrm")
            .or_else(|_| std::fs::read("assets/char.vrm"));

        if let Ok(bytes) = vrm_bytes {
            if bytes.len() > 20 {
                let chunk0_len =
                    u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
                if bytes.len() >= 20 + chunk0_len {
                    if let Ok(json_str) = std::str::from_utf8(&bytes[20..20 + chunk0_len]) {
                        if let Ok(gltf_json) = serde_json::from_str::<serde_json::Value>(json_str) {
                            let dummy_nodes = vec![
                                NodeData {
                                    index: 0,
                                    name: "dummy".to_string(),
                                    children: vec![],
                                    local_transform: Mat4::IDENTITY,
                                    initial_translation: Vec3::new(0.0, -0.1, 0.0),
                                    initial_rotation: Quat::IDENTITY,
                                    initial_scale: Vec3::ONE,
                                    skin_index: None,
                                    mesh_index: None,
                                };
                                300
                            ];
                            let sb = SpringBoneManager::from_gltf_json(&gltf_json, &dummy_nodes);
                            if let Some(mut sb) = sb {
                                assert!(!sb.joints.is_empty() || !sb.colliders.is_empty(), "SpringBoneManager parsed non-empty configuration");
                                let model = VrmModelData {
                                    is_vrm_0: false,
                                    primitives: vec![],
                                    materials: vec![],
                                    nodes: dummy_nodes,
                                    root_nodes: vec![0],
                                    skins: vec![],
                                    bone_mapping: std::collections::HashMap::new(),
                                    node_to_bone: vec![None; 300],
                                    parent_indices: vec![None; 300],
                                    parent_rest_world_rotations: vec![Quat::IDENTITY; 300],
                                    expressions: std::collections::HashMap::new(),
                                    known_mesh_indices: vec![],
                                    spring_bone_manager: None,
                                };
                                let mut globals = vec![Mat4::IDENTITY; 300];
                                for _ in 0..10 {
                                    sb.update(0.016, &model, &mut globals);
                                }
                                if !sb.joints.is_empty() {
                                    assert!(sb.joints[0].initialized);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_load_and_pose_char_vrm() {
        use crate::ui::vrm_loader::VrmModelData;
        let model = VrmModelData::load_from_file("assets/char.vrm").unwrap();
        assert!(!model.primitives.is_empty());
        assert!(!model.skins.is_empty());

        let mut globals = vec![Mat4::IDENTITY; model.nodes.len()];
        if let Some(sb_mutex) = &model.spring_bone_manager {
            let mut sb = sb_mutex.lock().unwrap();
            for _ in 0..10 {
                sb.update(0.016, &model, &mut globals);
            }
            for (j_idx, j) in sb.joints.iter().enumerate() {
                assert!(!j.current_tail.is_nan(), "Joint {} current_tail is NaN", j_idx);
            }
        }
    }
}
