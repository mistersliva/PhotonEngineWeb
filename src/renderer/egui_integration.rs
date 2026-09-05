use crate::map::MapManager;
use crate::math::Vec3;
use crate::player::Player;
use crate::scene::{Entity, Scene};

pub struct EguiState {
    pub ctx: egui::Context,
    pub winit_state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
    pub console_open: bool,
    pub console_input: String,
    pub console_log: Vec<String>,
    pub clipped_primitives: Option<Vec<egui::ClippedPrimitive>>,
    pub map_dialog_open: bool,
    pub new_map_name: String,
    pub ui_config: crate::config::UiConfig,
    // TODO: Video support — add back when video player is ported to wgpu
    // pub video_player: Option<crate::video::VideoPlayer>,
    // pub video_texture: Option<egui::TextureHandle>,
    pub hover_sound_timer: f32,
    pub last_hovered_btn: std::cell::Cell<Option<egui::Id>>,
    pub map_search: String,
    map_list_cache: Vec<String>,
    map_list_time: std::time::Instant,
}

impl EguiState {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, window: &winit::window::Window) -> Self {
        let ctx = egui::Context::default();

        let winit_state = egui_winit::State::new(
            ctx.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = egui_wgpu::Renderer::new(
            device,
            config.format,
            None,
            1,
            false,
        );

        Self {
            ctx,
            winit_state,
            renderer,
            console_open: false,
            console_input: String::new(),
            console_log: vec![
                "PhotonEngine Console v0.3".into(),
                "Type 'help' for commands".into(),
            ],
            clipped_primitives: None,
            map_dialog_open: false,
            new_map_name: String::new(),
            ui_config: crate::config::UiConfig::load(),
            hover_sound_timer: 0.0,
            last_hovered_btn: std::cell::Cell::new(None),
            map_search: String::new(),
            map_list_cache: Vec::new(),
            map_list_time: std::time::Instant::now()
                .checked_sub(std::time::Duration::from_secs(3600))
                .unwrap_or_else(std::time::Instant::now),
        }
    }

    pub fn handle_event(
        &mut self,
        window: &winit::window::Window,
        event: &winit::event::WindowEvent,
    ) -> bool {
        let response = self.winit_state.on_window_event(window, event);
        response.consumed
    }

    pub fn begin_frame(&mut self, window: &winit::window::Window) {
        let raw_input = self.winit_state.take_egui_input(window);
        self.ctx.begin_pass(raw_input);
    }

