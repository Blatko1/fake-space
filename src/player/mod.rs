pub mod camera;
pub mod render;

use winit::{
    event::{DeviceEvent, KeyEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::world::{self, RoomID, Segment, World};

use self::camera::Camera;

pub struct Player {
    camera: Camera,

    physics: bool,
    current_room: RoomID,
    in_portal: bool,
}

impl Player {
    pub fn new(camera: Camera, current_room: RoomID) -> Self {
        Self {
            camera,

            physics: false,
            current_room,
            in_portal: false,
        }
    }

    pub fn update(&mut self, world: &World, frame_time: f32) {
        self.camera.update(frame_time);
        // Teleportation between rooms
        let room = world.get_room_data(self.current_room);
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

        if self.physics {
            self.collision_detection(room.segment)
        }
    }

    fn collision_detection(&mut self, segment: &Segment) {
        let position = self.camera.origin;
        let current_tile = match segment.get_tile(position.x as i64, position.z as i64) {
            Some(t) => t,
            None => return,
        };
        if position.y <= current_tile.ground_level
            || position.y >= current_tile.ceiling_level
        {
            return;
        }
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
        self.camera.process_keyboard_input(event)
    }
}
