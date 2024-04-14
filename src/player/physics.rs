use glam::{Vec2, Vec3};

use crate::world::Segment;

use super::PlayerInputState;

const MOVEMENT_SPEED: f32 = 5.0;
const VERTICAL_MOVEMENT_SPEED: f32 = 10.0;
const TILE_COLLISION_OFFSET: f32 = 0.4;
const ACCELERATION_CONST: f32 = 10.0;
const SLOWDOWN_CONST: f32 = 10.0;

// TODO pub(super) are TEMP!!!!
pub struct CylinderBody {
    pub(super) radius: f32,
    height: f32,
    eye_height: f32,

    is_ghost: bool,
    pub(super) can_fly: bool,
    pub(super) movement_velocity: Vec2,
    air_velocity: f32,
    movement_accel: f32,
    max_movement_vel: f32,
    gravity_accel: f32,
    max_in_air_velocity: f32,
    jump_velocity: f32,
    slowdown_friction: f32,
    friction: f32,
    pub(super) is_grounded: bool,
}

impl CylinderBody {
    pub fn new(
        radius: f32,
        height: f32,
        eye_height_factor: f32,
        jump_velocity: f32,
        movement_accel: f32,
        max_movement_vel: f32,
        gravity_accel: f32,
        slowdown_friction: f32,
        friction: f32,
    ) -> Self {
        assert!(
            (0.0..=1.0).contains(&eye_height_factor),
            "Eye height not in range [0, 1]!"
        );
        Self {
            radius,
            height,
            eye_height: eye_height_factor * height,

            is_ghost: false,
            can_fly: false,
            movement_velocity: Vec2::ZERO,
            movement_accel,
            max_movement_vel,
            air_velocity: 0.0,
            gravity_accel,
            max_in_air_velocity: 100.0,
            jump_velocity,
            slowdown_friction,
            friction,
            is_grounded: false,
        }
    }

