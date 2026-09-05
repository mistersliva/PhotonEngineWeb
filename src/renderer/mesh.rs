use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

use super::buffers::BufferManager;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }

    pub fn create_cube(device: &wgpu::Device, _buffer_manager: &mut BufferManager) -> Self {
        let vertices = vec![
            Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0], uv: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0,  0.0,  1.0], uv: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  0.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0], uv: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0,  0.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  0.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [-0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0], uv: [0.0, 1.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 0.0,  1.0,  0.0], uv: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0], uv: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [ 0.0,  1.0,  0.0], uv: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0], uv: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 0.0, -1.0,  0.0], uv: [1.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0], uv: [1.0, 0.0] },
            Vertex { position: [-0.5, -0.5,  0.5], normal: [ 0.0, -1.0,  0.0], uv: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [ 1.0,  0.0,  0.0], uv: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [ 1.0,  0.0,  0.0], uv: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [ 1.0,  0.0,  0.0], uv: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], normal: [ 1.0,  0.0,  0.0], uv: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0,  0.0,  0.0], uv: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0,  0.0,  0.0], uv: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], normal: [-1.0,  0.0,  0.0], uv: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0,  0.0,  0.0], uv: [0.0, 0.0] },
        ];

        let indices: Vec<u32> = vec![
            0,  1,  2,  0,  2,  3,
            4,  5,  6,  4,  6,  7,
            8,  9,  10, 8,  10, 11,
            12, 13, 14, 12, 14, 15,
            16, 17, 18, 16, 18, 19,
            20, 21, 22, 20, 22, 23,
        ];

        Self::new(device, &vertices, &indices)
    }

    pub fn create_floor(device: &wgpu::Device, _buffer_manager: &mut BufferManager) -> Self {
        let size = 0.5_f32;
        let tiles = 32.0_f32;
        let vertices = vec![
            Vertex { position: [-size, 0.0,  size], normal: [0.0, 1.0, 0.0], uv: [0.0, tiles] },
            Vertex { position: [ size, 0.0,  size], normal: [0.0, 1.0, 0.0], uv: [tiles, tiles] },
            Vertex { position: [ size, 0.0, -size], normal: [0.0, 1.0, 0.0], uv: [tiles, 0.0] },
            Vertex { position: [-size, 0.0, -size], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0] },
        ];

        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        Self::new(device, &vertices, &indices)
    }

    pub fn create_ramp(device: &wgpu::Device, _buffer_manager: &mut BufferManager) -> Self {
        let n_slope = 1.0_f32 / (2.0_f32).sqrt();
        let vertices = vec![
            Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, n_slope, n_slope], uv: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, n_slope, n_slope], uv: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, n_slope, n_slope], uv: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, n_slope, n_slope], uv: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0] },
            Vertex { position: [-0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0] },
            Vertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0] },
        ];

        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3,
            4, 5, 6, 4, 6, 7,
            8, 9, 10, 8, 10, 11,
            12, 13, 14,
            15, 16, 17,
        ];

        Self::new(device, &vertices, &indices)
    }

    pub fn create_curved_ramp(device: &wgpu::Device, _buffer_manager: &mut BufferManager) -> Self {
        let segs_x = 16;
        let segs_z = 8;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for iz in 0..=segs_z {
            let tz = iz as f32 / segs_z as f32;
            let z = -0.5 + tz;

            for ix in 0..=segs_x {
                let tx = ix as f32 / segs_x as f32;
                let x = -0.5 + tx;

                let angle = tx * std::f32::consts::FRAC_PI_2;
                let y = -0.5 + (1.0 - angle.sin()) * 1.0;

                let nx = angle.cos();
                let ny = angle.sin();
                let nz = 0.0;

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [nx, ny, nz],
                    uv: [tx * 2.0, tz * 4.0],
                });
            }
        }

        let stride = segs_x + 1;
        for iz in 0..segs_z {
            for ix in 0..segs_x {
                let i0 = (iz * stride + ix) as u32;
                let i1 = (iz * stride + ix + 1) as u32;
                let i2 = ((iz + 1) * stride + ix + 1) as u32;
                let i3 = ((iz + 1) * stride + ix) as u32;

                indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
            }
        }

        Self::new(device, &vertices, &indices)
    }

    pub fn create_cylinder(device: &wgpu::Device, _buffer_manager: &mut BufferManager) -> Self {
        let segments = 24;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=segments {
            let theta = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let cos_t = theta.cos() * 0.5;
            let sin_t = theta.sin() * 0.5;
            let u = i as f32 / segments as f32;

            vertices.push(Vertex {
                position: [cos_t, -0.5, sin_t],
                normal: [theta.cos(), 0.0, theta.sin()],
                uv: [u, 0.0],
            });
            vertices.push(Vertex {
                position: [cos_t, 0.5, sin_t],
                normal: [theta.cos(), 0.0, theta.sin()],
                uv: [u, 1.0],
            });
        }

        for i in 0..segments as u32 {
            let i0 = i * 2;
            let i1 = i * 2 + 1;
            let i2 = i * 2 + 3;
            let i3 = i * 2 + 2;
            indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
        }

        let top_center_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: [0.0, 0.5, 0.0],
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
        });
        let bot_center_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: [0.0, -0.5, 0.0],
            normal: [0.0, -1.0, 0.0],
            uv: [0.5, 0.5],
        });

        for i in 0..segments as u32 {
            let theta0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let theta1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let v_top0 = vertices.len() as u32;
            vertices.push(Vertex {
                position: [theta0.cos() * 0.5, 0.5, theta0.sin() * 0.5],
                normal: [0.0, 1.0, 0.0],
                uv: [theta0.cos() * 0.5 + 0.5, theta0.sin() * 0.5 + 0.5],
            });
            let v_top1 = vertices.len() as u32;
            vertices.push(Vertex {
                position: [theta1.cos() * 0.5, 0.5, theta1.sin() * 0.5],
                normal: [0.0, 1.0, 0.0],
                uv: [theta1.cos() * 0.5 + 0.5, theta1.sin() * 0.5 + 0.5],
            });
            indices.extend_from_slice(&[top_center_idx, v_top0, v_top1]);

            let v_bot0 = vertices.len() as u32;
            vertices.push(Vertex {
                position: [theta0.cos() * 0.5, -0.5, theta0.sin() * 0.5],
                normal: [0.0, -1.0, 0.0],
                uv: [theta0.cos() * 0.5 + 0.5, theta0.sin() * 0.5 + 0.5],
            });
            let v_bot1 = vertices.len() as u32;
            vertices.push(Vertex {
                position: [theta1.cos() * 0.5, -0.5, theta1.sin() * 0.5],
                normal: [0.0, -1.0, 0.0],
                uv: [theta1.cos() * 0.5 + 0.5, theta1.sin() * 0.5 + 0.5],
            });
            indices.extend_from_slice(&[bot_center_idx, v_bot1, v_bot0]);
        }

        Self::new(device, &vertices, &indices)
    }

    pub fn create_sphere(device: &wgpu::Device, _buffer_manager: &mut BufferManager) -> Self {
        let lats = 16;
        let lons = 24;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=lats {
            let theta = (i as f32 / lats as f32) * std::f32::consts::PI;
            let sin_t = theta.sin();
            let cos_t = theta.cos();

            for j in 0..=lons {
                let phi = (j as f32 / lons as f32) * std::f32::consts::TAU;
                let sin_p = phi.sin();
                let cos_p = phi.cos();

                let x = cos_p * sin_t * 0.5;
                let y = cos_t * 0.5;
                let z = sin_p * sin_t * 0.5;

                let nx = cos_p * sin_t;
                let ny = cos_t;
                let nz = sin_p * sin_t;

                let u = j as f32 / lons as f32;
                let v = i as f32 / lats as f32;

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [nx, ny, nz],
                    uv: [u, v],
                });
            }
        }

        let stride = lons + 1;
        for i in 0..lats {
            for j in 0..lons {
                let i0 = (i * stride + j) as u32;
                let i1 = (i * stride + j + 1) as u32;
                let i2 = ((i + 1) * stride + j + 1) as u32;
                let i3 = ((i + 1) * stride + j) as u32;

                indices.extend_from_slice(&[i0, i1, i2, i0, i2, i3]);
            }
        }

        Self::new(device, &vertices, &indices)
    }
}
