pub use glam::{Mat4, Vec2, Vec3, Vec4};

pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_HEIGHT: u32 = 720;
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub fn perspective_vk(fov_y_radians: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let mut proj = Mat4::perspective_rh(fov_y_radians, aspect, near, far);
    proj.y_axis.y = -proj.y_axis.y;
    proj
}

pub fn look_at_view(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(eye, target, up)
}

/// Vulkan-style orthographic projection (Y flipped like `perspective_vk`).
pub fn ortho_vk(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    let mut proj = Mat4::orthographic_rh(left, right, bottom, top, near, far);
    proj.y_axis.y = -proj.y_axis.y;
    proj
}
