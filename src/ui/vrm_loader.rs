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
    pub last_target_weights: Vec<f32>,
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
    pub is_unlit: bool,
    pub category: MaterialCategory,
    pub is_transparent: bool,
    pub render_queue: i32,
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
    pub is_vrm_0: bool,
    pub primitives: Vec<MeshPrimitiveData>,
    pub materials: Vec<MaterialData>,
    pub nodes: Vec<NodeData>,
    pub root_nodes: Vec<usize>,
    pub skins: Vec<SkinData>,
    pub bone_mapping: HashMap<String, usize>,
    pub node_to_bone: Vec<Option<String>>,
    pub parent_indices: Vec<Option<usize>>,
    pub parent_rest_world_rotations: Vec<Quat>,
    pub spring_bone_manager: Option<std::sync::Mutex<crate::ui::spring_bone::SpringBoneManager>>,
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
            image::load_from_memory(data)
                .ok()
                .map(|dyn_img| dyn_img.to_rgba8())
        }
        gltf::image::Source::Uri { .. } => None,
    };

    if let Some(mut img) = raw_img_opt {
        let (w, h) = img.dimensions();
        // Memory optimization: Clamp texture resolution to 1024px
        if w > 1024 || h > 1024 {
            let new_w = (w / 2).max(512);
            let new_h = (h / 2).max(512);
            img =
                image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Triangle);
        }
        let arc_img = std::sync::Arc::new(img);
        cache.insert(img_idx, arc_img.clone());
        Some(arc_img)
    } else {
        None
    }
}

fn compute_rest_world_rotations(
    node_idx: usize,
    parent_rot: Quat,
    nodes: &[NodeData],
    rest_world_rots: &mut [Quat],
) {
    let world_rot = parent_rot * nodes[node_idx].initial_rotation;
    rest_world_rots[node_idx] = world_rot;
    for &child_idx in &nodes[node_idx].children {
        if child_idx < nodes.len() {
            compute_rest_world_rotations(child_idx, world_rot, nodes, rest_world_rots);
        }
    }
}

impl VrmModelData {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Box<Self>> {
        let path_buf = path.as_ref().to_path_buf();

        // Execute VRM parsing and texture decoding in dedicated thread with 32MB stack to prevent stack overflow
        let handle = std::thread::Builder::new()
            .name("vrm-loader-worker".to_string())
            .stack_size(32 * 1024 * 1024)
            .spawn(move || Self::load_from_file_internal(path_buf).map(Box::new))?;

        match handle.join() {
            Ok(res) => res,
            Err(_) => Err(anyhow::anyhow!(
                "VRM Loader thread panicked during model parsing"
            )),
        }
    }

