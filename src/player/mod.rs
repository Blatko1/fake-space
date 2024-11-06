mod physics;

use glam::{Vec2, Vec3};
use winit::event::MouseScrollDelta;

use crate::{
    control::GameInput, map::{room::RoomID, Map}
};

use self::{
    physics::{CylinderBody, PhysicsStateDebugData},
};

pub struct Player {
    body: CylinderBody,

    score: u32,
    current_room: RoomID,
    use_flashlight: bool,
}

impl Player {
    pub fn new(current_room: RoomID) -> Self {
        let body = CylinderBody::new(Vec3::new(10.5, 1.0, 14.5), 90.0f32.to_radians(), 0.0, 0.2, 2.0, 0.9, 1.2, 3.5, 3.0, -4.0, 2.5, 0.0);

        Self {
            score: 0,
            body,
            current_room,
            use_flashlight: false,
        }
    }

    pub fn update(&mut self, map: &Map, delta: f32) {
        let mut room = map.get_room_data(self.current_room);

        self.body.update_physics(delta);

        // Teleportation between rooms
        if let Some(tile) = room
            .segment
            .get_tile_checked(self.body.feet_position.x as i64, self.body.feet_position.z as i64)
        {
            // Check if tile has a portal
            if let Some(src_dummy_portal) = tile.portal {
                // Check if portal has a linked portal
                let src_portal = room.get_portal(src_dummy_portal.id);
                if let Some((room_id, portal_id)) = src_portal.link {
                    // Teleport the player
                    self.current_room = room_id;
                    let dest_room = map.get_room_data(room_id);
                    let dest_portal = dest_room.get_portal(portal_id);
                    room = dest_room;

                    let (new_origin, yaw_angle_difference) =
                        src_portal.teleport(self.body.feet_position, dest_portal);
                    self.body.feet_position = new_origin;
                    self.body.add_yaw(yaw_angle_difference);
                }
            }
        }
        self
            .body
            .collision_detection_resolution(room.segment);
    }

    pub fn increase_score(&mut self, add: u32) {
        self.score += add
    }

    pub fn current_room_id(&self) -> RoomID {
        self.current_room
    }

    pub fn current_tile_pos(&self) -> (i64, i64) {
        (self.body.feet_position.x as i64, self.body.feet_position.z as i64)
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
