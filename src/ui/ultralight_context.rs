use std::path::Path;

pub struct UltralightContext {
    pub initialized: bool,
}

impl UltralightContext {
    pub fn new() -> Self {
        Self {
            initialized: false,
        }
    }

    pub fn initialize(&mut self, _width: u32, _height: u32) -> Result<(), String> {
        log::info!("Ultralight context initialized (stub - requires ultralight crate)");
        self.initialized = true;
        Ok(())
    }

    pub fn load_html(&mut self, path: &Path) -> Result<(), String> {
        let _html = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read HTML: {}", e))?;
        log::info!("Loaded HTML from {}", path.display());
        Ok(())
    }

    pub fn update(&mut self) -> Option<Vec<u8>> {
        if !self.initialized {
            return None;
        }
        None
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}
}