    fn load_from_file_internal(path_buf: PathBuf) -> anyhow::Result<Self> {
        let path_ref = &path_buf;
        println!(
            "[VRM Loader] Reading Three.js 1:1 VRM model: {:?}",
            path_ref
        );

        let gltf = gltf::Gltf::open(path_ref)?;
        let buffers = gltf::import_buffers(&gltf.document, path_ref.parent(), gltf.blob)?;
        let document = gltf.document;

        // Read raw JSON from VRM binary container for accurate parsing of MToon and VRMC_vrm extensions
        let mut gltf_json: Option<serde_json::Value> = None;
        if let Ok(mut file) = File::open(path_ref) {
            let mut bytes = Vec::new();
            if file.read_to_end(&mut bytes).is_ok() && bytes.len() > 20 {
                let chunk0_len =
                    u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
                if bytes.len() >= 20 + chunk0_len {
                    if let Ok(json_str) = std::str::from_utf8(&bytes[20..20 + chunk0_len]) {
                        gltf_json = serde_json::from_str(json_str).ok();
                    }
                }
            }
        }

        // Determine VRM version:
        // VRM 1.0 has extensions.VRMC_vrm
        // VRM 0.X has extensions.VRM
        let is_vrm_0 = gltf_json
            .as_ref()
            .and_then(|j| j.get("extensions"))
            .map(|e| e.get("VRMC_vrm").is_none() && e.get("VRM").is_some())
            .unwrap_or(false);

        if is_vrm_0 {
            println!("[VRM Loader] Detected legacy VRM 0.X model - applying VRM 1.0 humanoid normalization");
        } else {
            println!("[VRM Loader] Detected modern VRM 1.0 model");
        }

        // 1. Ekstrak Material, MToon & KHR_materials_unlit Extensions
        let mut materials = Vec::new();
        let mut image_cache = HashMap::new();
        let json_materials = gltf_json
            .as_ref()
            .and_then(|j| j.get("materials"))
            .and_then(|m| m.as_array());

        for (mat_idx, mat) in document.materials().enumerate() {
            let name = mat.name().unwrap_or("Material").to_string();
            let name_lower = name.to_lowercase();

            let category = if name_lower.contains("hair") || name_lower.contains("hairpin") {
                MaterialCategory::Hair
            } else if name_lower.contains("eye") || name_lower.contains("iris") || name_lower.contains("brow") {
                MaterialCategory::Eyes
            } else if name_lower.contains("body3")
                || name_lower.contains("accesory")
                || name_lower.contains("accessory")
                || name_lower.contains("dress")
                || name_lower.contains("cloth")
                || name_lower.contains("shoes")
                || name_lower.contains("sleeves")
                || name_lower.contains("element")
            {
                MaterialCategory::Outfit
            } else if name_lower.contains("face")
                || name_lower.contains("body")
                || name_lower.contains("skin")
            {
                MaterialCategory::Skin
            } else {
                MaterialCategory::General
            };

            let pbr = mat.pbr_metallic_roughness();
            let mut base_color_factor = pbr.base_color_factor();
            let mut double_sided = mat.double_sided();
            let mut is_blend = matches!(mat.alpha_mode(), gltf::material::AlphaMode::Blend);
            let mut render_queue: i32 = if is_blend { 3000 } else { 2000 };

            let mut is_unlit = json_materials
                .and_then(|arr| arr.get(mat_idx))
                .and_then(|m| m.get("extensions"))
                .and_then(|e| e.get("KHR_materials_unlit"))
                .is_some();

            let mut alpha_cutoff = match mat.alpha_mode() {
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

            // For OPAQUE materials, glTF 2.0 spec mandates that alpha is ignored and treated as 1.0.
            // For BLEND/MASK materials, preserve the original base_color_factor alpha so that
            // intentionally transparent helper quads (e.g. eye_touming / eye_bantou with alpha 0.0)
            // stay completely transparent and are not rendered as solid white occluders.
            if matches!(mat.alpha_mode(), gltf::material::AlphaMode::Opaque) {
                base_color_factor[3] = 1.0;
            }

            // Emissive Texture & Factor
            let emissive_factor = mat.emissive_factor();
            let emissive_texture = if let Some(tex_info) = mat.emissive_texture() {
                extract_image_rgba(tex_info.texture().source(), &buffers, &mut image_cache)
            } else {
                None
            };

            // MToon Parsing from raw JSON (supporting both VRM 0.X and VRM 1.0)
            let mut has_mtoon = false;
            let mut shade_texture = None;
            let mut shade_color_factor = [1.0, 1.0, 1.0];
            let mut matcap_texture = None;
            let mut matcap_factor = [1.0, 1.0, 1.0];
            let mut shading_shift = -0.2;
            let mut shading_toony = 0.85;

            // 1. Try parsing VRMC_materials_mtoon (VRM 1.0)
            if let Some(mat_json) = json_materials.and_then(|arr| arr.get(mat_idx)) {
                if let Some(mtoon_json) = mat_json
                    .get("extensions")
                    .and_then(|e| e.get("VRMC_materials_mtoon"))
                {
                    has_mtoon = true;
                    if let Some(cm) = mtoon_json.get("cullMode").and_then(|v| v.as_str()) {
                        if cm.eq_ignore_ascii_case("off") {
                            double_sided = true;
                        }
                    }
                    if let Some(q_offset) = mtoon_json
                        .get("renderQueueOffsetNumber")
                        .and_then(|v| v.as_i64())
                    {
                        render_queue += q_offset as i32;
                    }
                    if let Some(sc) = mtoon_json
                        .get("shadeColorFactor")
                        .and_then(|v| v.as_array())
                    {
                        if sc.len() >= 3 {
                            shade_color_factor = [
                                sc[0].as_f64().unwrap_or(1.0) as f32,
                                sc[1].as_f64().unwrap_or(1.0) as f32,
                                sc[2].as_f64().unwrap_or(1.0) as f32,
                            ];
                        }
                    }
                    if let Some(shift) = mtoon_json
                        .get("shadingShiftFactor")
                        .and_then(|v| v.as_f64())
                    {
                        shading_shift = shift as f32;
                    }
                    if let Some(toony) = mtoon_json
                        .get("shadingToonyFactor")
                        .and_then(|v| v.as_f64())
                    {
                        shading_toony = toony as f32;
                    }
                    if let Some(mf) = mtoon_json.get("matcapFactor").and_then(|v| v.as_array()) {
                        if mf.len() >= 3 {
                            matcap_factor = [
                                mf[0].as_f64().unwrap_or(1.0) as f32,
                                mf[1].as_f64().unwrap_or(1.0) as f32,
                                mf[2].as_f64().unwrap_or(1.0) as f32,
                            ];
                        }
                    }
                    if let Some(st_idx) = mtoon_json
                        .get("shadeMultiplyTexture")
                        .and_then(|t| t.get("index"))
                        .and_then(|i| i.as_u64())
                    {
                        if let Some(tex) = document.textures().nth(st_idx as usize) {
                            shade_texture =
                                extract_image_rgba(tex.source(), &buffers, &mut image_cache);
                        }
                    }
                    if let Some(mt_idx) = mtoon_json
                        .get("matcapTexture")
                        .and_then(|t| t.get("index"))
                        .and_then(|i| i.as_u64())
                    {
                        if let Some(tex) = document.textures().nth(mt_idx as usize) {
                            matcap_texture =
                                extract_image_rgba(tex.source(), &buffers, &mut image_cache);
                        }
                    }
                }
            }

            // 2. Fallback to VRM 0.X (extensions.VRM.materialProperties)
            if let Some(vrm_0) = gltf_json
                .as_ref()
                .and_then(|j| j.get("extensions"))
                .and_then(|e| e.get("VRM"))
            {
                if let Some(mat_props) = vrm_0.get("materialProperties").and_then(|p| p.as_array()) {
                    // Match by material name or by index
                    let prop_opt = mat_props.iter().find(|p| {
                        p.get("name").and_then(|n| n.as_str()) == Some(&name)
                    }).or_else(|| mat_props.get(mat_idx));

                    if let Some(prop) = prop_opt {
                        let shader_name = prop.get("shader").and_then(|s| s.as_str()).unwrap_or("");
                        let is_vrm0_mtoon = shader_name.contains("MToon")
                            || (!shader_name.is_empty() && !shader_name.contains("Unlit"))
                            || (shader_name.is_empty() && is_vrm_0);
                        if is_vrm0_mtoon {
                            has_mtoon = true;
                        }

                        if let Some(rq) = prop.get("renderQueue").and_then(|v| v.as_i64()) {
                            if rq > 0 {
                                render_queue = rq as i32;
                            }
                        }
                        let fprops = prop.get("floatProperties");
                        let vprops = prop.get("vectorProperties");
                        let tprops = prop.get("textureProperties");

                        // VRM 0.X _CullMode (0 = Off/DoubleSided, 1 = Front, 2 = Back)
                        if let Some(cm) = fprops.and_then(|f| f.get("_CullMode")).and_then(|v| v.as_f64()) {
                            if (cm as i32) == 0 {
                                double_sided = true;
                            }
                        }

                        // VRM 0.X blend mode mapping:
                        // UniVRM 0.X exporter often bugs and marks ALL glTF materials as 'alphaMode: BLEND'.
                        // The actual authoritative blend mode is stored in floatProperties._BlendMode:
                        // 0 = Opaque, 1 = Cutout (AlphaTest), 2 = Transparent, 3 = TransparentWithZWrite
                        let blend_mode = fprops
                            .and_then(|f| f.get("_BlendMode"))
                            .and_then(|v| v.as_f64())
                            .map(|b| b as i32);

                        if let Some(bm) = blend_mode {
                            match bm {
                                0 => {
                                    is_blend = false;
                                    alpha_cutoff = 0.0;
                                    base_color_factor[3] = 1.0;
                                    if render_queue <= 0 {
                                        render_queue = 2000;
                                    }
                                }
                                1 => {
                                    is_blend = false;
                                    let cutoff = fprops
                                        .and_then(|f| f.get("_Cutoff"))
                                        .and_then(|v| v.as_f64())
                                        .unwrap_or(0.5) as f32;
                                    alpha_cutoff = cutoff;
                                    if render_queue <= 0 {
                                        render_queue = 2450;
                                    }
                                }
                                2 => {
                                    is_blend = true;
                                    alpha_cutoff = 0.005;
                                    if render_queue <= 0 {
                                        render_queue = 3000;
                                    }
                                }
                                3 => {
                                    is_blend = true;
                                    alpha_cutoff = 0.005;
                                    if render_queue <= 0 {
                                        render_queue = 2500;
                                    }
                                }
                                _ => {}
                            }
                        } else if render_queue < 2450 {
                            is_blend = false;
                            alpha_cutoff = 0.0;
                            base_color_factor[3] = 1.0;
                        } else if render_queue < 2500 {
                            is_blend = false;
                            let cutoff = fprops
                                .and_then(|f| f.get("_Cutoff"))
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.5) as f32;
                            alpha_cutoff = cutoff;
                        } else {
                            is_blend = true;
                            alpha_cutoff = 0.005;
                        }

                        if let Some(col) = vprops.and_then(|v| v.get("_Color")).and_then(|a| a.as_array()) {
                            if col.len() >= 4 {
                                let c_alpha = col[3].as_f64().unwrap_or(1.0) as f32;
                                // For standard OPAQUE materials, glTF 2.0 treats alpha as 1.0.
                                // For MASK (Cutout) or BLEND (Transparent), preserve the MToon _Color alpha
                                // so that intentionally transparent helper quads/layers (alpha 0.0) are discarded.
                                let final_alpha = if render_queue < 2450 && matches!(mat.alpha_mode(), gltf::material::AlphaMode::Opaque) {
                                    1.0
                                } else {
                                    c_alpha
                                };
                                base_color_factor = [
                                    col[0].as_f64().unwrap_or(1.0) as f32,
                                    col[1].as_f64().unwrap_or(1.0) as f32,
                                    col[2].as_f64().unwrap_or(1.0) as f32,
                                    final_alpha,
                                ];
                            }
                        }

                        if let Some(sc) = vprops.and_then(|v| v.get("_ShadeColor")).and_then(|a| a.as_array()) {
                            if sc.len() >= 3 {
                                shade_color_factor = [
                                    sc[0].as_f64().unwrap_or(1.0) as f32,
                                    sc[1].as_f64().unwrap_or(1.0) as f32,
                                    sc[2].as_f64().unwrap_or(1.0) as f32,
                                ];
                            }
                        }

                        // VRM 0.X to VRM 1.0 MToon formula conversion:
                        // shadingShift = -_ShadeShift + (1.0 - _ShadeToony) * _ShadingGradeRate
                        // Three.js VRM0 parser converts:
                        // shift = -_ShadeShift, toony = _ShadeToony
                        let s_shift = fprops.and_then(|f| f.get("_ShadeShift")).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        let s_toony = fprops.and_then(|f| f.get("_ShadeToony")).and_then(|v| v.as_f64()).unwrap_or(0.9) as f32;
                        shading_shift = -s_shift;
                        shading_toony = s_toony;

                        if let Some(st_idx) = tprops.and_then(|t| t.get("_ShadeTexture")).and_then(|i| i.as_u64()) {
                            if let Some(tex) = document.textures().nth(st_idx as usize) {
                                shade_texture = extract_image_rgba(tex.source(), &buffers, &mut image_cache);
                            }
                        }

                        if let Some(mt_idx) = tprops.and_then(|t| t.get("_SphereAdd")).and_then(|i| i.as_u64()) {
                            if let Some(tex) = document.textures().nth(mt_idx as usize) {
                                matcap_texture = extract_image_rgba(tex.source(), &buffers, &mut image_cache);
                            }
                        }
                    }
                }
            }

            // Prioritize MToon over Unlit:
            // If material defines MToon (VRMC_materials_mtoon or VRM 0.x MToon),
            // KHR_materials_unlit is merely a glTF fallback, so do not mark as is_unlit.
            if has_mtoon {
                is_unlit = false;
            }

            // Fallback Shade Tint:
            // If skin material lacks a dedicated shade texture and shade color is still white/default (>= 0.95),
            // assign a warm peach tint (~0.85) so anime cel-shading contours emerge naturally under lighting.
            if matches!(category, MaterialCategory::Skin) && shade_texture.is_none() {
                if shade_color_factor[0] >= 0.95 && shade_color_factor[1] >= 0.95 && shade_color_factor[2] >= 0.95 {
                    shade_color_factor = [0.85, 0.81, 0.78];
                }
            }

            let is_transparent = if is_vrm_0 {
                is_blend && render_queue >= 2500
            } else {
                is_blend || render_queue >= 2500
            };

            println!(
                "   [Material] '{}' -> BaseTex: {}, ShadeTex: {}, EmissiveTex: {}, MatcapTex: {}, Queue: {}, Transparent: {}",
                name,
                base_color_texture.is_some(),
                shade_texture.is_some(),
                emissive_texture.is_some(),
                matcap_texture.is_some(),
                render_queue,
                is_transparent
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
                is_unlit,
                category,
                is_transparent,
                render_queue,
            });
        }

