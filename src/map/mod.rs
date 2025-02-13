// TODO check if these really need to be pub
pub mod tilemap;
pub mod parser;
pub mod portal;
pub mod room;

use std::path::PathBuf;

use glam::Vec2;
use tilemap::Tilemap;
use portal::{Orientation, Rotation};
use rand::seq::SliceRandom;
use room::{Room, RoomID, RoomRef};

const DEFAULT_ROOM_DIRECTION: Vec2 = Vec2::Y;

pub struct Map {
    tilemaps: Vec<Tilemap>,
    rooms: Vec<Room>,
}

impl Map {
    // TODO starting blueprint is always '0' and main room is '1'
    pub fn new(tilemaps: Vec<Tilemap>) -> Self {
        let mut rooms = Vec::new();
        let mut room_counter = 0;
        let mut rng = rand::thread_rng();

        // Select the first blueprint which repeats only once
        let tilemap = &tilemaps[0];
        let mut starting_room = Room {
            id: RoomID(room_counter),
            tilemap_id: tilemap.id,
            portals: tilemap.unlinked_portals.clone(),
            is_fully_generated: false,
            skybox: tilemap.default_skybox,
            ambient_light_intensity: tilemap.default_ambient_light,
            direction: DEFAULT_ROOM_DIRECTION,
        };
        room_counter += 1;

        let root_segment = &tilemaps[1];
        let mut adjacent_rooms: Vec<Room> = starting_room
            .portals
            .iter_mut()
            .map(|portal| {
                let mut new_room = Room {
                    id: RoomID(room_counter),
                    tilemap_id: root_segment.id,
                    portals: root_segment.unlinked_portals.clone(),
                    //objects: blueprint.object_placeholders.clone(),
                    is_fully_generated: false,
                    skybox: root_segment.default_skybox,
                    ambient_light_intensity: root_segment.default_ambient_light,

                    // Temporary value
                    direction: Vec2::ZERO
                };
                let dest_portal = new_room.portals.choose_mut(&mut rng).unwrap();
                let src_angle = f32::atan2(portal.direction.y, portal.direction.x);
                let dest_angle = f32::atan2(-dest_portal.direction.y, -dest_portal.direction.x);
                // Angle for how much to rotate the destination room
                let diff = dest_angle - src_angle;
                let rotation = glam::mat2(Vec2::new(diff.cos(), diff.sin()), Vec2::new(-diff.sin(), diff.cos()));
                let dest_room_direction = rotation * DEFAULT_ROOM_DIRECTION;
                new_room.direction = dest_room_direction;
                // Connect the two portals:
                // Connect the starting room with new random room
                portal.destination = Some((new_room.id, dest_portal.id));
                // Connect the new random room with the starting room
                dest_portal.destination = Some((starting_room.id, portal.id));
                room_counter += 1;

                new_room
            })
            .collect();
        rooms.push(starting_room);
        rooms.append(&mut adjacent_rooms);

        Self { tilemaps, rooms }
    }

    pub fn get_room_data(&self, index: RoomID) -> RoomRef {
        let room = &self.rooms[index.0];
        RoomRef {
            tilemap: &self.tilemaps[room.tilemap_id.0],
            data: room,
        }
    }
}
