use glam::{Mat4, Quat, Vec3};
use serde_json::Value;

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
}

impl SpringBoneManager {
    pub fn from_gltf_json(gltf_json: &Value, nodes: &[NodeData]) -> Option<Self> {
        let sb = gltf_json.get("extensions")?.get("VRMC_springBone")?;

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
                // Kumpulkan seluruh collider yang relevan untuk rantai pegas ini
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
                            bone_axis,
                            bone_length,
                            hit_radius,
                            stiffness,
                            gravity_power,
                            gravity_dir,
                            drag_force,
                            collider_indices: spring_colliders.clone(),
                            current_tail: Vec3::ZERO,
                            prev_tail: Vec3::ZERO,
                            initialized: false,
                        });
                    }
                }
            }
        }

        if joints.is_empty() {
            None
        } else {
            println!(
                "🍃 [SpringBone] Diinisialisasi: {} joints pegas dinamis, {} colliders tabrakan",
                joints.len(),
                colliders.len()
            );
            Some(Self { colliders, joints })
        }
    }

    pub fn update(
        &mut self,
        delta_seconds: f32,
        model: &VrmModelData,
        globals: &mut [Mat4],
    ) {
        let dt = delta_seconds.clamp(0.001, 0.05);

        // Precompute posisi bola collider di world space
        let mut collider_spheres: Vec<Option<(Vec3, f32)>> = Vec::with_capacity(self.colliders.len());
        for c in &self.colliders {
            if c.node_idx < globals.len() {
                let center = globals[c.node_idx].transform_point3(c.offset);
                collider_spheres.push(Some((center, c.radius)));
            } else {
                collider_spheres.push(None);
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

            // Rest world transform untuk joint ini
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

            // Inisialisasi awal tail pada frame pertama
            if !joint.initialized {
                let init_pos = bone_world_pos + world_rest_dir * joint.bone_length;
                joint.current_tail = init_pos;
                joint.prev_tail = init_pos;
                joint.initialized = true;
            }

            // 1. Verlet Integration (Inersia + Kekakuan Pegas + Gravitasi) - 100% Identik Three-VRM Pixiv Standard
            let inertia = (joint.current_tail - joint.prev_tail) * (1.0 - joint.drag_force);
            let stiffness_force = world_rest_dir * (joint.stiffness * dt);
            let gravity_force = joint.gravity_dir * (joint.gravity_power * dt);
            let mut next_tail = joint.current_tail + inertia + stiffness_force + gravity_force;

            // 2. Length Constraint (Panjang ruas tulang pegas konsisten di world space)
            let to_tail = next_tail - bone_world_pos;
            let dist = to_tail.length();
            if dist > 0.0001 {
                next_tail = bone_world_pos + (to_tail / dist) * joint.bone_length;
            } else {
                next_tail = bone_world_pos + world_rest_dir * joint.bone_length;
            }

            // 3. Collision Resolution (Cegah rambut/rok menembus bola collider kepala/dada)
            for &c_idx in &joint.collider_indices {
                if let Some(Some((sphere_center, sphere_radius))) = collider_spheres.get(c_idx) {
                    let r_sum = sphere_radius + joint.hit_radius;
                    let diff = next_tail - *sphere_center;
                    let d = diff.length();
                    if d < r_sum && d > 0.0001 {
                        let push_dir = diff / d;
                        next_tail = *sphere_center + push_dir * r_sum;

                        // Rekalibrasi panjang tulang setelah dipantulkan collider
                        let to_tail2 = next_tail - bone_world_pos;
                        let dist2 = to_tail2.length();
                        if dist2 > 0.0001 {
                            next_tail = bone_world_pos + (to_tail2 / dist2) * joint.bone_length;
                        }
                    }
                }
            }

            // 4. Update history posisi
            joint.prev_tail = joint.current_tail;
            joint.current_tail = next_tail;

            // 5. Arahkan rotasi tulang ke next_tail (100% Identik Three-VRM: setFromUnitVectors premultiply initialLocalRotation)
            let inv_rest_world = rest_world_mat.inverse();
            let local_tail_pos = inv_rest_world.transform_point3(next_tail);
            let local_tail_dir = local_tail_pos.normalize_or_zero();

            if local_tail_dir.length_squared() > 0.0001 && joint.bone_axis.length_squared() > 0.0001 {
                let rot_from_axis = Quat::from_rotation_arc(joint.bone_axis, local_tail_dir);
                let final_rotation = node.initial_rotation * rot_from_axis;

                let current_local = Mat4::from_scale_rotation_translation(
                    node.initial_scale,
                    final_rotation,
                    node.initial_translation,
                );
                let new_global = parent_mat * current_local;
                globals[joint.node_idx] = new_global;

                // Update seluruh hierarki anak dari tulang ini agar rantai rambut berikutnya mengikuti
                update_children_transforms(joint.node_idx, new_global, model, globals);
            }
        }
    }
}

fn update_children_transforms(
    parent_idx: usize,
    parent_global: Mat4,
    model: &VrmModelData,
    globals: &mut [Mat4],
) {
    let node = &model.nodes[parent_idx];
    for &child_idx in &node.children {
        if child_idx < model.nodes.len() {
            let child_node = &model.nodes[child_idx];
            let child_local = Mat4::from_scale_rotation_translation(
                child_node.initial_scale,
                child_node.initial_rotation,
                child_node.initial_translation,
            );
            let child_global = parent_global * child_local;
            globals[child_idx] = child_global;
            update_children_transforms(child_idx, child_global, model, globals);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_bone_from_char_vrm() {
        if let Ok(bytes) = std::fs::read("assets/char.vrm") {
            if bytes.len() > 20 {
                let chunk0_len =
                    u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
                if bytes.len() >= 20 + chunk0_len {
                    if let Ok(json_str) = std::str::from_utf8(&bytes[20..20 + chunk0_len]) {
                        if let Ok(gltf_json) = serde_json::from_str::<serde_json::Value>(json_str)
                        {
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
                            assert!(
                                sb.is_some(),
                                "SpringBoneManager should parse VRMC_springBone successfully"
                            );
                            let mut sb = sb.unwrap();
                            assert_eq!(sb.colliders.len(), 28);
                            assert_eq!(sb.joints.len(), 145);

                            let model = VrmModelData {
                                primitives: vec![],
                                materials: vec![],
                                nodes: dummy_nodes,
                                root_nodes: vec![0],
                                skins: vec![],
                                bone_mapping: std::collections::HashMap::new(),
                                node_to_bone: vec![None; 300],
                                parent_indices: vec![None; 300],
                                parent_rest_world_rotations: vec![Quat::IDENTITY; 300],
                                spring_bone_manager: None,
                                expressions: std::collections::HashMap::new(),
                                known_mesh_indices: vec![],
                            };
                            let mut globals = vec![Mat4::IDENTITY; 300];
                            for _ in 0..10 {
                                sb.update(0.016, &model, &mut globals);
                            }
                            assert!(sb.joints[0].initialized);
                        }
                    }
                }
            }
        }
    }
}
