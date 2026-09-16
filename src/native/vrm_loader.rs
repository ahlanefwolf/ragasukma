use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub joints: [u32; 4],
    pub weights: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct MorphTargetData {
    pub position_displacements: Vec<[f32; 3]>,
}

pub struct MeshPrimitiveData {
    pub base_vertices: Vec<Vertex>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_index: Option<usize>,
    pub node_index: usize,
    pub mesh_index: usize,
    pub morph_targets: Vec<MorphTargetData>,
}

#[derive(Clone, Debug, Default)]
pub struct BlendShapeBind {
    pub mesh_index: usize,
    pub primitive_index: usize,
    pub target_index: usize,
    pub weight: f32, // 0.0 - 1.0
}

#[derive(Clone, Debug, Default)]
pub struct VrmExpressionData {
    pub name: String,
    pub binds: Vec<BlendShapeBind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialCategory {
    Hair,
    Skin,
    Outfit,
    Eyes,
    General,
}

pub struct MaterialData {
    pub name: String,
    pub base_color_texture: Option<std::sync::Arc<image::RgbaImage>>,
    pub base_color_factor: [f32; 4],
    pub shade_texture: Option<std::sync::Arc<image::RgbaImage>>,
    pub shade_color_factor: [f32; 3],
    pub emissive_texture: Option<std::sync::Arc<image::RgbaImage>>,
    pub emissive_factor: [f32; 3],
    pub matcap_texture: Option<std::sync::Arc<image::RgbaImage>>,
    pub matcap_factor: [f32; 3],
    pub shading_shift: f32,
    pub shading_toony: f32,
    pub alpha_cutoff: f32,
    pub double_sided: bool,
    pub category: MaterialCategory,
}

#[derive(Clone, Debug)]
pub struct NodeData {
    pub index: usize,
    pub name: String,
    pub children: Vec<usize>,
    pub local_transform: Mat4,
    pub initial_translation: Vec3,
    pub initial_rotation: Quat,
    pub initial_scale: Vec3,
    pub skin_index: Option<usize>,
    pub mesh_index: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct SkinData {
    pub joints: Vec<usize>,
    pub inverse_bind_matrices: Vec<Mat4>,
}

pub struct VrmModelData {
    pub primitives: Vec<MeshPrimitiveData>,
    pub materials: Vec<MaterialData>,
    pub nodes: Vec<NodeData>,
    pub root_nodes: Vec<usize>,
    pub skins: Vec<SkinData>,
    pub bone_mapping: HashMap<String, usize>,
    pub node_to_bone: Vec<Option<String>>,
    pub expressions: HashMap<String, VrmExpressionData>,
    pub known_mesh_indices: Vec<usize>,
}

fn extract_image_rgba(
    image: gltf::Image,
    buffers: &[gltf::buffer::Data],
    cache: &mut HashMap<usize, std::sync::Arc<image::RgbaImage>>,
) -> Option<std::sync::Arc<image::RgbaImage>> {
    let img_idx = image.index();
    if let Some(cached) = cache.get(&img_idx) {
        return Some(cached.clone());
    }

    let raw_img_opt = match image.source() {
        gltf::image::Source::View { view, .. } => {
            let parent_buffer = &buffers[view.buffer().index()];
            let begin = view.offset();
            let end = begin + view.length();
            let data = &parent_buffer[begin..end];
            image::load_from_memory(data).ok().map(|dyn_img| dyn_img.to_rgba8())
        }
        gltf::image::Source::Uri { .. } => None,
    };

    if let Some(mut img) = raw_img_opt {
        let (w, h) = img.dimensions();
        // Optimasi Memori: Batasi resolusi tekstur ke 1024px
        if w > 1024 || h > 1024 {
            let new_w = (w / 2).max(512);
            let new_h = (h / 2).max(512);
            img = image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Triangle);
        }
        let arc_img = std::sync::Arc::new(img);
        cache.insert(img_idx, arc_img.clone());
        Some(arc_img)
    } else {
        None
    }
}

impl VrmModelData {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Box<Self>> {
        let path_buf = path.as_ref().to_path_buf();

        // Eksekusi parsing VRM & Texture Decoding di thread khusus ber-stack 32MB untuk mencegah stack overflow
        let handle = std::thread::Builder::new()
            .name("vrm-loader-worker".to_string())
            .stack_size(32 * 1024 * 1024)
            .spawn(move || Self::load_from_file_internal(path_buf).map(Box::new))?;

