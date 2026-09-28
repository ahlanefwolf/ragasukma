struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) joints: vec4<u32>,
    @location(4) weights: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_pos: vec3<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct BoneMatrices {
    matrices: array<mat4x4<f32>, 256>,
}

@group(1) @binding(0)
var<uniform> bones: BoneMatrices;

struct MaterialUniform {
    base_color_factor: vec4<f32>,
    shade_color_factor: vec4<f32>, // rgb + padding
    emissive_factor: vec4<f32>,    // rgb + padding
    matcap_factor: vec4<f32>,      // rgb + padding
    shading_shift: f32,
    shading_toony: f32,
    alpha_cutoff: f32,
    has_emissive: u32,
    has_matcap: u32,
    has_shade_tex: u32,
    _pad0: u32,
    _pad1: u32,
}

@group(2) @binding(0)
var<uniform> material: MaterialUniform;

@group(2) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(2)
var t_base: texture_2d<f32>;

@group(2) @binding(3)
var t_shade: texture_2d<f32>;

@group(2) @binding(4)
var t_emissive: texture_2d<f32>;

@group(2) @binding(5)
var t_matcap: texture_2d<f32>;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Linear Blend Skinning (LBS)
    var skin_matrix: mat4x4<f32> = 
        in.weights.x * bones.matrices[in.joints.x] +
        in.weights.y * bones.matrices[in.joints.y] +
        in.weights.z * bones.matrices[in.joints.z] +
        in.weights.w * bones.matrices[in.joints.w];

    let weight_sum = in.weights.x + in.weights.y + in.weights.z + in.weights.w;
    if (weight_sum < 0.001) {
        skin_matrix = mat4x4<f32>(
            vec4<f32>(1.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 1.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 1.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 1.0)
        );
    }

    let local_pos = vec4<f32>(in.position, 1.0);
    let world_pos = skin_matrix * local_pos;
    
    // Normal transform
    let normal_mat = mat3x3<f32>(
        skin_matrix[0].xyz,
        skin_matrix[1].xyz,
        skin_matrix[2].xyz,
    );
    let world_normal = normalize(normal_mat * in.normal);

    out.uv = in.uv;
    out.world_normal = world_normal;
    out.world_pos = world_pos.xyz;
    out.clip_position = camera.view_proj * world_pos;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Sample Base Color
    let base_sample = textureSample(t_base, s_diffuse, in.uv);
    let lit_color = base_sample.rgb * material.base_color_factor.rgb;
    let alpha = base_sample.a * material.base_color_factor.a;

    // 2. Alpha Cutoff
    if (alpha < material.alpha_cutoff) {
        discard;
    }

    // 3. Three.js MToon Shading Formulation (1:1 @pixiv/three-vrm)
    // Key Light Direction yang lembut dari depan kamera (sedikit miring ke atas-kanan)
    let light_dir = normalize(vec3<f32>(0.2, 0.6, 1.0));
    let n = normalize(in.world_normal);
    let dotNL = clamp(dot(n, light_dir), -1.0, 1.0);

    // MToon Toony Step
    let toony_min = -1.0 + material.shading_toony;
    let toony_max = 1.0 - material.shading_toony;
    let raw_shading = dotNL + material.shading_shift;
    let shading = clamp((raw_shading - toony_min) / max(toony_max - toony_min, 0.0001), 0.0, 1.0);

    // Shade Color (Sample Shade Texture & Shade Color Factor)
    var shade_color: vec3<f32>;
    if (material.has_shade_tex == 1u) {
        let shade_sample = textureSample(t_shade, s_diffuse, in.uv).rgb;
        shade_color = shade_sample * material.shade_color_factor.rgb * material.base_color_factor.rgb;
    } else {
        shade_color = lit_color * material.shade_color_factor.rgb;
    }

    // Interpolasi halus antara Shade dan Lit
    var final_rgb = mix(shade_color, lit_color, shading);

    // 4. Three.js MToon MatCap (Spherical UV reflection) & Parametric Rim Lighting
    let view_dir = normalize(camera.camera_pos - in.world_pos);

    // 4a. MatCap
    var matcap_rgb = vec3<f32>(0.0);
    if (material.has_matcap == 1u) {
        var x = vec3<f32>(view_dir.z, 0.0, -view_dir.x);
        let len_x = length(x);
        if (len_x > 0.0001) {
            x = x / len_x;
        } else {
            x = vec3<f32>(1.0, 0.0, 0.0);
        }
        let y = cross(view_dir, x);
        let sphere_uv = vec2<f32>(
            0.5 + 0.5 * dot(x, n),
            0.5 - 0.5 * dot(y, n)
        );
        let matcap_sample = textureSample(t_matcap, s_diffuse, sphere_uv).rgb;
        matcap_rgb = material.matcap_factor.rgb * matcap_sample;
    }

    // 4b. Parametric Rim Lighting (Fresnel anime highlight pada siluet model)
    let v_dot_n = clamp(dot(view_dir, n), 0.0, 1.0);
    let rim_fresnel = pow(clamp(1.0 - v_dot_n, 0.0, 1.0), 4.5);
    let rim_dir_factor = clamp(dotNL * 0.5 + 0.5, 0.0, 1.0);
    let rim_light = lit_color * (rim_fresnel * rim_dir_factor * 0.65);

    // Gabungkan Shading + Rim + MatCap
    final_rgb = final_rgb + rim_light + matcap_rgb;

    // 5. Emissive Texture (Warna Asli Rambut & Glow VRoid)
    if (material.has_emissive == 1u) {
        let em_sample = textureSample(t_emissive, s_diffuse, in.uv).rgb;
        let em_color = em_sample * material.emissive_factor.rgb;
        final_rgb = final_rgb + em_color;
    }

    return vec4<f32>(final_rgb, alpha);
}
