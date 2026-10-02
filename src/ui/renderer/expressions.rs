use crate::ui::pose_system::AssistantVisualState;
use crate::ui::renderer::types::GpuPrimitive;
use crate::ui::vrm_loader::{VrmExpressionData, VrmModelData};
use std::collections::HashMap;

/// Pencari ekspresi di VRM berdasarkan nama/alias tanpa tumpang tindih
pub fn find_expression<'a>(
    expressions: &'a HashMap<String, VrmExpressionData>,
    target: &str,
) -> Option<&'a VrmExpressionData> {
    let target_low = target.to_lowercase();
    if let Some(expr) = expressions.get(&target_low) {
        return Some(expr);
    }

    // 1. Exact match case-insensitive
    for (k, v) in expressions {
        if k.eq_ignore_ascii_case(&target_low) {
            return Some(v);
        }
    }

    // 2. Substring match jika nama memiliki prefix seperti "fcl_all_happy", "vrm_joy", "preset_smile"
    for (k, v) in expressions {
        let k_low = k.to_lowercase();
        if k_low.contains(&target_low) {
            return Some(v);
        }
    }

    None
}

/// Cari ekspresi pertama yang cocok dari daftar prioritas alias
fn find_first_matching<'a>(
    expressions: &'a HashMap<String, VrmExpressionData>,
    aliases: &[&str],
) -> Option<&'a VrmExpressionData> {
    for alias in aliases {
        if let Some(expr) = find_expression(expressions, alias) {
            return Some(expr);
        }
    }
    None
}

/// Struktur komposisi bobot ekspresi yang aktif pada frame ini
pub struct ActiveExpressionLayer<'a> {
    pub expr: &'a VrmExpressionData,
    pub weight: f32,
}

/// Menghitung bobot blend shapes untuk 8 emosi kanonikal (VRM 1.0 standard Three.js).
pub fn resolve_emotion_layers<'a>(
    expressions: &'a HashMap<String, VrmExpressionData>,
    emotion: &str,
) -> Vec<ActiveExpressionLayer<'a>> {
    let mut layers = Vec::new();
    let emo = emotion.to_lowercase();

    match emo.as_str() {
        "happy" => {
            if let Some(expr) = find_expression(expressions, "happy") {
                layers.push(ActiveExpressionLayer { expr, weight: 0.75 });
            }
        }
        "shy" => {
            if let Some(joy) = find_expression(expressions, "happy") {
                layers.push(ActiveExpressionLayer {
                    expr: joy,
                    weight: 0.40,
                });
            }
            if let Some(blink) = find_expression(expressions, "blink") {
                layers.push(ActiveExpressionLayer {
                    expr: blink,
                    weight: 0.30,
                });
            }
        }
        "excited" => {
            if let Some(joy) = find_expression(expressions, "happy") {
                layers.push(ActiveExpressionLayer {
                    expr: joy,
                    weight: 0.75,
                });
            }
            if let Some(sparkle) = find_expression(expressions, "surprised") {
                layers.push(ActiveExpressionLayer {
                    expr: sparkle,
                    weight: 0.15,
                });
            }
        }
        "surprised" => {
            if let Some(expr) = find_expression(expressions, "surprised") {
                // Bobot proporsional agar ekspresi kaget tetap jelas tanpa merusak bentuk rahang
                layers.push(ActiveExpressionLayer { expr, weight: 0.50 });
            }
        }
        "thinking" => {
            if let Some(u_mouth) = find_first_matching(expressions, &["ou", "u"]) {
                layers.push(ActiveExpressionLayer {
                    expr: u_mouth,
                    weight: 0.35,
                });
            }
            if let Some(sad) = find_expression(expressions, "sad") {
                layers.push(ActiveExpressionLayer {
                    expr: sad,
                    weight: 0.20,
                });
            }
        }
        "sad" => {
            if let Some(expr) = find_expression(expressions, "sad") {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            }
        }
        "angry" => {
            if let Some(expr) = find_expression(expressions, "angry") {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            }
        }
        "neutral" | "" => {
            // Netral: 100% base mesh tanpa morph
        }
        _ => {
            // Fallback jika nama emosi langsung cocok dengan nama morph
            if let Some(expr) = find_expression(expressions, &emo) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.75 });
            }
        }
    }

    layers
}

