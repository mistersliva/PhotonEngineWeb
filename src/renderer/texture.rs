pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn load_from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &str,
    ) -> Result<Self, String> {
        log::info!("[Texture] Loading: {}", path);
        let img = image::open(path)
            .map_err(|e| {
                let msg = format!("Failed to open image {}: {}", path, e);
                log::error!("[Texture] {}", msg);
                msg
            })?
            .to_rgba8();

        let width = img.width();
        let height = img.height();
        let pixels = img.into_raw();
        log::info!(
            "[Texture] Image {}x{}, {} bytes, format=Rgba8Unorm",
            width,
            height,
            pixels.len()
        );

        let tex = Self::create_from_pixels(device, queue, &pixels, width, height);
        log::info!("[Texture] {} loaded successfully!", path);
        Ok(tex)
    }

    pub fn create_white_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        log::info!("[Texture] Creating 1x1 white fallback texture");
        let pixels: [u8; 4] = [255, 255, 255, 255];
        let tex = Self::create_from_pixels(device, queue, &pixels, 1, 1);
        log::info!("[Texture] White fallback texture created");
        tex
    }

    pub fn create_detail_normal(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const SIZE: u32 = 256;
        let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let n1 = hash_noise(x / 4, y / 4);
                let n2 = hash_noise(x, y);
                let dx = (hash_noise(x.wrapping_add(1), y) - n2) * 0.6
                    + (hash_noise(x.wrapping_add(1) / 4, y / 4) - n1) * 1.4;
                let dy = (hash_noise(x, y.wrapping_add(1)) - n2) * 0.6
                    + (hash_noise(x / 4, y.wrapping_add(1) / 4) - n1) * 1.4;
                let inv = 1.0 / (dx * dx + dy * dy + 1.0).sqrt();
                pixels.push(((dx * inv * 0.5 + 0.5) * 255.0) as u8);
                pixels.push(((dy * inv * 0.5 + 0.5) * 255.0) as u8);
                pixels.push(((inv * 0.5 + 0.5) * 255.0) as u8);
                pixels.push(255);
            }
        }

        let tex = Self::create_from_pixels(device, queue, &pixels, SIZE, SIZE);
        log::info!("[Texture] Procedural detail normal map created");
        tex
    }

    pub fn create_from_pixels(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
        }
    }
}

fn hash_noise(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(374761393).wrapping_add(y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) % 1024) as f32 / 1023.0
}
