use crate::math::Vec3;

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
}

pub struct PhysicsWorld {
    pub colliders: Vec<AABB>,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            colliders: Vec::new(),
        }
    }

    pub fn add_collider(&mut self, aabb: AABB) {
        self.colliders.push(aabb);
    }

    pub fn resolve_collision(&self, pos: &mut Vec3, vel: &mut Vec3, aabb: &AABB) {
        for collider in &self.colliders {
            if !aabb.intersects(collider) {
                continue;
            }

            let aabb_center = aabb.center();
            let col_center = collider.center();
            let diff = aabb_center - col_center;
            let aabb_he = aabb.half_extents();
            let col_he = collider.half_extents();

            let overlap_x = (aabb_he.x + col_he.x) - diff.x.abs();
            let overlap_y = (aabb_he.y + col_he.y) - diff.y.abs();
            let overlap_z = (aabb_he.z + col_he.z) - diff.z.abs();

            if overlap_x > 0.0 && overlap_y > 0.0 && overlap_z > 0.0 {
                if overlap_x < overlap_y && overlap_x < overlap_z {
                    if diff.x > 0.0 {
                        pos.x += overlap_x;
                    } else {
                        pos.x -= overlap_x;
                    }
                    vel.x = 0.0;
                } else if overlap_y < overlap_z {
                    if diff.y > 0.0 {
                        pos.y += overlap_y;
                    } else {
                        pos.y -= overlap_y;
                    }
                    vel.y = 0.0;
                } else {
                    if diff.z > 0.0 {
                        pos.z += overlap_z;
                    } else {
                        pos.z -= overlap_z;
                    }
                    vel.z = 0.0;
                }
            }
        }
    }
}
