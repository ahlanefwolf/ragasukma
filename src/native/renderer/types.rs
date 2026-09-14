use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct BoneMatricesUniform {
    pub matrices: [[[f32; 4]; 4]; 256],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialUniform {
    pub base_color_factor: [f32; 4],
    pub shade_color_factor: [f32; 4], // rgb + 0.0
    pub emissive_factor: [f32; 4],    // rgb + 0.0
    pub matcap_factor: [f32; 4],      // rgb + 0.0
    pub shading_shift: f32,
    pub shading_toony: f32,
    pub alpha_cutoff: f32,
    pub has_emissive: u32,
    pub has_matcap: u32,
    pub has_shade_tex: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Avatar2dUniform {
    pub window_size: [f32; 2],
    pub image_size: [f32; 2],
    pub camera_pan: [f32; 2],
    pub camera_zoom: f32,
    pub breath_offset: f32,
    pub _pad0: f32,
    pub _pad1: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct OverlayUniform {
    pub window_size: [f32; 2],
    pub cursor_pos: [f32; 2],
    pub is_visible: f32,
    pub _pad0: f32,
    pub _pad1: f32,
    pub _pad2: f32,
}

pub struct GpuPrimitive {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub material_index: usize,
}

pub struct GpuMaterial {
    pub bind_group: wgpu::BindGroup,
}
