use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};
use bytemuck::{Pod, Zeroable};

use crate::camera::Camera;
use crate::scene::PointLightData;

pub const MAX_POINT_LIGHTS: usize = 12;

pub const SUN_DIR: [f32; 3] = [0.55, 0.80, 0.25];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
pub struct UniformBufferObject {
    pub view: Mat4,
    pub projection: Mat4,
    pub light_dir: [f32; 4],
    pub light_color: [f32; 4],
    pub view_pos: [f32; 4],
    pub ambient_sky: [f32; 4],
    pub ambient_ground: [f32; 4],
    pub light_vp: Mat4,
    pub flash_pos: [f32; 4],
    pub flash_dir: [f32; 4],
    pub flash_params: [f32; 4],
    pub light_info: [f32; 4],
    pub lights_pos: [[f32; 4]; MAX_POINT_LIGHTS],
    pub lights_color: [[f32; 4]; MAX_POINT_LIGHTS],
}

pub struct BufferManager {
    pub uniform_buffers: Vec<wgpu::Buffer>,
}

impl BufferManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let frame_count = crate::math::MAX_FRAMES_IN_FLIGHT;
        let _buffer_size = std::mem::size_of::<UniformBufferObject>();

        let uniform_buffers: Vec<wgpu::Buffer> = (0..frame_count)
            .map(|_| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[UniformBufferObject::default()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                })
            })
            .collect();

        Self { uniform_buffers }
    }

    pub fn update_uniform_buffer(
        &self,
        queue: &wgpu::Queue,
        current_frame: usize,
        camera: &Camera,
        aspect: f32,
        point_lights: &[PointLightData],
        flash_pos: Vec3,
        flash_dir: Vec3,
        flashlight_on: bool,
    ) {
        let light_vp = sun_shadow_matrix();
        let mut lights_pos = [[0.0f32; 4]; MAX_POINT_LIGHTS];
        let mut lights_color = [[0.0f32; 4]; MAX_POINT_LIGHTS];
        let count = point_lights.len().min(MAX_POINT_LIGHTS);
        for i in 0..count {
            let l = &point_lights[i];
            lights_pos[i] = [l.position.x, l.position.y, l.position.z, l.radius];
            lights_color[i] = [l.color[0], l.color[1], l.color[2], l.intensity];
        }

        let ubo = UniformBufferObject {
            view: camera.view_matrix,
            projection: crate::math::perspective_vk(camera.fov, aspect, camera.near, camera.far),
            light_dir: [SUN_DIR[0], SUN_DIR[1], SUN_DIR[2], 1.0],
            light_color: [1.0, 0.88, 0.72, 1.0],
            view_pos: [camera.position.x, camera.position.y, camera.position.z, 1.0],
            ambient_sky: [0.22, 0.25, 0.32, 1.0],
            ambient_ground: [0.10, 0.08, 0.06, 1.0],
            light_vp,
            flash_pos: [
                flash_pos.x,
                flash_pos.y,
                flash_pos.z,
                if flashlight_on { 1.0 } else { 0.0 },
            ],
            flash_dir: [flash_dir.x, flash_dir.y, flash_dir.z, 0.0],
            flash_params: [0.976, 0.940, 2500.0, 2.2],
            light_info: [count as f32, 0.0, 0.0, 0.0],
            lights_pos,
            lights_color,
        };

        let ubo_array = [ubo];
        let data = bytemuck::cast_slice(&ubo_array);
        queue.write_buffer(&self.uniform_buffers[current_frame], 0, data);
    }
}

pub fn sun_shadow_matrix() -> Mat4 {
    let dir = glam::Vec3::new(SUN_DIR[0], SUN_DIR[1], SUN_DIR[2]).normalize();
    let center = glam::Vec3::ZERO;
    let eye = center + dir * 4500.0;
    let view = crate::math::look_at_view(eye, center, glam::Vec3::Y);
    let proj = crate::math::ortho_vk(-2600.0, 2600.0, -2600.0, 2600.0, 1.0, 9000.0);
    proj * view
}