const CANONICAL_EMOTIONS: &[(&str, &str)] = &[
    ("*happy*", "happy"),
    ("[happy]", "happy"),
    ("*senang*", "happy"),
    ("[senang]", "happy"),
    ("*gembira*", "happy"),
    ("*shy*", "shy"),
    ("[shy]", "shy"),
    ("*malu*", "shy"),
    ("[malu]", "shy"),
    ("*malu-malu*", "shy"),
    ("*tersipu*", "shy"),
    ("*excited*", "excited"),
    ("[excited]", "excited"),
    ("*semangat*", "excited"),
    ("*antusias*", "excited"),
    ("*surprised*", "surprised"),
    ("[surprised]", "surprised"),
    ("*kaget*", "surprised"),
    ("*terkejut*", "surprised"),
    ("*thinking*", "thinking"),
    ("[thinking]", "thinking"),
    ("*mikir*", "thinking"),
    ("*berpikir*", "thinking"),
    ("*bingung*", "thinking"),
    ("*sad*", "sad"),
    ("[sad]", "sad"),
    ("*sedih*", "sad"),
    ("*kecewa*", "sad"),
    ("*angry*", "angry"),
    ("[angry]", "angry"),
    ("*marah*", "angry"),
    ("*kesal*", "angry"),
    ("*neutral*", "neutral"),
    ("[neutral]", "neutral"),
    ("*biasa*", "neutral"),
    ("*santai*", "neutral"),
];

/// Ekstrak emosi dari tag eksplisit standar AI (*happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, *neutral*, dll.)
pub fn detect_emotion(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    for &(tag, emo) in CANONICAL_EMOTIONS {
        if lower.contains(tag) {
            return Some(emo);
        }
    }
    None
}

/// Ekstrak emosi dengan fallback default ke "neutral" jika tidak terdeteksi
pub fn detect_emotion_with_fallback(text: &str) -> &'static str {
    detect_emotion(text).unwrap_or("neutral")
}

