use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

const CONFIG_FILE: &str = "config.json";

fn strip_json_comments(json: &str) -> String {
    let mut result = String::with_capacity(json.len());
    for line in json.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        // Also strip inline comments:  "key": value  // comment
        if let Some(pos) = line.find("//") {
            // Make sure it's not inside a string
            let before = &line[..pos];
            let quotes = before.matches('"').count();
            if quotes % 2 == 0 {
                result.push_str(&line[..pos]);
                result.push('\n');
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    #[serde(default = "default_volume")]
    pub volume: u32,
    #[serde(default = "default_sensitivity")]
    pub sensitivity: u32,
    #[serde(default = "default_fps_limit")]
    pub fps_limit: u32,
    #[serde(default)]
    pub window_mode: WindowModeConfig,
    #[serde(default = "default_fov")]
    pub fov: u32,
    #[serde(default = "default_true")]
    pub shadows_enabled: bool,
}

fn default_volume() -> u32 { 75 }
fn default_sensitivity() -> u32 { 50 }
fn default_fps_limit() -> u32 { 400 }
fn default_fov() -> u32 { 75 }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WindowModeConfig {
    #[default]
    Windowed,
    Borderless,
    Exclusive,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            volume: default_volume(),
            sensitivity: default_sensitivity(),
            fps_limit: default_fps_limit(),
            window_mode: WindowModeConfig::Windowed,
            fov: default_fov(),
            shadows_enabled: true,
        }
    }
}

