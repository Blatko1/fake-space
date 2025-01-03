pub mod portal;
pub mod room;
pub mod segment;

use std::path::PathBuf;

use nom::{error::convert_error, Finish};
use rand::seq::SliceRandom;
use room::{Room, RoomID, RoomRef};
use segment::Segment;

use crate::map_parser::{cleanup_input, MapParser};

pub struct Map {
    segments: Vec<Segment>,
    rooms: Vec<Room>,
}

impl Map {
    // TODO starting segment is always '0' and main room is '1'
    pub fn new(segments: Vec<Segment>) -> Self {
        let mut rooms = Vec::new();
        let mut room_counter = 0;
        let mut rng = rand::thread_rng();

        // Select the first segment which repeats only once
        let segment = &segments[0];
        let mut starting_room = Room::new(RoomID(room_counter), segment);
        room_counter += 1;

        let root_segment = &segments[1];
        let mut adjacent_rooms: Vec<Room> = starting_room
            .portals
            .iter_mut()
            .map(|portal| {
                let mut new_room = Room::new(RoomID(room_counter), root_segment);
                let room_rand_portal = new_room.portals.choose_mut(&mut rng).unwrap();
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

        Self { segments, rooms }
    }

    pub fn get_room_data(&self, index: RoomID) -> RoomRef {
        let room = &self.rooms[index.0];
        RoomRef {
            segment: &self.segments[room.segment_id.0],
            data: room,
        }
    }
}
