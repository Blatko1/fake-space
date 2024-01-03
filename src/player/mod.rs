use std::f32::consts::PI;

use winit::event::{DeviceEvent, ElementState, VirtualKeyCode, KeyboardInput};

use crate::{world::world::{RoomID, World}, render::camera::Camera};

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
        if let Some(tile) = room
            .segment
            .get_tile(position.x as i32, position.z as i32)
        {
            if let Some(portal) = tile.portal {
                if !self.in_portal {
                    if let Some((room_id, portal_id)) = room.portals[portal.id.0].connection {
                        self.current_room = room_id;
                        let connected_room = world.get_room_data(room_id);
                        let portal_pos =
                            connected_room.get_portal(portal_id).local_position;
                        let offset_x = (portal_pos.0 as i64 - position.x as i64) as f32;
                        let offset_z = (portal_pos.1 as i64 - position.z as i64) as f32;
                        self.camera.increase_origin(offset_x, 0.0, offset_z);
                        self.in_portal = true;
                    }
                }
            } else {
                self.in_portal = false;
            }            
        }
    }

    pub fn cast_and_draw(&self, world: &World, data: &mut [u8]) {
        //self.raycaster.cast_and_draw(world, self.current_room, data)
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