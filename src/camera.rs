use crate::math::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
}

impl Camera {
    pub fn new() -> Self {
        let mut cam = Self {
            position: Vec3::new(0.0, 1.0, 3.0),
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
            fov: 70.0_f32.to_radians(),
            aspect: 1280.0 / 720.0,
            near: 0.1,
            far: 50000.0,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
        };
        cam.update_matrices();
        cam
    }

    pub fn forward(&self) -> Vec3 {
        Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn up(&self) -> Vec3 {
        Vec3::Y
    }

    pub fn set_aspect(&mut self, width: u32, height: u32) {
        if height > 0 {
            self.aspect = width as f32 / height as f32;
        }
    }

    pub fn set_fov_deg(&mut self, deg: f32) {
        self.fov = deg.clamp(50.0, 110.0).to_radians();
    }

    pub fn update_matrices(&mut self) {
        let target = self.position + self.forward();
        self.view_matrix = crate::math::look_at_view(self.position, target, Vec3::Y);
        self.projection_matrix = crate::math::perspective_vk(self.fov, self.aspect, self.near, self.far);
    }
}
