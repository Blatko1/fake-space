pub mod camera;
pub mod render;

use glam::Vec3;
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::world::{RoomID, Segment, World};

use self::camera::Camera;

const VELOCITY_FACTOR: f32 = 5.0;
const JUMP_VELOCITY: f32 = 3.0;
const MAX_WALK_HEIGHT_OFFSET: f32 = 0.5;

pub struct Player {
    camera: Camera,

    physics: bool,
    gravity: f32,
    bb: CylinderBoundingBox,
    vel: Vec3,
    is_in_air: bool,

    current_room: RoomID,
    in_portal: bool,

    /// Physics jump
    jump: bool,
}

impl Player {
    pub fn new(camera: Camera, current_room: RoomID) -> Self {
        // TODO currently hard-coded
        // TODO radius < 0.5
        let bb = CylinderBoundingBox {
            radius: 0.4,
            height: 1.8,
            eye_height: 0.9 * 1.8,
            //position: Vec3::new(camera.origin.x, camera.origin.y, camera.origin.z),
        };
        Self {
            camera,

            physics: false,
            gravity: 2.0,
            bb,
            vel: Vec3::ZERO,
            is_in_air: false,

            current_room,
            in_portal: false,

            jump: false,
        }
    }

    pub fn update(&mut self, world: &World, frame_time: f32) {
        self.camera.update(frame_time);

        let room = world.get_room_data(self.current_room);
        if self.physics {
            if self.jump && !self.is_in_air {
                self.vel.y = JUMP_VELOCITY;
            }
            self.process_physics(room.segment, frame_time)
        }

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

    fn process_physics(&mut self, segment: &Segment, frame_time: f32) {
        let mut bb_pos = self.camera.origin;
        bb_pos.y = self.camera.origin.y - self.bb.eye_height;
        let current_tile = match segment.get_tile(bb_pos.x as i64, bb_pos.z as i64) {
            Some(t) => t,
            None => return,
        };
        let mut current_ground_level = current_tile.ground_level;

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
                        if !self.is_in_air
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
                        if !self.is_in_air
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
                        if !self.is_in_air
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
                        if !self.is_in_air
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

        // Reposition according to velocity and apply gravity
        bb_pos += self.vel * frame_time * VELOCITY_FACTOR;
        self.vel.y -= self.gravity * frame_time * VELOCITY_FACTOR;

        // Check if the bb is too low or too high
        if bb_pos.y < current_ground_level {
            bb_pos.y = current_ground_level;
            self.vel.y = 0.0;
        } else if (bb_pos.y + self.bb.height) > current_tile.ceiling_level {
            bb_pos.y = current_tile.ceiling_level - self.bb.height;
            self.vel.y = 0.0;
        }

        // Check if bb is in air
        self.is_in_air = if bb_pos.y <= current_ground_level {
            false
        } else {
            true
        };

        // Apply the new position
        self.camera.origin = bb_pos;
        self.camera.origin.y += self.bb.eye_height;
    }

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
    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        self.camera.process_mouse_input(event)
    }

    #[inline]
    pub fn process_keyboard_input(&mut self, event: KeyEvent) {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::KeyP) if !event.state.is_pressed() => {
                self.physics = !self.physics
            }
            _ => (),
        }

        let value = match event.state {
            ElementState::Pressed => 1.0,
            ElementState::Released => 0.0,
        };
        if !self.physics {
            if let PhysicalKey::Code(key) = event.physical_key {
                match key {
                    KeyCode::KeyQ => self.camera.turn_left = value,
                    KeyCode::KeyE => self.camera.turn_right = value,
                    KeyCode::KeyW => self.camera.forward = value,
                    KeyCode::KeyS => self.camera.backward = value,
                    KeyCode::Space => self.camera.fly_up = value,
                    KeyCode::ShiftLeft => self.camera.fly_down = value,
                    KeyCode::KeyA => self.camera.strafe_left = value,
                    KeyCode::KeyD => self.camera.strafe_right = value,
                    KeyCode::ArrowUp => self.camera.increase_fov = value,
                    KeyCode::ArrowDown => self.camera.decrease_fov = value,
                    // Look more up (y_shearing):
                    KeyCode::PageUp => self.camera.increase_y_shearing = value,
                    // Look more down (y_shearing):
                    KeyCode::PageDown => self.camera.decrease_y_shearing = value,
                    // Reset look (y_shearing):
                    KeyCode::Home => self.camera.y_shearing = 0.0,
                    _ => (),
                }
            }
        } else {
            let is_pressed = event.state.is_pressed();
            if let PhysicalKey::Code(key) = event.physical_key {
                match key {
                    KeyCode::KeyW => self.camera.forward = value,
                    KeyCode::KeyS => self.camera.backward = value,
                    KeyCode::KeyA => self.camera.strafe_left = value,
                    KeyCode::KeyD => self.camera.strafe_right = value,
                    KeyCode::Space => self.jump = is_pressed,
                    KeyCode::ArrowUp => self.camera.increase_fov = value,
                    KeyCode::ArrowDown => self.camera.decrease_fov = value,
                    // Look more up (y_shearing):
                    KeyCode::PageUp => self.camera.increase_y_shearing = value,
                    // Look more down (y_shearing):
                    KeyCode::PageDown => self.camera.decrease_y_shearing = value,
                    // Reset look (y_shearing):
                    KeyCode::Home => self.camera.y_shearing = 0.0,
                    _ => (),
                }
            }
        }
    }
}

enum TileSide {
    Left,
    Right,
    Front,
    Back,
}

/// Player bounding box which is a cylinder
pub struct CylinderBoundingBox {
    radius: f32,
    height: f32,
    eye_height: f32,
}