impl GameConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        if !Path::new(CONFIG_FILE).exists() {
            log::info!("No config file found, using defaults");
            return Self::default();
        }
        match std::fs::read_to_string(CONFIG_FILE) {
            Ok(data) => match serde_json::from_str(&strip_json_comments(&data)) {
                Ok(config) => {
                    log::info!("Loaded config from {}", CONFIG_FILE);
                    config
                }
                Err(e) => {
                    log::warn!("Failed to parse config: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to read config: {}, using defaults", e);
                Self::default()
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return Self::default(),
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return Self::default(),
        };
        let data = match storage.get_item(CONFIG_FILE) {
            Ok(Some(d)) => d,
            _ => return Self::default(),
        };
        match serde_json::from_str::<GameConfig>(&strip_json_comments(&data)) {
            Ok(config) => {
                log::info!("Loaded config from localStorage");
                config
            }
            Err(e) => {
                log::warn!("Failed to parse config from localStorage: {}, using defaults", e);
                Self::default()
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(CONFIG_FILE, json) {
                    log::error!("Failed to save config: {}", e);
                } else {
                    log::info!("Config saved to {}", CONFIG_FILE);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize config: {}", e);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return,
        };
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if storage.set_item(CONFIG_FILE, &json).is_err() {
                log::error!("Failed to save config to localStorage");
            } else {
                log::info!("Config saved to localStorage");
            }
        }
    }
}

impl From<&crate::app::WindowMode> for WindowModeConfig {
    fn from(mode: &crate::app::WindowMode) -> Self {
        match mode {
            crate::app::WindowMode::Windowed => WindowModeConfig::Windowed,
            crate::app::WindowMode::Borderless => WindowModeConfig::Borderless,
            crate::app::WindowMode::Exclusive => WindowModeConfig::Exclusive,
        }
    }
}

impl From<&WindowModeConfig> for crate::app::WindowMode {
    fn from(cfg: &WindowModeConfig) -> Self {
        match cfg {
            WindowModeConfig::Windowed => crate::app::WindowMode::Windowed,
            WindowModeConfig::Borderless => crate::app::WindowMode::Borderless,
            WindowModeConfig::Exclusive => crate::app::WindowMode::Exclusive,
        }
    }
}

// ── UI Theme Configuration ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub title: TitleConfig,
    pub buttons: ButtonsConfig,
    pub menu: MenuConfig,
    pub hud: HudConfig,
    pub crosshair: CrosshairConfig,
    pub console: ConsoleConfig,
    pub video: VideoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    pub enabled: bool,
    pub filename: String,
    pub loop_video: bool,
    pub alpha: f32,
    pub brightness: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleConfig {
    pub text: String,
    pub size: f32,
    pub color: [u8; 4],
    pub subtitle: String,
    pub subtitle_size: f32,
    pub subtitle_color: [u8; 4],
    pub version: String,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonsConfig {
    pub width: f32,
    pub height: f32,
    pub text_size: f32,
    pub play_color: [u8; 4],
    pub play_hover: [u8; 4],
    pub maps_color: [u8; 4],
    pub maps_hover: [u8; 4],
    pub settings_color: [u8; 4],
    pub settings_hover: [u8; 4],
    pub quit_color: [u8; 4],
    pub quit_hover: [u8; 4],
    pub text_color: [u8; 4],
    pub slide_in_speed: f32,
    pub glow_pulse_speed: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuConfig {
    pub bg_alpha: u8,
    pub particles: u32,
    pub particle_color: [u8; 4],
    pub particle_speed: f32,
    pub wave_count: u32,
    pub wave_speed: f32,
    pub wave_color: [[u8; 4]; 5],
    pub hints: String,
    pub hints_color: [u8; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudConfig {
    pub panel_bg: [u8; 4],
    pub panel_border: [u8; 4],
    pub text_primary: [u8; 4],
    pub text_secondary: [u8; 4],
    pub text_accent: [u8; 4],
    pub fps_good: [u8; 4],
    pub fps_mid: [u8; 4],
    pub fps_bad: [u8; 4],
    pub speed_low: [u8; 4],
    pub speed_mid: [u8; 4],
    pub speed_high: [u8; 4],
    pub speed_max: [u8; 4],
    pub telemetry_pos: [f32; 2],
    pub timer_pos_x: f32,
    pub speedometer_pos_y: f32,
    pub keys_pos_x: f32,
    pub keys_pos_y: f32,
    pub health_pos_y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrosshairConfig {
    pub color: [u8; 4],
    pub gap: f32,
    pub length: f32,
    pub thickness: f32,
    pub dot_radius: f32,
    pub pulse_speed: f32,
    pub pulse_amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    pub header_color: [u8; 4],
    pub input_color: [u8; 4],
    pub error_color: [u8; 4],
    pub help_color: [u8; 4],
    pub output_color: [u8; 4],
    pub max_height: f32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            title: TitleConfig {
                text: "PHOTON ENGINE".into(),
                size: 42.0,
                color: [255, 255, 255, 255],
                subtitle: "Vulkan 3D Engine".into(),
                subtitle_size: 14.0,
                subtitle_color: [140, 165, 210, 255],
                version: "v0.2".into(),
                offset_x: 0.0,
                offset_y: 0.0,
            },
            buttons: ButtonsConfig {
                width: 240.0,
                height: 44.0,
                text_size: 18.0,
                play_color: [30, 140, 60, 255],
                play_hover: [0, 255, 100, 255],
                maps_color: [0, 100, 220, 255],
                maps_hover: [0, 150, 255, 255],
                settings_color: [70, 80, 100, 255],
                settings_hover: [120, 140, 200, 255],
                quit_color: [180, 40, 50, 255],
                quit_hover: [255, 80, 80, 255],
                text_color: [255, 255, 255, 255],
                slide_in_speed: 3.0,
                glow_pulse_speed: 4.0,
                offset_x: 0.0,
                offset_y: 0.0,
            },
            menu: MenuConfig {
                bg_alpha: 200,
                particles: 30,
                particle_color: [100, 160, 255, 80],
                particle_speed: 0.2,
                wave_count: 5,
                wave_speed: 0.3,
                wave_color: [
                    [10, 20, 50, 20],
                    [25, 30, 70, 28],
                    [40, 40, 90, 36],
                    [55, 50, 110, 44],
                    [70, 60, 130, 52],
                ],
                hints: "M = Menu  |  F = Flashlight  |  F12 = Screenshot  |  ~ = Console".into(),
                hints_color: [80, 100, 140, 200],
            },
            hud: HudConfig {
                panel_bg: [15, 17, 26, 210],
                panel_border: [255, 255, 255, 30],
                text_primary: [255, 255, 255, 255],
                text_secondary: [160, 170, 190, 255],
                text_accent: [0, 229, 255, 255],
                fps_good: [0, 230, 118, 255],
                fps_mid: [255, 171, 0, 255],
                fps_bad: [255, 61, 0, 255],
                speed_low: [0, 229, 255, 255],
                speed_mid: [0, 230, 118, 255],
                speed_high: [255, 214, 0, 255],
                speed_max: [255, 61, 0, 255],
                telemetry_pos: [20.0, 20.0],
                timer_pos_x: 0.0,
                speedometer_pos_y: 60.0,
                keys_pos_x: 20.0,
                keys_pos_y: 25.0,
                health_pos_y: 25.0,
            },
            crosshair: CrosshairConfig {
                color: [0, 240, 255, 230],
                gap: 4.0,
                length: 6.0,
                thickness: 1.5,
                dot_radius: 1.2,
                pulse_speed: 3.0,
                pulse_amount: 0.15,
            },
            console: ConsoleConfig {
                header_color: [180, 200, 255, 255],
                input_color: [0, 229, 255, 255],
                error_color: [255, 82, 82, 255],
                help_color: [140, 160, 200, 255],
                output_color: [160, 200, 160, 255],
                max_height: 300.0,
            },
            video: VideoConfig {
                enabled: false,
                filename: "background.mp4".into(),
                loop_video: true,
                alpha: 1.0,
                brightness: 0.4,
            },
        }
    }
}

impl UiConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn find_assets_dir() -> std::path::PathBuf {
        // Try relative to the executable first
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let p = exe_dir.join("assets/ui.json");
                if p.exists() {
                    return exe_dir.to_path_buf();
                }
                // Also try one level up (e.g. target/release -> project root)
                if let Some(parent) = exe_dir.parent() {
                    let p = parent.join("assets/ui.json");
                    if p.exists() {
                        return parent.to_path_buf();
                    }
                }
            }
        }
        // Fallback to cwd
        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn find_assets_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(".")
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        let assets_dir = Self::find_assets_dir();
        let ui_json = assets_dir.join("assets/ui.json");

        log::info!("[UI] Looking for ui.json at: {}", ui_json.display());

        if ui_json.exists() {
            match std::fs::read_to_string(&ui_json) {
                Ok(data) => match serde_json::from_str::<UiConfig>(&strip_json_comments(&data)) {
                    Ok(config) => {
                        log::info!("[UI] Loaded UI config from {}", ui_json.display());
                        log::info!("[UI] Title: '{}' size={}", config.title.text, config.title.size);
                        log::info!("[UI] Play btn: {:?} hover: {:?}", config.buttons.play_color, config.buttons.play_hover);
                        log::info!("[UI] Particles: {} speed={}", config.menu.particles, config.menu.particle_speed);
                        return config;
                    }
                    Err(e) => {
                        log::warn!("[UI] Failed to parse {}: {}", ui_json.display(), e);
                    }
                },
                Err(e) => {
                    log::warn!("[UI] Failed to read {}: {}", ui_json.display(), e);
                }
            }
        }

        log::info!("[UI] No ui.json found, creating defaults at {}", ui_json.display());
        let default = Self::default();
        // Save to the exe-relative location
        if let Some(parent) = ui_json.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&default) {
            let _ = std::fs::write(&ui_json, json);
        }
        default
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return Self::default(),
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return Self::default(),
        };
        let data = match storage.get_item("ui_config") {
            Ok(Some(d)) => d,
            _ => return Self::default(),
        };
        match serde_json::from_str::<UiConfig>(&strip_json_comments(&data)) {
            Ok(config) => {
                log::info!("[UI] Loaded UI config from localStorage");
                config
            }
            Err(e) => {
                log::warn!("[UI] Failed to parse UI config from localStorage: {}", e);
                Self::default()
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        let assets_dir = Self::find_assets_dir();
        let ui_json = assets_dir.join("assets/ui.json");
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Some(parent) = ui_json.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&ui_json, json) {
                    log::error!("Failed to save ui.json: {}", e);
                } else {
                    log::info!("[UI] ui.json saved to {}", ui_json.display());
                }
            }
            Err(e) => {
                log::error!("Failed to serialize UI config: {}", e);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };
        let storage = match window.local_storage() {
            Ok(Some(s)) => s,
            _ => return,
        };
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if storage.set_item("ui_config", &json).is_err() {
                log::error!("Failed to save ui_config to localStorage");
            } else {
                log::info!("[UI] ui_config saved to localStorage");
            }
        }
    }

    pub fn to_color(c: [u8; 4]) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }
}
