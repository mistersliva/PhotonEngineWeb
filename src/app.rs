use crate::input::InputState;
use crate::camera::Camera;
use crate::player::Player;
use crate::scene::Scene;
use crate::renderer::Renderer;
use crate::map::MapManager;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
#[cfg(not(target_arch = "wasm32"))]
use winit::window::{Window, WindowId, CursorGrabMode};
#[cfg(target_arch = "wasm32")]
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowMode {
    Windowed,
    Borderless,
    Exclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuPage {
    Main,
    Settings,
    MapSelect,
}

pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    camera: Camera,
    player: Player,
    scene: Scene,
    input: InputState,
    last_frame_time: Instant,
    session_start: Instant,
    _running: bool,
    show_menu: bool,
    menu_page: MenuPage,
    fps_counter: FpsCounter,
    settings_volume: u32,
    settings_sensitivity: u32,
    settings_fps_limit: u32,
    settings_fov: u32,
    shadows_enabled: bool,
    window_mode: WindowMode,
    max_speed: f32,
    jumps: u32,
    last_speed: f32,
    accel: f32,
    last_mouse_dx: f32,
    pub map_manager: MapManager,
    pub current_map: String,
    audio: crate::audio::AudioManager,
}

struct FpsCounter {
    frame_count: u32,
    last_time: Instant,
    fps: f32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            last_time: Instant::now(),
            fps: 0.0,
        }
    }

    fn tick(&mut self) -> f32 {
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_time).as_secs_f32();
        if elapsed >= 1.0 {
            self.fps = self.frame_count as f32 / elapsed;
            self.frame_count = 0;
            self.last_time = now;
        }
        self.fps
    }
}

impl App {
    pub fn new() -> Self {
        let config = crate::config::GameConfig::load();
        let window_mode = match config.window_mode {
            crate::config::WindowModeConfig::Windowed => WindowMode::Windowed,
            crate::config::WindowModeConfig::Borderless => WindowMode::Borderless,
            crate::config::WindowModeConfig::Exclusive => WindowMode::Exclusive,
        };
        let mut camera = Camera::new();
        camera.set_fov_deg(config.fov as f32);
        Self {
            window: None,
            renderer: None,
            camera,
            player: Player::new(),
            scene: Scene::new(),
            input: InputState::new(),
            last_frame_time: Instant::now(),
            session_start: Instant::now(),
            _running: true,
            show_menu: true,
            menu_page: MenuPage::Main,
            fps_counter: FpsCounter::new(),
            settings_volume: config.volume,
            settings_sensitivity: config.sensitivity,
            settings_fps_limit: config.fps_limit,
            settings_fov: config.fov.clamp(50, 110),
            shadows_enabled: config.shadows_enabled,
            window_mode,
            max_speed: 0.0,
            jumps: 0,
            last_speed: 0.0,
            accel: 0.0,
            last_mouse_dx: 0.0,
            map_manager: MapManager::new(),
            current_map: String::new(),
            audio: crate::audio::AudioManager::new(),
        }
    }

    fn save_config(&self) {
        let config = crate::config::GameConfig {
            volume: self.settings_volume,
            sensitivity: self.settings_sensitivity,
            fps_limit: self.settings_fps_limit,
            window_mode: crate::config::WindowModeConfig::from(&self.window_mode),
            fov: self.settings_fov,
            shadows_enabled: self.shadows_enabled,
        };
        config.save();
    }