        match handle.join() {
            Ok(res) => res,
            Err(_) => Err(anyhow::anyhow!("VRM Loader thread panicked during model parsing")),
        }
    }

    fn load_from_file_internal(path_buf: PathBuf) -> anyhow::Result<Self> {
        let path_ref = &path_buf;
        println!("📦 [VRM Loader] Membaca model VRM 1:1 Three.js: {:?}", path_ref);

        let gltf = gltf::Gltf::open(path_ref)?;
        let buffers = gltf::import_buffers(&gltf.document, path_ref.parent(), gltf.blob)?;
        let document = gltf.document;

        // Baca raw JSON dari GLB untuk parsing extensions MToon & VRMC_vrm secara akurat
        let mut gltf_json: Option<serde_json::Value> = None;
        if let Ok(mut file) = File::open(path_ref) {
            let mut bytes = Vec::new();
            if file.read_to_end(&mut bytes).is_ok() && bytes.len() > 20 {
                let chunk0_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
                if bytes.len() >= 20 + chunk0_len {
                    if let Ok(json_str) = std::str::from_utf8(&bytes[20..20 + chunk0_len]) {
                        gltf_json = serde_json::from_str(json_str).ok();
                    }
                }
            }
        }

        // 1. Ekstrak Material, MToon & KHR_materials_unlit Extensions
        let mut materials = Vec::new();
        let mut image_cache = HashMap::new();
        let json_materials = gltf_json.as_ref().and_then(|j| j.get("materials")).and_then(|m| m.as_array());

        for (mat_idx, mat) in document.materials().enumerate() {
            let name = mat.name().unwrap_or("Material").to_string();
            let name_lower = name.to_lowercase();

            let category = if name_lower.contains("hair") || name_lower.contains("rambut") {
                MaterialCategory::Hair
            } else if name_lower.contains("face") || name_lower.contains("body") || name_lower.contains("skin") {
                MaterialCategory::Skin
            } else if name_lower.contains("eye") || name_lower.contains("iris") {
                MaterialCategory::Eyes
            } else if name_lower.contains("cloth") || name_lower.contains("tops") || name_lower.contains("bottoms") || name_lower.contains("shoes") || name_lower.contains("onepiece") {
                MaterialCategory::Outfit
            } else {
                MaterialCategory::General
            };

            let pbr = mat.pbr_metallic_roughness();
            let base_color_factor = pbr.base_color_factor();
            let double_sided = mat.double_sided();

            let alpha_cutoff = match mat.alpha_mode() {
                gltf::material::AlphaMode::Opaque => 0.0,
                gltf::material::AlphaMode::Mask => mat.alpha_cutoff().unwrap_or(0.5),
                gltf::material::AlphaMode::Blend => 0.005,
            };

            // Base Color Texture
            let base_color_texture = if let Some(tex_info) = pbr.base_color_texture() {
                extract_image_rgba(tex_info.texture().source(), &buffers, &mut image_cache)
            } else {
                None
            };

            // Emissive Texture & Factor
            let emissive_factor = mat.emissive_factor();
            let emissive_texture = if let Some(tex_info) = mat.emissive_texture() {
                extract_image_rgba(tex_info.texture().source(), &buffers, &mut image_cache)
            } else {
                None
            };

            // MToon Parsing dari raw JSON
            let mut shade_texture = None;
            let mut shade_color_factor = [1.0, 1.0, 1.0];
            let mut matcap_texture = None;
            let mut matcap_factor = [0.0, 0.0, 0.0];
            let mut shading_shift = -0.2;
            let mut shading_toony = 0.85;

            if let Some(mat_json) = json_materials.and_then(|arr| arr.get(mat_idx)) {
                if let Some(mtoon_json) = mat_json.get("extensions").and_then(|e| e.get("VRMC_materials_mtoon")) {
                    if let Some(sc) = mtoon_json.get("shadeColorFactor").and_then(|v| v.as_array()) {
                        if sc.len() >= 3 {
                            shade_color_factor = [
                                sc[0].as_f64().unwrap_or(1.0) as f32,
                                sc[1].as_f64().unwrap_or(1.0) as f32,
                                sc[2].as_f64().unwrap_or(1.0) as f32,
                            ];
                        }
                    }
                    if let Some(shift) = mtoon_json.get("shadingShiftFactor").and_then(|v| v.as_f64()) {
                        shading_shift = shift as f32;
                    }
                    if let Some(toony) = mtoon_json.get("shadingToonyFactor").and_then(|v| v.as_f64()) {
                        shading_toony = toony as f32;
                    }
                    if let Some(mf) = mtoon_json.get("matcapFactor").and_then(|v| v.as_array()) {
                        if mf.len() >= 3 {
                            matcap_factor = [
                                mf[0].as_f64().unwrap_or(0.0) as f32,
                                mf[1].as_f64().unwrap_or(0.0) as f32,
                                mf[2].as_f64().unwrap_or(0.0) as f32,
                            ];
                        }
                    }
                    if let Some(st_idx) = mtoon_json.get("shadeMultiplyTexture").and_then(|t| t.get("index")).and_then(|i| i.as_u64()) {
                        if let Some(tex) = document.textures().nth(st_idx as usize) {
                            shade_texture = extract_image_rgba(tex.source(), &buffers, &mut image_cache);
                        }
                    }
                    if let Some(mt_idx) = mtoon_json.get("matcapTexture").and_then(|t| t.get("index")).and_then(|i| i.as_u64()) {
                        if let Some(tex) = document.textures().nth(mt_idx as usize) {
                            matcap_texture = extract_image_rgba(tex.source(), &buffers, &mut image_cache);
                        }
                    }
                }
            }

            println!(
                "  🎨 [Material] '{}' -> BaseTex: {}, ShadeTex: {}, EmissiveTex: {}, MatcapTex: {}",
                name,
                base_color_texture.is_some(),
                shade_texture.is_some(),
                emissive_texture.is_some(),
                matcap_texture.is_some()
            );

            materials.push(MaterialData {
                name,
                base_color_texture,
                base_color_factor,
                shade_texture,
                shade_color_factor,
                emissive_texture,
                emissive_factor,
                matcap_texture,
                matcap_factor,
                shading_shift,
                shading_toony,
                alpha_cutoff,
                double_sided,
                category,
            });
        }

        // 2. Ekstrak Node Hierarki & Bone Mapping
        let mut nodes = Vec::new();
        let mut bone_mapping = HashMap::new();

        for node in document.nodes() {
            let (t, r, s) = node.transform().decomposed();
            let initial_translation = Vec3::from(t);
            let initial_rotation = Quat::from_xyzw(r[0], r[1], r[2], r[3]);
            let initial_scale = Vec3::from(s);

            let local_transform = Mat4::from_scale_rotation_translation(
                initial_scale,
                initial_rotation,
                initial_translation,
            );

            let name = node.name().unwrap_or("").to_string();
            let node_idx = node.index();

            if let Some(mapped_name) = map_vrm_bone_str(&name) {
                bone_mapping.insert(mapped_name.to_string(), node_idx);
            }

            nodes.push(NodeData {
                index: node_idx,
                name,
                children: node.children().map(|c| c.index()).collect(),
                local_transform,
                initial_translation,
                initial_rotation,
                initial_scale,
                skin_index: node.skin().map(|s| s.index()),
                mesh_index: node.mesh().map(|m| m.index()),
            });
        }

        // Mapping Humanoid Bones dari VRMC_vrm extension di raw JSON
        if let Some(vrm_json) = gltf_json.as_ref().and_then(|j| j.get("extensions")).and_then(|e| e.get("VRMC_vrm")) {
            if let Some(bones) = vrm_json.get("humanoid").and_then(|h| h.get("humanBones")).and_then(|b| b.as_object()) {
                for (bone_name, bone_obj) in bones {
                    if let Some(node_idx) = bone_obj.get("node").and_then(|n| n.as_u64()) {
                        bone_mapping.insert(bone_name.clone(), node_idx as usize);
                    }
                }
            }
        }

        // 3. Ekstrak Mesh & Primitives dengan Relasi Node Mesh
        let mut primitives = Vec::new();
        for node in &nodes {
            if let Some(mesh_idx) = node.mesh_index {
                if let Some(mesh) = document.meshes().nth(mesh_idx) {
                    for prim in mesh.primitives() {
                        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

                        let positions: Vec<[f32; 3]> = reader.read_positions().map(|p| p.collect()).unwrap_or_default();
                        let normals: Vec<[f32; 3]> = reader.read_normals().map(|n| n.collect()).unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
                        let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|u| u.into_f32().collect()).unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                        let joints: Vec<[u32; 4]> = reader.read_joints(0).map(|j| j.into_u16().map(|arr| [arr[0] as u32, arr[1] as u32, arr[2] as u32, arr[3] as u32]).collect()).unwrap_or_else(|| vec![[0, 0, 0, 0]; positions.len()]);
                        let weights: Vec<[f32; 4]> = reader.read_weights(0).map(|w| w.into_f32().collect()).unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]);

                        let mut vertices = Vec::with_capacity(positions.len());
                        for i in 0..positions.len() {
                            vertices.push(Vertex {
                                position: positions[i],
                                normal: normals[i],
                                uv: uvs[i],
                                joints: joints[i],
                                weights: weights[i],
                            });
                        }

                        let indices: Vec<u32> = reader.read_indices().map(|idx| idx.into_u32().collect()).unwrap_or_else(|| (0..positions.len() as u32).collect());

                        // Ekstrak Morph Targets (Blend Shapes untuk Facial Expression & Lip Sync)
                        let mut morph_targets = Vec::new();
                        for (pos_opt, _norm_opt, _tan_opt) in reader.read_morph_targets() {
                            if let Some(morph_pos_iter) = pos_opt {
                                let position_displacements: Vec<[f32; 3]> = morph_pos_iter.collect();
                                morph_targets.push(MorphTargetData {
                                    position_displacements,
                                });
                            }
                        }

                        let base_vertices = vertices.clone();

                        primitives.push(MeshPrimitiveData {
                            base_vertices,
                            vertices,
                            indices,
                            material_index: prim.material().index(),
                            node_index: node.index,
                            mesh_index: mesh_idx,
                            morph_targets,
                        });
                    }
                }
            }
        }

        // 4. Ekstrak Skins (Joints & Inverse Bind Matrices)
        let mut skins = Vec::new();
        for skin in document.skins() {
            let reader = skin.reader(|buffer| Some(&buffers[buffer.index()]));
            let joints: Vec<usize> = skin.joints().map(|j| j.index()).collect();
            let inverse_bind_matrices: Vec<Mat4> = reader
                .read_inverse_bind_matrices()
                .map(|mats| mats.map(|m| Mat4::from_cols_array_2d(&m)).collect())
                .unwrap_or_else(|| vec![Mat4::IDENTITY; joints.len()]);

            skins.push(SkinData {
                joints,
                inverse_bind_matrices,
            });
        }

        // 5. Ekstrak Blend Shapes & Expressions (VRM 0.X & VRM 1.0)
        let mut expressions = HashMap::new();

        // VRM 0.X BlendShapeMaster
        if let Some(vrm_0) = gltf_json.as_ref().and_then(|j| j.get("extensions")).and_then(|e| e.get("VRM")) {
            if let Some(groups) = vrm_0.get("blendShapeMaster").and_then(|b| b.get("blendShapeGroups")).and_then(|g| g.as_array()) {
                for group in groups {
                    let name = group.get("name").and_then(|n| n.as_str())
                        .or_else(|| group.get("presetName").and_then(|n| n.as_str()))
                        .unwrap_or("").to_lowercase();
                    if name.is_empty() {
                        continue;
                    }

                    let mut binds = Vec::new();
                    if let Some(bind_arr) = group.get("binds").and_then(|b| b.as_array()) {
                        for b in bind_arr {
                            let mesh_index = b.get("mesh").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                            let target_index = b.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                            let weight = (b.get("weight").and_then(|v| v.as_f64()).unwrap_or(100.0) / 100.0) as f32;
                            binds.push(BlendShapeBind {
                                mesh_index,
                                primitive_index: 0,
                                target_index,
                                weight,
                            });
                        }
                    }
                    expressions.insert(name.clone(), VrmExpressionData {
                        name,
                        binds,
                    });
                }
            }
        }

        // VRM 1.0 Expressions (preset & custom)
        if let Some(vrm_1) = gltf_json.as_ref().and_then(|j| j.get("extensions")).and_then(|e| e.get("VRMC_vrm")) {
            if let Some(exprs_obj) = vrm_1.get("expressions").and_then(|e| e.as_object()) {
                for (_category, sub_val) in exprs_obj {
                    if let Some(obj) = sub_val.as_object() {
                        for (expr_name, expr_val) in obj {
                            let name = expr_name.to_lowercase();
                            let mut binds = Vec::new();
                            if let Some(morph_binds) = expr_val.get("morphTargetBinds").and_then(|m| m.as_array()) {
                                for b in morph_binds {
                                    let node_idx = b.get("node").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                                    let mesh_index = nodes.iter().find(|n| n.index == node_idx).and_then(|n| n.mesh_index).unwrap_or(0);
                                    let target_index = b.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                                    let weight = b.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                                    binds.push(BlendShapeBind {
                                        mesh_index,
                                        primitive_index: 0,
                                        target_index,
                                        weight,
                                    });
                                }
                            }
                            expressions.insert(name.clone(), VrmExpressionData {
                                name,
                                binds,
                            });
                        }
                    }
                }
            }
        }

        let root_nodes = if let Some(scene) = document.default_scene().or_else(|| document.scenes().next()) {
            scene.nodes().map(|n| n.index()).collect()
        } else {
            (0..nodes.len()).collect()
        };

        // Precompute node_to_bone reverse mapping & known_mesh_indices untuk O(1) direct lookup saat render loop
        let mut node_to_bone = vec![None; nodes.len()];
        for (bone_name, &node_idx) in &bone_mapping {
            if node_idx < node_to_bone.len() {
                node_to_bone[node_idx] = Some(bone_name.clone());
            }
        }
        let known_mesh_indices: Vec<usize> = primitives.iter().map(|p| p.mesh_index).collect();

        println!("✨ [VRM Loader] Sukses memuat {} mesh parts, {} material layers, {} blend expressions", primitives.len(), materials.len(), expressions.len());

        Ok(Self {
            primitives,
            materials,
            nodes,
            root_nodes,
            skins,
            bone_mapping,
            node_to_bone,
            expressions,
            known_mesh_indices,
        })
    }
}

