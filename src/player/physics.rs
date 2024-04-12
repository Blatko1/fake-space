use glam::{Vec2, Vec3};

use crate::world::Segment;

use super::{camera::Camera, PlayerInputState};

const MOVEMENT_SPEED: f32 = 5.0;
const VERTICAL_MOVEMENT_SPEED: f32 = 10.0;
const FREE_FLY_SPEED: f32 = 0.8;
const TILE_COLLISION_OFFSET: f32 = 0.4;
const ACCELERATION_CONST: f32 = 10.0;
const SLOWDOWN_CONST: f32 = 10.0;

pub struct CylinderBody {
    radius: f32,
    height: f32,
    eye_height: f32,

    physics_switch: bool,
    movement_velocity: Vec2,
    air_velocity: f32,
    movement_accel: f32,
    max_movement_vel: f32,
    gravity_accel: f32,
    max_in_air_velocity: f32,
    jump_velocity: f32,
    slowdown_friction: f32,
    friction: f32,
    is_grounded: bool,
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
            eye_height_factor <= 1.0 && eye_height_factor >= 0.0,
            "Eye height not in range [0, 1]!"
        );
        Self {
            radius,
            height,
            eye_height: eye_height_factor * height,

            physics_switch: true,
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

    pub fn move_with_camera(
        &mut self,
        camera: &mut Camera,
        segment: &Segment,
        input_state: &PlayerInputState,
        delta: f32,
    ) {
        let movement = input_state.move_direction();
        let (horizontal_movement, vertical_movement) = (movement.x, movement.y);
        let movement_dir = camera.forward_dir * vertical_movement
            + camera.right_dir * horizontal_movement;
        let acceleration =
            Vec2::new(movement_dir.x, movement_dir.z) * self.movement_accel * ACCELERATION_CONST;
            
        if self.physics_switch {
            if acceleration.x != 0.0 {
                self.movement_velocity.x += acceleration.x * delta;
            } else {
                self.movement_velocity.x /= 1.0 + self.slowdown_friction * delta * SLOWDOWN_CONST;
            }
            if acceleration.y != 0.0 {
                self.movement_velocity.y += acceleration.y * delta;
            } else {
                self.movement_velocity.y /= 1.0 + self.slowdown_friction * delta * SLOWDOWN_CONST;
            }
            self.movement_velocity = self.movement_velocity.clamp_length_max(self.max_movement_vel);
    
            if input_state.jump && self.is_grounded {
                self.air_velocity = self.jump_velocity;
            }
        } else {
            self.movement_velocity = Vec2::new(MOVEMENT_SPEED, MOVEMENT_SPEED);
            self.air_velocity = input_state.fly_direction() * FREE_FLY_SPEED;
        }

        let eye_position = camera.origin;
        let mut feet_position = Vec3::new(
            eye_position.x,
            eye_position.y - self.eye_height,
            eye_position.z,
        );
        feet_position.x += self.movement_velocity.x * delta;
        feet_position.z += self.movement_velocity.y * delta;
        feet_position.y += self.air_velocity * delta * VERTICAL_MOVEMENT_SPEED;

        // If physics are turned off, skip the physics
        if !self.physics_switch {
            camera.origin = Vec3::new(
                feet_position.x,
                feet_position.y + self.eye_height,
                feet_position.z,
            );
            return;
        }

        // Apply friction and gravity
        self.movement_velocity /= 1.0 + self.friction * delta;
        self.air_velocity = (self.air_velocity + self.gravity_accel * delta)
            .clamp(-self.max_in_air_velocity, self.max_in_air_velocity);

        let current_tile =
            match segment.get_tile(feet_position.x as i64, feet_position.z as i64) {
                Some(t) => t,
                None => return,
            };
        let mut ground_level = current_tile.ground_level;
        let mut ceiling_level = current_tile.ceiling_level;

        let pos_x = feet_position.x as i64;
        let pos_z = feet_position.z as i64;
        let intersected_vertical =
            if (feet_position.x + self.radius) > (current_tile.position.x as f32 + 1.0) {
                if let Some(tile) = segment.get_tile(pos_x + 1, pos_z) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.x =
                            (current_tile.position.x as f32 + 1.0) - self.radius;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedVerticalSide::Right)
            } else if (feet_position.x - self.radius) < current_tile.position.x as f32 {
                if let Some(tile) = segment.get_tile(pos_x - 1, pos_z) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.x = current_tile.position.x as f32 + self.radius;
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
            if (feet_position.z + self.radius) > (current_tile.position.z as f32 + 1.0) {
                if let Some(tile) = segment.get_tile(pos_x, pos_z + 1) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.z =
                            (current_tile.position.z as f32 + 1.0) - self.radius;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedHorizontalSide::Top)
            } else if (feet_position.z - self.radius) < current_tile.position.z as f32 {
                if let Some(tile) = segment.get_tile(pos_x, pos_z - 1) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > feet_position.y
                        || tile.ceiling_level < (feet_position.y + self.height)
                    {
                        feet_position.z = current_tile.position.z as f32 + self.radius;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedHorizontalSide::Bottom)
            } else {
                None
            };
        match (intersected_vertical, intersected_horizontal) {
            (Some(v), Some(h)) => match (v, h) {
                (IntersectedVerticalSide::Left, IntersectedHorizontalSide::Top) => {}
                (IntersectedVerticalSide::Left, IntersectedHorizontalSide::Bottom) => {}
                (IntersectedVerticalSide::Right, IntersectedHorizontalSide::Top) => {}
                (IntersectedVerticalSide::Right, IntersectedHorizontalSide::Bottom) => {}
            },
            _ => (),
        }

        if feet_position.y <= ground_level {
            feet_position.y = ground_level;
            self.air_velocity = 0.0;
        } else if (feet_position.y + self.height) >= ceiling_level {
            feet_position.y = ceiling_level - self.height;
            self.air_velocity = 0.0;
        }
        self.is_grounded = feet_position.y <= ground_level;

        camera.origin = Vec3::new(
            feet_position.x,
            feet_position.y + self.eye_height,
            feet_position.z,
        );
    }

    pub fn toggle_physics(&mut self) {
        self.physics_switch = !self.physics_switch
    }
}

enum IntersectedVerticalSide {
    Left,
    Right,
}

enum IntersectedHorizontalSide {
    Top,
    Bottom,
}
