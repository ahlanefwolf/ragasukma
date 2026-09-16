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

/// Menghitung bobot blend shapes untuk 7+ ekspresi emosi utama.
/// Jika model VRM tidak memiliki blend shape khusus (misal "shy" atau "thinking"),
/// sistem otomatis menyusun procedural blend dari blend shape dasar (joy, blink, wink, sorrow, dll)
/// sehingga SETIAP emosi DIJAMIN memiliki tampilan visual yang khas, unik, dan berbeda.
pub fn resolve_emotion_layers<'a>(
    expressions: &'a HashMap<String, VrmExpressionData>,
    emotion: &str,
) -> Vec<ActiveExpressionLayer<'a>> {
    let mut layers = Vec::new();
    let emo = emotion.to_lowercase();

    match emo.as_str() {
        "happy" => {
            // Senyum ceria penuh
            if let Some(expr) = find_first_matching(
                expressions,
                &["happy", "joy", "smile", "fcl_mth_smile", "fcl_eye_joy"],
            ) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            }
        }
        "shy" => {
            // Tersipu / malu: Prioritas blend shape blush/shy.
            // Jika tidak ada, padukan senyum lembut (0.45) + kedip sebagian (0.35)
            if let Some(expr) = find_first_matching(
                expressions,
                &[
                    "shy",
                    "blush",
                    "tere",
                    "tersipu",
                    "fcl_all_shy",
                    "fcl_all_blush",
                ],
            ) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            } else {
                if let Some(joy) = find_first_matching(expressions, &["happy", "joy", "smile"]) {
                    layers.push(ActiveExpressionLayer {
                        expr: joy,
                        weight: 0.40,
                    });
                }
                if let Some(blink) =
                    find_first_matching(expressions, &["blink", "eyeblink", "fcl_eye_close"])
                {
                    layers.push(ActiveExpressionLayer {
                        expr: blink,
                        weight: 0.30,
                    });
                }
            }
        }
        "surprised" => {
            // Terkejut / kaget: Mata membelalak lebar + mulut 'O'
            if let Some(expr) = find_first_matching(
                expressions,
                &[
                    "surprised",
                    "surprise",
                    "odoroki",
                    "shock",
                    "fcl_eye_surprised",
                ],
            ) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            } else if let Some(oh) = find_first_matching(expressions, &["oh", "o", "fcl_mth_o"]) {
                layers.push(ActiveExpressionLayer {
                    expr: oh,
                    weight: 0.60,
                });
            }
        }
        "sad" => {
            // Sedih / haru: Alis turun + sudut bibir melengkung ke bawah
            if let Some(expr) = find_first_matching(
                expressions,
                &["sad", "sorrow", "kanashimi", "cry", "fcl_eye_sad"],
            ) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            }
        }
        "thinking" => {
            // Berpikir / heran: Alis berkerut / ekspresi ragu.
            // Jika tidak ada preset khusus, padukan senyum kecil (0.2) + bibir 'U' (0.35)
            if let Some(expr) = find_first_matching(
                expressions,
                &["thinking", "confused", "doubt", "mikir", "fcl_all_thinking"],
            ) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.80 });
            } else {
                if let Some(u_mouth) = find_first_matching(expressions, &["ou", "u", "fcl_mth_u"]) {
                    layers.push(ActiveExpressionLayer {
                        expr: u_mouth,
                        weight: 0.35,
                    });
                }
                if let Some(sad) = find_first_matching(expressions, &["sad", "sorrow"]) {
                    layers.push(ActiveExpressionLayer {
                        expr: sad,
                        weight: 0.20,
                    });
                }
            }
        }
        "excited" => {
            // Antusias / bersemangat: Kedip sebelah (wink) + senyum gembira
            if let Some(expr) =
                find_first_matching(expressions, &["excited", "cheer", "fcl_all_excited"])
            {
                layers.push(ActiveExpressionLayer { expr, weight: 0.90 });
            } else {
                if let Some(wink) = find_first_matching(
                    expressions,
                    &["wink", "wink_l", "wink_r", "blink_l", "fcl_eye_wink_l"],
                ) {
                    layers.push(ActiveExpressionLayer {
                        expr: wink,
                        weight: 0.85,
                    });
                }
                if let Some(joy) = find_first_matching(expressions, &["happy", "joy", "smile"]) {
                    layers.push(ActiveExpressionLayer {
                        expr: joy,
                        weight: 0.70,
                    });
                }
            }
        }
        "angry" => {
            // Kesal / marah: Alis menukik ke bawah + tatapan tajam
            if let Some(expr) =
                find_first_matching(expressions, &["angry", "ikari", "fcl_eye_angry"])
            {
                layers.push(ActiveExpressionLayer { expr, weight: 0.85 });
            }
        }
        "neutral" | "" => {
            // Netral: Kosong (wajah rileks alami 100% base vertices)
        }
        _ => {
            // Fallback: Coba cocokkan nama emosi apa pun secara langsung
            if let Some(expr) = find_expression(expressions, &emo) {
                layers.push(ActiveExpressionLayer { expr, weight: 0.75 });
            }
        }
    }

    layers
}

