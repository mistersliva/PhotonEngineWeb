use crate::scene::{Entity, MeshType};
use crate::math::Vec3;

const MAGIC: &[u8; 4] = b"MPHT";
const VERSION: u16 = 3;
const MAPS_DIR: &str = "assets/maps";

#[derive(Debug, Clone)]
pub struct MapData {
    pub name: String,
    pub spawn_position: Vec3,
    pub spawn_angles: Vec3,
    pub entities: Vec<Entity>,
}

impl MapData {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            spawn_position: Vec3::new(0.0, 28.0, 0.0),
            spawn_angles: Vec3::ZERO,
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn remove_entity(&mut self, index: usize) -> Option<Entity> {
        if index < self.entities.len() {
            Some(self.entities.remove(index))
        } else {
            None
        }
    }
}

pub struct MapManager {
    pub current_map: String,
    pub available_maps: Vec<String>,
}

impl MapManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_map: String::new(),
            available_maps: Vec::new(),
        };
        manager.refresh_list();
        manager
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn refresh_list(&mut self) {
        self.available_maps.clear();
        let maps_dir = std::path::Path::new(MAPS_DIR);
        if !maps_dir.exists() {
            let _ = std::fs::create_dir_all(maps_dir);
            return;
        }
        if let Ok(entries) = std::fs::read_dir(maps_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("mpht") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        self.available_maps.push(name.to_string());
                    }
                }
            }
        }
        self.available_maps.sort();
    }

    #[cfg(target_arch = "wasm32")]
    pub fn refresh_list(&mut self) {
        self.available_maps.clear();
        // No directory listing on web; return empty or embedded list.
    }

    pub fn map_path(name: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("{}/{}.mpht", MAPS_DIR, name))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_map(name: &str) -> Result<MapData, String> {
        let path = Self::map_path(name);
        if !path.exists() {
            return Err(format!("Map '{}' not found at {}", name, path.display()));
        }
        Self::load_from_file(&path)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_map(name: &str) -> Result<MapData, String> {
        // On web, maps need to be loaded via async fetch.
        // For now, try to fetch synchronously using XMLHttpRequest as a fallback,
        // or return an error indicating maps must be pre-loaded.
        log::warn!("Map loading on web requires pre-loaded assets. Map '{}' not available.", name);
        Err(format!("Map '{}' not available on web. Use pre-loaded maps.", name))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_map(name: &str, data: &MapData) -> Result<(), String> {
        let path = Self::map_path(name);
        let parent = path.parent().ok_or("Invalid map path")?;
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create maps directory: {}", e))?;
        Self::save_to_file(&path, data)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn save_map(_name: &str, _data: &MapData) -> Result<(), String> {
        log::warn!("save_map is not supported on web (read-only)");
        Ok(())
    }

    pub fn create_default_map() -> MapData {
        let mut map = MapData::new("default");
        map.spawn_position = Vec3::new(0.0, 290.0, -850.0);
        map.spawn_angles = Vec3::new(0.0, 0.0, 0.0);

        let mut scene = crate::scene::Scene::new();
        scene.create_demo_scene();
        map.entities = scene.entities;
        map
    }

pub fn create_empty_map(name: &str) -> MapData {
        let mut map = MapData::new(name);
        map.spawn_position = Vec3::new(0.0, 28.0, 0.0);

        map.add_entity(Entity {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(2048.0, 1.0, 2048.0),
            mesh_type: MeshType::Floor,
            color: [1.0, 1.0, 1.0],
            texture_index: 1,
            has_collision: true,
            group_id: 0,
        });

        map
    }

    /// Creates a test lighting map with a house and lights inside/outside.
    /// This map is designed to demonstrate HL2-style point light baking:
    /// - Multiple `Light` and `Glow` entities act as dynamic point lights
    /// - A simple house structure with walls, floor, and ceiling
    /// - Lights placed indoors and outdoors for good coverage
    pub fn create_test_lights_map(name: &str) -> MapData {
        let mut map = MapData::new(name);
        map.spawn_position = Vec3::new(0.0, 28.0, 0.0);
        map.spawn_angles = Vec3::ZERO;

        // 1. Floor
        map.add_entity(Entity {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(4096.0, 1.0, 4096.0),
            mesh_type: MeshType::Floor,
            color: [0.95, 0.92, 0.85],
            texture_index: 1,
            has_collision: true,
            group_id: 0,
        });

        // 2. House walls - a simple enclosed structure
        let wall_h = 240.0;
        let wall_t = 20.0;
        let house_w = 400.0;
        let house_d = 300.0;

        // Front wall
        map.add_entity(Entity {
            position: Vec3::new(0.0, wall_h, house_d / 2.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(house_w, wall_h, wall_t),
            mesh_type: MeshType::Wall,
            color: [0.8, 0.75, 0.7],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // Back wall
        map.add_entity(Entity {
            position: Vec3::new(0.0, wall_h, -house_d / 2.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(house_w, wall_h, wall_t),
            mesh_type: MeshType::Wall,
            color: [0.8, 0.75, 0.7],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // Left wall
        map.add_entity(Entity {
            position: Vec3::new(-house_w / 2.0, wall_h, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(wall_t, wall_h, house_d),
            mesh_type: MeshType::Wall,
            color: [0.8, 0.75, 0.7],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // Right wall
        map.add_entity(Entity {
            position: Vec3::new(house_w / 2.0, wall_h, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(wall_t, wall_h, house_d),
            mesh_type: MeshType::Wall,
            color: [0.8, 0.75, 0.7],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // 3. Ceiling beams (a ring around the top)
        map.add_entity(Entity {
            position: Vec3::new(0.0, house_d + wall_h + 20.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::new(house_w + wall_t * 2.0, 20.0, house_d + wall_t * 2.0),
            mesh_type: MeshType::Cube,
            color: [0.6, 0.55, 0.5],
            texture_index: 0,
            has_collision: true,
            group_id: 0,
        });

        // 4. Indoor lights (inside the house)
        for (pos, color, mesh_type) in [
            // Ceiling light #1
            (Vec3::new(-80.0, 200.0, 60.0), [1.0, 0.95, 0.6], MeshType::Glow),
            // Ceiling light #2
            (Vec3::new(80.0, 200.0, 60.0), [1.0, 0.92, 0.5], MeshType::Glow),
            // Table light #1
            (Vec3::new(-150.0, 120.0, -50.0), [1.0, 0.85, 0.4], MeshType::Light),
            // Table light #2
            (Vec3::new(150.0, 120.0, -50.0), [1.0, 0.82, 0.35], MeshType::Light),
            // Floor lamp corner
            (Vec3::new(-180.0, 100.0, -120.0), [1.0, 0.80, 0.3], MeshType::Light),
            // Floor lamp other corner
            (Vec3::new(180.0, 100.0, -120.0), [1.0, 0.78, 0.25], MeshType::Light),
        ] {
            map.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale: Vec3::new(30.0, 30.0, 30.0),
                mesh_type,
                color,
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        // 5. Outdoor lights around the house
        for (pos, color, mesh_type) in [
            // Porch light front
            (Vec3::new(0.0, 250.0, 220.0), [1.0, 0.95, 0.7], MeshType::Glow),
            // Left outdoor floodlight
            (Vec3::new(-250.0, 200.0, 0.0), [1.0, 0.85, 0.5], MeshType::Light),
            // Right outdoor floodlight
            (Vec3::new(250.0, 200.0, 0.0), [1.0, 0.82, 0.35], MeshType::Light),
            // Backyard post light
            (Vec3::new(0.0, 220.0, -250.0), [1.0, 0.92, 0.55], MeshType::Glow),
            // Garden light left
            (Vec3::new(-200.0, 80.0, -200.0), [1.0, 0.80, 0.4], MeshType::Light),
            // Garden light right
            (Vec3::new(200.0, 80.0, -200.0), [1.0, 0.78, 0.3], MeshType::Light),
        ] {
            map.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale: Vec3::new(40.0, 40.0, 40.0),
                mesh_type,
                color,
                texture_index: 0,
                has_collision: true,
                group_id: 0,
            });
        }

        map
    }

    /// Showcase map: a small house with warm interior lighting, a porch,
    /// garden lamps, trees and a shadow-casting tower.
    ///
    /// Designed to show off the forward renderer: sun shadows from the
    /// directional light + up to 12 dynamic point lights from Light/Glow
    /// fixtures. Spawn is on the garden path looking at the house.
    pub fn create_house_lighting_map(name: &str) -> MapData {
        let mut map = MapData::new(name);
        map.spawn_position = Vec3::new(0.0, 90.0, 620.0);
        map.spawn_angles = Vec3::new(std::f32::consts::PI, 0.0, 0.0);

        let mut add = |pos: Vec3, scale: Vec3, mesh: MeshType, color: [f32; 3], tex: usize, collision: bool| {
            map.add_entity(Entity {
                position: pos,
                rotation: Vec3::ZERO,
                scale,
                mesh_type: mesh,
                color,
                texture_index: tex,
                has_collision: collision,
                group_id: 0,
            });
        };

        // --- Terrain: big grass ground ---
        add(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4096.0, 1.0, 4096.0),
            MeshType::Floor,
            [0.55, 0.72, 0.42],
            1,
            true,
        );

        // --- Garden path (stone slabs) ---
        for (i, z) in [180.0, 300.0, 420.0, 540.0].iter().enumerate() {
            let w = if i % 2 == 0 { 150.0 } else { 130.0 };
            add(
                Vec3::new(0.0, 4.0, *z),
                Vec3::new(w, 8.0, 90.0),
                MeshType::Cube,
                [0.62, 0.60, 0.58],
                0,
                true,
            );
        }

        // --- House foundation + wooden floor ---
        let hw = 420.0; // house width (x)
        let hd = 320.0; // house depth (z)
        let wall_t = 24.0;
        let wall_h = 200.0;
        let base_y = 10.0;
        add(
            Vec3::new(0.0, base_y, 0.0),
            Vec3::new(hw + 60.0, 20.0, hd + 60.0),
            MeshType::Cube,
            [0.45, 0.43, 0.42],
            0,
            true,
        );
        add(
            Vec3::new(0.0, base_y + 14.0, 0.0),
            Vec3::new(hw, 8.0, hd),
            MeshType::Cube,
            [0.72, 0.55, 0.34],
            0,
            true,
        );

        let cy = base_y + 20.0 + wall_h / 2.0; // wall center y
        let front_z = hd / 2.0;
        let back_z = -hd / 2.0;

        // Back wall (solid)
        add(
            Vec3::new(0.0, cy, back_z),
            Vec3::new(hw, wall_h, wall_t),
            MeshType::Wall,
            [0.86, 0.80, 0.70],
            0,
            true,
        );
        // Side walls (solid)
        add(
            Vec3::new(-hw / 2.0, cy, 0.0),
            Vec3::new(wall_t, wall_h, hd),
            MeshType::Wall,
            [0.84, 0.78, 0.68],
            0,
            true,
        );
        add(
            Vec3::new(hw / 2.0, cy, 0.0),
            Vec3::new(wall_t, wall_h, hd),
            MeshType::Wall,
            [0.84, 0.78, 0.68],
            0,
            true,
        );
        // Front wall with door hole + two window holes:
        // door 90 wide in center, windows 110 wide on each side.
        let seg_y = cy;
        // left segment
        add(
            Vec3::new(-152.5, seg_y, front_z),
            Vec3::new(115.0, wall_h, wall_t),
            MeshType::Wall,
            [0.88, 0.82, 0.72],
            0,
            true,
        );
        // right segment
        add(
            Vec3::new(152.5, seg_y, front_z),
            Vec3::new(115.0, wall_h, wall_t),
            MeshType::Wall,
            [0.88, 0.82, 0.72],
            0,
            true,
        );
        // middle-left (between door and left window)
        add(
            Vec3::new(-67.5, seg_y, front_z),
            Vec3::new(55.0, wall_h, wall_t),
            MeshType::Wall,
            [0.88, 0.82, 0.72],
            0,
            true,
        );
        // middle-right
        add(
            Vec3::new(67.5, seg_y, front_z),
            Vec3::new(55.0, wall_h, wall_t),
            MeshType::Wall,
            [0.88, 0.82, 0.72],
            0,
            true,
        );
        // lintel above door (door height 140)
        add(
            Vec3::new(0.0, seg_y + (wall_h / 2.0) - 30.0, front_z),
            Vec3::new(90.0, 60.0, wall_t),
            MeshType::Wall,
            [0.88, 0.82, 0.72],
            0,
            true,
        );
        // window sills + lintels (window band y 80..150 relative to floor)
        for x in [-152.5, 152.5] {
            // below window
            add(
                Vec3::new(x, base_y + 50.0, front_z),
                Vec3::new(110.0, 60.0, wall_t),
                MeshType::Wall,
                [0.88, 0.82, 0.72],
                0,
                true,
            );
            // above window
            add(
                Vec3::new(x, base_y + 170.0, front_z),
                Vec3::new(110.0, 60.0, wall_t),
                MeshType::Wall,
                [0.88, 0.82, 0.72],
                0,
                true,
            );
            // glowing window glass (no collision, emissive at dusk)
            add(
                Vec3::new(x, base_y + 115.0, front_z),
                Vec3::new(100.0, 60.0, 6.0),
                MeshType::Glow,
                [1.0, 0.85, 0.45],
                0,
                false,
            );
        }

        // --- Pitched roof: two slabs + gable ends + ridge beam ---
        let roof_y = base_y + 20.0 + wall_h + 55.0;
        for (x_sign, _yaw) in [(-1.0, 0.5_f32), (1.0, -0.5_f32)] {
            add(
                Vec3::new(x_sign * hw * 0.22, roof_y, 0.0),
                Vec3::new(hw * 0.62, 14.0, hd + 80.0),
                MeshType::Cube,
                [0.55, 0.28, 0.18],
                0,
                true,
            );
        }
        // gable triangles approximated with rotated cubes
        for z in [front_z, back_z] {
            add(
                Vec3::new(0.0, roof_y - 30.0, z),
                Vec3::new(hw * 0.7, 90.0, wall_t),
                MeshType::Wall,
                [0.82, 0.75, 0.65],
                0,
                true,
            );
        }
        // chimney (casts a nice long shadow)
        add(
            Vec3::new(120.0, roof_y + 90.0, -60.0),
            Vec3::new(50.0, 160.0, 50.0),
            MeshType::Cube,
            [0.50, 0.32, 0.28],
            0,
            true,
        );
        add(
            Vec3::new(120.0, roof_y + 175.0, -60.0),
            Vec3::new(66.0, 18.0, 66.0),
            MeshType::Cube,
            [0.35, 0.33, 0.34],
            0,
            true,
        );

        // --- Porch: slab + 4 posts + flat roof ---
        let porch_z = front_z + 110.0;
        add(
            Vec3::new(0.0, base_y + 6.0, porch_z),
            Vec3::new(300.0, 12.0, 200.0),
            MeshType::Cube,
            [0.60, 0.57, 0.54],
            0,
            true,
        );
        for x in [-135.0, 135.0] {
            for z in [porch_z - 85.0, porch_z + 85.0] {
                add(
                    Vec3::new(x, base_y + 110.0, z),
                    Vec3::new(18.0, 200.0, 18.0),
                    MeshType::Cube,
                    [0.42, 0.30, 0.20],
                    0,
                    true,
                );
            }
        }
        add(
            Vec3::new(0.0, base_y + 220.0, porch_z),
            Vec3::new(330.0, 16.0, 230.0),
            MeshType::Cube,
            [0.50, 0.26, 0.17],
            0,
            true,
        );

        // --- Interior furniture (all shadow casters) ---
        // big wooden table
        add(
            Vec3::new(-60.0, base_y + 55.0, -20.0),
            Vec3::new(160.0, 18.0, 100.0),
            MeshType::Cube,
            [0.48, 0.33, 0.20],
            0,
            true,
        );
        for (lx, lz) in [(-130.0, -60.0), (10.0, -60.0), (-130.0, 20.0), (10.0, 20.0)] {
            add(
                Vec3::new(lx, base_y + 25.0, lz),
                Vec3::new(12.0, 60.0, 12.0),
                MeshType::Cube,
                [0.40, 0.27, 0.16],
                0,
                true,
            );
        }
        // two stools
        add(
            Vec3::new(-60.0, base_y + 35.0, 90.0),
            Vec3::new(50.0, 50.0, 50.0),
            MeshType::Cube,
            [0.55, 0.38, 0.22],
            0,
            true,
        );
        add(
            Vec3::new(120.0, base_y + 35.0, -20.0),
            Vec3::new(50.0, 50.0, 50.0),
            MeshType::Cube,
            [0.55, 0.38, 0.22],
            0,
            true,
        );
        // bed in the back corner
        add(
            Vec3::new(-120.0, base_y + 35.0, -100.0),
            Vec3::new(140.0, 30.0, 90.0),
            MeshType::Cube,
            [0.30, 0.45, 0.70],
            0,
            true,
        );
        add(
            Vec3::new(-120.0, base_y + 55.0, -100.0),
            Vec3::new(130.0, 14.0, 80.0),
            MeshType::Cube,
            [0.85, 0.87, 0.90],
            0,
            false,
        );
        // fireplace against back wall
        add(
            Vec3::new(120.0, base_y + 60.0, back_z + 40.0),
            Vec3::new(120.0, 100.0, 50.0),
            MeshType::Cube,
            [0.45, 0.40, 0.38],
            0,
            true,
        );

        // --- Interior lights (warm) ---
        // ceiling lamp
        add(
            Vec3::new(0.0, base_y + 185.0, 0.0),
            Vec3::new(34.0, 34.0, 34.0),
            MeshType::Glow,
            [1.0, 0.90, 0.55],
            0,
            false,
        );
        // table candle
        add(
            Vec3::new(-60.0, base_y + 85.0, -20.0),
            Vec3::new(16.0, 22.0, 16.0),
            MeshType::Light,
            [1.0, 0.72, 0.30],
            0,
            false,
        );
        // fireplace fire glow
        add(
            Vec3::new(120.0, base_y + 70.0, back_z + 70.0),
            Vec3::new(60.0, 30.0, 20.0),
            MeshType::Glow,
            [1.0, 0.45, 0.12],
            0,
            false,
        );
        // bedside lamp
        add(
            Vec3::new(-180.0, base_y + 90.0, -120.0),
            Vec3::new(24.0, 40.0, 24.0),
            MeshType::Light,
            [1.0, 0.82, 0.50],
            0,
            false,
        );

        // --- Outdoor lights ---
        // porch lamp
        add(
            Vec3::new(0.0, base_y + 190.0, porch_z + 40.0),
            Vec3::new(26.0, 26.0, 26.0),
            MeshType::Glow,
            [1.0, 0.92, 0.60],
            0,
            false,
        );
        // garden lamps along path
        add(
            Vec3::new(-140.0, 80.0, 300.0),
            Vec3::new(22.0, 60.0, 22.0),
            MeshType::Light,
            [1.0, 0.80, 0.40],
            0,
            true,
        );
        add(
            Vec3::new(140.0, 80.0, 420.0),
            Vec3::new(22.0, 60.0, 22.0),
            MeshType::Light,
            [0.65, 0.85, 1.0],
            0,
            true,
        );
        // tall street lamp (cool white, big radius)
        add(
            Vec3::new(260.0, 260.0, 180.0),
            Vec3::new(18.0, 480.0, 18.0),
            MeshType::Metal,
            [0.25, 0.27, 0.30],
            0,
            true,
        );
        add(
            Vec3::new(260.0, 510.0, 180.0),
            Vec3::new(46.0, 46.0, 46.0),
            MeshType::Glow,
            [0.85, 0.92, 1.0],
            0,
            false,
        );

        // --- Shadow-casting set dressing ---
        // trees (trunk cylinder + foliage sphere)
        for (tx, tz, s) in [(-420.0, 120.0, 1.2), (430.0, -80.0, 1.5), (-350.0, -350.0, 1.0), (380.0, 420.0, 0.9)] {
            add(
                Vec3::new(tx, 90.0 * s, tz),
                Vec3::new(30.0 * s, 180.0 * s, 30.0 * s),
                MeshType::Cube,
                [0.40, 0.28, 0.17],
                0,
                true,
            );
            add(
                Vec3::new(tx, 230.0 * s, tz),
                Vec3::new(150.0 * s, 150.0 * s, 150.0 * s),
                MeshType::Sphere,
                [0.25, 0.55, 0.28],
                0,
                false,
            );
        }
        // fence posts along garden
        for i in 0..7 {
            let x = -300.0 + i as f32 * 100.0;
            add(
                Vec3::new(x, 50.0, 560.0),
                Vec3::new(16.0, 80.0, 16.0),
                MeshType::Cube,
                [0.50, 0.38, 0.25],
                0,
                true,
            );
        }
        add(
            Vec3::new(0.0, 70.0, 560.0),
            Vec3::new(640.0, 12.0, 12.0),
            MeshType::Cube,
            [0.50, 0.38, 0.25],
            0,
            false,
        );
        // tall obelisk tower to demo long sun shadows
        add(
            Vec3::new(-650.0, 250.0, -500.0),
            Vec3::new(90.0, 500.0, 90.0),
            MeshType::Metal,
            [0.70, 0.72, 0.78],
            0,
            true,
        );
        add(
            Vec3::new(-650.0, 520.0, -500.0),
            Vec3::new(60.0, 60.0, 60.0),
            MeshType::Glow,
            [1.0, 0.80, 0.35],
            0,
            false,
        );

        map
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_from_file(path: &std::path::Path) -> Result<MapData, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("Failed to open map: {}", e))?;
        let mut reader = std::io::BufReader::new(file);

        let mut magic = [0u8; 4];
        std::io::Read::read_exact(&mut reader, &mut magic).map_err(|e| format!("Read error: {}", e))?;
        if &magic != MAGIC {
            return Err("Invalid map file (bad magic)".into());
        }

        let mut version = [0u8; 2];
        std::io::Read::read_exact(&mut reader, &mut version).map_err(|e| format!("Read error: {}", e))?;
        let version = u16::from_le_bytes(version);
        if version > VERSION {
            return Err(format!("Unsupported map version: {}", version));
        }

        let mut entity_count = [0u8; 4];
        std::io::Read::read_exact(&mut reader, &mut entity_count).map_err(|e| format!("Read error: {}", e))?;
        let entity_count = u32::from_le_bytes(entity_count) as usize;

        let mut spawn_pos = [0u8; 12];
        std::io::Read::read_exact(&mut reader, &mut spawn_pos).map_err(|e| format!("Read error: {}", e))?;
        let spawn_position = Vec3::new(
            f32::from_le_bytes([spawn_pos[0], spawn_pos[1], spawn_pos[2], spawn_pos[3]]),
            f32::from_le_bytes([spawn_pos[4], spawn_pos[5], spawn_pos[6], spawn_pos[7]]),
            f32::from_le_bytes([spawn_pos[8], spawn_pos[9], spawn_pos[10], spawn_pos[11]]),
        );

        let mut spawn_ang = [0u8; 12];
        std::io::Read::read_exact(&mut reader, &mut spawn_ang).map_err(|e| format!("Read error: {}", e))?;
        let spawn_angles = Vec3::new(
            f32::from_le_bytes([spawn_ang[0], spawn_ang[1], spawn_ang[2], spawn_ang[3]]),
            f32::from_le_bytes([spawn_ang[4], spawn_ang[5], spawn_ang[6], spawn_ang[7]]),
            f32::from_le_bytes([spawn_ang[8], spawn_ang[9], spawn_ang[10], spawn_ang[11]]),
        );

        let mut reserved = [0u8; 14];
        std::io::Read::read_exact(&mut reader, &mut reserved).map_err(|e| format!("Read error: {}", e))?;

        let mut entities = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            let mut buf = [0u8; 52];
            std::io::Read::read_exact(&mut reader, &mut buf).map_err(|e| format!("Read error: {}", e))?;

            let position = Vec3::new(
                f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
                f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
                f32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            );
            let rotation = Vec3::new(
                f32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
                f32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]),
                f32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]),
            );
            let scale = Vec3::new(
                f32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]),
                f32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]),
                f32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]),
            );
            let color = [
                f32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]),
                f32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]]),
                f32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]),
            ];
            let mesh_type = match buf[48] {
                0 => MeshType::Cube,
                1 => MeshType::Floor,
                2 => MeshType::Sphere,
                3 => MeshType::Light,
                4 => MeshType::Wall,
                5 => MeshType::Metal,
                6 => MeshType::Glow,
                7 => MeshType::Ramp,
                8 => MeshType::CurvedRamp,
                9 => MeshType::Cylinder,
                _ => MeshType::Cube,
            };
            let texture_index = buf[49] as usize;

            let (has_collision, group_id) = if version >= 3 {
                let collision_byte = buf[50];
                (collision_byte & 1 != 0, (collision_byte >> 1) & 0x7F)
            } else if version >= 2 {
                (buf[50] != 0, 0u8)
            } else {
                (true, 0u8)
            };

            entities.push(Entity {
                position,
                rotation,
                scale,
                mesh_type,
                color,
                texture_index,
                has_collision,
                group_id,
            });
        }

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(MapData {
            name,
            spawn_position,
            spawn_angles,
            entities,
        })
    }

    #[cfg(target_arch = "wasm32")]
    fn load_from_bytes(bytes: &[u8], name: &str) -> Result<MapData, String> {
        if bytes.len() < 4 || &bytes[..4] != MAGIC {
            return Err("Invalid map data (bad magic)".into());
        }

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version > VERSION {
            return Err(format!("Unsupported map version: {}", version));
        }

        let entity_count = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;

        let spawn_position = Vec3::new(
            f32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]),
            f32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]),
            f32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]),
        );

        let spawn_angles = Vec3::new(
            f32::from_le_bytes([bytes[22], bytes[23], bytes[24], bytes[25]]),
            f32::from_le_bytes([bytes[26], bytes[27], bytes[28], bytes[29]]),
            f32::from_le_bytes([bytes[30], bytes[31], bytes[32], bytes[33]]),
        );

        // Skip 14 bytes reserved (offset 34..48)
        let mut offset = 48;

        let mut entities = Vec::with_capacity(entity_count);
        for _ in 0..entity_count {
            if offset + 52 > bytes.len() {
                return Err("Unexpected end of map data".into());
            }
            let buf = &bytes[offset..offset + 52];
            offset += 52;

            let position = Vec3::new(
                f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
                f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]),
                f32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]),
            );
            let rotation = Vec3::new(
                f32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]),
                f32::from_le_bytes([buf[16], buf[17], buf[18], buf[19]]),
                f32::from_le_bytes([buf[20], buf[21], buf[22], buf[23]]),
            );
            let scale = Vec3::new(
                f32::from_le_bytes([buf[24], buf[25], buf[26], buf[27]]),
                f32::from_le_bytes([buf[28], buf[29], buf[30], buf[31]]),
                f32::from_le_bytes([buf[32], buf[33], buf[34], buf[35]]),
            );
            let color = [
                f32::from_le_bytes([buf[36], buf[37], buf[38], buf[39]]),
                f32::from_le_bytes([buf[40], buf[41], buf[42], buf[43]]),
                f32::from_le_bytes([buf[44], buf[45], buf[46], buf[47]]),
            ];
            let mesh_type = match buf[48] {
                0 => MeshType::Cube,
                1 => MeshType::Floor,
                2 => MeshType::Sphere,
                3 => MeshType::Light,
                4 => MeshType::Wall,
                5 => MeshType::Metal,
                6 => MeshType::Glow,
                7 => MeshType::Ramp,
                8 => MeshType::CurvedRamp,
                9 => MeshType::Cylinder,
                _ => MeshType::Cube,
            };
            let texture_index = buf[49] as usize;

            let (has_collision, group_id) = if version >= 3 {
                let collision_byte = buf[50];
                (collision_byte & 1 != 0, (collision_byte >> 1) & 0x7F)
            } else if version >= 2 {
                (buf[50] != 0, 0u8)
            } else {
                (true, 0u8)
            };

            entities.push(Entity {
                position,
                rotation,
                scale,
                mesh_type,
                color,
                texture_index,
                has_collision,
                group_id,
            });
        }

        Ok(MapData {
            name: name.to_string(),
            spawn_position,
            spawn_angles,
            entities,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_to_file(path: &std::path::Path, data: &MapData) -> Result<(), String> {
        let file = std::fs::File::create(path).map_err(|e| format!("Failed to create map: {}", e))?;
        let mut writer = std::io::BufWriter::new(file);

        std::io::Write::write_all(&mut writer, MAGIC).map_err(|e| format!("Write error: {}", e))?;
        std::io::Write::write_all(&mut writer, &VERSION.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
        std::io::Write::write_all(&mut writer, &(data.entities.len() as u32).to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

        std::io::Write::write_all(&mut writer, &data.spawn_position.x.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
        std::io::Write::write_all(&mut writer, &data.spawn_position.y.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
        std::io::Write::write_all(&mut writer, &data.spawn_position.z.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

        std::io::Write::write_all(&mut writer, &data.spawn_angles.x.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
        std::io::Write::write_all(&mut writer, &data.spawn_angles.y.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
        std::io::Write::write_all(&mut writer, &data.spawn_angles.z.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

        let reserved = [0u8; 14];
        std::io::Write::write_all(&mut writer, &reserved).map_err(|e| format!("Write error: {}", e))?;

        for entity in &data.entities {
            std::io::Write::write_all(&mut writer, &entity.position.x.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.position.y.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.position.z.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

            std::io::Write::write_all(&mut writer, &entity.rotation.x.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.rotation.y.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.rotation.z.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

            std::io::Write::write_all(&mut writer, &entity.scale.x.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.scale.y.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.scale.z.to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

            std::io::Write::write_all(&mut writer, &entity.color[0].to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.color[1].to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;
            std::io::Write::write_all(&mut writer, &entity.color[2].to_le_bytes()).map_err(|e| format!("Write error: {}", e))?;

            let mesh_byte = match entity.mesh_type {
                MeshType::Cube => 0u8,
                MeshType::Floor => 1u8,
                MeshType::Sphere => 2u8,
                MeshType::Light => 3u8,
                MeshType::Wall => 4u8,
                MeshType::Metal => 5u8,
                MeshType::Glow => 6u8,
                MeshType::Ramp => 7u8,
                MeshType::CurvedRamp => 8u8,
                MeshType::Cylinder => 9u8,
            };
            std::io::Write::write_all(&mut writer, &[mesh_byte, entity.texture_index as u8]).map_err(|e| format!("Write error: {}", e))?;

            // v3 packs bit0 = collision, bits1-7 = group_id (see load path).
            let collision_byte =
                ((entity.group_id & 0x7F) << 1) | if entity.has_collision { 1u8 } else { 0u8 };
            let entity_reserved = [collision_byte, 0u8];
            std::io::Write::write_all(&mut writer, &entity_reserved).map_err(|e| format!("Write error: {}", e))?;
        }

        std::io::Write::flush(&mut writer).map_err(|e| format!("Flush error: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_default_map() {
        let default_map = MapManager::create_default_map();
        assert!(!default_map.entities.is_empty());
        let res = MapManager::save_map("default", &default_map);
        assert!(res.is_ok());

        let loaded = MapManager::load_map("default").unwrap();
        assert_eq!(loaded.entities.len(), default_map.entities.len());
    }

    #[test]
    fn test_save_and_load_house_lighting_map() {
        let map = MapManager::create_house_lighting_map("house_lighting");
        assert!(!map.entities.is_empty());
        let res = MapManager::save_map("house_lighting", &map);
        assert!(res.is_ok());

        let loaded = MapManager::load_map("house_lighting").unwrap();
        assert_eq!(loaded.entities.len(), map.entities.len());

        // Verify group_id round-trips correctly (v3 format).
        for (orig, loaded) in map.entities.iter().zip(loaded.entities.iter()) {
            assert_eq!(orig.group_id, loaded.group_id, "group_id mismatch");
            assert_eq!(orig.has_collision, loaded.has_collision, "collision mismatch");
        }
    }
}