pub fn map_vrm_bone_str(name_str: &str) -> Option<&'static str> {
    if name_str.contains("Head") || name_str == "head" {
        Some("head")
    } else if name_str.contains("Chest") || name_str == "chest" {
        Some("chest")
    } else if name_str.contains("Spine") || name_str == "spine" {
        Some("spine")
    } else if name_str.contains("Hips") || name_str == "hips" {
        Some("hips")
    } else if name_str.contains("LeftUpperArm") || name_str.contains("L_UpperArm") || name_str == "leftUpperArm" {
        Some("leftUpperArm")
    } else if name_str.contains("LeftLowerArm") || name_str.contains("L_LowerArm") || name_str == "leftLowerArm" {
        Some("leftLowerArm")
    } else if name_str.contains("LeftHand") || name_str.contains("L_Hand") || name_str == "leftHand" {
        Some("leftHand")
    } else if name_str.contains("RightUpperArm") || name_str.contains("R_UpperArm") || name_str == "rightUpperArm" {
        Some("rightUpperArm")
    } else if name_str.contains("RightLowerArm") || name_str.contains("R_LowerArm") || name_str == "rightLowerArm" {
        Some("rightLowerArm")
    } else if name_str.contains("RightHand") || name_str.contains("R_Hand") || name_str == "rightHand" {
        Some("rightHand")
    } else if name_str.contains("LeftShoulder") || name_str.contains("L_Shoulder") || name_str == "leftShoulder" {
        Some("leftShoulder")
    } else if name_str.contains("RightShoulder") || name_str.contains("R_Shoulder") || name_str == "rightShoulder" {
        Some("rightShoulder")
    } else if name_str.contains("LeftUpperLeg") || name_str.contains("L_UpperLeg") || name_str == "leftUpperLeg" {
        Some("leftUpperLeg")
    } else if name_str.contains("LeftLowerLeg") || name_str.contains("L_LowerLeg") || name_str == "leftLowerLeg" {
        Some("leftLowerLeg")
    } else if name_str.contains("LeftFoot") || name_str.contains("L_Foot") || name_str == "leftFoot" {
        Some("leftFoot")
    } else if name_str.contains("RightUpperLeg") || name_str.contains("R_UpperLeg") || name_str == "rightUpperLeg" {
        Some("rightUpperLeg")
    } else if name_str.contains("RightLowerLeg") || name_str.contains("R_LowerLeg") || name_str == "rightLowerLeg" {
        Some("rightLowerLeg")
    } else if name_str.contains("RightFoot") || name_str.contains("R_Foot") || name_str == "rightFoot" {
        Some("rightFoot")
    } else {
        None
    }
}
