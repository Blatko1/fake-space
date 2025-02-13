mod physics;

use std::f32::consts::PI;

use glam::{Vec2, Vec3};
use winit::event::MouseScrollDelta;

use crate::{
    control::GameInput,
    map::{portal::Rotation, room::RoomID, Map},
};

use self::physics::{CylinderBody, PhysicsStateDebugData};

pub struct Player {
    body: CylinderBody,

    current_room: RoomID,
    use_flashlight: bool,
}

impl Player {
    // TODO positive pitch should make camera look UP, not DOWN!!!?
    pub fn new(current_room: RoomID) -> Self {
        let body = CylinderBody::new(
            Vec3::new(5.5, 2.0, 4.5),
            90.0f32.to_radians(),
            0.0,
            0.2,
            2.0,
            0.9,
            1.2,
            3.5,
            3.0,
            -4.0,
            2.5,
            0.0,
        );

        Self {
            body,
            current_room,
            use_flashlight: false,
        }
    }

    pub fn update(&mut self, map: &Map, delta: f32) {
        let mut room = map.get_room_data(self.current_room);

        self.body.update_physics(delta);

        // Teleportation between rooms
        if let Some(tile) = room.tilemap.get_tile_checked(
            self.body.feet_position.x as i64,
            self.body.feet_position.z as i64,
        ) {
            // Check if tile has a portal
            if let Some(id) = tile.portal_id {
                // Check if portal has a linked portal
                let src_portal = room.get_portal(id);
                if let Some((room_id, dest_id)) = src_portal.destination {
                    // Teleport the player
                    self.current_room = room_id;
                    let dest_room = map.get_room_data(room_id);
                    let dest_portal = dest_room.get_portal(dest_id);
                    room = dest_room;

                    let player_map_pos = Vec2::new(self.body.feet_position.x, self.body.feet_position.z);
                    let offset = player_map_pos - src_portal.center;

                    let src_angle = f32::atan2(src_portal.direction.y, src_portal.direction.x);
                    let dest_angle = f32::atan2(-dest_portal.direction.y, -dest_portal.direction.x);
                    let diff = dest_angle - src_angle;
                    let rotation = glam::mat2(Vec2::new(diff.cos(), diff.sin()), Vec2::new(-diff.sin(), diff.cos()));
                    let rotated_offset = rotation * offset;
                    println!("src: {}, dest: {}", src_portal.direction, dest_portal.direction);
                    let new_position = dest_portal.center + rotated_offset + (-dest_portal.direction);
                    self.body.feet_position = Vec3::new(new_position.x, self.body.feet_position.y + dest_portal.ground_height - src_portal.ground_height, new_position.y);

                    self.body.add_yaw(diff);

                    //let dest_room = map.get_room_data(room_id);
                    //let dest_portal = dest_room.get_portal(portal_id);

                    //let new_origin =
                    //    src_portal.teleport_to(self.body.feet_position, dest_portal);
                    //let yaw_angle_difference =
                    //    match src_portal.direction_difference(&dest_portal) {
                    //        Rotation::Deg0 => PI,
                    //        Rotation::AnticlockwiseDeg90 => PI * 0.5,
                    //        Rotation::ClockwiseDeg90 => -PI * 0.5,
                    //        Rotation::Deg180 => 0.0,
                    //    };
                    //self.body.feet_position = new_origin;
                    //self.body.add_yaw(yaw_angle_difference);
                }
            }
        }
        self.body.collision_detection_resolution(room.tilemap);
    }
    
    pub fn current_room_id(&self) -> RoomID {
        self.current_room
    }

    pub fn current_tile_pos(&self) -> (i64, i64) {
        (
            self.body.feet_position.x as i64,
            self.body.feet_position.z as i64,
        )
    }

    pub fn use_flashlight(&self) -> bool {
        self.use_flashlight
    }

    pub fn handle_mouse_motion(&mut self, delta: (f64, f64)) {
        self.body.handle_mouse_motion(delta);
    }

    pub fn handle_game_input(&mut self, input: GameInput, is_pressed: bool) {
        match input {
            GameInput::FlashlightSwitch if !is_pressed => {
                self.use_flashlight = !self.use_flashlight
            }
            _ => self.body.handle_game_input(input, is_pressed),
        }
    }

    pub fn get_camera_target(&self) -> &CylinderBody {
        &self.body
    }

    /*pub fn collect_dbg_data(&self) -> PlayerDebugData {
        PlayerDebugData {
            score: self.score,
            eye_pos: self.camera.origin,
            forward_dir: self.camera.forward_dir,
            yaw_angle: self.camera.yaw_angle.to_degrees(),
            y_shearing: self.camera.y_shearing,
            current_room_id: self.current_room.0,
            physics_state: self.body.collect_dbg_data(),
        }
    }*/
}

#[derive(Debug)]
pub struct PlayerDebugData {
    pub score: u32,
    pub eye_pos: Vec3,
    pub forward_dir: Vec3,
    pub yaw_angle: f32,
    pub y_shearing: f32,
    pub current_room_id: usize,
    pub physics_state: PhysicsStateDebugData,
}
