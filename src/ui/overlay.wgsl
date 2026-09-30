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
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_overlay(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Fullscreen quad via single oversized triangle (0..3)
    let x = f32(i32(in_vertex_index == 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index == 2u) * 4 - 1);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Vector SDF Helper: Jarak ke Garis
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

    let pixel_pos = in.clip_position.xy;
    let w = overlay.window_size.x;
    let h = overlay.window_size.y;
    let cur = overlay.cursor_pos;

    // Koordinat pusat ketiga tombol kontrol jendela (Header 28px)
    let close_c = vec2<f32>(w - 16.0, 14.0);
    let max_c   = vec2<f32>(w - 44.0, 14.0);
    let min_c   = vec2<f32>(w - 72.0, 14.0);

    let btn_half = vec2<f32>(11.0, 9.5);
    let btn_radius = 0.0; // Tegak lurus tajam tanpa border-radius

    // Deteksi Hover kursor mouse
    let hover_close = abs(cur.x - close_c.x) <= 12.0 && abs(cur.y - close_c.y) <= 11.0;
    let hover_max   = abs(cur.x - max_c.x)   <= 12.0 && abs(cur.y - max_c.y)   <= 11.0;
    let hover_min   = abs(cur.x - min_c.x)   <= 12.0 && abs(cur.y - min_c.y)   <= 11.0;

    // ── 1. Tombol Close (✕) ──
    let p_close = pixel_pos - close_c;
    let d_bg_close = sdf_rounded_box(p_close, btn_half, btn_radius);
    let aa_bg_close = smoothstep(0.75, -0.75, d_bg_close);

    let d_x1 = sdf_line(p_close, vec2<f32>(-3.8, -3.8), vec2<f32>(3.8, 3.8));
    let d_x2 = sdf_line(p_close, vec2<f32>(-3.8, 3.8), vec2<f32>(3.8, -3.8));
    let d_cross = min(d_x1, d_x2) - 0.85;
    let aa_icon_close = smoothstep(0.75, -0.75, d_cross);

    if (aa_bg_close > 0.001) {
        let bg_col = select(
            vec3<f32>(0.831, 0.784, 0.686), // Default: Warm Earth Tan (#D4C8AF)
            vec3<f32>(0.75, 0.22, 0.22),     // Hover: Vibrant Terracotta Red (#C03838)
            hover_close
        );
        let icon_col = select(
            vec3<f32>(0.106, 0.176, 0.141), // Default: Deep Forest Green (#1B2D24)
            vec3<f32>(1.0, 1.0, 1.0),       // Hover: Solid White
            hover_close
        );
        let final_rgb = mix(bg_col, icon_col, aa_icon_close);
        return vec4<f32>(final_rgb, aa_bg_close);
    }

    // ── 2. Tombol Maximize (⛶) ──
    let p_max = pixel_pos - max_c;
    let d_bg_max = sdf_rounded_box(p_max, btn_half, btn_radius);
    let aa_bg_max = smoothstep(0.75, -0.75, d_bg_max);

    let d_max_outer = sdf_rounded_box(p_max, vec2<f32>(4.5, 4.5), 0.0);
    let d_max_inner = sdf_rounded_box(p_max, vec2<f32>(3.1, 3.1), 0.0);
    let d_max_box = max(d_max_outer, -d_max_inner);
    let aa_icon_max = smoothstep(0.75, -0.75, d_max_box);

    if (aa_bg_max > 0.001) {
        let bg_col = select(
            vec3<f32>(0.831, 0.784, 0.686), // Default: Warm Earth Tan (#D4C8AF)
            vec3<f32>(0.29, 0.42, 0.24),     // Hover: Moss Accent Green (#4A6B3E)
            hover_max
        );
        let icon_col = select(
            vec3<f32>(0.106, 0.176, 0.141), // Default: Deep Forest Green (#1B2D24)
            vec3<f32>(1.0, 1.0, 1.0),       // Hover: Solid White
            hover_max
        );
        let final_rgb = mix(bg_col, icon_col, aa_icon_max);
        return vec4<f32>(final_rgb, aa_bg_max);
    }

    // ── 3. Tombol Minimize (—) ──
    let p_min = pixel_pos - min_c;
    let d_bg_min = sdf_rounded_box(p_min, btn_half, btn_radius);
    let aa_bg_min = smoothstep(0.75, -0.75, d_bg_min);

    let d_min_line = sdf_line(p_min, vec2<f32>(-4.5, 0.5), vec2<f32>(4.5, 0.5)) - 0.85;
    let aa_icon_min = smoothstep(0.75, -0.75, d_min_line);

    if (aa_bg_min > 0.001) {
        let bg_col = select(
            vec3<f32>(0.831, 0.784, 0.686), // Default: Warm Earth Tan (#D4C8AF)
            vec3<f32>(0.29, 0.42, 0.24),     // Hover: Moss Accent Green (#4A6B3E)
            hover_min
        );
        let icon_col = select(
            vec3<f32>(0.106, 0.176, 0.141), // Default: Deep Forest Green (#1B2D24)
            vec3<f32>(1.0, 1.0, 1.0),       // Hover: Solid White
            hover_min
        );
        let final_rgb = mix(bg_col, icon_col, aa_icon_min);
        return vec4<f32>(final_rgb, aa_bg_min);
    }

    // ── 4. Window Frame Border (Moss Accent #596B45) ──
    let border_thickness = 1.5;
    let min_dist_edge = min(min(pixel_pos.x, w - pixel_pos.x), min(pixel_pos.y, h - pixel_pos.y));
    let border_aa = smoothstep(border_thickness + 0.5, border_thickness - 0.5, min_dist_edge);
    if (border_aa > 0.01) {
        let moss_col = vec3<f32>(0.349, 0.420, 0.271); // Moss Accent (#596B45)
        return vec4<f32>(moss_col, 0.85 * border_aa);
    }

    discard;
}
