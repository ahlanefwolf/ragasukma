use super::types::{BoneMatricesUniform, CameraUniform};
use crate::ui::pose_system::PoseManager;
use crate::ui::vrm_loader::VrmModelData;
use glam::{Mat4, Vec3};

pub struct CameraState {
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_zoom: f32,
    pub camera_pan_x: f32,
    pub camera_pan_y: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            camera_zoom: 1.0,
            camera_pan_x: 0.0,
            camera_pan_y: 0.0,
        }
    }
}

impl CameraState {
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.camera_yaw += delta_yaw;
        self.camera_pitch = (self.camera_pitch + delta_pitch).clamp(-1.2, 1.2);
    }

    pub fn pan(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        screen_h: f32,
        model_min_y: f32,
        model_max_y: f32,
    ) {
        let fov_rad = 28.0_f32.to_radians();
        let half_fov_tan = (fov_rad / 2.0).tan();
        let height = screen_h.max(1.0);
        let model_height = (model_max_y - model_min_y).max(1.2);
        let base_radius = (model_height * 1.12 / 2.0) / half_fov_tan;
        let current_radius = base_radius / self.camera_zoom.clamp(0.15, 6.0);

        let pan_factor = (2.0 * half_fov_tan * current_radius) / height;
        self.camera_pan_x -= delta_x * pan_factor;
        self.camera_pan_y += delta_y * pan_factor;
    }

    pub fn zoom(&mut self, scroll_delta: f32) {
        let zoom_factor = 1.0 + (scroll_delta * 0.08);
        self.camera_zoom = (self.camera_zoom * zoom_factor).clamp(0.2, 5.0);
    }

    pub fn reset(&mut self) {
        self.camera_yaw = 0.0;
        self.camera_pitch = 0.0;
        self.camera_zoom = 1.0;
        self.camera_pan_x = 0.0;
        self.camera_pan_y = 0.0;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_camera_and_bones(
    queue: &wgpu::Queue,
    camera_buffer: &wgpu::Buffer,
    bone_buffer: &wgpu::Buffer,
    model_opt: Option<&VrmModelData>,
    model_min_y: f32,
    model_max_y: f32,
    screen_width: f32,
    screen_height: f32,
    cam: &CameraState,
    pose_manager: &mut PoseManager,
    delta_seconds: f32,
    is_tuning: bool,
    global_transforms: &mut Vec<Mat4>,
    bone_uniform: &mut BoneMatricesUniform,
) {
    if !is_tuning {
        pose_manager.update_poses(delta_seconds);
    }

    let width = screen_width;
    let height = screen_height.max(1.0);
    let aspect = width / height;

    let fov_rad = 28.0_f32.to_radians();

    // 1. The actual center point and height of the model
    let center_y = (model_min_y + model_max_y) * 0.5;
    let model_height = (model_max_y - model_min_y).max(1.2);

    let safe_margin = 1.12_f32;
    let fit_height = model_height * safe_margin;

    let dist_v = (fit_height / 2.0) / (fov_rad / 2.0).tan();
    let radius = dist_v / cam.camera_zoom.clamp(0.15, 6.0);

    // 2. Standard RH Orbit Camera Transformation (Yaw & Pitch)
    let cos_pitch = cam.camera_pitch.cos();
    let sin_pitch = cam.camera_pitch.sin();
    let sin_yaw = cam.camera_yaw.sin();
    let cos_yaw = cam.camera_yaw.cos();

    let target = Vec3::new(cam.camera_pan_x, center_y + cam.camera_pan_y, 0.0);
    let camera_pos = Vec3::new(
        target.x + radius * cos_pitch * sin_yaw,
        target.y + radius * sin_pitch,
        target.z + radius * cos_pitch * cos_yaw,
    );
    let view = Mat4::look_at_rh(camera_pos, target, Vec3::Y);

    let near = 0.05_f32;
    let far = 50.0_f32;
    let proj = Mat4::perspective_rh(fov_rad, aspect, near, far);
    let view_proj = proj * view;

    let camera_uniform = CameraUniform {
        view_proj: view_proj.to_cols_array_2d(),
        camera_pos: camera_pos.to_array(),
        _padding: 0.0,
    };
    queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

    // Bone Matrix Update (Three.js Pose Hierarchy) - 0 Allocation
    if let Some(model) = model_opt {
        if global_transforms.len() != model.nodes.len() {
            global_transforms.resize(model.nodes.len(), Mat4::IDENTITY);
        } else {
            global_transforms.fill(Mat4::IDENTITY);
        }

        // VRM 0.X Coordinate Alignment:
        // In VRM 0.X (Unity standard), models naturally face towards -Z (away from camera / looking backwards in glTF).
        // VRM 1.0 models face towards +Z (directly facing camera).
        // Rotate the root model by 180 deg (PI) around Y axis so VRM 0.X faces forward towards the camera.
        let root_transform = if model.is_vrm_0 {
            Mat4::from_rotation_y(std::f32::consts::PI)
        } else {
            Mat4::IDENTITY
        };

        for &root_idx in &model.root_nodes {
            compute_global_transforms(
                root_idx,
                root_transform,
                pose_manager,
                model,
                global_transforms,
            );
        }

        // VRM SpringBone Verlet Physics (Hair, Bangs, Dynamic swaying skirt)
        if let Some(sb_mutex) = &model.spring_bone_manager {
            if let Ok(mut sb) = sb_mutex.lock() {
                sb.update(delta_seconds, model, global_transforms);
            }
        }

        if let Some(skin) = model.skins.first() {
            for (joint_i, &node_idx) in skin.joints.iter().enumerate() {
                if joint_i < 512 {
                    let inv_bind = skin
                        .inverse_bind_matrices
                        .get(joint_i)
                        .copied()
                        .unwrap_or(Mat4::IDENTITY);
                    let global_joint = global_transforms[node_idx];
                    let final_mat = global_joint * inv_bind;
                    bone_uniform.matrices[joint_i] = final_mat.to_cols_array_2d();
                }
            }
        }

        queue.write_buffer(bone_buffer, 0, bytemuck::cast_slice(&[*bone_uniform]));
    }
}

fn compute_global_transforms(
    node_idx: usize,
    parent_transform: Mat4,
    pose_manager: &PoseManager,
    model: &VrmModelData,
    globals: &mut [Mat4],
) {
    let node = &model.nodes[node_idx];
    let mut rotation = node.initial_rotation;

    if let Some(Some(bone_name)) = model.node_to_bone.get(node_idx) {
        if let Some(mut pose_rot) = pose_manager.get_bone_quat(bone_name) {
            // VRM 0.X Coordinate Alignment:
            // VRM 0.X models natively face -Z (Unity convention) while VRM 1.0 faces +Z (glTF standard).
            // Under 180° Y flip (R_y(π)), the bone rotation transforms as:
            // Q_0 = R_y(π) * Q_1 * R_y(π) = (-qx, qy, -qz, qw).
            // This ensures both VRM 0.X and VRM 1.0 start in standard T-Pose and animate identically.
            if model.is_vrm_0 {
                pose_rot = glam::Quat::from_xyzw(-pose_rot.x, pose_rot.y, -pose_rot.z, pose_rot.w);
            }

            // Three-VRM Normalized Humanoid Rig standard (@pixiv/three-vrm)
            // boneNode.quaternion = inv(P) * rigBoneNode.quaternion * P * boneRotation
            let parent_rot = model
                .parent_rest_world_rotations
                .get(node_idx)
                .copied()
                .unwrap_or(glam::Quat::IDENTITY);
            let inv_p = parent_rot.inverse();
            let r_local = inv_p * pose_rot * parent_rot;
            rotation = r_local * node.initial_rotation;
        }
    }

    let local_transform = Mat4::from_scale_rotation_translation(
        node.initial_scale,
        rotation,
        node.initial_translation,
    );

    let global_transform = parent_transform * local_transform;
    globals[node_idx] = global_transform;

    for &child_idx in &node.children {
        compute_global_transforms(child_idx, global_transform, pose_manager, model, globals);
    }
}