/// Realtime Facial Expressions, Natural Eye Blinking & Procedural Lip Sync
pub fn update_facial_expressions_and_lip_sync(
    queue: &wgpu::Queue,
    model: &mut VrmModelData,
    gpu_primitives: &[GpuPrimitive],
    current_state: AssistantVisualState,
    current_emotion: &str,
    elapsed_time: f32,
    is_tuning: bool,
) {
    if model.primitives.is_empty() {
        return;
    }

    // 1. Hitung layer emosi utama
    let emotion_layers = resolve_emotion_layers(&model.expressions, current_emotion);
    let mut active_weights: Vec<(&VrmExpressionData, f32)> = emotion_layers
        .iter()
        .map(|l| {
            // Saat berbicara (Speaking), redam emosi yang membuka rahang/mulut (surprised)
            // agar tidak terjadi efek dobel (combo mangap ekstrem) bersamaan dengan Lip Sync vokal
            let w = if current_state == AssistantVisualState::Speaking {
                let name = l.expr.name.to_lowercase();
                if name.contains("surprised") {
                    (l.weight * 0.30).min(0.20)
                } else if name.contains("happy") || name.contains("joy") {
                    (l.weight * 0.70).min(0.50)
                } else {
                    l.weight
                }
            } else {
                l.weight
            };
            (l.expr, w)
        })
        .collect();

    let has_strong_emotion = !active_weights.is_empty();

    // 2. Natural Eye Blinking (Kedipan mata santai alami)
    if !is_tuning {
        let blink_period = if current_state == AssistantVisualState::Speaking {
            6.2
        } else {
            5.2
        };
        let blink_phase = elapsed_time % blink_period;
        let blink_duration = 0.24; // Durasi kedip ~240ms
        if blink_phase < blink_duration {
            let t = blink_phase / blink_duration;
            let blink_damp = if has_strong_emotion { 0.6 } else { 1.0 };
            let blink_weight = (t * std::f32::consts::PI).sin().powf(1.5) * blink_damp;
            if blink_weight > 0.02 {
                if let Some(expr) = find_first_matching(
                    &model.expressions,
                    &["blink", "eyeblink", "blink_both", "fcl_eye_close"],
                ) {
                    active_weights.push((expr, blink_weight));
                }
            }
        }
    }

    // 3. Audio-Driven Lip Sync Vokal Vowel (A, I, U, E, O) saat Speaking
    if current_state == AssistantVisualState::Speaking && !is_tuning {
        let lip_weights = crate::audio::player::get_current_lip_sync_weights();

        if lip_weights.mouth_open > 0.015 {
            // Suara aktif dari speaker hardware: gunakan analisis frekuensi formant audio nyata
            let vowel_map: &[(&[&str], f32)] = &[
                (&["aa", "a", "viseme_a", "preset_a", "fcl_mth_a"], lip_weights.aa),
                (&["ih", "i", "viseme_i", "preset_i", "fcl_mth_i"], lip_weights.ih),
                (&["ou", "u", "viseme_u", "preset_u", "fcl_mth_u"], lip_weights.ou),
                (&["ee", "e", "viseme_e", "preset_e", "fcl_mth_e"], lip_weights.ee),
                (&["oh", "o", "viseme_o", "preset_o", "fcl_mth_o"], lip_weights.oh),
            ];

            for (aliases, weight) in vowel_map {
                if *weight > 0.02 {
                    if let Some(expr) = find_first_matching(&model.expressions, aliases) {
                        active_weights.push((expr, *weight));
                    }
                }
            }
        } else if !crate::audio::player::is_native_voice_playing() {
            // Fallback animasi artikulasi vokal natural saat audio dialihkan ke remote web client:
            let vowels = ["aa", "ih", "ou", "ee", "oh"];
            let vowel_idx = ((elapsed_time * 3.2) as usize) % vowels.len();
            let target_vowel = vowels[vowel_idx];

            let vowel_aliases: &[&str] = match target_vowel {
                "aa" => &["aa", "a", "viseme_a", "preset_a", "fcl_mth_a"],
                "ih" => &["ih", "i", "viseme_i", "preset_i", "fcl_mth_i"],
                "ou" => &["ou", "u", "viseme_u", "preset_u", "fcl_mth_u"],
                "ee" => &["ee", "e", "viseme_e", "preset_e", "fcl_mth_e"],
                "oh" => &["oh", "o", "viseme_o", "preset_o", "fcl_mth_o"],
                _ => &[],
            };

            let vowel_mouth_open = ((elapsed_time * 5.0).sin().abs()).powf(1.4) * 0.45;
            if vowel_mouth_open > 0.04 {
                if let Some(expr) = find_first_matching(&model.expressions, vowel_aliases) {
                    active_weights.push((expr, vowel_mouth_open));
                }
            }
        }
    }

    // 4. Update Posisi Vertex Morph Targets pada Mesh Wajah di GPU Buffer
    let known_mesh_indices = &model.known_mesh_indices;

    for (prim_idx, prim) in model.primitives.iter_mut().enumerate() {
        if prim.morph_targets.is_empty() || prim.base_vertices.is_empty() {
            continue;
        }

        let vert_count = prim.vertices.len().min(prim.base_vertices.len());
        // Selalu reset ke posisi dasar (base_vertices)
        for i in 0..vert_count {
            prim.vertices[i].position = prim.base_vertices[i].position;
        }

        // Kumpulkan total bobot per target_index agar morph tidak menumpuk melebihi batas aman (anti-distorsi rahang)
        let mut target_weights = vec![0.0f32; prim.morph_targets.len()];
        for &(expr_data, expr_weight) in &active_weights {
            if expr_weight <= 0.001 {
                continue;
            }
            for bind in &expr_data.binds {
                if bind.mesh_index != prim.mesh_index
                    && known_mesh_indices.contains(&bind.mesh_index)
                {
                    continue;
                }

                let target_idx = bind.target_index;
                if target_idx < target_weights.len() {
                    target_weights[target_idx] += expr_weight * bind.weight;
                }
            }
        }

        // Terapkan morph target dengan batas clamping estetis (maks 0.85) agar tidak over-stretch
        for (target_idx, &raw_w) in target_weights.iter().enumerate() {
            if raw_w <= 0.001 {
                continue;
            }
            let effective_weight = raw_w.min(0.85);
            let morph = &prim.morph_targets[target_idx];
            let count = vert_count.min(morph.position_displacements.len());
            for i in 0..count {
                let disp = morph.position_displacements[i];
                prim.vertices[i].position[0] += disp[0] * effective_weight;
                prim.vertices[i].position[1] += disp[1] * effective_weight;
                prim.vertices[i].position[2] += disp[2] * effective_weight;
            }
        }

        if prim_idx < gpu_primitives.len() {
            queue.write_buffer(
                &gpu_primitives[prim_idx].vertex_buffer,
                0,
                bytemuck::cast_slice(&prim.vertices),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_canonical_emotions() {
        assert_eq!(detect_emotion("*happy* hello!"), Some("happy"));
        assert_eq!(detect_emotion("[happy] hello!"), Some("happy"));
        assert_eq!(detect_emotion("*shy* oh don't say that..."), Some("shy"));
        assert_eq!(detect_emotion("*excited* let's go!"), Some("excited"));
        assert_eq!(detect_emotion("*surprised* really?"), Some("surprised"));
        assert_eq!(
            detect_emotion("*thinking* hmm give me a second..."),
            Some("thinking")
        );
        assert_eq!(detect_emotion("*sad* sorry about that..."), Some("sad"));
        assert_eq!(detect_emotion("*angry* stop that!"), Some("angry"));
        assert_eq!(
            detect_emotion("*neutral* here is the info."),
            Some("neutral")
        );
        assert_eq!(detect_emotion("*malu-malu* halo..."), Some("shy"));
        assert_eq!(detect_emotion("*senang* akhirnya selesai!"), Some("happy"));

        // Fallback or without tag
        assert_eq!(detect_emotion("hello there, how are you?"), None);
        assert_eq!(detect_emotion_with_fallback("hello"), "neutral");
    }

    #[test]
    fn test_resolve_emotion_layers() {
        let mut expressions = HashMap::new();
        expressions.insert(
            "happy".to_string(),
            VrmExpressionData {
                name: "happy".into(),
                binds: vec![],
            },
        );
        expressions.insert(
            "blink".to_string(),
            VrmExpressionData {
                name: "blink".into(),
                binds: vec![],
            },
        );
        expressions.insert(
            "blinkleft".to_string(),
            VrmExpressionData {
                name: "blinkleft".into(),
                binds: vec![],
            },
        );
        expressions.insert(
            "surprised".to_string(),
            VrmExpressionData {
                name: "surprised".into(),
                binds: vec![],
            },
        );
        expressions.insert(
            "sad".to_string(),
            VrmExpressionData {
                name: "sad".into(),
                binds: vec![],
            },
        );
        expressions.insert(
            "angry".to_string(),
            VrmExpressionData {
                name: "angry".into(),
                binds: vec![],
            },
        );
        expressions.insert(
            "ou".to_string(),
            VrmExpressionData {
                name: "ou".into(),
                binds: vec![],
            },
        );

        let happy_layers = resolve_emotion_layers(&expressions, "happy");
        assert_eq!(happy_layers.len(), 1);
        assert!((happy_layers[0].weight - 0.75).abs() < 1e-4);

        let shy_layers = resolve_emotion_layers(&expressions, "shy");
        assert_eq!(shy_layers.len(), 2);

        let excited_layers = resolve_emotion_layers(&expressions, "excited");
        assert_eq!(excited_layers.len(), 2);

        let neutral_layers = resolve_emotion_layers(&expressions, "neutral");
        assert_eq!(neutral_layers.len(), 0);
    }
}
