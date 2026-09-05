use crate::bitmap_font;
use crate::renderer::HudData;

const MAX_UI_VERTS: usize = 3000;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UiVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

impl UiVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UiVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[allow(dead_code)]
pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    white_bind_group: wgpu::BindGroup,
    font_bind_group: wgpu::BindGroup,
    white_texture: wgpu::Texture,
    font_texture: wgpu::Texture,
    screen_w: f32,
    screen_h: f32,
}

impl UiRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, output_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("ui_shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&Self::create_bind_group_layout(device)],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[UiVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Vertex Buffer"),
            size: (MAX_UI_VERTS * std::mem::size_of::<UiVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let white_texture = Self::create_white_texture(device, queue);
        let font_texture = Self::create_font_atlas(device, queue);

        let bind_group_layout = Self::create_bind_group_layout(device);

        let white_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI White Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&white_texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        ..Default::default()
                    })),
                },
            ],
        });

        let font_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI Font Texture Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&font_texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&device.create_sampler(&wgpu::SamplerDescriptor {
                        mag_filter: wgpu::FilterMode::Linear,
                        min_filter: wgpu::FilterMode::Linear,
                        address_mode_u: wgpu::AddressMode::ClampToEdge,
                        address_mode_v: wgpu::AddressMode::ClampToEdge,
                        address_mode_w: wgpu::AddressMode::ClampToEdge,
                        ..Default::default()
                    })),
                },
            ],
        });

        Self {
            pipeline,
            vertex_buffer,
            white_bind_group,
            font_bind_group,
            white_texture,
            font_texture,
            screen_w: 0.0,
            screen_h: 0.0,
        }
    }

    fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("UI Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    pub fn set_screen_size(&mut self, w: f32, h: f32) {
        self.screen_w = w;
        self.screen_h = h;
    }

    fn upload_vertices(queue: &wgpu::Queue, buffer: &wgpu::Buffer, verts: &[UiVertex]) {
        queue.write_buffer(buffer, 0, bytemuck::cast_slice(verts));
    }

    fn rect(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Vec<UiVertex> {
        vec![
            UiVertex { position: [x, y], uv: [0.0, 0.0], color },
            UiVertex { position: [x + w, y], uv: [1.0, 0.0], color },
            UiVertex { position: [x, y + h], uv: [0.0, 1.0], color },
            UiVertex { position: [x + w, y], uv: [1.0, 0.0], color },
            UiVertex { position: [x + w, y + h], uv: [1.0, 1.0], color },
            UiVertex { position: [x, y + h], uv: [0.0, 1.0], color },
        ]
    }

    fn text_verts(text: &str, x: f32, y: f32, char_w: f32, char_h: f32, color: [f32; 4]) -> Vec<UiVertex> {
        let mut verts = Vec::new();
        for (i, ch) in text.chars().enumerate() {
            if let Some((u0, v0, u1, v1)) = bitmap_font::char_uvs(ch) {
                let cx = x + i as f32 * char_w;
                verts.extend_from_slice(&[
                    UiVertex { position: [cx, y], uv: [u0, v1], color },
                    UiVertex { position: [cx + char_w, y], uv: [u1, v1], color },
                    UiVertex { position: [cx, y + char_h], uv: [u0, v0], color },
                    UiVertex { position: [cx + char_w, y], uv: [u1, v1], color },
                    UiVertex { position: [cx + char_w, y + char_h], uv: [u1, v0], color },
                    UiVertex { position: [cx, y + char_h], uv: [u0, v0], color },
                ]);
            }
        }
        verts
    }

    fn build_menu_rects(&self) -> Vec<UiVertex> {
        let mut v = Vec::new();
        let sw = self.screen_w;
        let sh = self.screen_h;
        let aspect = sw / sh;

        let pw = 0.55 * aspect;
        let ph = 0.65;
        let px = -pw * 0.5;
        let py = -ph * 0.5;

        v.extend(Self::rect(-aspect, -1.0, aspect * 2.0, 2.0, [0.0, 0.0, 0.0, 0.6]));
        v.extend(Self::rect(px, py, pw, ph, [0.10, 0.10, 0.12, 0.97]));
        v.extend(Self::rect(px + 0.02, py + ph - 0.15, pw - 0.04, 0.10, [0.15, 0.30, 0.65, 1.0]));
        v.extend(Self::rect(px + 0.02, py + ph - 0.06, pw - 0.04, 0.01, [0.30, 0.50, 1.0, 0.8]));

        let btn_x = px + pw * 0.15;
        let btn_w = pw * 0.70;
        let btn_h = 0.065;
        v.extend(Self::rect(btn_x, py + 0.35, btn_w, btn_h, [0.18, 0.58, 0.24, 1.0]));
        v.extend(Self::rect(btn_x, py + 0.35, btn_w, 0.008, [0.28, 0.78, 0.34, 1.0]));

        v.extend(Self::rect(btn_x, py + 0.22, btn_w, btn_h, [0.28, 0.28, 0.30, 1.0]));
        v.extend(Self::rect(btn_x, py + 0.22, btn_w, 0.008, [0.38, 0.38, 0.42, 0.9]));

        v.extend(Self::rect(btn_x, py + 0.09, btn_w, btn_h, [0.55, 0.15, 0.15, 1.0]));
        v.extend(Self::rect(btn_x, py + 0.09, btn_w, 0.008, [0.75, 0.25, 0.25, 0.9]));

        v.extend(Self::rect(px + 0.04, py + 0.02, pw - 0.08, 0.04, [0.15, 0.15, 0.17, 1.0]));
        v
    }

    fn build_menu_text(&self) -> Vec<UiVertex> {
        let sw = self.screen_w;
        let sh = self.screen_h;
        let aspect = sw / sh;

        let pw = 0.55 * aspect;
        let ph = 0.65;
        let px = -pw * 0.5;
        let py = -ph * 0.5;

        let btn_x = px + pw * 0.15;
        let btn_w = pw * 0.70;
        let btn_h = 0.065;

        let title_size = 0.028;
        let btn_text_size = 0.018;
        let hint_size = 0.011;

        let mut v = Vec::new();

        let title_text = "PHOTON ENGINE";
        let title_tw = title_text.len() as f32 * title_size * 0.6;
        let title_x = px + (pw - title_tw) * 0.5;
        let title_y = py + ph - 0.13;
        v.extend(Self::text_verts(title_text, title_x, title_y, title_size, title_size * 1.2, [1.0, 1.0, 1.0, 1.0]));

        let play_text = "PLAY";
        let play_tw = play_text.len() as f32 * btn_text_size * 0.6;
        let play_x = btn_x + (btn_w - play_tw) * 0.5;
        let play_y = py + 0.35 + (btn_h - btn_text_size) * 0.5;
        v.extend(Self::text_verts(play_text, play_x, play_y, btn_text_size, btn_text_size * 1.2, [1.0, 1.0, 1.0, 1.0]));

        let settings_text = "SETTINGS";
        let settings_tw = settings_text.len() as f32 * btn_text_size * 0.6;
        let settings_x = btn_x + (btn_w - settings_tw) * 0.5;
        let settings_y = py + 0.22 + (btn_h - btn_text_size) * 0.5;
        v.extend(Self::text_verts(settings_text, settings_x, settings_y, btn_text_size, btn_text_size * 1.2, [0.9, 0.9, 0.9, 1.0]));

        let quit_text = "QUIT";
        let quit_tw = quit_text.len() as f32 * btn_text_size * 0.6;
        let quit_x = btn_x + (btn_w - quit_tw) * 0.5;
        let quit_y = py + 0.09 + (btn_h - btn_text_size) * 0.5;
        v.extend(Self::text_verts(quit_text, quit_x, quit_y, btn_text_size, btn_text_size * 1.2, [1.0, 0.9, 0.9, 1.0]));

        let hint = "ESC to return";
        let hint_tw = hint.len() as f32 * hint_size * 0.6;
        let hint_x = px + (pw - hint_tw) * 0.5;
        let hint_y = py + 0.025;
        v.extend(Self::text_verts(hint, hint_x, hint_y, hint_size, hint_size * 1.2, [0.5, 0.5, 0.55, 1.0]));
        v
    }

    fn build_hud_rects(&self) -> Vec<UiVertex> {
        let mut v = Vec::new();
        let cx = 0.0f32;
        let cy = 0.0f32;
        v.extend(Self::rect(cx - 0.020, cy - 0.002, 0.040, 0.004, [1.0, 1.0, 1.0, 0.9]));
        v.extend(Self::rect(cx - 0.002, cy - 0.020, 0.004, 0.040, [1.0, 1.0, 1.0, 0.9]));
        v.extend(Self::rect(-0.96, -0.94, 0.40, 0.04, [0.10, 0.10, 0.10, 0.80]));
        v.extend(Self::rect(-0.955, -0.935, 0.39, 0.03, [0.15, 0.65, 0.20, 0.95]));
        v.extend(Self::rect(0.56, -0.94, 0.40, 0.04, [0.10, 0.10, 0.10, 0.80]));
        v
    }

    fn build_hud_text(&self, hud: &HudData) -> Vec<UiVertex> {
        let mut v = Vec::new();
        let cw = 0.022;
        let ch = 0.028;
        v.extend(Self::text_verts("100", -0.95, -0.955, cw, ch, [0.2, 0.9, 0.3, 1.0]));
        v.extend(Self::text_verts("HP", -0.86, -0.950, cw * 0.7, ch * 0.85, [0.6, 0.6, 0.6, 1.0]));
        let speed_text = format!("{:.0}", hud.speed * 3.6);
        v.extend(Self::text_verts(&speed_text, 0.575, -0.955, cw, ch, [0.8, 0.8, 0.8, 1.0]));
        v.extend(Self::text_verts("km/h", 0.70, -0.950, cw * 0.7, ch * 0.85, [0.6, 0.6, 0.6, 1.0]));
        v
    }

    fn create_white_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("UI White Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let white = [255u8, 255, 255, 255];
        queue.write_texture(
            tex.as_image_copy(),
            &white,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        tex
    }

    fn create_font_atlas(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let pixels = bitmap_font::generate_atlas_pixels();
        let w = bitmap_font::ATLAS_W;
        let h = bitmap_font::ATLAS_H;
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("UI Font Atlas"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            tex.as_image_copy(),
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        tex
    }

    pub fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, output_format: wgpu::TextureFormat) {
        let new = Self::new(device, queue, output_format);
        *self = new;
    }

    fn draw_pass(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'_>, verts: &[UiVertex], bind_group: &wgpu::BindGroup) {
        if verts.is_empty() { return; }
        Self::upload_vertices(queue, &self.vertex_buffer, verts);
        let vertex_count = verts.len() as u32;
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..vertex_count, 0..1);
    }

    pub fn draw_menu(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'_>) {
        let rects = self.build_menu_rects();
        self.draw_pass(queue, render_pass, &rects, &self.white_bind_group.clone());
        let text = self.build_menu_text();
        self.draw_pass(queue, render_pass, &text, &self.font_bind_group.clone());
    }

    pub fn draw_hud(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'_>) {
        let rects = self.build_hud_rects();
        self.draw_pass(queue, render_pass, &rects, &self.white_bind_group.clone());
        let text = self.build_hud_text(&HudData::default());
        self.draw_pass(queue, render_pass, &text, &self.font_bind_group.clone());
    }

    pub fn draw_settings(&mut self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'_>, volume: u32, sensitivity: u32) {
        let rects = self.build_settings_rects(volume, sensitivity);
        self.draw_pass(queue, render_pass, &rects, &self.white_bind_group.clone());
        let text = self.build_settings_text(volume, sensitivity);
        self.draw_pass(queue, render_pass, &text, &self.font_bind_group.clone());
    }

    fn build_settings_rects(&self, volume: u32, sensitivity: u32) -> Vec<UiVertex> {
        let mut v = Vec::new();
        let sw = self.screen_w;
        let sh = self.screen_h;
        let aspect = sw / sh;

        let pw = 0.50 * aspect;
        let ph = 0.55;
        let px = -pw * 0.5;
        let py = -ph * 0.5;

        v.extend(Self::rect(-aspect, -1.0, aspect * 2.0, 2.0, [0.0, 0.0, 0.0, 0.6]));
        v.extend(Self::rect(px, py, pw, ph, [0.10, 0.10, 0.12, 0.97]));
        v.extend(Self::rect(px + 0.02, py + ph - 0.14, pw - 0.04, 0.10, [0.15, 0.15, 0.18, 1.0]));
        v.extend(Self::rect(px + 0.02, py + ph - 0.05, pw - 0.04, 0.01, [0.25, 0.25, 0.30, 0.8]));

        let row1_y = py + ph - 0.28;
        let row2_y = py + ph - 0.40;
        v.extend(Self::rect(px + 0.04, row1_y, pw - 0.08, 0.06, [0.18, 0.18, 0.20, 1.0]));
        v.extend(Self::rect(px + 0.04, row2_y, pw - 0.08, 0.06, [0.18, 0.18, 0.20, 1.0]));

        let bar_x = px + pw * 0.52;
        let bar_w = pw * 0.34;
        let bar_h = 0.015;
        let bar1_y = row1_y + 0.022;
        let bar2_y = row2_y + 0.022;
        v.extend(Self::rect(bar_x, bar1_y, bar_w, bar_h, [0.3, 0.3, 0.32, 1.0]));
        let vol_fill = volume as f32 / 100.0;
        v.extend(Self::rect(bar_x, bar1_y, bar_w * vol_fill, bar_h, [0.20, 0.58, 0.85, 1.0]));
        let sens_fill = sensitivity as f32 / 100.0;
        v.extend(Self::rect(bar_x, bar2_y, bar_w, bar_h, [0.3, 0.3, 0.32, 1.0]));
        v.extend(Self::rect(bar_x, bar2_y, bar_w * sens_fill, bar_h, [0.85, 0.55, 0.20, 1.0]));

        let btn_x = px + pw * 0.25;
        let btn_w = pw * 0.50;
        let btn_h = 0.055;
        let btn_y = py + 0.04;
        v.extend(Self::rect(btn_x, btn_y, btn_w, btn_h, [0.15, 0.30, 0.65, 1.0]));
        v.extend(Self::rect(btn_x, btn_y, btn_w, 0.008, [0.25, 0.45, 0.85, 0.9]));

        v
    }

    fn build_settings_text(&self, volume: u32, sensitivity: u32) -> Vec<UiVertex> {
        let sw = self.screen_w;
        let sh = self.screen_h;
        let aspect = sw / sh;

        let pw = 0.50 * aspect;
        let ph = 0.55;
        let px = -pw * 0.5;
        let py = -ph * 0.5;

        let title_size = 0.024;
        let label_size = 0.016;
        let val_size = 0.014;
        let btn_text_size = 0.016;

        let mut v = Vec::new();

        let title = "SETTINGS";
        let title_tw = title.len() as f32 * title_size * 0.6;
        let title_x = px + (pw - title_tw) * 0.5;
        let title_y = py + ph - 0.13;
        v.extend(Self::text_verts(title, title_x, title_y, title_size, title_size * 1.2, [1.0, 1.0, 1.0, 1.0]));

        let row1_y = py + ph - 0.28;
        let row2_y = py + ph - 0.40;

        v.extend(Self::text_verts("Volume", px + 0.06, row1_y + 0.015, label_size, label_size * 1.2, [0.85, 0.85, 0.85, 1.0]));
        let vol_text = format!("{}%", volume);
        let vol_x = px + pw * 0.42;
        v.extend(Self::text_verts(&vol_text, vol_x, row1_y + 0.017, val_size, val_size * 1.2, [0.7, 0.85, 1.0, 1.0]));

        v.extend(Self::text_verts("Sensitivity", px + 0.06, row2_y + 0.015, label_size, label_size * 1.2, [0.85, 0.85, 0.85, 1.0]));
        let sens_text = format!("{}%", sensitivity);
        let sens_x = px + pw * 0.42;
        v.extend(Self::text_verts(&sens_text, sens_x, row2_y + 0.017, val_size, val_size * 1.2, [1.0, 0.85, 0.6, 1.0]));

        let back = "BACK";
        let back_tw = back.len() as f32 * btn_text_size * 0.6;
        let back_x = px + (pw - back_tw) * 0.5;
        let back_y = py + 0.04 + (0.055 - btn_text_size) * 0.5;
        v.extend(Self::text_verts(back, back_x, back_y, btn_text_size, btn_text_size * 1.2, [1.0, 1.0, 1.0, 1.0]));

        v
    }

    pub fn hit_test_settings(&self, x: f32, y: f32) -> Option<usize> {
        let aspect = self.screen_w / self.screen_h;
        let pw = 0.50 * aspect;
        let ph = 0.55;
        let px = -pw * 0.5;
        let py = -ph * 0.5;

        let row1_y = py + ph - 0.28;
        let row2_y = py + ph - 0.40;
        let bar_x = px + pw * 0.52;
        let bar_w = pw * 0.34;
        let btn_x = px + pw * 0.25;
        let btn_w = pw * 0.50;

        // volume bar (click right half = +, left half = -)
        if x >= bar_x && x <= bar_x + bar_w && y >= row1_y && y <= row1_y + 0.06 {
            return if x > bar_x + bar_w * 0.5 { Some(1) } else { Some(2) };
        }
        // sensitivity bar
        if x >= bar_x && x <= bar_x + bar_w && y >= row2_y && y <= row2_y + 0.06 {
            return if x > bar_x + bar_w * 0.5 { Some(3) } else { Some(4) };
        }
        // back button
        if x >= btn_x && x <= btn_x + btn_w && y >= py + 0.04 && y <= py + 0.04 + 0.055 {
            return Some(0);
        }
        None
    }

    pub fn update_hud(&mut self, _hud: &HudData) {}

    pub fn hit_test_menu(&self, x: f32, y: f32) -> Option<usize> {
        let aspect = self.screen_w / self.screen_h;
        let pw = 0.55 * aspect;
        let ph = 0.65;
        let px = -pw * 0.5;
        let py = -ph * 0.5;
        let btn_x = px + pw * 0.15;
        let btn_w = pw * 0.70;
        let btn_h = 0.065;

        let buttons = [
            [btn_x, py + 0.35, btn_w, btn_h],
            [btn_x, py + 0.22, btn_w, btn_h],
            [btn_x, py + 0.09, btn_w, btn_h],
        ];
        for (i, r) in buttons.iter().enumerate() {
            if x >= r[0] && x <= r[0]+r[2] && y >= r[1] && y <= r[1]+r[3] {
                return Some(i);
            }
        }
        None
    }
}
