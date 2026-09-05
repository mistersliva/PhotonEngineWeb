use crate::math::Vec3;
use crate::map::MapData;

#[derive(Debug, Clone)]
pub struct Entity {
    pub position: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub mesh_type: MeshType,
    pub color: [f32; 3],
    pub texture_index: usize,
    pub has_collision: bool,
    pub group_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeshType {
    Cube,
    Floor,
    Sphere,
    Light,
    Wall,
    Metal,
    Glow,
    Ramp,
    CurvedRamp,
    Cylinder,
}

pub struct Scene {
    pub entities: Vec<Entity>,
}

/// A runtime point light derived from emissive map entities
/// (Half-Life 2 style: `Light` / `Glow` brushes act as light sources).
#[derive(Debug, Clone, Copy)]
pub struct PointLightData {
    pub position: Vec3,
    pub color: [f32; 3],
    pub radius: f32,
    pub intensity: f32,
}

/// Maximum forward point lights supported by the shaders.
pub const MAX_POINT_LIGHTS: usize = 12;

impl Scene {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Collects up to `MAX_POINT_LIGHTS` point lights from emissive
    /// entities (`Light` / `Glow`), brightest-first, so maps get
    /// Half-Life 2 style dynamic lighting without any extra editing.
    pub fn collect_point_lights(&self) -> Vec<PointLightData> {
        let mut lights: Vec<PointLightData> = self
            .entities
            .iter()
            .filter(|e| {
                matches!(e.mesh_type, MeshType::Light | MeshType::Glow)
            })
            .map(|e| {
                let emissive = match e.mesh_type {
                    MeshType::Light => 1.2,
                    MeshType::Glow => 2.0,
                    _ => 0.0,
                };
                // Light radius scales with the fixture size so big
                // glow panels illuminate more of the map.
                let radius = (e.scale.x.max(e.scale.y).max(e.scale.z))
                    .clamp(60.0, 1200.0)
                    * 2.5;
                PointLightData {
                    position: e.position,
                    color: e.color,
                    radius,
                    intensity: emissive,
                }
            })
            .collect();

        // Brightest lights win when a map has more fixtures than the
        // forward renderer can handle in one pass.
        lights.sort_by(|a, b| {
            b.intensity
                .partial_cmp(&a.intensity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        lights.truncate(MAX_POINT_LIGHTS);
        lights
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn create_demo_scene(&mut self) {
        self.entities.clear();

        // 1. === THE WARM GOLDEN SUN (Visible in Sky) ===
        let sun_pos = Vec3::new(1200.0, 1600.0, -1000.0);
        self.add_entity(Entity {
            position: sun_pos,
            rotation: Vec3::ZERO,
            scale: Vec3::new(320.0, 320.0, 320.0),
            mesh_type: MeshType::Sphere,
            color: [1.0, 0.95, 0.70],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });
        self.add_entity(Entity {
            position: sun_pos,
            rotation: Vec3::ZERO,
            scale: Vec3::new(420.0, 420.0, 420.0),
            mesh_type: MeshType::Glow,
            color: [1.0, 0.85, 0.40],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // 2. Warm Sunlit Arena Floor (textured)
        self.add_entity(Entity {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(4096.0, 1.0, 4096.0),
            mesh_type: MeshType::Floor,
            color: [0.95, 0.92, 0.85],
            texture_index: 1,
            has_collision: true,
            group_id: 0,
        });

        // 3. Arena Enclosure Walls (Warm sunlit architectural concrete)
        let wall_h = 160.0;
        let wall_t = 16.0;
        let arena_r = 1300.0;
        for (pos, scale) in [
            (Vec3::new(0.0, wall_h, -arena_r), Vec3::new(arena_r * 2.0, wall_h * 2.0, wall_t)),
            (Vec3::new(0.0, wall_h, arena_r), Vec3::new(arena_r * 2.0, wall_h * 2.0, wall_t)),
            (Vec3::new(-arena_r, wall_h, 0.0), Vec3::new(wall_t, wall_h * 2.0, arena_r * 2.0)),
            (Vec3::new(arena_r, wall_h, 0.0), Vec3::new(wall_t, wall_h * 2.0, arena_r * 2.0)),
        ] {
            self.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale,
                mesh_type: MeshType::Wall,
                color: [0.88, 0.82, 0.74],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // 4. Central Golden Monument Tower
        self.add_entity(Entity {
            position: Vec3::new(0.0, 140.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(50.0, 280.0, 50.0),
            mesh_type: MeshType::Cylinder,
            color: [1.0, 0.82, 0.25],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });
        self.add_entity(Entity {
            position: Vec3::new(0.0, 290.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(45.0, 45.0, 45.0),
            mesh_type: MeshType::Sphere,
            color: [1.0, 0.90, 0.35],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });
        self.add_entity(Entity {
            position: Vec3::new(0.0, 290.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(65.0, 65.0, 65.0),
            mesh_type: MeshType::Glow,
            color: [1.0, 0.78, 0.20],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // 5. Elevated Start / Launch Platform (Player spawns here overlooking the sunny surf arena)
        self.add_entity(Entity {
            position: Vec3::new(0.0, 260.0, -850.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(220.0, 16.0, 200.0),
            mesh_type: MeshType::Metal,
            color: [0.30, 0.38, 0.48],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });
        // Glowing start pad trim
        self.add_entity(Entity {
            position: Vec3::new(0.0, 268.0, -850.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(210.0, 2.0, 190.0),
            mesh_type: MeshType::Glow,
            color: [0.0, 0.90, 1.0],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });
        // Drop launch ramp leading straight into surf course
        self.add_entity(Entity {
            position: Vec3::new(0.0, 210.0, -700.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(130.0, 100.0, 180.0),
            mesh_type: MeshType::Ramp,
            color: [0.0, 0.85, 1.0],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // 6. === VIBRANT ROUND CURVED SURF RAMPS ===
        // Track 1 (East Arc - Vibrant Electric Cyan & Sky Blue)
        let arc_center_e = Vec3::new(350.0, 0.0, -300.0);
        let arc_count = 14;
        for i in 0..arc_count {
            let t = i as f32 / arc_count as f32;
            let angle = t * std::f32::consts::PI * 0.85;
            let radius = 420.0;
            let x = arc_center_e.x + angle.sin() * radius;
            let z = arc_center_e.z + angle.cos() * radius;
            let y = 180.0 - t * 120.0;

            let yaw = angle;
            let r = 0.0 + t * 0.95;
            let g = 0.90 - t * 0.15;
            let b = 1.0 - t * 0.70;

            self.add_entity(Entity {
                position: Vec3::new(x, y, z),
                rotation: Vec3::new(0.0, yaw, 0.0),
                scale: Vec3::new(130.0, 65.0, 120.0),
                mesh_type: MeshType::CurvedRamp,
                color: [r, g, b],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // Track 2 (West Arc - Sunset Coral & Glowing Orange)
        let arc_center_w = Vec3::new(-350.0, 0.0, -300.0);
        for i in 0..arc_count {
            let t = i as f32 / arc_count as f32;
            let angle = -t * std::f32::consts::PI * 0.85;
            let radius = 420.0;
            let x = arc_center_w.x + angle.sin() * radius;
            let z = arc_center_w.z + angle.cos() * radius;
            let y = 180.0 - t * 120.0;

            let yaw = angle;
            let r = 1.0;
            let g = 0.50 + t * 0.40;
            let b = 0.10;

            self.add_entity(Entity {
                position: Vec3::new(x, y, z),
                rotation: Vec3::new(0.0, yaw, 0.0),
                scale: Vec3::new(130.0, 65.0, 120.0),
                mesh_type: MeshType::CurvedRamp,
                color: [r, g, b],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // 7. === ROUND SPIRAL SURF TOWER (South-West Quadrant) ===
        let spiral_center = Vec3::new(-520.0, 0.0, 450.0);
        let spiral_segs = 28;
        let spiral_r = 260.0;
        let spiral_h_max = 300.0;
        for i in 0..spiral_segs {
            let t = i as f32 / spiral_segs as f32;
            let angle = t * std::f32::consts::TAU * 1.5;
            let x = spiral_center.x + angle.cos() * spiral_r;
            let z = spiral_center.z + angle.sin() * spiral_r;
            let y = 30.0 + t * spiral_h_max;

            let yaw = -angle + std::f32::consts::FRAC_PI_2;
            let warm_r = 1.0;
            let warm_g = 0.70 + (t * 0.5).sin() * 0.25;
            let warm_b = 0.15 + (t * 0.5).cos() * 0.15;

            self.add_entity(Entity {
                position: Vec3::new(x, y, z),
                rotation: Vec3::new(0.0, yaw, 0.0),
                scale: Vec3::new(120.0, 55.0, 95.0),
                mesh_type: MeshType::CurvedRamp,
                color: [warm_r, warm_g, warm_b],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }
        // Center cylinder for spiral tower
        self.add_entity(Entity {
            position: Vec3::new(spiral_center.x, 180.0, spiral_center.z),
            rotation: Vec3::ZERO,
            scale: Vec3::new(80.0, 360.0, 80.0),
            mesh_type: MeshType::Cylinder,
            color: [0.92, 0.78, 0.45],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // 8. === CLASSIC TRIANGULAR SURF RIDGES (South-East Quadrant) ===
        for i in 0..10 {
            let z = 150.0 + (i as f32) * 90.0;
            let y = 140.0 - (i as f32) * 8.0;
            // Left slope (Emerald Green)
            self.add_entity(Entity {
                position: Vec3::new(450.0, y, z),
                rotation: Vec3::ZERO,
                scale: Vec3::new(85.0, 95.0, 100.0),
                mesh_type: MeshType::Ramp,
                color: [0.10, 0.90, 0.55],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
            // Right slope (Hot Pink / Magenta)
            self.add_entity(Entity {
                position: Vec3::new(540.0, y, z),
                rotation: Vec3::new(0.0, std::f32::consts::PI, 0.0),
                scale: Vec3::new(85.0, 95.0, 100.0),
                mesh_type: MeshType::Ramp,
                color: [1.0, 0.20, 0.65],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // 9. === SUNLIT PLATFORMS & SPEED JUMPS ===
        let platforms = [
            (Vec3::new(0.0, 80.0, 450.0), Vec3::new(260.0, 20.0, 180.0), [1.0, 0.85, 0.25], MeshType::Metal),
            (Vec3::new(300.0, 110.0, 50.0), Vec3::new(150.0, 20.0, 150.0), [0.95, 0.35, 0.35], MeshType::Cube),
            (Vec3::new(-300.0, 110.0, 50.0), Vec3::new(150.0, 20.0, 150.0), [0.20, 0.85, 0.95], MeshType::Cube),
            (Vec3::new(0.0, 50.0, -150.0), Vec3::new(180.0, 16.0, 180.0), [0.35, 0.90, 0.45], MeshType::Metal),
        ];

        for (pos, scale, color, mt) in platforms {
            self.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale,
                mesh_type: mt,
                color,
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // 10. === WARM GOLDEN GLOW LIGHT NODES & SCONCES ===
        for (pos, scale, color) in [
            (Vec3::new(0.0, 450.0, 0.0), 70.0, [1.0, 0.90, 0.40]),
            (Vec3::new(450.0, 280.0, -450.0), 45.0, [1.0, 0.80, 0.30]),
            (Vec3::new(-450.0, 280.0, -450.0), 45.0, [1.0, 0.80, 0.30]),
            (Vec3::new(450.0, 280.0, 450.0), 45.0, [1.0, 0.85, 0.35]),
            (Vec3::new(-450.0, 280.0, 450.0), 45.0, [1.0, 0.85, 0.35]),
            (Vec3::new(0.0, 340.0, -850.0), 50.0, [1.0, 0.92, 0.50]),
            (Vec3::new(0.0, 240.0, 750.0), 50.0, [1.0, 0.82, 0.35]),
        ] {
            self.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale: Vec3::new(scale, 14.0, scale),
                mesh_type: MeshType::Glow,
                color,
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // Wall sconces with warm ambient illumination
        for pos in [
            Vec3::new(-arena_r + 25.0, 160.0, 0.0),
            Vec3::new(arena_r - 25.0, 160.0, 0.0),
            Vec3::new(0.0, 160.0, -arena_r + 25.0),
            Vec3::new(0.0, 160.0, arena_r - 25.0),
            Vec3::new(-arena_r + 25.0, 160.0, -arena_r + 25.0),
            Vec3::new(arena_r - 25.0, 160.0, -arena_r + 25.0),
            Vec3::new(-arena_r + 25.0, 160.0, arena_r - 25.0),
            Vec3::new(arena_r - 25.0, 160.0, arena_r - 25.0),
        ] {
            self.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale: Vec3::new(32.0, 22.0, 32.0),
                mesh_type: MeshType::Glow,
                color: [1.0, 0.75, 0.25],
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }
    }

    pub fn load_from_map(&mut self, map_data: &MapData) {
        self.entities.clear();
        for entity in &map_data.entities {
            self.entities.push(Entity {
                position: entity.position,
                rotation: entity.rotation,
                scale: entity.scale,
                mesh_type: entity.mesh_type,
                color: entity.color,
                texture_index: entity.texture_index,
                has_collision: entity.has_collision,
                group_id: entity.group_id,
            });
        }
    }
}