    pub fn draw_menu(
        &mut self,
        settings_volume: &mut u32,
        settings_sensitivity: &mut u32,
        settings_fps_limit: &mut u32,
        settings_fov: &mut u32,
        shadows_enabled: &mut bool,
        window_mode: &mut crate::app::WindowMode,
        menu_page: &mut crate::app::MenuPage,
        show_menu: &mut bool,
        map_manager: &mut MapManager,
        scene: &mut Scene,
        player: &mut Player,
        current_map: &mut String,
        elapsed: f32,
        audio: &crate::audio::AudioManager,
    ) {
        let ctx = self.ctx.clone();

        match *menu_page {
            crate::app::MenuPage::Main => {
                let t = elapsed;
                let cfg = self.ui_config.clone();

                    egui::CentralPanel::default()
                        .frame(egui::Frame::NONE.fill(egui::Color32::from_black_alpha(cfg.menu.bg_alpha)))
                        .show(&ctx, |ui| {
                        let screen = ui.clip_rect();

                    // TODO: Video background (port video player to wgpu)
                    // if has_video { ... }

                    {
                        let painter = ui.painter();

                        // Diagonal grid lines
                        let grid_spacing = 60.0;
                        let grid_color = egui::Color32::from_rgba_unmultiplied(60, 55, 40, 18);
                        let offset = (t * 8.0) % grid_spacing;
                        let diag = screen.width() + screen.height();
                        let mut x = -diag + offset;
                        while x < diag {
                            let p1 = egui::pos2(screen.left() + x, screen.top());
                            let p2 = egui::pos2(screen.left() + x + screen.height(), screen.bottom());
                            painter.line_segment([p1, p2], egui::Stroke::new(0.5_f32, grid_color));
                            x += grid_spacing;
                        }

                        // Radial vignette effect (darken edges)
                        let center = screen.center();
                        let max_r = screen.width().max(screen.height()) * 0.7;
                        for ring in 0..8u32 {
                            let r = max_r * (ring as f32 / 8.0);
                            let alpha = ((ring as f32 / 8.0) * 40.0) as u8;
                            painter.circle_stroke(
                                center,
                                r,
                                egui::Stroke::new(max_r * 0.12, egui::Color32::from_rgba_unmultiplied(0, 0, 0, alpha)),
                            );
                        }

                        // Floating ember particles
                        let pc = cfg.menu.particle_color;
                        for i in 0..cfg.menu.particles {
                            let seed = i as f32 * 97.317;
                            let speed = cfg.menu.particle_speed;
                            let px = ((seed + t * (speed * 0.6 + (i % 7) as f32 * speed * 0.3)).sin() * 0.5 + 0.5) * screen.width();
                            let py = screen.bottom() - (((seed * 1.3 + t * (speed * 0.4 + (i % 4) as f32 * speed * 0.2)).sin() * 0.5 + 0.5) * screen.height());
                            let size = 1.2 + (t * 0.3 + seed).sin() * 0.6;
                            let alpha = ((t * 0.5 + seed).sin() * 0.5 + 0.5) * (pc[3] as f32) * 0.6 + pc[3] as f32 * 0.15;
                            painter.circle_filled(
                                egui::pos2(px, py),
                                size,
                                egui::Color32::from_rgba_unmultiplied(pc[0], pc[1], pc[2], alpha as u8),
                            );
                        }

                        // Thin horizontal rule lines for depth
                        for i in 0..3u32 {
                            let y = screen.top() + screen.height() * (0.3 + i as f32 * 0.2);
                            let alpha = 12u8;
                            painter.line_segment(
                                [egui::pos2(screen.left() + 40.0, y), egui::pos2(screen.right() - 40.0, y)],
                                egui::Stroke::new(0.5_f32, egui::Color32::from_rgba_unmultiplied(212, 175, 85, alpha)),
                            );
                        }
                    }

                    let center_x = screen.center().x + cfg.title.offset_x + cfg.buttons.offset_x;
                    let center_y = screen.center().y + cfg.title.offset_y - 30.0;
                    egui::Area::new(egui::Id::new("menu_center"))
                        .fixed_pos(egui::pos2(center_x, center_y))
                        .interactable(false)
                        .show(&ctx, |ui| {
                        ui.vertical_centered(|ui| {
                        ui.add_space(-190.0);

                        // Title
                        let tc = cfg.title.color;
                        let title_color = egui::Color32::from_rgb(tc[0], tc[1], tc[2]);

                        ui.heading(
                            egui::RichText::new(&cfg.title.text)
                                .size(cfg.title.size)
                                .strong()
                                .color(title_color),
                        );
                        ui.add_space(2.0);
                        let sc = cfg.title.subtitle_color;
                        ui.label(
                            egui::RichText::new(&cfg.title.subtitle)
                                .size(cfg.title.subtitle_size)
                                .color(egui::Color32::from_rgb(sc[0], sc[1], sc[2])),
                        );
                        ui.label(
                            egui::RichText::new(&cfg.title.version)
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 90, 65)),
                        );

                        // Decorative horizontal rule under title
                        ui.add_space(6.0);
                        let (rule_rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 1.0), egui::Sense::hover());
                        ui.painter().rect_filled(rule_rect, 0.0, egui::Color32::from_rgb(tc[0], tc[1], tc[2]));
                        ui.add_space(6.0);

                        let btn_w = cfg.buttons.width;
                        let btn_h = cfg.buttons.height;

                        ui.vertical_centered(|ui| {
                            let tc = cfg.buttons.text_color;
                            let text_col = egui::Color32::from_rgb(tc[0], tc[1], tc[2]);

                            let flat_btn = |ui: &mut egui::Ui, label: &str, base: [u8; 4], accent: [u8; 4], idx: f32| -> bool {
                                let delay = idx * 0.06;
                                let slide_in = ((t * cfg.buttons.slide_in_speed - delay).clamp(0.0, 1.0)).powf(2.5);

                                let resp = ui
                                    .add_sized(
                                        [btn_w * slide_in, btn_h],
                                        egui::Button::new(
                                            egui::RichText::new(label)
                                                .size(cfg.buttons.text_size)
                                                .strong()
                                                .color(text_col),
                                        )
                                        .fill(egui::Color32::from_rgba_unmultiplied(base[0], base[1], base[2], base[3])),
                                    );

                                if resp.hovered() {
                                    let prev = self.last_hovered_btn.get();
                                    if prev != Some(resp.id) {
                                        self.last_hovered_btn.set(Some(resp.id));
                                        audio.play_ui_sound();
                                    }
                                    let rect = resp.rect;
                                    let painter = ui.painter();
                                    // Subtle hover fill
                                    painter.rect_filled(rect, 2.0, egui::Color32::from_rgba_unmultiplied(accent[0], accent[1], accent[2], 25));
                                    // Left accent bar
                                    let bar = egui::Rect::from_min_size(
                                        egui::pos2(rect.left(), rect.top()),
                                        egui::vec2(3.0, rect.height()),
                                    );
                                    painter.rect_filled(bar, 1.0, egui::Color32::from_rgb(accent[0], accent[1], accent[2]));
                                }
                                resp.clicked()
                            };

                            let bc = &cfg.buttons;
                            if flat_btn(ui, "PLAY", bc.play_color, bc.play_hover, 0.0) {
                                audio.play_ui_sound();
                                if scene.entities.is_empty() {
                                    *menu_page = crate::app::MenuPage::MapSelect;
                                } else {
                                    *show_menu = false;
                                }
                            }

                            if scene.entities.is_empty() {
                                ui.add_space(2.0);
                                let warn_alpha = ((t * 2.0).sin() * 0.3 + 0.7) * 255.0;
                                ui.label(
                                    egui::RichText::new("No map loaded \u{2014} pick one in MAPS")
                                        .size(11.0)
                                        .color(egui::Color32::from_rgba_unmultiplied(212, 175, 85, warn_alpha as u8)),
                                );
                            }

                            ui.add_space(6.0);
                            if flat_btn(ui, "MAPS", bc.maps_color, bc.maps_hover, 1.0) {
                                audio.play_ui_sound();
                                *menu_page = crate::app::MenuPage::MapSelect;
                            }
                            ui.add_space(6.0);
                            if flat_btn(ui, "SETTINGS", bc.settings_color, bc.settings_hover, 2.0) {
                                audio.play_ui_sound();
                                *menu_page = crate::app::MenuPage::Settings;
                            }
                            ui.add_space(6.0);
                            if flat_btn(ui, "QUIT", bc.quit_color, bc.quit_hover, 3.0) {
                                audio.play_ui_sound();
                                std::process::exit(0);
                            }
                        });

                        ui.add_space(30.0);
                        let hc = cfg.menu.hints_color;
                        let hint_alpha = ((t * 1.5).sin() * 0.15 + 0.85) * hc[3] as f32;
                        ui.label(
                            egui::RichText::new(&cfg.menu.hints)
                                .size(10.0)
                                .color(egui::Color32::from_rgba_unmultiplied(hc[0], hc[1], hc[2], hint_alpha as u8)),
                        );
                    });
                    }); // Area for menu center
                }); // CentralPanel
            }
            crate::app::MenuPage::MapSelect => {
                self.draw_map_select(ctx, menu_page, show_menu, map_manager, scene, player, current_map, audio);
            }
            crate::app::MenuPage::Settings => {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.fill(egui::Color32::from_black_alpha(210)))
                    .show(&ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() / 2.0 - 200.0);
                        ui.heading(
                            egui::RichText::new("SETTINGS")
                                .size(28.0)
                                .color(egui::Color32::from_rgb(212, 175, 85)),
                        );
                        // Decorative rule
                        ui.add_space(4.0);
                        let (rule_rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 1.0), egui::Sense::hover());
                        ui.painter().rect_filled(rule_rect, 0.0, egui::Color32::from_rgb(212, 175, 85));
                        ui.add_space(16.0);

                        // -- DISPLAY --
                        ui.label(
                            egui::RichText::new("DISPLAY")
                                .size(11.0)
                                .strong()
                                .color(egui::Color32::from_rgb(140, 125, 90)),
                        );
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Window Mode")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            );
                            ui.add_space(20.0);
                            let modes = [
                                (crate::app::WindowMode::Windowed, "Windowed"),
                                (crate::app::WindowMode::Borderless, "Borderless"),
                                (crate::app::WindowMode::Exclusive, "Exclusive"),
                            ];
                            for (mode, label) in modes {
                                let is_active = *window_mode == mode;
                                let fill = if is_active {
                                    egui::Color32::from_rgb(160, 130, 40)
                                } else {
                                    egui::Color32::from_rgb(45, 42, 35)
                                };
                                let text_col = if is_active {
                                    egui::Color32::from_rgb(232, 220, 200)
                                } else {
                                    egui::Color32::from_rgb(120, 115, 100)
                                };
                                if ui
                                    .add_sized(
                                        [80.0, 26.0],
                                        egui::Button::new(
                                            egui::RichText::new(label)
                                                .size(12.0)
                                                .color(text_col),
                                        )
                                        .fill(fill),
                                    )
                                    .clicked()
                                {
                                    *window_mode = mode;
                                }
                            }
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("FOV")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            );
                            ui.add(egui::Slider::new(settings_fov, 50..=110).suffix("\u{00b0}"));
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Shadows")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            );
                            let shadow_label = if *shadows_enabled { "ON" } else { "OFF" };
                            let shadow_color = if *shadows_enabled {
                                egui::Color32::from_rgb(130, 180, 70)
                            } else {
                                egui::Color32::from_rgb(190, 70, 50)
                            };
                            if ui
                                .add_sized(
                                    [60.0, 26.0],
                                    egui::Button::new(
                                        egui::RichText::new(shadow_label)
                                            .size(12.0)
                                            .color(shadow_color),
                                    )
                                    .fill(egui::Color32::from_rgb(45, 42, 35)),
                                )
                                .clicked()
                            {
                                *shadows_enabled = !*shadows_enabled;
                            }
                        });

                        ui.add_space(12.0);

                        // -- GAMEPLAY --
                        ui.label(
                            egui::RichText::new("GAMEPLAY")
                                .size(11.0)
                                .strong()
                                .color(egui::Color32::from_rgb(140, 125, 90)),
                        );
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Volume")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            );
                            ui.add(egui::Slider::new(settings_volume, 0..=100));
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Sensitivity")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            );
                            ui.add(egui::Slider::new(settings_sensitivity, 0..=100));
                        });

                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("FPS Limit")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            );
                            let fps_text = if *settings_fps_limit == 0 {
                                "Unlimited".to_string()
                            } else {
                                format!("{} FPS", *settings_fps_limit)
                            };
                            ui.add(egui::Slider::new(settings_fps_limit, 0..=1000).text(fps_text));
                        });

                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width() / 2.0 - 80.0);
                            if ui
                                .add_sized(
                                    [150.0, 35.0],
                                    egui::Button::new(
                                        egui::RichText::new("APPLY")
                                            .size(16.0)
                                            .color(egui::Color32::from_rgb(232, 220, 200)),
                                    )
                                    .fill(egui::Color32::from_rgb(160, 130, 40)),
                                )
                                .clicked()
                            {
                                let config = crate::config::GameConfig {
                                    volume: *settings_volume,
                                    sensitivity: *settings_sensitivity,
                                    fps_limit: *settings_fps_limit,
                                    window_mode: crate::config::WindowModeConfig::from(&*window_mode),
                                    fov: *settings_fov,
                                    shadows_enabled: *shadows_enabled,
                                };
                                config.save();
                                log::info!("Settings applied and saved");
                            }
                        });

                        ui.add_space(8.0);

                        ui.label(
                            egui::RichText::new("ESC to go back")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(100, 90, 65)),
                        );
                    });
                });
            }
        }
    }

    fn draw_map_select(
        &mut self,
        ctx: egui::Context,
        menu_page: &mut crate::app::MenuPage,
        show_menu: &mut bool,
        map_manager: &mut MapManager,
        scene: &mut Scene,
        player: &mut Player,
        current_map: &mut String,
        audio: &crate::audio::AudioManager,
    ) {
        // Refresh list at most once per second
        if self.map_list_time.elapsed().as_millis() > 1000 {
            map_manager.refresh_list();
            self.map_list_cache = map_manager.available_maps.clone();
            self.map_list_time = std::time::Instant::now();
        }

        let search_lower = self.map_search.to_lowercase();

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_black_alpha(200)))
            .show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading(
                    egui::RichText::new("MAPS")
                        .size(28.0)
                        .color(egui::Color32::from_rgb(212, 175, 85)),
                );
                // Decorative rule
                ui.add_space(4.0);
                let (rule_rect, _) = ui.allocate_exact_size(egui::vec2(60.0, 1.0), egui::Sense::hover());
                ui.painter().rect_filled(rule_rect, 0.0, egui::Color32::from_rgb(212, 175, 85));
                ui.add_space(12.0);

                // Search box
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Search:").size(13.0).color(egui::Color32::from_rgb(140, 125, 90)));
                    ui.add(egui::TextEdit::singleline(&mut self.map_search).desired_width(220.0));
                });
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("New map name:").size(14.0).color(egui::Color32::from_rgb(232, 220, 200)));
                    ui.text_edit_singleline(&mut self.new_map_name);
                    if ui
                        .add_sized(
                            [100.0, 30.0],
                            egui::Button::new(
                                egui::RichText::new("CREATE")
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            )
                            .fill(egui::Color32::from_rgb(80, 100, 50)),
                        )
                        .clicked()
                    {
                        let name = self.new_map_name.trim().to_string();
                        if !name.is_empty() && !name.contains(' ') {
                            let new_map = MapManager::create_empty_map(&name);
                            if let Err(e) = MapManager::save_map(&name, &new_map) {
                                self.console_log.push(format!("Error creating map: {}", e));
                            } else {
                                map_manager.refresh_list();
                                self.map_list_cache = map_manager.available_maps.clone();
                                self.map_list_time = std::time::Instant::now();
                                self.new_map_name.clear();
                            }
                        }
                    }
                    if ui
                        .add_sized(
                            [140.0, 30.0],
                            egui::Button::new(
                                egui::RichText::new("HOUSE DEMO")
                                    .size(13.0)
                                    .color(egui::Color32::from_rgb(232, 220, 200)),
                            )
                            .fill(egui::Color32::from_rgb(120, 90, 30)),
                        )
                        .clicked()
                    {
                        let demo = MapManager::create_house_lighting_map("house_lighting");
                        let _ = MapManager::save_map("house_lighting", &demo);
                        map_manager.refresh_list();
                        self.map_list_cache = map_manager.available_maps.clone();
                        self.map_list_time = std::time::Instant::now();
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(6.0);

                // Filtered + cached list
                let maps: Vec<_> = self.map_list_cache.iter()
                    .filter(|n| search_lower.is_empty() || n.to_lowercase().contains(&search_lower))
                    .cloned()
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 80.0)
                    .show(ui, |ui| {
                        if maps.is_empty() {
                            ui.label(
                                egui::RichText::new("No maps found")
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(120, 110, 85)),
                            );
                        } else {
                            for map_name in &maps {
                                let is_current = *current_map == *map_name;

                                let btn_bg = if is_current {
                                    egui::Color32::from_rgb(35, 40, 25)
                                } else {
                                    egui::Color32::from_rgb(28, 26, 22)
                                };

                                ui.add_space(4.0);
                                egui::Frame::NONE.fill(btn_bg).inner_margin(egui::Margin::symmetric(12, 8)).show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(4.0);
                                        let label = if is_current {
                                            format!("{}  (LOADED)", map_name)
                                        } else {
                                            map_name.clone()
                                        };
                                        ui.label(
                                            egui::RichText::new(&label)
                                                .size(20.0)
                                                .color(if is_current {
                                                    egui::Color32::from_rgb(130, 180, 70)
                                                } else {
                                                    egui::Color32::from_rgb(232, 220, 200)
                                                }),
                                        );

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui
                                                .add_sized(
                                                    [80.0, 36.0],
                                                    egui::Button::new(
                                                        egui::RichText::new("DELETE")
                                                            .size(13.0)
                                                            .color(egui::Color32::from_rgb(232, 220, 200)),
                                                    )
                                                    .fill(egui::Color32::from_rgb(130, 55, 45)),
                                                )
                                                .clicked()
                                            {
                                                audio.play_ui_sound();
                                                let path = MapManager::map_path(map_name);
                                                let _ = std::fs::remove_file(path);
                                                map_manager.refresh_list();
                                                self.map_list_cache = map_manager.available_maps.clone();
                                                self.map_list_time = std::time::Instant::now();
                                            }

                                            if !is_current {
                                                if ui
                                                    .add_sized(
                                                        [120.0, 36.0],
                                                        egui::Button::new(
                                                            egui::RichText::new("LOAD")
                                                                .size(15.0)
                                                                .color(egui::Color32::from_rgb(232, 220, 200)),
                                                        )
                                                        .fill(egui::Color32::from_rgb(160, 130, 40)),
                                                    )
                                                    .clicked()
                                                {
                                                    audio.play_ui_sound();
                                                    match MapManager::load_map(map_name) {
                                                        Ok(map_data) => {
                                                            scene.load_from_map(&map_data);
                                                            player.position = map_data.spawn_position;
                                                            player.velocity = glam::Vec3::ZERO;
                                                            player.yaw = map_data.spawn_angles.x;
                                                            player.pitch = map_data.spawn_angles.y;
                                                            *current_map = map_name.clone();
                                                            *show_menu = false;
                                                        }
                                                        Err(e) => {
                                                            self.console_log.push(format!("Failed to load map: {}", e));
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    });
                                });
                                ui.add_space(2.0);
                            }
                        }
                    });

                ui.add_space(10.0);
                if ui
                    .add_sized(
                        [150.0, 40.0],
                        egui::Button::new(
                            egui::RichText::new("BACK")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(232, 220, 200)),
                        )
                        .fill(egui::Color32::from_rgb(50, 48, 42)),
                    )
                    .clicked()
                {
                    audio.play_ui_sound();
                    *menu_page = crate::app::MenuPage::Main;
                }
            });
        });
    }

    pub fn draw_hud(&mut self, data: &crate::renderer::HudData) {
        let ctx = self.ctx.clone();
        let cfg = &self.ui_config;

        let screen_rect = ctx.screen_rect();
        let screen_w = screen_rect.width();
        let screen_h = screen_rect.height();
        let t = data.elapsed;

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("hud_layer"),
        ));

        // 1. Animated Crosshair
        let center = egui::pos2(screen_w * 0.5, screen_h * 0.5);
        let pulse = (t * cfg.crosshair.pulse_speed).sin() * cfg.crosshair.pulse_amount + (1.0 - cfg.crosshair.pulse_amount);
        let cc = cfg.crosshair.color;
        let ch_color = egui::Color32::from_rgba_unmultiplied(cc[0], cc[1], cc[2], (cc[3] as f32 * pulse) as u8);
        let ch_shadow = egui::Color32::from_black_alpha(160);
        let ch_gap = cfg.crosshair.gap;
        let ch_len = cfg.crosshair.length * pulse;
        let ch_thick = cfg.crosshair.thickness;

        painter.line_segment([egui::pos2(center.x - ch_gap - ch_len, center.y), egui::pos2(center.x - ch_gap, center.y)], egui::Stroke::new(ch_thick + 1.0_f32, ch_shadow));
        painter.line_segment([egui::pos2(center.x + ch_gap, center.y), egui::pos2(center.x + ch_gap + ch_len, center.y)], egui::Stroke::new(ch_thick + 1.0_f32, ch_shadow));
        painter.line_segment([egui::pos2(center.x, center.y - ch_gap - ch_len), egui::pos2(center.x, center.y - ch_gap)], egui::Stroke::new(ch_thick + 1.0_f32, ch_shadow));
        painter.line_segment([egui::pos2(center.x, center.y + ch_gap), egui::pos2(center.x, center.y + ch_gap + ch_len)], egui::Stroke::new(ch_thick + 1.0_f32, ch_shadow));

        painter.line_segment([egui::pos2(center.x - ch_gap - ch_len, center.y), egui::pos2(center.x - ch_gap, center.y)], egui::Stroke::new(ch_thick, ch_color));
        painter.line_segment([egui::pos2(center.x + ch_gap, center.y), egui::pos2(center.x + ch_gap + ch_len, center.y)], egui::Stroke::new(ch_thick, ch_color));
        painter.line_segment([egui::pos2(center.x, center.y - ch_gap - ch_len), egui::pos2(center.x, center.y - ch_gap)], egui::Stroke::new(ch_thick, ch_color));
        painter.line_segment([egui::pos2(center.x, center.y + ch_gap), egui::pos2(center.x, center.y + ch_gap + ch_len)], egui::Stroke::new(ch_thick, ch_color));

        painter.circle_filled(center, cfg.crosshair.dot_radius * pulse, ch_color);

        let hcfg = &cfg.hud;
        let glass_frame = egui::Frame::new()
            .fill(egui::Color32::from_rgba_unmultiplied(hcfg.panel_bg[0], hcfg.panel_bg[1], hcfg.panel_bg[2], hcfg.panel_bg[3]))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(hcfg.panel_border[0], hcfg.panel_border[1], hcfg.panel_border[2], hcfg.panel_border[3])))
            .corner_radius(8.0_f32)
            .inner_margin(egui::Margin::symmetric(14, 10));

        // 2. Telemetry / Diagnostics (Top Left) with scan line
        egui::Area::new(egui::Id::new("hud_telemetry"))
            .fixed_pos(egui::pos2(hcfg.telemetry_pos[0], hcfg.telemetry_pos[1]))
            .interactable(false)
            .show(&ctx, |ui| {
                let frame = glass_frame.show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let fps_color = if data.fps >= 60.0 {
                                egui::Color32::from_rgb(hcfg.fps_good[0], hcfg.fps_good[1], hcfg.fps_good[2])
                            } else if data.fps >= 30.0 {
                                egui::Color32::from_rgb(hcfg.fps_mid[0], hcfg.fps_mid[1], hcfg.fps_mid[2])
                            } else {
                                egui::Color32::from_rgb(hcfg.fps_bad[0], hcfg.fps_bad[1], hcfg.fps_bad[2])
                            };
                            ui.label(egui::RichText::new(format!("{:.0}", data.fps)).size(18.0).strong().color(fps_color));
                            ui.label(egui::RichText::new(format!("FPS ({:.1} ms)", data.frame_time_ms)).size(12.0).color(egui::Color32::from_rgb(hcfg.text_secondary[0], hcfg.text_secondary[1], hcfg.text_secondary[2])));
                        });
                        ui.add_space(3.0);
                        // Map name
                        if !data.map_name.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("MAP").size(11.0).strong().color(egui::Color32::from_rgb(hcfg.text_accent[0], hcfg.text_accent[1], hcfg.text_accent[2])));
                                ui.label(egui::RichText::new(&data.map_name).size(12.0).color(egui::Color32::from_rgb(hcfg.text_primary[0], hcfg.text_primary[1], hcfg.text_primary[2])));
                            });
                        }
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("POS").size(11.0).strong().color(egui::Color32::from_rgb(hcfg.text_accent[0], hcfg.text_accent[1], hcfg.text_accent[2])));
                            ui.label(egui::RichText::new(format!("{:.1}  {:.1}  {:.1}", data.position.x, data.position.y, data.position.z)).size(12.0).monospace().color(egui::Color32::from_rgb(hcfg.text_primary[0], hcfg.text_primary[1], hcfg.text_primary[2])));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("ANG").size(11.0).strong().color(egui::Color32::from_rgb(hcfg.text_accent[0], hcfg.text_accent[1], hcfg.text_accent[2])));
                            ui.label(egui::RichText::new(format!("Y: {:.0}\u{00b0}  P: {:.0}\u{00b0}", data.yaw, data.pitch)).size(12.0).monospace().color(egui::Color32::from_rgb(hcfg.text_secondary[0], hcfg.text_secondary[1], hcfg.text_secondary[2])));
                        });
                        // Scene info + lights/shadows
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("SCN").size(11.0).strong().color(egui::Color32::from_rgb(hcfg.text_accent[0], hcfg.text_accent[1], hcfg.text_accent[2])));
                            let shadow_tag = if data.shadows_enabled { "SHD" } else { "NO-SHD" };
                            ui.label(egui::RichText::new(format!("{} ents | {} lights | {}", data.entity_count, data.light_count, shadow_tag)).size(12.0).color(egui::Color32::from_rgb(hcfg.text_secondary[0], hcfg.text_secondary[1], hcfg.text_secondary[2])));
                        });
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("LAMP").size(11.0).strong().color(egui::Color32::from_rgb(hcfg.text_accent[0], hcfg.text_accent[1], hcfg.text_accent[2])));
                            ui.label(egui::RichText::new(if data.flashlight { "ON  (F)" } else { "OFF (F)" }).size(12.0).monospace().color(if data.flashlight {
                                egui::Color32::from_rgb(255, 220, 120)
                            } else {
                                egui::Color32::from_rgb(hcfg.text_secondary[0], hcfg.text_secondary[1], hcfg.text_secondary[2])
                            }));
                        });
                        if data.noclip {
                            ui.label(egui::RichText::new("NOCLIP").size(10.0).strong().color(egui::Color32::from_rgb(190, 140, 50)));
                        }
                    });
                });
                // Scan line effect (amber)
                let scan_y = frame.response.rect.top() + (t * 40.0 % frame.response.rect.height());
                let scan_rect = egui::Rect::from_min_size(
                    egui::pos2(frame.response.rect.left(), scan_y),
                    egui::vec2(frame.response.rect.width(), 1.0),
                );
                painter.rect_filled(
                    scan_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(212, 175, 85, 25),
                );
            });

        // 3. Timer & Run Session (Top Center)
        let timer_w = 240.0;
        egui::Area::new(egui::Id::new("hud_timer"))
            .fixed_pos(egui::pos2((screen_w - timer_w) * 0.5 + hcfg.timer_pos_x, 20.0))
            .interactable(false)
            .show(&ctx, |ui| {
                glass_frame.show(ui, |ui| {
                    ui.set_width(timer_w - 28.0);
                    ui.vertical_centered(|ui| {
                        let mins = (data.session_time / 60.0).floor() as u32;
                        let secs = data.session_time % 60.0;
                        ui.label(
                            egui::RichText::new(format!("{:02}:{:05.2}", mins, secs))
                                .size(24.0)
                                .monospace()
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("JUMPS: {}", data.jumps)).size(11.0).color(egui::Color32::from_rgb(140, 130, 110)));
                            ui.label(egui::RichText::new("\u{2022}").size(10.0).color(egui::Color32::from_rgb(80, 75, 60)));
                            ui.label(egui::RichText::new(format!("MAX: {:.0} u/s", data.max_speed)).size(11.0).color(egui::Color32::from_rgb(212, 175, 85)));
                        });
                    });
                });
            });

        // 4. Momentum Speedometer (Center Bottom)
        let speed_color = if data.speed < 320.0 {
            egui::Color32::from_rgb(hcfg.speed_low[0], hcfg.speed_low[1], hcfg.speed_low[2])
        } else if data.speed < 600.0 {
            egui::Color32::from_rgb(hcfg.speed_mid[0], hcfg.speed_mid[1], hcfg.speed_mid[2])
        } else if data.speed < 900.0 {
            egui::Color32::from_rgb(hcfg.speed_high[0], hcfg.speed_high[1], hcfg.speed_high[2])
        } else {
            egui::Color32::from_rgb(hcfg.speed_max[0], hcfg.speed_max[1], hcfg.speed_max[2])
        };

        let speed_w = 260.0;
        let speed_h = 100.0;
        egui::Area::new(egui::Id::new("hud_speedometer"))
            .fixed_pos(egui::pos2((screen_w - speed_w) * 0.5, screen_h - speed_h - hcfg.speedometer_pos_y))
            .interactable(false)
            .show(&ctx, |ui| {
                glass_frame.show(ui, |ui| {
                    ui.set_width(speed_w - 28.0);
                    ui.vertical_centered(|ui| {
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(
                                egui::RichText::new(format!("{:.0}", data.speed))
                                    .size(38.0)
                                    .strong()
                                    .color(speed_color),
                            );
                            ui.vertical(|ui| {
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("UPS").size(12.0).strong().color(egui::Color32::from_rgb(120, 110, 85)));
                                let (accel_text, accel_col) = if data.accel > 20.0 {
                                    (format!("\u{25b2} +{:.0}", data.accel), egui::Color32::from_rgb(130, 180, 70))
                                } else if data.accel < -20.0 {
                                    (format!("\u{25bc} {:.0}", data.accel), egui::Color32::from_rgb(190, 70, 50))
                                } else {
                                    ("\u{2015}\u{2015}".into(), egui::Color32::from_rgb(100, 95, 75))
                                };
                                ui.label(egui::RichText::new(accel_text).size(11.0).strong().color(accel_col));
                            });
                        });

                        ui.add_space(4.0);

                        // Speed Gauge Bar (animated)
                        let bar_w = speed_w - 40.0;
                        let bar_h = 6.0;
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, bar_h), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgba_unmultiplied(28, 26, 22, 200));

                        let fill_pct = (data.speed / 1000.0).clamp(0.0, 1.0);
                        let shimmer = (t * 2.0).sin() * 0.5 + 0.5;
                        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(bar_w * fill_pct, bar_h));
                        ui.painter().rect_filled(fill_rect, 3.0, speed_color);
                        // Shimmer highlight
                        if fill_pct > 0.05 {
                            let shimmer_x = rect.left() + bar_w * fill_pct * shimmer;
                            let shimmer_rect = egui::Rect::from_min_size(
                                egui::pos2(shimmer_x - 8.0, rect.top()),
                                egui::vec2(16.0, bar_h),
                            ).intersect(fill_rect);
                            ui.painter().rect_filled(
                                shimmer_rect,
                                3.0,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 60),
                            );
                        }
                    });
                });
            });

        // 5. Key Overlay (Bottom Right)
        let key_w = 150.0;
        let key_h = 110.0;
        egui::Area::new(egui::Id::new("hud_keys"))
            .fixed_pos(egui::pos2(screen_w - key_w - hcfg.keys_pos_x, screen_h - key_h - hcfg.keys_pos_y))
            .interactable(false)
            .show(&ctx, |ui| {
                glass_frame.show(ui, |ui| {
                    ui.set_width(key_w - 28.0);
                    ui.vertical(|ui| {
                        let render_key = |ui: &mut egui::Ui, text: &str, active: bool, width: f32, height: f32| {
                            let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
                            let bg_color = if active {
                                egui::Color32::from_rgb(212, 175, 85)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(28, 26, 22, 180)
                            };
                            let text_color = if active {
                                egui::Color32::from_rgb(22, 20, 16)
                            } else {
                                egui::Color32::from_rgb(110, 100, 80)
                            };
                            let border_color = if active {
                                egui::Color32::from_rgb(232, 220, 200)
                            } else {
                                egui::Color32::from_rgba_unmultiplied(212, 175, 85, 20)
                            };

                            ui.painter().rect_filled(rect, 4.0, bg_color);
                            ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, border_color), egui::StrokeKind::Inside);
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::proportional(12.0),
                                text_color,
                            );
                        };

                        // Row 1: W
                        ui.horizontal(|ui| {
                        ui.add_space(30.0 + cfg.buttons.offset_y);
                            render_key(ui, "W", data.key_w, 36.0, 24.0);
                        });

                        ui.add_space(3.0);

                        // Row 2: A / S / D
                        ui.horizontal(|ui| {
                            render_key(ui, "A", data.key_a, 30.0, 24.0);
                            render_key(ui, "S", data.key_s, 30.0, 24.0);
                            render_key(ui, "D", data.key_d, 30.0, 24.0);
                        });

                        ui.add_space(3.0);

                        // Row 3: JUMP / DUCK
                        ui.horizontal(|ui| {
                            render_key(ui, "JUMP", data.key_jump, 50.0, 22.0);
                            render_key(ui, "DUCK", data.key_duck, 44.0, 22.0);
                        });
                    });
                });
            });

        // 6. Health & Status (Bottom Left)
        let hp_w = 180.0;
        let hp_h = 85.0;
        egui::Area::new(egui::Id::new("hud_health"))
            .fixed_pos(egui::pos2(20.0, screen_h - hp_h - hcfg.health_pos_y))
            .interactable(false)
            .show(&ctx, |ui| {
                glass_frame.show(ui, |ui| {
                    ui.set_width(hp_w - 28.0);
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("+").size(24.0).strong().color(egui::Color32::from_rgb(130, 180, 70)));
                            ui.label(egui::RichText::new(format!("{}", data.health)).size(24.0).strong().color(egui::Color32::from_rgb(232, 220, 200)));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let (badge_text, badge_col) = if data.crouching {
                                    ("DUCK", egui::Color32::from_rgb(212, 175, 85))
                                } else if data.grounded {
                                    ("GROUND", egui::Color32::from_rgb(130, 180, 70))
                                } else {
                                    ("AIR", egui::Color32::from_rgb(140, 130, 110))
                                };
                                ui.label(egui::RichText::new(badge_text).size(10.0).strong().color(badge_col));
                            });
                        });

                        ui.add_space(4.0);

                        // Health Bar
                        let bar_w = hp_w - 30.0;
                        let bar_h = 6.0;
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(bar_w, bar_h), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 3.0, egui::Color32::from_rgba_unmultiplied(28, 26, 22, 200));
                        let fill_pct = (data.health as f32 / 100.0).clamp(0.0, 1.0);
                        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(bar_w * fill_pct, bar_h));
                        ui.painter().rect_filled(fill_rect, 3.0, egui::Color32::from_rgb(130, 180, 70));
                    });
                });
            });
    }

    pub fn draw_console(
        &mut self,
        map_manager: &mut MapManager,
        scene: &mut Scene,
        player: &mut Player,
        current_map: &mut String,
    ) {
        if !self.console_open {
            return;
        }

        let ctx = self.ctx.clone();
        let cc = &self.ui_config.console;
        let header_color = cc.header_color;
        let input_color = cc.input_color;
        let error_color = cc.error_color;
        let help_color = cc.help_color;
        let output_color = cc.output_color;
        let max_h = cc.max_height;

        egui::TopBottomPanel::bottom("console")
            .max_height(max_h)
            .show(&ctx, |ui| {
                ui.label(
                    egui::RichText::new("Console  (~ to close)")
                        .strong()
                        .color(egui::Color32::from_rgb(header_color[0], header_color[1], header_color[2]))
                        .size(12.0),
                );
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(max_h - 100.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &self.console_log {
                            let color = if line.starts_with('>') {
                                egui::Color32::from_rgb(input_color[0], input_color[1], input_color[2])
                            } else if line.contains("Error") || line.contains("Failed") {
                                egui::Color32::from_rgb(error_color[0], error_color[1], error_color[2])
                            } else if line.starts_with("  ") {
                                egui::Color32::from_rgb(help_color[0], help_color[1], help_color[2])
                            } else {
                                egui::Color32::from_rgb(output_color[0], output_color[1], output_color[2])
                            };
                            ui.label(
                                egui::RichText::new(line.as_str())
                                    .color(color)
                                    .monospace()
                                    .size(12.0),
                            );
                        }
                    });

                ui.separator();

                let response = ui.text_edit_singleline(&mut self.console_input);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let cmd = self.console_input.trim().to_string();
                    if !cmd.is_empty() {
                        self.console_log.push(format!("> {}", cmd));
                        self.execute_command(&cmd, map_manager, scene, player, current_map);
                    }
                    self.console_input.clear();
                    response.request_focus();
                }

                if response.gained_focus() {
                    response.request_focus();
                }
            });
    }

    fn execute_command(
        &mut self,
        cmd: &str,
        map_manager: &mut MapManager,
        scene: &mut Scene,
        player: &mut Player,
        current_map: &mut String,
    ) {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        match parts.first().copied() {
            Some("help") => {
                self.console_log.push("Available commands:".into());
                self.console_log.push("  help              - Show this help".into());
                self.console_log.push("  clear             - Clear console".into());
                self.console_log.push("  version           - Show version".into());
                self.console_log.push("  quit              - Exit game".into());
                self.console_log.push("  tp x y z          - Teleport to position".into());
                self.console_log.push("  noclip            - Toggle noclip fly mode".into());
                self.console_log.push("  flashlight        - Toggle flashlight".into());
                self.console_log.push("  shadows           - Toggle sun shadows".into());
                self.console_log.push("  fov <degrees>     - Set field of view (50-110)".into());
                self.console_log.push("  map <name>        - Load a map".into());
                self.console_log.push("  map list          - List available maps".into());
                self.console_log.push("  map new <name>    - Create a new empty map".into());
                self.console_log.push("  map save          - Save current map".into());
            }
            Some("clear") => {
                self.console_log.clear();
            }
            Some("version") => {
                self.console_log.push("PhotonEngine v0.1".into());
            }
            Some("quit") | Some("exit") => {
                std::process::exit(0);
            }
            Some("tp") => {
                if parts.len() == 4 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    ) {
                        player.position = Vec3::new(x, y, z);
                        self.console_log
                            .push(format!("Teleported to ({}, {}, {})", x, y, z));
                    } else {
                        self.console_log
                            .push("Usage: tp <x> <y> <z>".into());
                    }
                } else {
                    self.console_log
                        .push("Usage: tp <x> <y> <z>".into());
                }
            }
            Some("noclip") => {
                player.noclip = !player.noclip;
                if player.noclip {
                    player.velocity = glam::Vec3::ZERO;
                    self.console_log.push("Noclip ON (WASD=move, Space=up, C/Ctrl=down, Shift=fast)".into());
                } else {
                    player.velocity = glam::Vec3::ZERO;
                    self.console_log.push("Noclip OFF".into());
                }
            }
            Some("flashlight") | Some("flash") => {
                player.flashlight = !player.flashlight;
                self.console_log.push(format!(
                    "Flashlight {}",
                    if player.flashlight { "ON" } else { "OFF" }
                ));
            }
            Some("shadows") | Some("shadow") => {
                self.console_log.push("Toggle shadows in Settings (SHIFT+ESC \u{2192} Settings)".into());
            }
            Some("fov") => {
                if parts.len() < 2 {
                    self.console_log.push("Usage: fov <degrees> (50-110)".into());
                } else if let Ok(v) = parts[1].parse::<u32>() {
                    let clamped = v.clamp(50, 110);
                    self.console_log.push(format!("FOV set to {}\u{00b0} (restart applies)", clamped));
                } else {
                    self.console_log.push("Usage: fov <degrees> (50-110)".into());
                }
            }
            Some("map") => {
                if parts.len() < 2 {
                    self.console_log.push("Usage:".into());
                    self.console_log.push("  map <name>      - Load a map".into());
                    self.console_log.push("  map list        - List available maps".into());
                    self.console_log.push("  map new <name>  - Create new map".into());
                    self.console_log.push("  map save        - Save current map".into());
                    return;
                }
                match parts[1] {
                    "list" => {
                        map_manager.refresh_list();
                        if map_manager.available_maps.is_empty() {
                            self.console_log.push("No maps found in assets/maps/".into());
                        } else {
                            self.console_log.push("Available maps:".into());
                            for name in &map_manager.available_maps {
                                let marker = if *name == *current_map { " (current)" } else { "" };
                                self.console_log.push(format!("  {}{}", name, marker));
                            }
                        }
                    }
                    "new" => {
                        if parts.len() < 3 {
                            self.console_log.push("Usage: map new <name>".into());
                            return;
                        }
                        let name = parts[2];
                        if name.contains(' ') {
                            self.console_log.push("Map name cannot contain spaces".into());
                            return;
                        }
                        let new_map = MapManager::create_empty_map(name);
                        match MapManager::save_map(name, &new_map) {
                            Ok(()) => {
                                map_manager.refresh_list();
                                self.console_log.push(format!("Created new map: {}", name));
                            }
                            Err(e) => {
                                self.console_log.push(format!("Failed to create map: {}", e));
                            }
                        }
                    }
                    "save" => {
                        if current_map.is_empty() {
                            self.console_log.push("No map loaded to save".into());
                            return;
                        }
                        let editor_entities: Vec<Entity> = scene.entities.iter().cloned().collect();
                        let map_data = crate::map::MapData {
                            name: current_map.clone(),
                            spawn_position: player.position,
                            spawn_angles: Vec3::new(player.yaw, player.pitch, 0.0),
                            entities: editor_entities,
                        };
                        match MapManager::save_map(current_map, &map_data) {
                            Ok(()) => {
                                self.console_log.push(format!("Saved map: {}", current_map));
                            }
                            Err(e) => {
                                self.console_log.push(format!("Failed to save map: {}", e));
                            }
                        }
                    }
                    map_name => {
                        match MapManager::load_map(map_name) {
                            Ok(map_data) => {
                                scene.load_from_map(&map_data);
                                player.position = map_data.spawn_position;
                                player.velocity = glam::Vec3::ZERO;
                                player.yaw = map_data.spawn_angles.x;
                                player.pitch = map_data.spawn_angles.y;
                                *current_map = map_name.to_string();
                                self.console_log.push(format!(
                                    "Loaded map '{}' ({} entities)",
                                    map_name,
                                    map_data.entities.len()
                                ));
                            }
                            Err(e) => {
                                self.console_log.push(format!("Failed to load map: {}", e));
                            }
                        }
                    }
                }
            }
            Some(unknown) => {
                self.console_log
                    .push(format!("Unknown command: {}", unknown));
            }
            None => {}
        }
    }

    // TODO: Video support — port video player to wgpu
    pub fn init_video(&mut self) {}
    pub fn update_video(&mut self, _dt: f32) {}

    pub fn end_frame_and_upload_textures(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let output = self.ctx.end_pass();

        let paint_jobs = self.ctx.tessellate(output.shapes, self.ctx.pixels_per_point());

        // Upload textures
        let textures_delta_set: Vec<_> = output.textures_delta.set.into_iter().collect();
        if !textures_delta_set.is_empty() {
            for (id, image_delta) in &textures_delta_set {
                self.renderer.update_texture(device, queue, *id, image_delta);
            }
        }

        self.clipped_primitives = Some(paint_jobs);

        // Schedule texture frees for next frame
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }

    pub fn cmd_draw(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'_>,
        screen_width: u32,
        screen_height: u32,
        pixels_per_point: f32,
    ) {
        if let Some(clipped_primitives) = self.clipped_primitives.take() {
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [screen_width, screen_height],
                pixels_per_point,
            };
            // SAFETY: The render_pass and self.egui_state.renderer have compatible lifetimes
            // in practice — both are alive for the duration of this call.
            unsafe {
                let pass_static = std::mem::transmute::<
                    &mut wgpu::RenderPass<'_>,
                    &mut wgpu::RenderPass<'static>,
                >(render_pass);
                self.renderer.render(pass_static, &clipped_primitives, &screen_descriptor);
            }
        }
    }

    pub fn recreate(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.ctx.forget_all_images();
        self.renderer = egui_wgpu::Renderer::new(
            device,
            config.format,
            None,
            1,
            false,
        );
    }
}
