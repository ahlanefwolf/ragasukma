struct OverlayUniform {
    window_size: vec2<f32>,
    cursor_pos: vec2<f32>,
    is_visible: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

@group(0) @binding(0)
var<uniform> overlay: OverlayUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_overlay(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    var x = -1.0;
    var y = -1.0;
    if (in_vertex_index == 1u) {
        x = 3.0;
        y = -1.0;
    } else if (in_vertex_index == 2u) {
        x = -1.0;
        y = 3.0;
    }
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Vector SDF Helper: Jarak ke segmen garis (Capsule Line)
fn sdf_line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// Vector SDF Helper: Jarak ke Rounded Box
fn sdf_rounded_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r;
}

@fragment
fn fs_overlay(in: VertexOutput) -> @location(0) vec4<f32> {
    if (overlay.is_visible < 0.5) {
        discard;
    }

    let pixel_pos = in.uv * overlay.window_size;
    let w = overlay.window_size.x;
    let h = overlay.window_size.y;
    let cur = overlay.cursor_pos;

    let is_header = pixel_pos.y <= 28.0;

    // Koordinat pusat ketiga tombol
    let close_c = vec2<f32>(w - 16.0, 14.0);
    let max_c   = vec2<f32>(w - 44.0, 14.0);
    let min_c   = vec2<f32>(w - 72.0, 14.0);

    // Deteksi Hover
    let hover_close = abs(cur.x - close_c.x) <= 12.0 && abs(cur.y - close_c.y) <= 11.0;
    let hover_max   = abs(cur.x - max_c.x)   <= 12.0 && abs(cur.y - max_c.y)   <= 11.0;
    let hover_min   = abs(cur.x - min_c.x)   <= 12.0 && abs(cur.y - min_c.y)   <= 11.0;

    // Vector SDF Close (✕)
    let p_close = pixel_pos - close_c;
    let d_x1 = sdf_line(p_close, vec2<f32>(-3.8, -3.8), vec2<f32>(3.8, 3.8));
    let d_x2 = sdf_line(p_close, vec2<f32>(-3.8, 3.8), vec2<f32>(3.8, -3.8));
    let d_cross = min(d_x1, d_x2) - 0.75;
    let aa_close = clamp(0.5 - d_cross, 0.0, 1.0);

    // Vector SDF Maximize (⛶)
    let p_max = pixel_pos - max_c;
    let d_max_outer = sdf_rounded_box(p_max, vec2<f32>(4.6, 4.6), 1.2);
    let d_max_inner = sdf_rounded_box(p_max, vec2<f32>(3.2, 3.2), 0.6);
    let d_max_box = max(d_max_outer, -d_max_inner);
    let aa_max = clamp(0.5 - d_max_box, 0.0, 1.0);

    // Vector SDF Minimize (—)
    let p_min = pixel_pos - min_c;
    let d_min_line = sdf_line(p_min, vec2<f32>(-4.2, 1.0), vec2<f32>(4.2, 1.0)) - 0.75;
    let aa_min = clamp(0.5 - d_min_line, 0.0, 1.0);

    // Render Ikon (Forest Green saat default, Solid White saat hover)
    if (aa_close > 0.05) {
        let icon_col = select(vec3<f32>(0.161, 0.282, 0.227), vec3<f32>(1.0, 1.0, 1.0), hover_close);
        return vec4<f32>(icon_col, aa_close);
    }
    if (aa_max > 0.05) {
        let icon_col = select(vec3<f32>(0.161, 0.282, 0.227), vec3<f32>(1.0, 1.0, 1.0), hover_max);
        return vec4<f32>(icon_col, aa_max);
    }
    if (aa_min > 0.05) {
        let icon_col = select(vec3<f32>(0.161, 0.282, 0.227), vec3<f32>(1.0, 1.0, 1.0), hover_min);
        return vec4<f32>(icon_col, aa_min);
    }

    // Background Tombol (Solid, Kontras Jelas dengan Tema Earth Tone)
    let d_bg_close = sdf_rounded_box(p_close, vec2<f32>(11.0, 9.5), 3.5);
    if (d_bg_close < 0.0) {
        if (hover_close) {
            return vec4<f32>(0.659, 0.294, 0.294, 1.0); // Solid Terracotta Earth Red (#A84B4B)
        }
        return vec4<f32>(0.847, 0.808, 0.710, 1.0); // Solid Warm Cream (#D8CEB5)
    }

    let d_bg_max = sdf_rounded_box(p_max, vec2<f32>(11.0, 9.5), 3.5);
    if (d_bg_max < 0.0) {
        if (hover_max) {
            return vec4<f32>(0.349, 0.420, 0.271, 1.0); // Solid Moss Accent (#596B45)
        }
        return vec4<f32>(0.847, 0.808, 0.710, 1.0); // Solid Warm Cream (#D8CEB5)
    }

    let d_bg_min = sdf_rounded_box(p_min, vec2<f32>(11.0, 9.5), 3.5);
    if (d_bg_min < 0.0) {
        if (hover_min) {
            return vec4<f32>(0.349, 0.420, 0.271, 1.0); // Solid Moss Accent (#596B45)
        }
        return vec4<f32>(0.847, 0.808, 0.710, 1.0); // Solid Warm Cream (#D8CEB5)
    }

    // Header Bar Background (28px) - Warm Surface (#EBE4D5) dengan Moss Divider
    if (is_header) {
        if (pixel_pos.y >= 27.0) {
            return vec4<f32>(0.349, 0.420, 0.271, 0.75); // Moss Divider Line
        }
        return vec4<f32>(0.922, 0.894, 0.835, 0.92); // Warm Surface Header (#EBE4D5)
    }

    // Window frame border (Moss #596B45)
    let border_thickness = 1.5;
    let min_dist_edge = min(min(pixel_pos.x, w - pixel_pos.x), min(pixel_pos.y, h - pixel_pos.y));
    if (min_dist_edge < border_thickness) {
        return vec4<f32>(0.349, 0.420, 0.271, 0.85); // Moss Border
    }

    discard;
}
