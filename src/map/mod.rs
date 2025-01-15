pub mod portal;
pub mod room;
pub mod blueprint;
pub mod parser;

use std::path::PathBuf;

use nom::{error::convert_error, Finish};
use portal::{Orientation, Rotation};
use rand::seq::SliceRandom;
use room::{Room, RoomID, RoomRef};
use blueprint::Blueprint;

pub struct Map {
    blueprints: Vec<Blueprint>,
    rooms: Vec<Room>,
}

impl Map {
    // TODO starting blueprint is always '0' and main room is '1'
    pub fn new(blueprints: Vec<Blueprint>) -> Self {
        let mut rooms = Vec::new();
        let mut room_counter = 0;
        let mut rng = rand::thread_rng();

        // Select the first blueprint which repeats only once
        let blueprint = &blueprints[0];
        let mut starting_room = Room::new(RoomID(room_counter), blueprint, Orientation::North);
        room_counter += 1;

        let root_segment = &blueprints[1];
        let mut adjacent_rooms: Vec<Room> = starting_room
            .portals
            .iter_mut()
            .map(|portal| {
                let mut new_room = Room {
                    id: RoomID(room_counter),
                    segment_id: blueprint.id,
                    portals: blueprint.unlinked_portals.clone(),
                    //objects: blueprint.object_placeholders.clone(),
                    is_fully_generated: false,
                    skybox: blueprint.skybox,
                    ambient_light_intensity: blueprint.ambient_light_intensity,
        
                    // Temporary value
                    orientation: Orientation::North
                };
                let room_rand_portal = new_room.portals.choose_mut(&mut rng).unwrap();
                let rotation_difference = portal.direction_difference(&room_rand_portal);
                new_room.orientation = Orientation::from_angle((starting_room.orientation as i32 + 360 + rotation_difference as i32) % 360);
                // Connect the two portals:
                // Connect the starting room with new random room
                portal.link = Some((new_room.id, room_rand_portal.id));
                // Connect the new random room with the starting room
                room_rand_portal.link = Some((starting_room.id, portal.id));
                room_counter += 1;

                new_room
            })
            .collect();
        rooms.push(starting_room);
        rooms.append(&mut adjacent_rooms);

        Self { blueprints, rooms }
    }

    pub fn get_room_data(&self, index: RoomID) -> RoomRef {
        let room = &self.rooms[index.0];
        RoomRef {
            blueprint: &self.blueprints[room.segment_id.0],
            data: room,
        }
    }
}
