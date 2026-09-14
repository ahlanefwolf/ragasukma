struct Avatar2dUniform {
    window_size: vec2<f32>,
    image_size: vec2<f32>,
    camera_pan: vec2<f32>,
    camera_zoom: f32,
    breath_offset: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0)
var<uniform> u: Avatar2dUniform;

@group(1) @binding(0)
var t_avatar: texture_2d<f32>;
@group(1) @binding(1)
var s_avatar: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let win_w = u.window_size.x;
    let win_h = u.window_size.y;
    let img_w = u.image_size.x;
    let img_h = u.image_size.y;

    if (win_w <= 0.0 || win_h <= 0.0 || img_w <= 0.0 || img_h <= 0.0) {
        discard;
    }

    // Fit contain dalam ukuran window portrait avatar
    let scale_x = win_w / img_w;
    let scale_y = win_h / img_h;
    let base_scale = min(scale_x, scale_y) * 0.90 * u.camera_zoom;

    let draw_w = img_w * base_scale;
    let draw_h = img_h * base_scale;

    let center_x = win_w * 0.5 + u.camera_pan.x;
    let center_y = win_h * 0.52 + u.camera_pan.y + u.breath_offset;

    let min_x = center_x - draw_w * 0.5;
    let max_x = center_x + draw_w * 0.5;
    let min_y = center_y - draw_h * 0.5;
    let max_y = center_y + draw_h * 0.5;

    let pixel_pos = in.uv * u.window_size;

    if (pixel_pos.x < min_x || pixel_pos.x > max_x || pixel_pos.y < min_y || pixel_pos.y > max_y) {
        discard;
    }

    let tex_u = (pixel_pos.x - min_x) / draw_w;
    let tex_v = (pixel_pos.y - min_y) / draw_h;

    let color = textureSample(t_avatar, s_avatar, vec2<f32>(tex_u, tex_v));
    if (color.a < 0.02) {
        discard;
    }

    return vec4<f32>(color.rgb * color.a, color.a);
}