    pub fn apply_window_mode(window: &Window, mode: WindowMode) {
        match mode {
            WindowMode::Windowed => {
                window.set_fullscreen(None);
            }
            WindowMode::Borderless => {
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
            WindowMode::Exclusive => {
                if let Some(monitor) = window.current_monitor() {
                    let video_mode = monitor.video_modes().next();
                    if let Some(mode) = video_mode {
                        window.set_fullscreen(Some(winit::window::Fullscreen::Exclusive(mode)));
                    } else {
                        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor))));
                    }
                } else {
                    window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                }
            }
        }
    }

    pub fn set_window_mode(&mut self, mode: WindowMode) {
        self.window_mode = mode;
        if let Some(window) = &self.window {
            Self::apply_window_mode(window, mode);
        }
    }

    pub fn load_map_by_name(&mut self, name: &str) -> Result<(), String> {
        if let Some(renderer) = &self.renderer {
            renderer.gpu.device_wait_idle();
        }
        let map_data = MapManager::load_map(name)?;
        self.scene.load_from_map(&map_data);
        self.player.position = map_data.spawn_position;
        self.player.velocity = glam::Vec3::ZERO;
        self.player.yaw = map_data.spawn_angles.x;
        self.player.pitch = map_data.spawn_angles.y;
        self.current_map = name.to_string();
        log::info!("Loaded map: {}", name);
        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("PhotonEngine")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        #[cfg(target_arch = "wasm32")]
        let attrs = {
            use winit::platform::web::WindowAttributesExtWebSys;
            attrs.with_canvas(
                web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.get_element_by_id("canvas"))
                    .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok()),
            )
        };

        let window = event_loop.create_window(attrs).unwrap();
        self.renderer = Some(pollster::block_on(Renderer::new(&window)));
        self.window = Some(window);

        if let Some(renderer) = &mut self.renderer {
            renderer.egui_state.init_video();
            renderer.shadows_enabled = self.shadows_enabled;
        }
        self.audio.set_volume(self.settings_volume);
        let size = self.window.as_ref().map(|w| w.inner_size());
        if let Some(s) = size {
            self.camera.set_aspect(s.width, s.height);
            self.camera.set_fov_deg(self.settings_fov as f32);
            self.camera.update_matrices();
        }
        self.scene = Scene::new();
        self.player = Player::new();
        self.current_map = String::new();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
            let consumed = renderer.egui_state.handle_event(window, &event);
            if consumed {
                return;
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.save_config();
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let (Some(renderer), Some(_window)) = (&mut self.renderer, &self.window) {
                    if size.width > 0 && size.height > 0 {
                        let current = renderer.swapchain.extent;
                        if size.width != current.width || size.height != current.height {
                            log::info!("Window resized from {}x{} to {}x{}, recreating swapchain...", current.width, current.height, size.width, size.height);
                            renderer.resize(size.width, size.height);
                            log::info!("Swapchain recreated successfully");
                        }
                        self.camera.set_aspect(size.width, size.height);
                        self.camera.update_matrices();
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.handle_cursor_moved(position.x, position.y);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                self.input.handle_mouse_button(button, state);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input.handle_key_event(&event);

                if self.input.is_key_just_pressed(winit::keyboard::KeyCode::Backquote) {
                    if let Some(renderer) = &mut self.renderer {
                        renderer.egui_state.console_open = !renderer.egui_state.console_open;
                        if renderer.egui_state.console_open {
                            self.set_cursor_grab(false);
                        }
                    }
                }

                if self.input.is_key_just_pressed(winit::keyboard::KeyCode::Escape) {
                    if let Some(renderer) = &mut self.renderer {
                        if renderer.egui_state.console_open {
                            renderer.egui_state.console_open = false;
                        } else if self.show_menu {
                            if self.menu_page == MenuPage::Settings || self.menu_page == MenuPage::MapSelect {
                                self.menu_page = MenuPage::Main;
                            } else {
                                self.show_menu = false;
                                self.set_cursor_grab(true);
                            }
                        } else {
                            self.show_menu = true;
                            self.menu_page = MenuPage::Main;
                            self.set_cursor_grab(false);
                        }
                    }
                }

                if self.input.is_key_just_pressed(winit::keyboard::KeyCode::KeyM) {
                    if let Some(renderer) = &mut self.renderer {
                        if !renderer.egui_state.console_open {
                            self.show_menu = !self.show_menu;
                            if self.show_menu {
                                self.menu_page = MenuPage::Main;
                            }
                            self.set_cursor_grab(!self.show_menu);
                        }
                    }
                }

                if self.input.is_key_just_pressed(winit::keyboard::KeyCode::KeyF) {
                    self.player.flashlight = !self.player.flashlight;
                    log::info!(
                        "Flashlight {}",
                        if self.player.flashlight { "ON" } else { "OFF" }
                    );
                }

                if self.input.is_key_just_pressed(winit::keyboard::KeyCode::F11) {
                    let next_mode = match self.window_mode {
                        WindowMode::Windowed => WindowMode::Borderless,
                        WindowMode::Borderless => WindowMode::Exclusive,
                        WindowMode::Exclusive => WindowMode::Windowed,
                    };
                    self.set_window_mode(next_mode);
                }

                let alt = self.input.is_key_pressed(winit::keyboard::KeyCode::AltLeft)
                    || self.input.is_key_pressed(winit::keyboard::KeyCode::AltRight);
                if alt && self.input.is_key_just_pressed(winit::keyboard::KeyCode::Enter) {
                    let next_mode = match self.window_mode {
                        WindowMode::Windowed => WindowMode::Borderless,
                        _ => WindowMode::Windowed,
                    };
                    self.set_window_mode(next_mode);
                }
            }
            WindowEvent::RedrawRequested => {
                #[cfg(not(target_arch = "wasm32"))]
                if self.settings_fps_limit > 0 {
                    let target_dt = 1.0 / self.settings_fps_limit as f32;
                    let elapsed = self.last_frame_time.elapsed().as_secs_f32();
                    if elapsed < target_dt {
                        let sleep_ms = ((target_dt - elapsed) * 1000.0 - 1.0).max(0.0) as u64;
                        if sleep_ms > 0 {
                            std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
                        }
                        while self.last_frame_time.elapsed().as_secs_f32() < target_dt {
                            std::hint::spin_loop();
                        }
                    }
                }

                let now = Instant::now();
                let dt = now.duration_since(self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;

                let fps = self.fps_counter.tick();

                if let Some(renderer) = &mut self.renderer {
                    renderer.egui_state.update_video(dt);
                }

                let no_map = self.scene.entities.is_empty();
                if no_map && self.show_menu {
                    let t = self.session_start.elapsed().as_secs_f32() * 0.08;
                    self.camera.position = crate::math::Vec3::new(
                        t.sin() * 1100.0,
                        520.0,
                        t.cos() * 1100.0,
                    );
                    let fwd =
                        (crate::math::Vec3::new(0.0, 120.0, 0.0) - self.camera.position)
                            .normalize();
                    self.camera.pitch = fwd.y.asin();
                    self.camera.yaw = fwd.z.atan2(fwd.x);
                }

                if !self.show_menu && !no_map {
                    if self.input.is_key_just_pressed(winit::keyboard::KeyCode::Space) && self.player.grounded {
                        self.jumps += 1;
                    }
                    if self.input.is_key_just_pressed(winit::keyboard::KeyCode::KeyN) {
                        self.session_start = Instant::now();
                        self.max_speed = 0.0;
                        self.jumps = 0;
                    }

                    self.player.process_input(&self.input, dt, &self.audio);
                    self.player.update(dt, &self.scene.entities);
                    let eye_offset = if self.player.crouching { 18.0 } else { 28.0 };
                    self.camera.position = self.player.position + crate::math::Vec3::new(0.0, eye_offset, 0.0);
                    self.camera.yaw = self.player.yaw;
                    self.camera.pitch = self.player.pitch;

                    let current_speed = glam::Vec2::new(self.player.velocity.x, self.player.velocity.z).length();
                    if current_speed > self.max_speed {
                        self.max_speed = current_speed;
                    }
                    if dt > 0.0001 {
                        self.accel = (current_speed - self.last_speed) / dt;
                    }
                    self.last_speed = current_speed;
                }
                self.camera.update_matrices();

                let volume_saved = self.settings_volume;
                if let (Some(renderer), Some(window)) = (&mut self.renderer, &self.window) {
                    let size = window.inner_size();

                    renderer.egui_state.begin_frame(window);

                    let mut show_menu = self.show_menu;
                    let mut menu_page = self.menu_page;
                    let mut settings_volume = self.settings_volume;
                    let mut settings_sensitivity = self.settings_sensitivity;
                    let mut settings_fps_limit = self.settings_fps_limit;
                    let mut settings_fov = self.settings_fov;
                    let mut shadows_enabled = self.shadows_enabled;
                    let prev_window_mode = self.window_mode;
                    let mut window_mode = self.window_mode;

                    if show_menu {
                        let elapsed = self.session_start.elapsed().as_secs_f32();
                        renderer.egui_state.draw_menu(
                            &mut settings_volume,
                            &mut settings_sensitivity,
                            &mut settings_fps_limit,
                            &mut settings_fov,
                            &mut shadows_enabled,
                            &mut window_mode,
                            &mut menu_page,
                            &mut show_menu,
                            &mut self.map_manager,
                            &mut self.scene,
                            &mut self.player,
                            &mut self.current_map,
                            elapsed,
                            &self.audio,
                        );
                    }

                    if window_mode != prev_window_mode {
                        self.window_mode = window_mode;
                        Self::apply_window_mode(window, window_mode);
                    }
                    self.camera.set_fov_deg(settings_fov as f32);
                    self.audio.set_volume(settings_volume);
                    renderer.shadows_enabled = shadows_enabled;

                    if !show_menu {
                        let current_speed = glam::Vec2::new(self.player.velocity.x, self.player.velocity.z).length();
                        let hud_data = crate::renderer::HudData {
                            fps,
                            frame_time_ms: if fps > 0.0 { 1000.0 / fps } else { 0.0 },
                            position: self.player.position,
                            velocity: self.player.velocity,
                            speed: current_speed,
                            max_speed: self.max_speed,
                            accel: self.accel,
                            yaw: self.player.yaw.to_degrees(),
                            pitch: self.player.pitch.to_degrees(),
                            health: 100,
                            grounded: self.player.grounded,
                            crouching: self.player.crouching,
                            key_w: self.input.is_key_pressed(winit::keyboard::KeyCode::KeyW),
                            key_a: self.input.is_key_pressed(winit::keyboard::KeyCode::KeyA),
                            key_s: self.input.is_key_pressed(winit::keyboard::KeyCode::KeyS),
                            key_d: self.input.is_key_pressed(winit::keyboard::KeyCode::KeyD),
                            key_jump: self.input.is_key_pressed(winit::keyboard::KeyCode::Space),
                            key_duck: self.player.crouching,
                            mouse_dx: self.last_mouse_dx,
                            session_time: self.session_start.elapsed().as_secs_f32(),
                            jumps: self.jumps,
                            flashlight: self.player.flashlight,
                            elapsed: self.session_start.elapsed().as_secs_f32(),
                            map_name: self.current_map.clone(),
                            entity_count: self.scene.entities.len(),
                            light_count: self.scene.collect_point_lights().len(),
                            shadows_enabled: self.shadows_enabled,
                            noclip: self.player.noclip,
                        };
                        renderer.egui_state.draw_hud(&hud_data);
                        self.last_mouse_dx *= 0.5;
                    }

                    renderer.egui_state.draw_console(
                        &mut self.map_manager,
                        &mut self.scene,
                        &mut self.player,
                        &mut self.current_map,
                    );

                    self.show_menu = show_menu;
                    self.menu_page = menu_page;
                    self.settings_volume = settings_volume;
                    self.settings_sensitivity = settings_sensitivity;
                    self.settings_fps_limit = settings_fps_limit;
                    self.settings_fov = settings_fov.clamp(50, 110);
                    self.shadows_enabled = shadows_enabled;

                    let want_screenshot = self.input.is_key_just_pressed(winit::keyboard::KeyCode::F12);

                    let _image_index = renderer.draw(
                        &self.scene,
                        &self.camera,
                        size.width,
                        size.height,
                        &self.input,
                        self.camera.position,
                        self.camera.forward(),
                        self.player.flashlight && !self.show_menu,
                        want_screenshot,
                    );
                }
                let volume_changed = self.settings_volume != volume_saved;
                if volume_changed {
                    self.save_config();
                }
                if let Some(window) = &self.window {
                    let console_open = self.renderer.as_ref().map_or(false, |r| r.egui_state.console_open);
                    self.set_cursor_grab(!self.show_menu && !console_open);
                    window.request_redraw();
                }
                self.input.end_frame();
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let Some(renderer) = &self.renderer {
            if renderer.egui_state.console_open {
                return;
            }
        }
        if !self.show_menu {
            if let DeviceEvent::MouseMotion { delta } = event {
                self.last_mouse_dx = delta.0 as f32;
                self.player.handle_mouse(delta.0 as f32, delta.1 as f32, self.settings_sensitivity);
            }
        }
    }
}

impl App {
    fn set_cursor_grab(&self, grab: bool) {
        if let Some(window) = &self.window {
            #[cfg(not(target_arch = "wasm32"))]
            if grab {
                let _ = window.set_cursor_grab(CursorGrabMode::Confined);
                window.set_cursor_visible(false);
            } else {
                let _ = window.set_cursor_grab(CursorGrabMode::None);
                window.set_cursor_visible(true);
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = (window, grab);
            }
        }
    }
}
