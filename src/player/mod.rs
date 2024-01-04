use std::f32::consts::PI;

use winit::event::{DeviceEvent, ElementState, VirtualKeyCode, KeyboardInput};

use crate::{world::world::{RoomID, World, PortalRotationDifference}, render::{camera::Camera, self}};

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
            if let Some(src_portal) = tile.portal {
                if !self.in_portal {
                    if let Some((room_id, portal_id)) = room.portals[src_portal.id.0].connection {
                        self.current_room = room_id;
                        let dest_room = world.get_room_data(room_id);
                        let dest_room_portal = dest_room.get_portal(portal_id);
                        let portal_pos =
                        dest_room_portal.local_position;
                        let mut x = portal_pos.0 as f32 + position.x.fract(); //position.x + (portal_pos.0 as i64 - position.x as i64) as f32;
                        let y = position.y + dest_room_portal.ground_level - src_portal.ground_level;
                        let mut z = portal_pos.1 as f32 + position.z.fract();// position.z + (portal_pos.1 as i64 - position.z as i64) as f32;
                        match src_portal.direction.rotation_radian_difference(dest_room_portal.direction) {
                            PortalRotationDifference::None => (),
                            PortalRotationDifference::LeftDeg90 => todo!(),
                            PortalRotationDifference::RightDeg90 => todo!(),
                            PortalRotationDifference::Deg180 => {
                                self.camera.increase_direction_angle(PI);
                                x = portal_pos.0 as f32 + 0.5 + (src_portal.local_position.0 as f32 + 0.5) - position.x;
                                z = portal_pos.1 as f32 + 0.5 + (src_portal.local_position.1 as f32 + 0.5) - position.z;
                            },
                        }
                        self.camera.set_origin(x, y, z);
                        self.in_portal = true;
                    }
                }
            } else {
                self.in_portal = false;
            }            
        }
    }

    pub fn cast_and_draw<'a, C>(&self, world: &World, column_iter: C) where C: Iterator<Item = &'a mut [u8]> {
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