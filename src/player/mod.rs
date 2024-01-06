use std::f32::consts::PI;

use winit::event::{DeviceEvent, KeyboardInput};

use crate::{
    render::{self, camera::Camera},
    world::{RoomID, World},
};

const ONE_DEGREE_RAD: f32 = PI / 180.0;

pub struct Player {
    camera: Camera,

    current_room: RoomID,
    in_portal: bool,
}

impl Player {
    pub fn new(camera: Camera, current_room: RoomID) -> Self {
        Self {
            camera,

            current_room,
            in_portal: false,
        }
    }

    pub fn update(&mut self, world: &World) {
        self.camera.update();
        // Teleportation between rooms
        let room = world.get_room_data(self.current_room);
        let position = self.camera.get_origin();
        // Check if player is on a tile
        if let Some(tile) = room.segment.get_tile(position.x as i64, position.z as i64) {
            // Check if tile has a portal
            if let Some(src_dummy_portal) = tile.portal {
                // Check if player was just teleported
                if !self.in_portal {
                    // Check if portal has a linked portal
                    let src_portal = room.get_portal(src_dummy_portal.id);
                    if let Some((room_id, portal_id)) =
                        src_portal.link
                    {
                        // Teleport the player
                        self.current_room = room_id;
                        let dest_portal = world.get_room_data(room_id).get_portal(portal_id);
                        self.camera.portal_teleport(src_portal, dest_portal);
                        self.in_portal = true;
                    }
                }
            } else {
                self.in_portal = false;
            }
        }
    }

    pub fn cast_and_draw<'a, C>(&self, world: &World, column_iter: C)
    where
        C: Iterator<Item = &'a mut [u8]>,
    {
        render::cast_and_draw(self, world, column_iter)
    }

    pub fn get_camera(&self) -> &Camera {
        &self.camera
    }

    pub fn get_current_room_id(&self) -> RoomID {
        self.current_room
    }

    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        self.camera.process_mouse_input(event)
    }

    pub fn process_keyboard_input(&mut self, event: KeyboardInput) {
        self.camera.process_keyboard_input(event)
    }
}