    pub fn collision_detection_resolution(
        &mut self,
        origin: Vec3,
        segment: &Segment,
    ) -> Vec3 {
        if self.is_ghost {
            return origin;
        }

        let mut feet_position = Vec3::new(origin.x, origin.y - self.eye_height, origin.z);
        let current_tile =
            match segment.get_tile(feet_position.x as i64, feet_position.z as i64) {
                Some(t) => t,
                None => return origin,
            };
        let mut ground_level = current_tile.ground_level;
        let mut ceiling_level = current_tile.ceiling_level;

        let pos_x = current_tile.position.x as i64;
        let pos_z = current_tile.position.z as i64;
        let intersected_vertical =
            if (feet_position.x + self.radius) > (pos_x as f32 + 1.0) {
                if let Some(tile) = segment.get_tile(pos_x + 1, pos_z) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.x = (pos_x as f32 + 1.0) - self.radius;
                        self.movement_velocity.x = 0.0;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedVerticalSide::Right)
            } else if (feet_position.x - self.radius) < pos_x as f32 {
                if let Some(tile) = segment.get_tile(pos_x - 1, pos_z) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.x = pos_x as f32 + self.radius;
                        self.movement_velocity.x = 0.0;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedVerticalSide::Left)
            } else {
                None
            };
        let intersected_horizontal =
            if (feet_position.z + self.radius) > (pos_z as f32 + 1.0) {
                if let Some(tile) = segment.get_tile(pos_x, pos_z + 1) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.z = (pos_z as f32 + 1.0) - self.radius;
                        self.movement_velocity.y = 0.0;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedHorizontalSide::Top)
            } else if (feet_position.z - self.radius) < pos_z as f32 {
                if let Some(tile) = segment.get_tile(pos_x, pos_z - 1) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.z = pos_z as f32 + self.radius;
                        self.movement_velocity.y = 0.0;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedHorizontalSide::Bottom)
            } else {
                None
            };
        if let (Some(v), Some(h)) = (intersected_vertical, intersected_horizontal) {
            let offset = (v as i64, h as i64);
            let (offset_x, offset_z) = (offset.0 * 2 - 1, offset.1 * 2 - 1);
            if let Some(adjacent_tile) =
                segment.get_tile(pos_x + offset_x, pos_z + offset_z)
            {
                if (adjacent_tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                    || adjacent_tile.ceiling_level < (feet_position.y + self.height)
                {
                    let edge_x = (pos_x + offset.0) as f32;
                    let edge_z = (pos_z + offset.1) as f32;
                    let dist_x = edge_x - feet_position.x;
                    let dist_z = edge_z - feet_position.z;
                    if dist_x.abs() > dist_z.abs() {
                        feet_position.x = edge_x - offset_x as f32 * self.radius;
                        self.movement_velocity.x = 0.0;
                    } else {
                        feet_position.z = edge_z - offset_z as f32 * self.radius;
                        self.movement_velocity.y = 0.0;
                    }
                }
            }
        }

        if feet_position.y < ground_level {
            feet_position.y = ground_level;

            self.air_velocity = 0.0;
        } else if (feet_position.y + self.height) > ceiling_level {
            feet_position.y = ceiling_level - self.height;
            self.air_velocity = 0.0;
        }
        self.is_grounded = feet_position.y <= ground_level;

        Vec3::new(
            feet_position.x,
            feet_position.y + self.eye_height,
            feet_position.z,
        )
    }

    pub fn update_physics_state(
        &mut self,
        forward_dir: Vec3,
        right_dir: Vec3,
        input_state: &PlayerInputState,
        delta: f32,
    ) {
        let movement = input_state.movement();
        let (horizontal_movement, vertical_movement) = (movement.x, movement.y);
        let movement_dir =
            forward_dir * vertical_movement + right_dir * horizontal_movement;

        let acceleration = Vec2::new(movement_dir.x, movement_dir.z)
            * self.movement_accel
            * ACCELERATION_CONST;
        if acceleration.x != 0.0 {
            self.movement_velocity.x += acceleration.x * delta;
        } else {
            self.movement_velocity.x /=
                1.0 + self.slowdown_friction * delta * SLOWDOWN_CONST;
        }
        if acceleration.y != 0.0 {
            self.movement_velocity.y += acceleration.y * delta;
        } else {
            self.movement_velocity.y /=
                1.0 + self.slowdown_friction * delta * SLOWDOWN_CONST;
        }
        self.movement_velocity = self
            .movement_velocity
            .clamp_length_max(self.max_movement_vel);

        if self.can_fly {
            self.air_velocity = self.jump_velocity * input_state.fly_direction();
        } else if input_state.jump && self.is_grounded {
            self.air_velocity = self.jump_velocity;
        }
    }

    pub fn apply_physics(&mut self, mut origin: Vec3, delta: f32) -> Vec3 {
        origin.x += self.movement_velocity.x * delta;
        origin.z += self.movement_velocity.y * delta;
        origin.y += self.air_velocity * delta * VERTICAL_MOVEMENT_SPEED;

        // Apply friction and gravity
        self.movement_velocity /= 1.0 + self.friction * delta;
        if !self.can_fly {
            self.air_velocity = (self.air_velocity + self.gravity_accel * delta)
                .clamp(-self.max_in_air_velocity, self.max_in_air_velocity);
        }

        origin
    }

    pub fn rotate_velocity(&mut self, yaw_angle_rotate: f32) {
        let cos = yaw_angle_rotate.cos();
        let sin = yaw_angle_rotate.sin();
        let x = self.movement_velocity.x * cos - self.movement_velocity.y * sin;
        let z = self.movement_velocity.x * sin + self.movement_velocity.y * cos;
        self.movement_velocity.x = x;
        self.movement_velocity.y = z;
    }

    #[inline]
    pub fn toggle_ghost(&mut self) {
        self.is_ghost = !self.is_ghost
    }

    #[inline]
    pub fn toggle_fly(&mut self) {
        self.can_fly = !self.can_fly
    }
}

enum IntersectedVerticalSide {
    Left,
    Right,
}

enum IntersectedHorizontalSide {
    Bottom,
    Top,
}