        // 2. Ekstrak Node Hierarchy & Bone Mapping
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

        // Mapping Humanoid Bones from VRMC_vrm (VRM 1.0) or VRM (VRM 0.X) in raw JSON
        if let Some(vrm_1) = gltf_json
            .as_ref()
            .and_then(|j| j.get("extensions"))
            .and_then(|e| e.get("VRMC_vrm"))
        {
            if let Some(bones) = vrm_1
                .get("humanoid")
                .and_then(|h| h.get("humanBones"))
                .and_then(|b| b.as_object())
            {
                for (bone_name, bone_obj) in bones {
                    if let Some(node_idx) = bone_obj.get("node").and_then(|n| n.as_u64()) {
                        let mapped_key = match bone_name.as_str() {
                            "leftThumbMetacarpal" | "leftThumbProximal" => Some("leftThumb"),
                            "leftIndexProximal" => Some("leftIndex"),
                            "leftMiddleProximal" => Some("leftMiddle"),
                            "leftRingProximal" => Some("leftRing"),
                            "leftLittleProximal" => Some("leftLittle"),
                            "rightThumbMetacarpal" | "rightThumbProximal" => Some("rightThumb"),
                            "rightIndexProximal" => Some("rightIndex"),
                            "rightMiddleProximal" => Some("rightMiddle"),
                            "rightRingProximal" => Some("rightRing"),
                            "rightLittleProximal" => Some("rightLittle"),
                            _ => None,
                        };

                        if let Some(std_name) = mapped_key {
                            bone_mapping.insert(std_name.to_string(), node_idx as usize);
                        } else {
                            bone_mapping.insert(bone_name.clone(), node_idx as usize);
                        }
                    }
                }
            }
        } else if let Some(vrm_0) = gltf_json
            .as_ref()
            .and_then(|j| j.get("extensions"))
            .and_then(|e| e.get("VRM"))
        {
            // VRM 0.X Humanoid Bones mapping (Array of { bone: string, node: number })
            if let Some(bone_arr) = vrm_0
                .get("humanoid")
                .and_then(|h| h.get("humanBones"))
                .and_then(|b| b.as_array())
            {
                for bone_item in bone_arr {
                    let bone_name = bone_item.get("bone").and_then(|n| n.as_str()).unwrap_or("");
                    if let Some(node_idx) = bone_item.get("node").and_then(|n| n.as_u64()) {
                        let mapped_key = match bone_name {
                            "leftThumb" | "leftThumbProximal" => Some("leftThumb"),
                            "leftIndex" | "leftIndexProximal" => Some("leftIndex"),
                            "leftMiddle" | "leftMiddleProximal" => Some("leftMiddle"),
                            "leftRing" | "leftRingProximal" => Some("leftRing"),
                            "leftLittle" | "leftLittleProximal" => Some("leftLittle"),
                            "rightThumb" | "rightThumbProximal" => Some("rightThumb"),
                            "rightIndex" | "rightIndexProximal" => Some("rightIndex"),
                            "rightMiddle" | "rightMiddleProximal" => Some("rightMiddle"),
                            "rightRing" | "rightRingProximal" => Some("rightRing"),
                            "rightLittle" | "rightLittleProximal" => Some("rightLittle"),
                            _ => None,
                        };

                        if let Some(std_name) = mapped_key {
                            bone_mapping.insert(std_name.to_string(), node_idx as usize);
                        } else {
                            bone_mapping.insert(bone_name.to_string(), node_idx as usize);
                        }
                    }
                }
            }
        }

