pub mod device;
pub mod swapchain;
pub mod pipeline;
pub mod buffers;
pub mod command;
pub mod descriptor;
pub mod mesh;
pub mod shadow;
pub mod texture;
pub mod ui_renderer;
pub mod egui_integration;

pub use device::GpuState;
pub use swapchain::Swapchain;
pub use pipeline::PipelineManager;
pub use buffers::BufferManager;
pub use descriptor::BindGroupManager;
pub use mesh::{Mesh, Vertex};
pub use shadow::ShadowMap;
pub use texture::Texture;
pub use ui_renderer::UiRenderer;
pub use egui_integration::EguiState;

use crate::camera::Camera;
use crate::scene::Scene;
use crate::input::InputState;
use wgpu::util::DeviceExt;

#[derive(Clone, Debug, Default)]
pub struct HudData {
    pub fps: f32,
    pub frame_time_ms: f32,
    pub position: glam::Vec3,
    pub velocity: glam::Vec3,
    pub speed: f32,
    pub max_speed: f32,
    pub accel: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub health: u32,
    pub grounded: bool,
    pub crouching: bool,
    pub key_w: bool,
    pub key_a: bool,
    pub key_s: bool,
    pub key_d: bool,
    pub key_jump: bool,
    pub key_duck: bool,
    pub mouse_dx: f32,
    pub session_time: f32,
    pub jumps: u32,
    pub flashlight: bool,
    pub elapsed: f32,
    pub map_name: String,
    pub entity_count: usize,
    pub light_count: usize,
    pub shadows_enabled: bool,
    pub noclip: bool,
}