/// Ekstrak emosi dari teks mentah (sebelum atau sesudah dibersihkan).
/// Mendukung tag eksplisit seperti `*senang*`, `*malu*`, `[EMO:happy]`, `[happy]`
/// serta fallback deteksi kata kunci natural.
pub fn detect_emotion(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();

    // 1. Tag eksplisit format bintang (*senang*, *malu*, dll) atau kurung siku ([EMO:happy], [senang])
    if lower.contains("*senang*")
        || lower.contains("*happy*")
        || lower.contains("*gembira*")
        || lower.contains("*ceria*")
        || lower.contains("*senyum*")
        || lower.contains("[emo:happy]")
        || lower.contains("[emo:senang]")
        || lower.contains("[happy]")
        || lower.contains("[senang]")
    {
        return Some("happy");
    }
    if lower.contains("*malu*")
        || lower.contains("*shy*")
        || lower.contains("*blush*")
        || lower.contains("*tersipu*")
        || lower.contains("*manja*")
        || lower.contains("[emo:shy]")
        || lower.contains("[emo:malu]")
        || lower.contains("[shy]")
        || lower.contains("[malu]")
    {
        return Some("shy");
    }
    if lower.contains("*marah*")
        || lower.contains("*angry*")
        || lower.contains("*kesal*")
        || lower.contains("*ngambek*")
        || lower.contains("*judes*")
        || lower.contains("*protes*")
        || lower.contains("[emo:angry]")
        || lower.contains("[emo:marah]")
        || lower.contains("[angry]")
        || lower.contains("[marah]")
    {
        return Some("angry");
    }
    if lower.contains("*semangat*")
        || lower.contains("*excited*")
        || lower.contains("*antusias*")
        || lower.contains("*heboh*")
        || lower.contains("[emo:excited]")
        || lower.contains("[emo:semangat]")
        || lower.contains("[excited]")
        || lower.contains("[semangat]")
    {
        return Some("excited");
    }
    if lower.contains("*kaget*")
        || lower.contains("*surprised*")
        || lower.contains("*terkejut*")
        || lower.contains("*takjub*")
        || lower.contains("*heran*")
        || lower.contains("[emo:surprised]")
        || lower.contains("[emo:kaget]")
        || lower.contains("[surprised]")
        || lower.contains("[kaget]")
    {
        return Some("surprised");
    }
    if lower.contains("*mikir*")
        || lower.contains("*thinking*")
        || lower.contains("*bingung*")
        || lower.contains("*confused*")
        || lower.contains("*penasaran*")
        || lower.contains("[emo:thinking]")
        || lower.contains("[emo:mikir]")
        || lower.contains("[thinking]")
        || lower.contains("[mikir]")
    {
        return Some("thinking");
    }
    if lower.contains("*sedih*")
        || lower.contains("*sad*")
        || lower.contains("*kecewa*")
        || lower.contains("*prihatin*")
        || lower.contains("*galau*")
        || lower.contains("[emo:sad]")
        || lower.contains("[emo:sedih]")
        || lower.contains("[sad]")
        || lower.contains("[sedih]")
    {
        return Some("sad");
    }
    if lower.contains("*netral*")
        || lower.contains("*neutral*")
        || lower.contains("*biasa*")
        || lower.contains("[emo:neutral]")
        || lower.contains("[emo:netral]")
        || lower.contains("[neutral]")
        || lower.contains("[netral]")
    {
        return Some("neutral");
    }

    // 2. Keyword kontekstual natural jika tidak ada tag khusus
    if lower.contains("tersipu") || lower.contains("pipi merah") || lower.contains("malu-malu") {
        return Some("shy");
    }
    if lower.contains("kesal")
        || lower.contains("ngambek")
        || lower.contains("marah")
        || lower.contains("jangan gitu dong")
        || lower.contains("ihh")
    {
        return Some("angry");
    }
    if lower.contains("senang")
        || lower.contains("gembira")
        || lower.contains("alhamdulillah")
        || lower.contains("suka banget")
        || lower.contains("makasih")
        || lower.contains("terima kasih")
    {
        return Some("happy");
    }
    if lower.contains("semangat")
        || lower.contains("siap ")
        || lower.contains("tentu saja")
        || lower.contains("mantap")
        || lower.contains("hebat")
    {
        return Some("excited");
    }
    if lower.contains("kaget")
        || lower.contains("astaga")
        || lower.contains("terkejut")
        || lower.contains("beneran nih")
        || lower.contains("masa sih")
        || lower.contains("wah ")
    {
        return Some("surprised");
    }
    if lower.contains("bingung")
        || lower.contains("kurang paham")
        || lower.contains("hmm")
        || lower.contains("sebentar")
    {
        return Some("thinking");
    }
    if lower.contains("sedih")
        || lower.contains("kecewa")
        || lower.contains("maaf")
        || lower.contains("sayang sekali")
    {
        return Some("sad");
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
    let mut active_weights: Vec<(&VrmExpressionData, f32)> =
        emotion_layers.iter().map(|l| (l.expr, l.weight)).collect();

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

    // 3. Lip Sync Vokal Vowel (A, I, U, E, O) saat Speaking
    if current_state == AssistantVisualState::Speaking && !is_tuning {
        let vowels = ["a", "i", "u", "e", "o"];
        let vowel_idx = ((elapsed_time * 3.0) as usize) % vowels.len();
        let target_vowel = vowels[vowel_idx];

        let vowel_aliases: &[&str] = match target_vowel {
            "a" => &["aa", "a", "viseme_a", "preset_a", "fcl_mth_a"],
            "i" => &["ih", "i", "viseme_i", "preset_i", "fcl_mth_i"],
            "u" => &["ou", "u", "viseme_u", "preset_u", "fcl_mth_u"],
            "e" => &["ee", "e", "viseme_e", "preset_e", "fcl_mth_e"],
            "o" => &["oh", "o", "viseme_o", "preset_o", "fcl_mth_o"],
            _ => &[],
        };

        let vowel_mouth_open = ((elapsed_time * 5.5).sin().abs()).powf(1.4) * 0.60;
        if vowel_mouth_open > 0.04 {
            if let Some(expr) = find_first_matching(&model.expressions, vowel_aliases) {
                active_weights.push((expr, vowel_mouth_open));
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

        // Terapkan seluruh layer bobot morph yang aktif
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
                if target_idx < prim.morph_targets.len() {
                    let morph = &prim.morph_targets[target_idx];
                    let effective_weight = expr_weight * bind.weight;
                    let count = vert_count.min(morph.position_displacements.len());
                    for i in 0..count {
                        let disp = morph.position_displacements[i];
                        prim.vertices[i].position[0] += disp[0] * effective_weight;
                        prim.vertices[i].position[1] += disp[1] * effective_weight;
                        prim.vertices[i].position[2] += disp[2] * effective_weight;
                    }
                }
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
