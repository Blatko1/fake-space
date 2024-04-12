pub mod camera;
mod physics;
pub mod render;

use glam::{Vec2, Vec3};
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    control::GameInput,
    world::{RoomID, Segment, World},
};

use self::{camera::Camera, physics::CylinderBody};

const FLY_STRENGTH: f32 = 5.5;
const MOVEMENT_SPEED: f32 = 1.0;
const JUMP_VELOCITY: f32 = 1.0;
const MAX_WALK_HEIGHT_OFFSET: f32 = 0.5;

pub struct Player {
    camera: Camera,

    body: CylinderBody,

    current_room: RoomID,
    in_portal: bool,

    input_state: PlayerInputState,
}

impl Player {
    pub fn new(camera: Camera, current_room: RoomID) -> Self {
        let body = CylinderBody::new(0.2, 1.8, 0.9, 
            1.0, 3.5, 3.0, 
            -2.0, 2.0, 0.0);

        Self {
            camera,

            body,

            current_room,
            in_portal: false,

            input_state: PlayerInputState::default(),
        }
    }

    pub fn update(&mut self, world: &World, delta: f32) {
        let room = world.get_room_data(self.current_room);

        self.camera.update(&self.input_state, delta);
        self.body.move_with_camera(
            &mut self.camera,
            room.segment,
            &self.input_state,
            delta,
        );

        // Teleportation between rooms
        let position = self.camera.origin;
        // Check if player is on a tile
        if let Some(tile) = room.segment.get_tile(position.x as i64, position.z as i64) {
            // Check if tile has a portal
            if let Some(src_dummy_portal) = tile.portal {
                // Check if player was just teleported
                if !self.in_portal {
                    // Check if portal has a linked portal
                    let src_portal = room.get_portal(src_dummy_portal.id);
                    if let Some((room_id, portal_id)) = src_portal.link {
                        // Teleport the player
                        self.current_room = room_id;
                        let dest_portal =
                            world.get_room_data(room_id).get_portal(portal_id);
                        self.camera.portal_teleport(src_portal, dest_portal);
                        self.in_portal = true;
                    }
                }
            } else {
                self.in_portal = false;
            }
        }
    }