pub struct Renderer {
    pub gpu: GpuState,
    pub swapchain: Swapchain,
    pub pipeline_manager: PipelineManager,
    pub buffer_manager: BufferManager,
    pub bind_group_manager: BindGroupManager,
    pub ui_renderer: UiRenderer,
    pub egui_state: EguiState,
    pub floor_mesh: Mesh,
    pub cube_mesh: Mesh,
    pub ramp_mesh: Mesh,
    pub curved_ramp_mesh: Mesh,
    pub cylinder_mesh: Mesh,
    pub sphere_mesh: Mesh,
    pub current_frame: usize,
    pub framebuffer_resized: bool,
    pub textures: Vec<Texture>,
    pub texture_bind_groups: Vec<wgpu::BindGroup>,
    pub shadow_map: ShadowMap,
    pub shadow_bind_group: wgpu::BindGroup,
    pub normal_texture: Texture,
    pub normal_bind_group: wgpu::BindGroup,
    pub uniform_bind_groups: Vec<wgpu::BindGroup>,
    pub shadows_enabled: bool,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    #[cfg(target_arch = "wasm32")]
    pub draw_call_count: u32,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Self {
        log::info!("Creating GpuState...");
        let gpu = GpuState::new(window).await;

        let size = window.inner_size();
        log::info!("Creating Swapchain...");
        let swapchain = Swapchain::new(size.width, size.height);

        log::info!("Creating PipelineManager...");
        let pipeline_manager = PipelineManager::new(&gpu.device, gpu.config.format);

        log::info!("Creating BufferManager...");
        let mut buffer_manager = BufferManager::new(&gpu.device);

        log::info!("Creating BindGroupManager...");
        let bind_group_manager = BindGroupManager::new(&gpu.device);

        log::info!("Creating shadow map...");
        let shadow_map = ShadowMap::new(&gpu.device);
        let shadow_bind_group = bind_group_manager.create_shadow_bind_group(
            &gpu.device,
            &shadow_map.view,
            &shadow_map.sampler,
        );

        log::info!("Creating floor mesh...");
        let floor_mesh = Mesh::create_floor(&gpu.device, &mut buffer_manager);

        log::info!("Creating cube mesh...");
        let cube_mesh = Mesh::create_cube(&gpu.device, &mut buffer_manager);

        log::info!("Creating ramp mesh...");
        let ramp_mesh = Mesh::create_ramp(&gpu.device, &mut buffer_manager);

        log::info!("Creating curved ramp mesh...");
        let curved_ramp_mesh = Mesh::create_curved_ramp(&gpu.device, &mut buffer_manager);

        log::info!("Creating cylinder mesh...");
        let cylinder_mesh = Mesh::create_cylinder(&gpu.device, &mut buffer_manager);

        log::info!("Creating sphere mesh...");
        let sphere_mesh = Mesh::create_sphere(&gpu.device, &mut buffer_manager);

        log::info!("Creating uniform bind groups...");
        let uniform_bind_groups: Vec<wgpu::BindGroup> = buffer_manager
            .uniform_buffers
            .iter()
            .map(|buf| bind_group_manager.create_uniform_bind_group(&gpu.device, buf))
            .collect();

        log::info!("Loading textures...");
        let mut textures = Vec::new();
        let mut texture_bind_groups = Vec::new();

        log::info!("[Renderer] Creating white fallback texture...");
        let white_texture = Texture::create_white_texture(&gpu.device, &gpu.queue);
        let white_bg = bind_group_manager.create_texture_bind_group(
            &gpu.device,
            &white_texture.view,
            &white_texture.sampler,
        );
        textures.push(white_texture);
        texture_bind_groups.push(white_bg);

        log::info!("[Renderer] Attempting to load grass.jpg...");
        match Texture::load_from_file(&gpu.device, &gpu.queue, "assets/textures/grass.jpg") {
            Ok(grass_tex) => {
                log::info!("[Renderer] grass.jpg loaded, creating bind group...");
                let grass_bg = bind_group_manager.create_texture_bind_group(
                    &gpu.device,
                    &grass_tex.view,
                    &grass_tex.sampler,
                );
                textures.push(grass_tex);
                texture_bind_groups.push(grass_bg);
                log::info!(
                    "[Renderer] Total textures: {}, Total bind groups: {}",
                    textures.len(),
                    texture_bind_groups.len()
                );
            }
            Err(e) => {
                log::warn!("[Renderer] Failed to load grass.jpg: {}", e);
            }
        }

        log::info!("Creating detail normal map...");
        let normal_texture = Texture::create_detail_normal(&gpu.device, &gpu.queue);
        let normal_bind_group = bind_group_manager.create_normal_bind_group(
            &gpu.device,
            &normal_texture.view,
            &normal_texture.sampler,
        );

        log::info!("Creating UiRenderer...");
        let ui_renderer = UiRenderer::new(&gpu.device, &gpu.queue, gpu.config.format);

        log::info!("Creating EguiState...");
        let egui_state = EguiState::new(&gpu.device, &gpu.config, window);

        log::info!("Creating depth texture...");
        let depth_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d { width: size.width.max(1), height: size.height.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Depth32Float),
            aspect: wgpu::TextureAspect::DepthOnly,
            ..Default::default()
        });

        log::info!("All initialization complete!");

        Self {
            gpu,
            swapchain,
            pipeline_manager,
            buffer_manager,
            bind_group_manager,
            ui_renderer,
            egui_state,
            floor_mesh,
            cube_mesh,
            ramp_mesh,
            curved_ramp_mesh,
            cylinder_mesh,
            sphere_mesh,
            current_frame: 0,
            framebuffer_resized: false,
            textures,
            texture_bind_groups,
            shadow_map,
            shadow_bind_group,
            normal_texture,
            normal_bind_group,
            uniform_bind_groups,
            shadows_enabled: true,
            depth_texture,
            depth_view,
            #[cfg(target_arch = "wasm32")]
            draw_call_count: 0,
        }
    }

    pub fn add_texture(&mut self, texture: Texture) -> usize {
        let bg = self.bind_group_manager.create_texture_bind_group(
            &self.gpu.device,
            &texture.view,
            &texture.sampler,
        );
        let index = self.textures.len();
        self.textures.push(texture);
        self.texture_bind_groups.push(bg);
        index
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.gpu.resize(width, height);
        self.swapchain.resize(width, height);

        self.depth_texture = self.gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Depth32Float),
            aspect: wgpu::TextureAspect::DepthOnly,
            ..Default::default()
        });
    }

    pub fn draw(
        &mut self,
        scene: &Scene,
        camera: &Camera,
        width: u32,
        height: u32,
        _input: &InputState,
        flash_pos: glam::Vec3,
        flash_dir: glam::Vec3,
        flashlight_on: bool,
        _screenshot: bool,
    ) -> Option<()> {
        #[cfg(target_arch = "wasm32")]
        {
            self.draw_call_count += 1;
            if self.draw_call_count <= 3 {
                web_sys::console::log_1(&format!("PhotonEngine: draw() called #{}", self.draw_call_count).into());
            }
        }
        if width == 0 || height == 0 {
            return None;
        }

        let frame = match self.gpu.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.resize(width, height);
                return None;
            }
            Err(e) => {
                log::error!("Failed to acquire surface texture: {:?}", e);
                return None;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Main Encoder"),
            });

        let point_lights = scene.collect_point_lights();
        self.buffer_manager.update_uniform_buffer(
            &self.gpu.queue,
            self.current_frame,
            camera,
            width as f32 / height as f32,
            &point_lights,
            flash_pos,
            flash_dir,
            flashlight_on,
        );

        // Sun shadow-map pass
        if self.shadows_enabled {
            if let Some(shadow_pipeline) = self.pipeline_manager.shadow_pipeline.as_ref() {
                let mut shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Shadow Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.shadow_map.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                shadow_pass.set_pipeline(&shadow_pipeline.render_pipeline);

                for entity in &scene.entities {
                    if matches!(
                        entity.mesh_type,
                        crate::scene::MeshType::Glow | crate::scene::MeshType::Light
                    ) {
                        continue;
                    }
                    let mesh = match entity.mesh_type {
                        crate::scene::MeshType::Floor => &self.floor_mesh,
                        crate::scene::MeshType::Cube | crate::scene::MeshType::Wall => &self.cube_mesh,
                        crate::scene::MeshType::Sphere => &self.sphere_mesh,
                        crate::scene::MeshType::Light | crate::scene::MeshType::Glow => &self.cube_mesh,
                        crate::scene::MeshType::Metal => &self.cube_mesh,
                        crate::scene::MeshType::Ramp => &self.ramp_mesh,
                        crate::scene::MeshType::CurvedRamp => &self.curved_ramp_mesh,
                        crate::scene::MeshType::Cylinder => &self.cylinder_mesh,
                    };

                    let rot = glam::Quat::from_euler(
                        glam::EulerRot::YXZ,
                        entity.rotation.y,
                        entity.rotation.x,
                        entity.rotation.z,
                    );
                    let model =
                        glam::Mat4::from_scale_rotation_translation(entity.scale, rot, entity.position);
                    let light_vp = crate::renderer::buffers::sun_shadow_matrix();

                    let mut push_data = [0.0f32; 32];
                    push_data[..16].copy_from_slice(&model.to_cols_array());
                    push_data[16..].copy_from_slice(&light_vp.to_cols_array());
                    let push_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Shadow Push Constants"),
                        contents: bytemuck::cast_slice(&push_data),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                    let push_bg = self.bind_group_manager.create_push_constant_bind_group(
                        &self.gpu.device,
                        &push_buffer,
                    );
                    shadow_pass.set_bind_group(0, &push_bg, &[]);
                    shadow_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    shadow_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    shadow_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            }
        }

        // Main render pass
        {
            let mut main_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.20,
                            g: 0.32,
                            b: 0.48,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(mesh_pipeline) = self.pipeline_manager.mesh_pipeline.as_ref() {
                let vp = camera.projection_matrix * camera.view_matrix;
                main_pass.set_pipeline(&mesh_pipeline.render_pipeline);

                for entity in &scene.entities {
                    let half_extents = entity.scale * 0.5;
                    if !frustum_cull(&vp, entity.position, half_extents) {
                        continue;
                    }
                    let mesh = match entity.mesh_type {
                        crate::scene::MeshType::Floor => &self.floor_mesh,
                        crate::scene::MeshType::Cube | crate::scene::MeshType::Wall => &self.cube_mesh,
                        crate::scene::MeshType::Sphere => &self.sphere_mesh,
                        crate::scene::MeshType::Light | crate::scene::MeshType::Glow => &self.cube_mesh,
                        crate::scene::MeshType::Metal => &self.cube_mesh,
                        crate::scene::MeshType::Ramp => &self.ramp_mesh,
                        crate::scene::MeshType::CurvedRamp => &self.curved_ramp_mesh,
                        crate::scene::MeshType::Cylinder => &self.cylinder_mesh,
                    };

                    let tex_idx = if entity.texture_index < self.texture_bind_groups.len() {
                        entity.texture_index
                    } else if entity.mesh_type == crate::scene::MeshType::Floor
                        && self.texture_bind_groups.len() > 1
                    {
                        1
                    } else {
                        0
                    };

                    let rot = glam::Quat::from_euler(
                        glam::EulerRot::YXZ,
                        entity.rotation.y,
                        entity.rotation.x,
                        entity.rotation.z,
                    );
                    let model =
                        glam::Mat4::from_scale_rotation_translation(entity.scale, rot, entity.position);

                    let color = entity.color;
                    let (metallic, roughness, ao, emissive) = match entity.mesh_type {
                        crate::scene::MeshType::Floor => (0.05, 0.45, 1.0, 0.0),
                        crate::scene::MeshType::Cube => (0.15, 0.35, 1.0, 0.0),
                        crate::scene::MeshType::Sphere => (0.85, 0.15, 1.0, 0.0),
                        crate::scene::MeshType::Light => (0.0, 0.20, 1.0, 1.2),
                        crate::scene::MeshType::Wall => (0.02, 0.75, 1.0, 0.0),
                        crate::scene::MeshType::Metal => (0.88, 0.18, 1.0, 0.0),
                        crate::scene::MeshType::Glow => (0.0, 0.10, 1.0, 2.0),
                        crate::scene::MeshType::Ramp => (0.20, 0.30, 1.0, 0.0),
                        crate::scene::MeshType::CurvedRamp => (0.25, 0.25, 1.0, 0.0),
                        crate::scene::MeshType::Cylinder => (0.70, 0.25, 1.0, 0.0),
                    };

                    let push_data: [f32; 24] = [
                        model.to_cols_array()[0],
                        model.to_cols_array()[1],
                        model.to_cols_array()[2],
                        model.to_cols_array()[3],
                        model.to_cols_array()[4],
                        model.to_cols_array()[5],
                        model.to_cols_array()[6],
                        model.to_cols_array()[7],
                        model.to_cols_array()[8],
                        model.to_cols_array()[9],
                        model.to_cols_array()[10],
                        model.to_cols_array()[11],
                        model.to_cols_array()[12],
                        model.to_cols_array()[13],
                        model.to_cols_array()[14],
                        model.to_cols_array()[15],
                        color[0],
                        color[1],
                        color[2],
                        1.0,
                        metallic,
                        roughness,
                        ao,
                        emissive,
                    ];
                    let push_buffer = self.gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Mesh Push Constants"),
                        contents: bytemuck::cast_slice(&push_data),
                        usage: wgpu::BufferUsages::UNIFORM,
                    });
                    let push_bg = self.bind_group_manager.create_push_constant_bind_group(
                        &self.gpu.device,
                        &push_buffer,
                    );

                    main_pass.set_bind_group(0, &self.uniform_bind_groups[self.current_frame], &[]);
                    main_pass.set_bind_group(1, &self.texture_bind_groups[tex_idx], &[]);
                    main_pass.set_bind_group(2, &self.shadow_bind_group, &[]);
                    main_pass.set_bind_group(3, &self.normal_bind_group, &[]);
                    main_pass.set_bind_group(4, &push_bg, &[]);

                    main_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    main_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    main_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                }
            }
        }

        // Egui pass
        self.egui_state.end_frame_and_upload_textures(&self.gpu.device, &self.gpu.queue);
        {
            let mut egui_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.egui_state.cmd_draw(&mut egui_pass, width, height, 1.0);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        self.current_frame = (self.current_frame + 1) % crate::math::MAX_FRAMES_IN_FLIGHT;
        Some(())
    }

}

