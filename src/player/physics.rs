use core::f32;
use std::f32::consts::{FRAC_2_PI, FRAC_PI_2, PI};

use glam::{Vec2, Vec3};

use crate::{control::GameInput, map::segment::Segment, raycaster::camera::{normalize_rad, CameraTarget, CameraTargetData}};

const MOVEMENT_CONST: f32 = 1.5;
const VERTICAL_MOVEMENT_CONST: f32 = 10.0;
const TILE_COLLISION_OFFSET: f32 = 0.4;
const ACCELERATION_CONST: f32 = 10.0;
const SLOWDOWN_CONST: f32 = 10.0;

pub struct CylinderBody {
    pub(super) feet_position: Vec3,
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    forward_dir: Vec3,
    right_dir: Vec3,

    radius: f32,
    height: f32,
    eye_height: f32,

    is_ghost: bool,
    can_fly: bool,
    is_grounded: bool,

    movement_velocity: Vec2,
    air_velocity: f32,
    movement_accel: f32,
    max_movement_vel: f32,
    gravity_accel: f32,
    max_in_air_velocity: f32,
    jump_strength: f32,
    slowdown_friction: f32,
    friction: f32,
    input_state: InputState,
}

impl CylinderBody {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        feet_position: Vec3,
        yaw: f32,
        pitch: f32,

        radius: f32,
        height: f32,
        eye_height_factor: f32,