    /*fn process_physics(&mut self, world: &World, frame_time: f32) {
        let mut bb_pos = self.camera.origin;
        bb_pos.y = self.camera.origin.y - self.bb.eye_height;

        bb_pos.x += self.velocity.x;
        bb_pos.z += self.velocity.z;
        // Reposition according to velocity and apply gravity
        bb_pos.y += self.velocity.y * frame_time * FLY_STRENGTH;
        self.velocity.x /= 1.0 + self.friction * frame_time;
        self.velocity.z /= 1.0 + self.friction * frame_time;
        self.velocity.y += self.gravity * frame_time;

        let room = world.get_room_data(self.current_room);
        let segment = room.segment;

        let current_tile = match segment.get_tile(bb_pos.x as i64, bb_pos.z as i64) {
            Some(t) => t,
            None => return,
        };
        let mut current_ground_level = current_tile.ground_level;

        // Check if the bb is too low or too high
        if bb_pos.y <= current_ground_level {
            bb_pos.y = current_ground_level;
            self.velocity.y = 0.0;
        } else if (bb_pos.y + self.bb.height) >= current_tile.ceiling_level {
            bb_pos.y = current_tile.ceiling_level - self.bb.height;
            self.velocity.y = 0.0;
        }
        // Check if bounding box is in air
        self.on_ground = if bb_pos.y <= current_ground_level {
            false
        } else {
            true
        };

        // Check for intersections with adjacent tiles
        let vertical_intersection =
            if (bb_pos.x + self.bb.radius) > (current_tile.position.x as f32 + 1.0) {
                // Get the adjacent tile
                if let Some(adjacent_tile) =
                    segment.get_tile(bb_pos.x as i64 + 1, bb_pos.z as i64)
                {
                    // Check if adjacent tile is low enough or if adjacent ceiling level is high enough
                    if adjacent_tile.ground_level <= bb_pos.y
                        && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                    {
                        current_ground_level =
                            adjacent_tile.ground_level.max(current_ground_level);
                        bb_pos.y = bb_pos.y.max(current_ground_level);
                    } else {
                        // Maybe player can walk up the adjacent tile if player is not in air
                        if !self.on_ground
                            && adjacent_tile.ground_level - current_ground_level
                                <= MAX_WALK_HEIGHT_OFFSET
                        {
                            current_ground_level =
                                current_ground_level.max(adjacent_tile.ground_level);
                        } else {
                            bb_pos.x =
                                (current_tile.position.x as f32 + 1.0) - self.bb.radius;
                        }
                    }
                    Some(TileSide::Right)
                } else {
                    None
                }
            } else if (bb_pos.x - self.bb.radius) < current_tile.position.x as f32 {
                if let Some(adjacent_tile) =
                    segment.get_tile(bb_pos.x as i64 - 1, bb_pos.z as i64)
                {
                    // Check if adjacent tile is low enough or if adjacent ceiling level is high enough
                    if adjacent_tile.ground_level <= bb_pos.y
                        && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                    {
                        current_ground_level =
                            adjacent_tile.ground_level.max(current_ground_level);
                        bb_pos.y = bb_pos.y.max(current_ground_level);
                    } else {
                        // Maybe player can walk up the adjacent tile if player is not in air
                        if !self.on_ground
                            && adjacent_tile.ground_level - current_ground_level
                                <= MAX_WALK_HEIGHT_OFFSET
                        {
                            current_ground_level =
                                current_ground_level.max(adjacent_tile.ground_level);
                        } else {
                            bb_pos.x = current_tile.position.x as f32 + self.bb.radius;
                        }
                    }
                    Some(TileSide::Left)
                } else {
                    None
                }
            } else {
                None
            };
        let horizontal_intersection =
            if (bb_pos.z + self.bb.radius) > (current_tile.position.z as f32 + 1.0) {
                if let Some(adjacent_tile) =
                    segment.get_tile(bb_pos.x as i64, bb_pos.z as i64 + 1)
                {
                    // Check if adjacent tile is low enough or if adjacent ceiling level is high enough
                    if adjacent_tile.ground_level <= bb_pos.y
                        && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                    {
                        current_ground_level =
                            adjacent_tile.ground_level.max(current_ground_level);
                        bb_pos.y = bb_pos.y.max(current_ground_level);
                    } else {
                        // Maybe player can walk up the adjacent tile if player is not in air
                        if !self.on_ground
                            && adjacent_tile.ground_level - current_ground_level
                                <= MAX_WALK_HEIGHT_OFFSET
                        {
                            current_ground_level =
                                current_ground_level.max(adjacent_tile.ground_level);
                        } else {
                            bb_pos.z =
                                (current_tile.position.z as f32 + 1.0) - self.bb.radius;
                        }
                    }
                    Some(TileSide::Front)
                } else {
                    None
                }
            } else if (bb_pos.z - self.bb.radius) < current_tile.position.z as f32 {
                if let Some(adjacent_tile) =
                    segment.get_tile(bb_pos.x as i64, bb_pos.z as i64 - 1)
                {
                    // Check if adjacent tile is low enough or if adjacent ceiling level is high enough
                    if adjacent_tile.ground_level <= bb_pos.y
                        && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                    {
                        current_ground_level =
                            adjacent_tile.ground_level.max(current_ground_level);
                        bb_pos.y = bb_pos.y.max(current_ground_level);
                    } else {
                        // Maybe player can walk up the adjacent tile if player is not in air
                        if !self.on_ground
                            && adjacent_tile.ground_level - current_ground_level
                                <= MAX_WALK_HEIGHT_OFFSET
                        {
                            current_ground_level =
                                current_ground_level.max(adjacent_tile.ground_level);
                        } else {
                            bb_pos.z = current_tile.position.z as f32 + self.bb.radius;
                        }
                    }
                    Some(TileSide::Back)
                } else {
                    None
                }
            } else {
                None
            };
        match (vertical_intersection, horizontal_intersection) {
            (Some(v), Some(h)) => match (v, h) {
                (TileSide::Left, TileSide::Front) => {
                    if let Some(adjacent_tile) =
                        segment.get_tile(bb_pos.x as i64 - 1, bb_pos.z as i64 + 1)
                    {
                        if adjacent_tile.ground_level <= bb_pos.y
                            && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                        {
                        } else {
                            let dist_x = (adjacent_tile.position.x + 1) as f32 - bb_pos.x;
                            let dist_z = adjacent_tile.position.z as f32 - bb_pos.z;
                            if dist_x.abs() > dist_z.abs() {
                                bb_pos.x =
                                    current_tile.position.x as f32 + self.bb.radius;
                            } else {
                                bb_pos.z =
                                    (current_tile.position.z + 1) as f32 - self.bb.radius;
                            }
                        }
                    }
                }
                (TileSide::Left, TileSide::Back) => {
                    if let Some(adjacent_tile) =
                        segment.get_tile(bb_pos.x as i64 - 1, bb_pos.z as i64 - 1)
                    {
                        if adjacent_tile.ground_level <= bb_pos.y
                            && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                        {
                        } else {
                            let dist_x = (adjacent_tile.position.x + 1) as f32 - bb_pos.x;
                            let dist_z = (adjacent_tile.position.z + 1) as f32 - bb_pos.z;
                            if dist_x.abs() > dist_z.abs() {
                                bb_pos.x =
                                    current_tile.position.x as f32 + self.bb.radius;
                            } else {
                                bb_pos.z =
                                    current_tile.position.z as f32 + self.bb.radius;
                            }
                        }
                    }
                }
                (TileSide::Right, TileSide::Front) => {
                    if let Some(adjacent_tile) =
                        segment.get_tile(bb_pos.x as i64 + 1, bb_pos.z as i64 + 1)
                    {
                        if adjacent_tile.ground_level <= bb_pos.y
                            && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                        {
                        } else {
                            let dist_x = adjacent_tile.position.x as f32 - bb_pos.x;
                            let dist_z = adjacent_tile.position.z as f32 - bb_pos.z;
                            if dist_x.abs() > dist_z.abs() {
                                bb_pos.x =
                                    (current_tile.position.x + 1) as f32 - self.bb.radius;
                            } else {
                                bb_pos.z =
                                    (current_tile.position.z + 1) as f32 - self.bb.radius;
                            }
                        }
                    }
                }
                (TileSide::Right, TileSide::Back) => {
                    if let Some(adjacent_tile) =
                        segment.get_tile(bb_pos.x as i64 + 1, bb_pos.z as i64 - 1)
                    {
                        if adjacent_tile.ground_level <= bb_pos.y
                            && adjacent_tile.ceiling_level > (bb_pos.y + self.bb.height)
                        {
                        } else {
                            let dist_x = adjacent_tile.position.x as f32 - bb_pos.x;
                            let dist_z = (adjacent_tile.position.z + 1) as f32 - bb_pos.z;
                            if dist_x.abs() > dist_z.abs() {
                                bb_pos.x =
                                    (current_tile.position.x + 1) as f32 - self.bb.radius;
                            } else {
                                bb_pos.z =
                                    current_tile.position.z as f32 + self.bb.radius;
                            }
                        }
                    }
                }
                _ => unreachable!(),
            },
            _ => (),
        }

        // Apply the new position
        self.camera.origin = bb_pos;
        self.camera.origin.y += self.bb.eye_height;
    }*/