        // 3. Ekstrak Mesh & Primitives with Relasi Node Mesh
        let mut primitives = Vec::new();
        for node in &nodes {
            if let Some(mesh_idx) = node.mesh_index {
                if let Some(mesh) = document.meshes().nth(mesh_idx) {
                    for prim in mesh.primitives() {
                        let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));

                        let positions: Vec<[f32; 3]> = reader
                            .read_positions()
                            .map(|p| p.collect())
                            .unwrap_or_default();
                        let normals: Vec<[f32; 3]> = reader
                            .read_normals()
                            .map(|n| n.collect())
                            .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);
                        let uvs: Vec<[f32; 2]> = reader
                            .read_tex_coords(0)
                            .map(|u| u.into_f32().collect())
                            .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                        let joints: Vec<[u32; 4]> = reader
                            .read_joints(0)
                            .map(|j| {
                                j.into_u16()
                                    .map(|arr| {
                                        [arr[0] as u32, arr[1] as u32, arr[2] as u32, arr[3] as u32]
                                    })
                                    .collect()
                            })
                            .unwrap_or_else(|| vec![[0, 0, 0, 0]; positions.len()]);
                        let weights: Vec<[f32; 4]> = reader
                            .read_weights(0)
                            .map(|w| w.into_f32().collect())
                            .unwrap_or_else(|| vec![[1.0, 0.0, 0.0, 0.0]; positions.len()]);

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

                        let indices: Vec<u32> = reader
                            .read_indices()
                            .map(|idx| idx.into_u32().collect())
                            .unwrap_or_else(|| (0..positions.len() as u32).collect());

                        // Ekstrak Morph Targets (Blend Shapes for Facial Expression & Lip Sync)
                        let mut morph_targets = Vec::new();
                        for (pos_opt, _norm_opt, _tan_opt) in reader.read_morph_targets() {
                            if let Some(morph_pos_iter) = pos_opt {
                                let position_displacements: Vec<[f32; 3]> =
                                    morph_pos_iter.collect();
                                morph_targets.push(MorphTargetData {
                                    position_displacements,
                                });
                            }
                        }

                        // Compact vertices per primitive:
                        // Multi-material meshes from Blender / glTF often share a giant vertex buffer
                        // (e.g. 24,000+ vertices) across 16+ primitives.
                        // Compacting down to only the vertices indexed by this primitive saves up to 90%+
                        // of GPU vertex memory and vertex shader work while preserving 100% of UVs, normals,
                        // textures, and skin weights!
                        let (compact_vertices, compact_indices, compact_morph_targets) = {
                            let mut old_to_new = vec![u32::MAX; vertices.len()];
                            let mut used_old_indices = Vec::new();
                            let mut compact_vertices = Vec::new();
                            let mut compact_indices = Vec::with_capacity(indices.len());

                            for &old_idx in &indices {
                                let old_usize = old_idx as usize;
                                if old_usize < vertices.len() {
                                    let mut new_idx = old_to_new[old_usize];
                                    if new_idx == u32::MAX {
                                        new_idx = compact_vertices.len() as u32;
                                        old_to_new[old_usize] = new_idx;
                                        compact_vertices.push(vertices[old_usize]);
                                        used_old_indices.push(old_usize);
                                    }
                                    compact_indices.push(new_idx);
                                }
                            }

                            // Remap morph target displacements to match compact_vertices 1:1
                            let mut compact_morph_targets = Vec::with_capacity(morph_targets.len());
                            for mt in &morph_targets {
                                let mut new_disps = Vec::with_capacity(used_old_indices.len());
                                for &old_usize in &used_old_indices {
                                    let disp = mt
                                        .position_displacements
                                        .get(old_usize)
                                        .copied()
                                        .unwrap_or([0.0, 0.0, 0.0]);
                                    new_disps.push(disp);
                                }
                                compact_morph_targets.push(MorphTargetData {
                                    position_displacements: new_disps,
                                });
                            }

                            if compact_vertices.is_empty() {
                                (vertices, indices, morph_targets)
                            } else {
                                (compact_vertices, compact_indices, compact_morph_targets)
                            }
                        };

                        let base_vertices = compact_vertices.clone();

                        primitives.push(MeshPrimitiveData {
                            base_vertices,
                            vertices: compact_vertices,
                            indices: compact_indices,
                            material_index: prim.material().index(),
                            node_index: node.index,
                            mesh_index: mesh_idx,
                            morph_targets: compact_morph_targets,
                            last_target_weights: Vec::new(),
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

        // Remap primitive joint indices if a primitive belongs to a secondary skin (skin != 0)
        // so all primitives reference indices in the primary master skin (skins[0])
        if skins.len() > 1 {
            let mut node_to_primary_joint = HashMap::with_capacity(skins[0].joints.len());
            for (p_idx, &node_idx) in skins[0].joints.iter().enumerate() {
                node_to_primary_joint.insert(node_idx, p_idx as u32);
            }

            // Merge any missing joints from secondary skins into primary master skin
            for s_idx in 1..skins.len() {
                let sec_joints = skins[s_idx].joints.clone();
                let sec_inv_mats = skins[s_idx].inverse_bind_matrices.clone();
                for (sec_j_idx, joint_node) in sec_joints.into_iter().enumerate() {
                    if let std::collections::hash_map::Entry::Vacant(entry) =
                        node_to_primary_joint.entry(joint_node)
                    {
                        let new_p_idx = skins[0].joints.len() as u32;
                        skins[0].joints.push(joint_node);
                        let inv_mat = sec_inv_mats
                            .get(sec_j_idx)
                            .copied()
                            .unwrap_or(Mat4::IDENTITY);
                        skins[0].inverse_bind_matrices.push(inv_mat);
                        entry.insert(new_p_idx);
                    }
                }
            }

            for prim in &mut primitives {
                let node_skin_idx = nodes.get(prim.node_index).and_then(|n| n.skin_index);
                if let Some(skin_idx) = node_skin_idx {
                    if skin_idx > 0 && skin_idx < skins.len() {
                        let sec_skin = &skins[skin_idx];
                        let remap: Vec<u32> = sec_skin
                            .joints
                            .iter()
                            .map(|&joint_node| {
                                node_to_primary_joint
                                    .get(&joint_node)
                                    .copied()
                                    .unwrap_or(0)
                            })
                            .collect();

                        for v in &mut prim.vertices {
                            v.joints[0] = remap.get(v.joints[0] as usize).copied().unwrap_or(0);
                            v.joints[1] = remap.get(v.joints[1] as usize).copied().unwrap_or(0);
                            v.joints[2] = remap.get(v.joints[2] as usize).copied().unwrap_or(0);
                            v.joints[3] = remap.get(v.joints[3] as usize).copied().unwrap_or(0);
                        }
                        prim.base_vertices = prim.vertices.clone();
                    }
                }
            }
        }

        // Joint Compaction & Optimization:
        // Exporters (like Blender Saturday06 VRM exporter) often export all armature bones into
        // the skin joints list (> 1000 joints), but only a subset are actually weighted to mesh vertices.
        // GPU uniform buffers have a standard hardware limit of 1024 bone matrices (64KB).
        // If primary skin joints > 1024, or to optimize bone matrix calculations, compact the skin
        // to only joints referenced by vertices with non-zero weights.
        if let Some(primary_skin) = skins.first_mut() {
            let max_joint_in_prims = primitives
                .iter()
                .flat_map(|p| &p.vertices)
                .flat_map(|v| v.joints)
                .max()
                .unwrap_or(0) as usize;

            if primary_skin.joints.len() > 1024 || max_joint_in_prims >= 1024 {
                let mut used_joint_indices = std::collections::BTreeSet::new();
                for prim in &primitives {
                    for v in &prim.vertices {
                        if v.weights[0] > 0.0001 {
                            used_joint_indices.insert(v.joints[0] as usize);
                        }
                        if v.weights[1] > 0.0001 {
                            used_joint_indices.insert(v.joints[1] as usize);
                        }
                        if v.weights[2] > 0.0001 {
                            used_joint_indices.insert(v.joints[2] as usize);
                        }
                        if v.weights[3] > 0.0001 {
                            used_joint_indices.insert(v.joints[3] as usize);
                        }
                    }
                }

                // Ensure root joint 0 is preserved
                used_joint_indices.insert(0);

                let mut new_joints = Vec::with_capacity(used_joint_indices.len());
                let mut new_inv_mats = Vec::with_capacity(used_joint_indices.len());
                let mut old_to_new = HashMap::with_capacity(used_joint_indices.len());

                for &old_idx in &used_joint_indices {
                    if old_idx < primary_skin.joints.len() {
                        let new_idx = new_joints.len() as u32;
                        new_joints.push(primary_skin.joints[old_idx]);
                        let inv_mat = primary_skin
                            .inverse_bind_matrices
                            .get(old_idx)
                            .copied()
                            .unwrap_or(Mat4::IDENTITY);
                        new_inv_mats.push(inv_mat);
                        old_to_new.insert(old_idx as u32, new_idx);
                    }
                }

                println!(
                    "   [Skin Optimization] Compacted skin joints from {} down to {} active joints",
                    primary_skin.joints.len(),
                    new_joints.len()
                );

                primary_skin.joints = new_joints;
                primary_skin.inverse_bind_matrices = new_inv_mats;

                for prim in &mut primitives {
                    for v in &mut prim.vertices {
                        v.joints[0] = old_to_new.get(&v.joints[0]).copied().unwrap_or(0);
                        v.joints[1] = old_to_new.get(&v.joints[1]).copied().unwrap_or(0);
                        v.joints[2] = old_to_new.get(&v.joints[2]).copied().unwrap_or(0);
                        v.joints[3] = old_to_new.get(&v.joints[3]).copied().unwrap_or(0);

                        // Safety clamp for extreme edge cases (> 1024 active bones)
                        if v.joints[0] >= 1024 {
                            v.joints[0] = 0;
                        }
                        if v.joints[1] >= 1024 {
                            v.joints[1] = 0;
                        }
                        if v.joints[2] >= 1024 {
                            v.joints[2] = 0;
                        }
                        if v.joints[3] >= 1024 {
                            v.joints[3] = 0;
                        }
                    }
                    prim.base_vertices = prim.vertices.clone();
                }
            }
        }

        // 5. Ekstrak Blend Shapes & Expressions (VRM 0.X & VRM 1.0)
        let mut expressions = HashMap::new();

        // VRM 0.X BlendShapeMaster
        if let Some(vrm_0) = gltf_json
            .as_ref()
            .and_then(|j| j.get("extensions"))
            .and_then(|e| e.get("VRM"))
        {
            if let Some(groups) = vrm_0
                .get("blendShapeMaster")
                .and_then(|b| b.get("blendShapeGroups"))
                .and_then(|g| g.as_array())
            {
                for group in groups {
                    let name = group
                        .get("name")
                        .and_then(|n| n.as_str())
                        .or_else(|| group.get("presetName").and_then(|n| n.as_str()))
                        .unwrap_or("")
                        .to_lowercase();
                    if name.is_empty() {
                        continue;
                    }

                    let mut binds = Vec::new();
                    if let Some(bind_arr) = group.get("binds").and_then(|b| b.as_array()) {
                        for b in bind_arr {
                            let mesh_index =
                                b.get("mesh").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                            let target_index =
                                b.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                            let weight = (b.get("weight").and_then(|v| v.as_f64()).unwrap_or(100.0)
                                / 100.0) as f32;
                            binds.push(BlendShapeBind {
                                mesh_index,
                                primitive_index: 0,
                                target_index,
                                weight,
                            });
                        }
                    }
                    expressions.insert(name.clone(), VrmExpressionData { name, binds });
                }
            }
        }

        // VRM 1.0 Expressions (preset & custom)
        if let Some(vrm_1) = gltf_json
            .as_ref()
            .and_then(|j| j.get("extensions"))
            .and_then(|e| e.get("VRMC_vrm"))
        {
            if let Some(exprs_obj) = vrm_1.get("expressions").and_then(|e| e.as_object()) {
                for (_category, sub_val) in exprs_obj {
                    if let Some(obj) = sub_val.as_object() {
                        for (expr_name, expr_val) in obj {
                            let name = expr_name.to_lowercase();
                            let mut binds = Vec::new();
                            if let Some(morph_binds) =
                                expr_val.get("morphTargetBinds").and_then(|m| m.as_array())
                            {
                                for b in morph_binds {
                                    let node_idx =
                                        b.get("node").and_then(|v| v.as_u64()).unwrap_or(0)
                                            as usize;
                                    let mesh_index = nodes
                                        .iter()
                                        .find(|n| n.index == node_idx)
                                        .and_then(|n| n.mesh_index)
                                        .unwrap_or(0);
                                    let target_index =
                                        b.get("index").and_then(|v| v.as_u64()).unwrap_or(0)
                                            as usize;
                                    let weight =
                                        b.get("weight").and_then(|v| v.as_f64()).unwrap_or(1.0)
                                            as f32;
                                    binds.push(BlendShapeBind {
                                        mesh_index,
                                        primitive_index: 0,
                                        target_index,
                                        weight,
                                    });
                                }
                            }
                            expressions.insert(name.clone(), VrmExpressionData { name, binds });
                        }
                    }
                }
            }
        }

        let root_nodes = if let Some(scene) = document
            .default_scene()
            .or_else(|| document.scenes().next())
        {
            scene.nodes().map(|n| n.index()).collect()
        } else {
            (0..nodes.len()).collect()
        };

        // Precompute node_to_bone reverse mapping & known_mesh_indices for O(1) direct lookup when render loop
        let mut node_to_bone = vec![None; nodes.len()];
        for (bone_name, &node_idx) in &bone_mapping {
            if node_idx < node_to_bone.len() {
                node_to_bone[node_idx] = Some(bone_name.clone());
            }
        }
        let known_mesh_indices: Vec<usize> = primitives.iter().map(|p| p.mesh_index).collect();

        // Precompute parent indices and rest world rotations for Three-VRM Normalized Rig
        let mut parent_indices = vec![None; nodes.len()];
        for (i, node) in nodes.iter().enumerate() {
            for &child in &node.children {
                if child < parent_indices.len() {
                    parent_indices[child] = Some(i);
                }
            }
        }

        let mut rest_world_rotations = vec![Quat::IDENTITY; nodes.len()];
        for &root_idx in &root_nodes {
            if root_idx < nodes.len() {
                compute_rest_world_rotations(
                    root_idx,
                    Quat::IDENTITY,
                    &nodes,
                    &mut rest_world_rotations,
                );
            }
        }

        let mut parent_rest_world_rotations = vec![Quat::IDENTITY; nodes.len()];
        for i in 0..nodes.len() {
            if let Some(parent_idx) = parent_indices[i] {
                parent_rest_world_rotations[i] = rest_world_rotations[parent_idx];
            }
        }

        let spring_bone_manager = gltf_json
            .as_ref()
            .and_then(|j| crate::ui::spring_bone::SpringBoneManager::from_gltf_json(j, &nodes))
            .map(std::sync::Mutex::new);

        println!(
            "[VRM Loader] Successfully loaded {} mesh parts, {} material layers, {} blend expressions",
            primitives.len(),
            materials.len(),
            expressions.len()
        );

        Ok(Self {
            is_vrm_0,
            primitives,
            materials,
            nodes,
            root_nodes,
            skins,
            bone_mapping,
            node_to_bone,
            parent_indices,
            parent_rest_world_rotations,
            spring_bone_manager,
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
    } else if name_str.contains("LeftUpperArm")
        || name_str.contains("L_UpperArm")
        || name_str == "leftUpperArm"
    {
        Some("leftUpperArm")
    } else if name_str.contains("LeftLowerArm")
        || name_str.contains("L_LowerArm")
        || name_str == "leftLowerArm"
    {
        Some("leftLowerArm")
    } else if name_str.contains("LeftHand") || name_str.contains("L_Hand") || name_str == "leftHand"
    {
        Some("leftHand")
    } else if name_str.contains("L_Thumb1")
        || name_str.contains("LeftThumbProximal")
        || name_str == "leftThumb"
    {
        Some("leftThumb")
    } else if name_str.contains("L_Index1")
        || name_str.contains("LeftIndexProximal")
        || name_str == "leftIndex"
    {
        Some("leftIndex")
    } else if name_str.contains("L_Middle1")
        || name_str.contains("LeftMiddleProximal")
        || name_str == "leftMiddle"
    {
        Some("leftMiddle")
    } else if name_str.contains("L_Ring1")
        || name_str.contains("LeftRingProximal")
        || name_str == "leftRing"
    {
        Some("leftRing")
    } else if name_str.contains("L_Little1")
        || name_str.contains("LeftLittleProximal")
        || name_str == "leftLittle"
    {
        Some("leftLittle")
    } else if name_str.contains("RightUpperArm")
        || name_str.contains("R_UpperArm")
        || name_str == "rightUpperArm"
    {
        Some("rightUpperArm")
    } else if name_str.contains("RightLowerArm")
        || name_str.contains("R_LowerArm")
        || name_str == "rightLowerArm"
    {
        Some("rightLowerArm")
    } else if name_str.contains("RightHand")
        || name_str.contains("R_Hand")
        || name_str == "rightHand"
    {
        Some("rightHand")
    } else if name_str.contains("R_Thumb1")
        || name_str.contains("RightThumbProximal")
        || name_str == "rightThumb"
    {
        Some("rightThumb")
    } else if name_str.contains("R_Index1")
        || name_str.contains("RightIndexProximal")
        || name_str == "rightIndex"
    {
        Some("rightIndex")
    } else if name_str.contains("R_Middle1")
        || name_str.contains("RightMiddleProximal")
        || name_str == "rightMiddle"
    {
        Some("rightMiddle")
    } else if name_str.contains("R_Ring1")
        || name_str.contains("RightRingProximal")
        || name_str == "rightRing"
    {
        Some("rightRing")
    } else if name_str.contains("R_Little1")
        || name_str.contains("RightLittleProximal")
        || name_str == "rightLittle"
    {
        Some("rightLittle")
    } else if name_str.contains("LeftShoulder")
        || name_str.contains("L_Shoulder")
        || name_str == "leftShoulder"
    {
        Some("leftShoulder")
    } else if name_str.contains("RightShoulder")
        || name_str.contains("R_Shoulder")
        || name_str == "rightShoulder"
    {
        Some("rightShoulder")
    } else if name_str.contains("LeftUpperLeg")
        || name_str.contains("L_UpperLeg")
        || name_str == "leftUpperLeg"
    {
        Some("leftUpperLeg")
    } else if name_str.contains("LeftLowerLeg")
        || name_str.contains("L_LowerLeg")
        || name_str == "leftLowerLeg"
    {
        Some("leftLowerLeg")
    } else if name_str.contains("LeftFoot") || name_str.contains("L_Foot") || name_str == "leftFoot"
    {
        Some("leftFoot")
    } else if name_str.contains("RightUpperLeg")
        || name_str.contains("R_UpperLeg")
        || name_str == "rightUpperLeg"
    {
        Some("rightUpperLeg")
    } else if name_str.contains("RightLowerLeg")
        || name_str.contains("R_LowerLeg")
        || name_str == "rightLowerLeg"
    {
        Some("rightLowerLeg")
    } else if name_str.contains("RightFoot")
        || name_str.contains("R_Foot")
        || name_str == "rightFoot"
    {
        Some("rightFoot")
    } else {
        None
    }
}