        jump_strength: f32,
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
        let forward_dir = Vec3::new(yaw.cos(), 0.0, yaw.sin());
        let right_dir = Vec3::new(forward_dir.z, forward_dir.y, -forward_dir.x);
        Self {
            feet_position,
            yaw,
            pitch,
            forward_dir,
            right_dir,
            
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
            jump_strength,
            slowdown_friction,
            friction,
            is_grounded: false,
            input_state: InputState::default(),
        }
    }

    pub fn collision_detection_resolution(
        &mut self,
        segment: &Segment,
    ) {
        if self.is_ghost {
            return;
        }

        let Some(current_tile) = segment
            .get_tile_checked(self.feet_position.x as i64, self.feet_position.z as i64) else {
                return
            };
        let mut ground_level = current_tile.ground_level;
        let mut ceiling_level = current_tile.ceiling_level;

        let pos_x = current_tile.position.x as i64;
        let pos_z = current_tile.position.z as i64;
        let intersected_vertical =
            if (self.feet_position.x + self.radius) > (pos_x as f32 + 1.0) {
                if let Some(tile) = segment.get_tile_checked(pos_x + 1, pos_z) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > self.feet_position.y
                        || (tile.ceiling_level + TILE_COLLISION_OFFSET)
                            < (self.feet_position.y + self.height)
                    {
                        self.feet_position.x = (pos_x as f32 + 1.0) - self.radius;
                        //self.movement_velocity.x = 0.0;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedVerticalSide::Right)
            } else if (self.feet_position.x - self.radius) < pos_x as f32 {
                if let Some(tile) = segment.get_tile_checked(pos_x - 1, pos_z) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > self.feet_position.y
                        || (tile.ceiling_level + TILE_COLLISION_OFFSET)
                            < (self.feet_position.y + self.height)
                    {
                        self.feet_position.x = pos_x as f32 + self.radius;
                        //self.movement_velocity.x = 0.0;
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
            if (self.feet_position.z + self.radius) > (pos_z as f32 + 1.0) {
                if let Some(tile) = segment.get_tile_checked(pos_x, pos_z + 1) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > self.feet_position.y
                        || (tile.ceiling_level + TILE_COLLISION_OFFSET)
                            < (self.feet_position.y + self.height)
                    {
                        self.feet_position.z = (pos_z as f32 + 1.0) - self.radius;
                        //self.movement_velocity.y = 0.0;
                    } else {
                        ground_level = ground_level.max(tile.ground_level);
                        ceiling_level = ceiling_level.min(tile.ceiling_level);
                    }
                }
                Some(IntersectedHorizontalSide::Top)
            } else if (self.feet_position.z - self.radius) < pos_z as f32 {
                if let Some(tile) = segment.get_tile_checked(pos_x, pos_z - 1) {
                    if (tile.ground_level - TILE_COLLISION_OFFSET) > self.feet_position.y
                        || (tile.ceiling_level + TILE_COLLISION_OFFSET)
                            < (self.feet_position.y + self.height)
                    {
                        self.feet_position.z = pos_z as f32 + self.radius;
                        //self.movement_velocity.y = 0.0;
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
            if let Some(tile) =
                segment.get_tile_checked(pos_x + offset_x, pos_z + offset_z)
            {
                if (tile.ground_level - TILE_COLLISION_OFFSET) > self.feet_position.y
                    || (tile.ceiling_level + TILE_COLLISION_OFFSET)
                        < (self.feet_position.y + self.height)
                {
                    let edge_x = (pos_x + offset.0) as f32;
                    let edge_z = (pos_z + offset.1) as f32;
                    let dist_x = edge_x - self.feet_position.x;
                    let dist_z = edge_z - self.feet_position.z;
                    if dist_x.abs() > dist_z.abs() {
                        self.feet_position.x = edge_x - offset_x as f32 * self.radius;
                        //self.movement_velocity.x = 0.0;
                    } else {
                        self.feet_position.z = edge_z - offset_z as f32 * self.radius;
                        //self.movement_velocity.y = 0.0;
                    }
                } else {
                    ground_level = ground_level.max(tile.ground_level);
                    ceiling_level = ceiling_level.min(tile.ceiling_level);
                }
            }
        }

        if self.feet_position.y < ground_level {
            self.feet_position.y = ground_level;
            self.air_velocity = 0.0;
        } else if (self.feet_position.y + self.height) > ceiling_level {
            self.feet_position.y = ceiling_level - self.height;
            self.air_velocity = 0.0;
        }
        self.is_grounded = self.feet_position.y <= ground_level;
        if self.is_grounded {
            self.air_velocity = 0.0;
        }
    }

    pub fn update_physics(
        &mut self,
        delta: f32,
    ) {
        let movement = self.input_state.movement();
        let (horizontal_movement, vertical_movement) = (movement.x, movement.y);
        let movement_dir =
            self.forward_dir * vertical_movement + self.right_dir * horizontal_movement;

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
            self.air_velocity = self.jump_strength * self.input_state.fly_direction();
        } else if self.input_state.jump && self.is_grounded {
            self.air_velocity = self.jump_strength;
        }

        self.feet_position.x += self.movement_velocity.x * delta * MOVEMENT_CONST;
        self.feet_position.z += self.movement_velocity.y * delta * MOVEMENT_CONST;
        self.feet_position.y += self.air_velocity * delta * VERTICAL_MOVEMENT_CONST;

        // Maybe will be used when floor gets a friction attribute
        self.movement_velocity /= 1.0 + self.friction * delta;

        // Apply gravity
        if !self.can_fly {
            self.air_velocity = (self.air_velocity + self.gravity_accel * delta)
                .clamp(-self.max_in_air_velocity, self.max_in_air_velocity);
        }
    }

    /// Angle increases in a counter clockwise direction.
    pub fn add_yaw(&mut self, add: f32) {
        self.set_yaw(self.yaw + add);
    }

    pub fn add_pitch(&mut self, add: f32) {
        self.set_pitch(self.pitch + add);
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = normalize_rad(yaw);
        self.forward_dir = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
        self.right_dir = Vec3::new(self.forward_dir.z, 0.0, -self.forward_dir.x);
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        let (yaw_delta, pitch_delta) = (delta.0 as f32, delta.1 as f32);
        self.add_yaw(-yaw_delta * f32::consts::PI / 180.0 * 0.08);
        self.add_pitch(pitch_delta * PI / 180.0 * 0.08);
    }

    pub fn handle_game_input(&mut self, input: GameInput, is_pressed: bool) {
        match input {
            GameInput::MoveForward => self.input_state.forward = is_pressed,
            GameInput::MoveBackward => self.input_state.backward = is_pressed,
            GameInput::StrafeLeft => self.input_state.left = is_pressed,
            GameInput::StrafeRight => self.input_state.right = is_pressed,
            GameInput::PhysicsSwitch if !is_pressed => {
                self.is_ghost = !self.is_ghost;
                self.can_fly = !self.can_fly;
            }
            GameInput::Jump => self.input_state.jump = is_pressed,
            GameInput::FlyUp => self.input_state.fly_up = is_pressed,
            GameInput::FlyDown => self.input_state.fly_down = is_pressed,
            _ => ()
        }
    }

    pub fn collect_dbg_data(&self) -> PhysicsStateDebugData {
        PhysicsStateDebugData {
            radius: self.radius,
            height: self.height,
            is_ghost: self.is_ghost,
            can_fly: self.can_fly,
            movement_velocity: self.movement_velocity,
            air_velocity: self.air_velocity,
            is_grounded: self.is_grounded,
        }
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

#[derive(Debug, Default)]
pub struct InputState {
    pub jump: bool,
    pub fly_up: bool,
    pub fly_down: bool,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
}

impl InputState {
    pub fn movement(&self) -> Vec2 {
        let x = if self.left { -1.0 } else { 0.0 } + if self.right { 1.0 } else { 0.0 };
        let z =
            if self.forward { 1.0 } else { 0.0 } + if self.backward { -1.0 } else { 0.0 };
        Vec2::new(x, z).try_normalize().unwrap_or_default()
    }

    pub fn fly_direction(&self) -> f32 {
        (if self.fly_up { 1.0 } else { 0.0 } - if self.fly_down { 1.0 } else { 0.0 })
    }
}

impl CameraTarget for CylinderBody {
    fn get_target_data(&self) -> CameraTargetData {
        CameraTargetData {
            origin: Vec3::new(self.feet_position.x, self.feet_position.y + self.eye_height, self.feet_position.z),
            forward_dir: self.forward_dir,
            right_dir: self.right_dir,
            yaw: self.yaw,
            pitch: self.pitch,
        }
    }
}

#[derive(Debug)]
pub struct PhysicsStateDebugData {
    pub radius: f32,
    pub height: f32,
    pub is_ghost: bool,
    pub can_fly: bool,
    pub movement_velocity: Vec2,
    pub air_velocity: f32,
    pub is_grounded: bool,
}