    pub fn cast_and_draw<'a, C>(&self, world: &World, column_iter: C)
    where
        C: Iterator<Item = &'a mut [u8]>,
    {
        render::cast_and_draw(self, world, column_iter)
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn current_room_id(&self) -> RoomID {
        self.current_room
    }

    #[inline]
    pub fn on_mouse_move(&mut self, delta: Vec2) {
        self.camera.on_mouse_move(delta)
    }

    pub fn process_input(&mut self, input: GameInput, is_pressed: bool) {
        match input {
            GameInput::MoveForward => self.input_state.forward = is_pressed,
            GameInput::MoveBackward => self.input_state.backward = is_pressed,
            GameInput::StrafeLeft => self.input_state.left = is_pressed,
            GameInput::StrafeRight => self.input_state.right = is_pressed,
            GameInput::IncreaseFOV => self.input_state.increase_fov = is_pressed,
            GameInput::DecreaseFOV => self.input_state.decrease_fov = is_pressed,
            GameInput::PhysicsSwitch if !is_pressed => {
                self.body.toggle_physics();
            }
            GameInput::Jump => self.input_state.jump = is_pressed,
            GameInput::FlyUp => self.input_state.fly_up = is_pressed,
            GameInput::FlyDown => self.input_state.fly_down = is_pressed,
            // the rest is not interpretable by player
            _ => (),
        }
    }

    /*pub fn collect_dbg_data(&self) -> PlayerDebugData {
        PlayerDebugData {
            camera_origin: self.camera.origin,
            camera_direction: self.camera.dir,
            camera_angle: self.camera.yaw_angle,
            y_shearing: self.camera.y_shearing,

            is_in_air: self.on_ground,
            physics_switch: self.physics_switch,
            velocity: self.velocity,

            current_room_id: self.current_room.0,
        }
    }*/
}

enum TileSide {
    Left,
    Right,
    Front,
    Back,
}

#[derive(Debug)]
pub struct PlayerDebugData {
    pub camera_origin: Vec3,
    pub camera_direction: Vec3,
    pub camera_angle: f32,
    pub y_shearing: f32,

    pub is_in_air: bool,
    pub physics_switch: bool,
    pub velocity: Vec3,

    pub current_room_id: usize,
}

#[derive(Debug, Default)]
pub struct PlayerInputState {
    pub jump: bool,
    pub fly_up: bool,
    pub fly_down: bool,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub increase_fov: bool,
    pub decrease_fov: bool,
}

impl PlayerInputState {
    pub fn move_direction(&self) -> Vec2 {
        let x = if self.left { -1.0 } else { 0.0 } + if self.right { 1.0 } else { 0.0 };
        let z =
            if self.forward { 1.0 } else { 0.0 } + if self.backward { -1.0 } else { 0.0 };
        Vec2::new(x, z).try_normalize().unwrap_or_default()
    }

    pub fn fly_direction(&self) -> f32 {
        return if self.fly_up { 1.0 } else { 0.0 }
            - if self.fly_down { 1.0 } else { 0.0 };
    }

    pub fn fov_change(&self) -> f32 {
        return if self.increase_fov { 1.0 } else { 0.0 }
            + if self.decrease_fov { -1.0 } else { 0.0 };
    }
}