/// Conservative sphere test: rotation-proof (yawed ramps no longer pop
/// out of the frustum) at the cost of occasionally drawing a slightly
/// off-screen object.
fn frustum_cull(vp: &glam::Mat4, pos: glam::Vec3, half_extents: glam::Vec3) -> bool {
    let radius = half_extents.length();
    let cols = vp.to_cols_array();
    let rows = [
        glam::Vec4::new(cols[0], cols[4], cols[8], cols[12]),
        glam::Vec4::new(cols[1], cols[5], cols[9], cols[13]),
        glam::Vec4::new(cols[2], cols[6], cols[10], cols[14]),
        glam::Vec4::new(cols[3], cols[7], cols[11], cols[15]),
    ];

    for i in 0..3 {
        for s in [-1.0_f32, 1.0] {
            let plane = glam::Vec4::new(
                rows[3].x + s * rows[i].x,
                rows[3].y + s * rows[i].y,
                rows[3].z + s * rows[i].z,
                rows[3].w + s * rows[i].w,
            );
            let len = glam::Vec3::new(plane.x, plane.y, plane.z).length();
            if len < 0.0001 {
                continue;
            }
            let n = glam::Vec3::new(plane.x / len, plane.y / len, plane.z / len);
            let d = plane.w / len;
            if n.dot(pos) + d + radius < 0.0 {
                return false;
            }
        }
    }

    true
}
