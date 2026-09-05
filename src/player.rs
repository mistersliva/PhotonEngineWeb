use crate::math::Vec3;
use crate::input::InputState;
use glam::Vec3Swizzles;
use winit::keyboard::KeyCode;

const WALK_SPEED: f32 = 320.0;
const CROUCH_SPEED: f32 = 140.0;
const SPRINT_SPEED: f32 = 520.0;
const STOP_SPEED: f32 = 100.0;
const GROUND_FRICTION: f32 = 6.0;
const GROUND_ACCEL: f32 = 10.0;
const AIR_ACCEL: f32 = 12.0;
const AIR_SPEED_CAP: f32 = 30.0;
const JUMP_VELOCITY: f32 = 290.0;
const GRAVITY: f32 = -800.0;
const PLAYER_HEIGHT: f32 = 72.0;
const NOCLIP_SPEED: f32 = 600.0;
const NOCLIP_FAST_SPEED: f32 = 2000.0;
const CROUCH_HEIGHT: f32 = 54.0;
const PLAYER_WIDTH: f32 = 32.0;

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
    pub crouching: bool,
    pub noclip: bool,
    /// Half-Life 2 style flashlight (toggled with F or `flashlight`).
    pub flashlight: bool,
    pub step_timer: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(0.0, PLAYER_HEIGHT * 0.5, 0.0),
            velocity: Vec3::ZERO,
            yaw: -90.0_f32.to_radians(),
            pitch: 0.0,
            grounded: true,
            crouching: false,
            noclip: false,
            flashlight: false,
            step_timer: 0.0,
        }
    }

    /// Apply mouse look. `sensitivity` is the slider value 0-100,
    /// mapped so 50 == the original MOUSE_SENSITIVITY constant.
    pub fn handle_mouse(&mut self, dx: f32, dy: f32, sensitivity: u32) {
        // Linear ramp: 0 -> 0.0004, 50 -> 0.002, 100 -> 0.01
        let sens = 0.0004 + (sensitivity as f32) * 0.000096;
        self.yaw += dx * sens;
        self.pitch -= dy * sens;
        self.pitch = self.pitch.clamp(
            -89.0_f32.to_radians(),
            89.0_f32.to_radians(),
        );
    }

    pub fn process_input(&mut self, input: &InputState, dt: f32, audio: &crate::audio::AudioManager) {
        if self.noclip {
            self.noclip_move(input, dt);
            return;
        }

        let forward = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize();
        let right = Vec3::new(-self.yaw.sin(), 0.0, self.yaw.cos()).normalize();

        self.crouching = input.is_key_pressed(KeyCode::ControlLeft) || input.is_key_pressed(KeyCode::ControlRight);

        let speed = if self.crouching {
            CROUCH_SPEED
        } else if input.is_key_pressed(KeyCode::ShiftLeft)
            || input.is_key_pressed(KeyCode::ShiftRight)
        {
            SPRINT_SPEED
        } else {
            WALK_SPEED
        };

        let mut wish_dir = Vec3::ZERO;

        if input.is_key_pressed(KeyCode::KeyW) {
            wish_dir += forward;
        }
        if input.is_key_pressed(KeyCode::KeyS) {
            wish_dir -= forward;
        }
        if input.is_key_pressed(KeyCode::KeyD) {
            wish_dir += right;
        }
        if input.is_key_pressed(KeyCode::KeyA) {
            wish_dir -= right;
        }

        if wish_dir.length_squared() > 0.0 {
            wish_dir = wish_dir.normalize();
        }

        if self.grounded {
            self.ground_move(wish_dir, speed, dt);
        } else {
            self.air_move(wish_dir, speed, dt);
        }

        if input.is_key_pressed(KeyCode::Space) && self.grounded {
            self.velocity.y = JUMP_VELOCITY;
            self.grounded = false;
            audio.play_jump();
        }

        // Footstep sounds
        let moving = self.grounded && wish_dir.length_squared() > 0.0 && self.velocity.xz().length() > 10.0;
        if moving {
            self.step_timer -= dt;
            if self.step_timer <= 0.0 {
                let speed = self.velocity.xz().length();
                let speed_factor = speed / WALK_SPEED;
                let step_interval = 0.32 / speed_factor.max(0.5);
                self.step_timer = step_interval;
                audio.play_footstep();
            }
        } else {
            self.step_timer = 0.15;
        }
    }

    fn noclip_move(&mut self, input: &InputState, dt: f32) {
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize();
        let right = forward.cross(Vec3::Y).normalize();

        let speed = if input.is_key_pressed(KeyCode::ShiftLeft) {
            NOCLIP_FAST_SPEED
        } else {
            NOCLIP_SPEED
        };

        let mut wish_dir = Vec3::ZERO;
        if input.is_key_pressed(KeyCode::KeyW) {
            wish_dir += forward;
        }
        if input.is_key_pressed(KeyCode::KeyS) {
            wish_dir -= forward;
        }
        if input.is_key_pressed(KeyCode::KeyD) {
            wish_dir += right;
        }
        if input.is_key_pressed(KeyCode::KeyA) {
            wish_dir -= right;
        }
        if input.is_key_pressed(KeyCode::Space) {
            wish_dir.y += 1.0;
        }
        if input.is_key_pressed(KeyCode::ControlLeft) || input.is_key_pressed(KeyCode::KeyC) {
            wish_dir.y -= 1.0;
        }

        if wish_dir.length_squared() > 0.0 {
            wish_dir = wish_dir.normalize();
        }

        self.velocity = wish_dir * speed;
        self.position += self.velocity * dt;
    }

    fn apply_friction(&mut self, dt: f32) {
        let speed = self.velocity.xz().length();
        if speed < 0.01 {
            self.velocity.x = 0.0;
            self.velocity.z = 0.0;
            return;
        }

        let control = if speed < STOP_SPEED { STOP_SPEED } else { speed };
        let drop = control * GROUND_FRICTION * dt;
        let new_speed = (speed - drop).max(0.0);

        if speed > 0.0 {
            let factor = new_speed / speed;
            self.velocity.x *= factor;
            self.velocity.z *= factor;
        }
    }

    fn ground_move(&mut self, wish_dir: Vec3, speed: f32, dt: f32) {
        self.apply_friction(dt);

        if wish_dir.length_squared() > 0.0 {
            let current_speed = glam::Vec2::dot(self.velocity.xz(), wish_dir.xz());
            let add_speed = speed - current_speed;
            if add_speed > 0.0 {
                let accel_speed = (GROUND_ACCEL * speed * dt).min(add_speed);
                self.velocity.x += wish_dir.x * accel_speed;
                self.velocity.z += wish_dir.z * accel_speed;
            }
        }

        self.velocity.y += GRAVITY * dt;
    }

    fn air_move(&mut self, wish_dir: Vec3, speed: f32, dt: f32) {
        if wish_dir.length_squared() > 0.0 {
            let wish_speed = speed.min(AIR_SPEED_CAP);
            let cur_speed = glam::Vec2::dot(self.velocity.xz(), wish_dir.xz());
            let add_speed = wish_speed - cur_speed;
            if add_speed > 0.0 {
                let accel_speed = (AIR_ACCEL * speed * dt).min(add_speed);
                self.velocity.x += wish_dir.x * accel_speed;
                self.velocity.z += wish_dir.z * accel_speed;
            }
        }

        self.velocity.y += GRAVITY * dt;
    }

    pub fn update(&mut self, dt: f32, entities: &[crate::scene::Entity]) {
        if self.noclip {
            self.grounded = false;
            return;
        }

        self.position += self.velocity * dt;

        let height = if self.crouching { CROUCH_HEIGHT } else { PLAYER_HEIGHT };
        let half_w = PLAYER_WIDTH * 0.5;
        let half_h = height * 0.5;

        let mut on_surface = false;

        let player_min_x = self.position.x - half_w;
        let player_max_x = self.position.x + half_w;
        let player_min_y = self.position.y - half_h;
        let player_max_y = self.position.y + half_h;
        let player_min_z = self.position.z - half_w;
        let player_max_z = self.position.z + half_w;

        for entity in entities {
            if !entity.has_collision {
                continue;
            }

            if entity.mesh_type == crate::scene::MeshType::Ramp || entity.mesh_type == crate::scene::MeshType::CurvedRamp {
                // Transform player into local space of the ramp
                let to_player = self.position - entity.position;
                let rot_inv = glam::Quat::from_euler(
                    glam::EulerRot::YXZ,
                    entity.rotation.y,
                    entity.rotation.x,
                    entity.rotation.z,
                ).inverse();
                let local_pos = rot_inv * to_player;
                let half_scale = entity.scale * 0.5;

                // Check bounding box in local space
                if local_pos.x >= -half_scale.x - half_w && local_pos.x <= half_scale.x + half_w
                    && local_pos.z >= -half_scale.z - half_w && local_pos.z <= half_scale.z + half_w
                    && local_pos.y >= -half_scale.y - half_h && local_pos.y <= half_scale.y + half_h
                {
                    // For Ramp: slope connects (Z = half_scale.z, Y = -half_scale.y) to (Z = -half_scale.z, Y = half_scale.y)
                    let slope_y = -local_pos.z * (entity.scale.y / entity.scale.z);
                    let dist = local_pos.y - slope_y;

                    if dist < half_h && dist > -half_scale.y {
                        let local_normal = glam::Vec3::new(0.0, entity.scale.z, entity.scale.y).normalize();
                        let world_normal = glam::Quat::from_euler(
                            glam::EulerRot::YXZ,
                            entity.rotation.y,
                            entity.rotation.x,
                            entity.rotation.z,
                        ) * local_normal;

                        let push = (half_h - dist).max(0.0);
                        self.position += world_normal * push;

                        // Surf physics: slide along surface normal without friction
                        let n_dot_v = self.velocity.dot(world_normal);
                        if n_dot_v < 0.0 {
                            self.velocity -= world_normal * n_dot_v;
                        }

                        if world_normal.y > 0.7 {
                            on_surface = true;
                        }
                    }
                }
                continue;
            }

            let entity_half = entity.scale * 0.5;
            let ent_min = entity.position - entity_half;
            let ent_max = entity.position + entity_half;

            let overlap_x = (player_max_x - ent_min.x).min(ent_max.x - player_min_x);
            let overlap_y = (player_max_y - ent_min.y).min(ent_max.y - player_min_y);
            let overlap_z = (player_max_z - ent_min.z).min(ent_max.z - player_min_z);

            if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {
                if overlap_y <= overlap_x && overlap_y <= overlap_z {
                    if self.position.y > entity.position.y {
                        self.position.y += overlap_y;
                        if self.velocity.y < 0.0 {
                            self.velocity.y = 0.0;
                        }
                        on_surface = true;
                    } else {
                        self.position.y -= overlap_y;
                        if self.velocity.y > 0.0 {
                            self.velocity.y = 0.0;
                        }
                    }
                } else if overlap_x <= overlap_z {
                    if self.position.x > entity.position.x {
                        self.position.x += overlap_x;
                    } else {
                        self.position.x -= overlap_x;
                    }
                    self.velocity.x = 0.0;
                } else {
                    if self.position.z > entity.position.z {
                        self.position.z += overlap_z;
                    } else {
                        self.position.z -= overlap_z;
                    }
                    self.velocity.z = 0.0;
                }
            }
        }

        self.grounded = on_surface;
    }
}
