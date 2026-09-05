pub struct Swapchain {
    pub extent: wgpu::Extent3d,
}

impl Swapchain {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            extent: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.extent.width = width;
        self.extent.height = height;
    }
}
