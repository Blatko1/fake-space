pub mod camera;
mod physics;
pub mod render;

use glam::{Vec2, Vec3};

use crate::{
    control::GameInput,
    world::{RoomID, World},
};

use self::{
    camera::Camera,
    physics::{CylinderBody, PhysicsStateDebugData},
};

pub struct Player {
    camera: Camera,

    body: CylinderBody,

    current_room: RoomID,

    input_state: PlayerInputState,
}

impl Player {
    pub fn new(camera: Camera, current_room: RoomID) -> Self {
        let body = CylinderBody::new(0.2, 2.0, 0.9, 
            1.2, 3.5, 3.0, 
            -4.0, 2.5, 0.0);

        Self {
            camera,

            body,

            current_room,

            input_state: PlayerInputState::default(),
        }
    }

    pub fn update(&mut self, world: &World, delta: f32) {
        let mut room = world.get_room_data(self.current_room);

        self.camera.update(&self.input_state, delta);
        self.body.update_physics_state(
            self.camera.forward_dir,
            self.camera.right_dir,
            &self.input_state,
            delta,
        );
        let new_origin = self.body.apply_physics(self.camera.origin, delta);
        self.camera.origin = new_origin;

        // Teleportation between rooms
        if let Some(tile) = room
            .segment
            .get_tile(self.camera.origin.x as i64, self.camera.origin.z as i64)
        {
            // Check if tile has a portal
            if let Some(src_dummy_portal) = tile.portal {
                // Check if portal has a linked portal
                let src_portal = room.get_portal(src_dummy_portal.id);
                if let Some((room_id, portal_id)) = src_portal.link {
                    // Teleport the player
                    self.current_room = room_id;
                    let dest_room = world.get_room_data(room_id);
                    let dest_portal = dest_room.get_portal(portal_id);
                    room = dest_room;

                    let (new_origin, yaw_angle_difference) =
                        src_portal.teleport_to(self.camera.origin, dest_portal);
                    self.camera.origin = new_origin;
                    self.camera.add_yaw_angle(yaw_angle_difference);
                    self.body.rotate_velocity(yaw_angle_difference);
                }
            }
        }
        let new_origin = self
            .body
            .collision_detection_resolution(self.camera.origin, room.segment);
        self.camera.origin = new_origin;
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
    pub fn on_mouse_move(&mut self, delta: Vec2) {
        self.camera.on_mouse_move(delta)
    }

    pub fn process_input(&mut self, input: GameInput, is_pressed: bool) {
        match input {
            GameInput::MoveForward => self.input_state.forward = is_pressed,
            GameInput::MoveBackward => self.input_state.backward = is_pressed,
            GameInput::StrafeLeft => self.input_state.left = is_pressed,
            GameInput::StrafeRight => self.input_state.right = is_pressed,
            GameInput::IncreaseFOV => self.input_state.increase_fov = is_pressed,
            GameInput::DecreaseFOV => self.input_state.decrease_fov = is_pressed,
            GameInput::PhysicsSwitch if !is_pressed => {
                self.body.toggle_ghost();
                self.body.toggle_fly();
            }
            GameInput::Jump => self.input_state.jump = is_pressed,
            GameInput::FlyUp => self.input_state.fly_up = is_pressed,
            GameInput::FlyDown => self.input_state.fly_down = is_pressed,
            // the rest is not interpretable by player
            _ => (),
        }
    }

    pub fn collect_dbg_data(&self) -> PlayerDebugData {
        PlayerDebugData {
            eye_pos: self.camera.origin,
            forward_dir: self.camera.forward_dir,
            yaw_angle: self.camera.yaw_angle.to_degrees(),
            y_shearing: self.camera.y_shearing,
            fov: self.camera.fov.to_degrees(),
            current_room_id: self.current_room.0,
            physics_state: self.body.collect_dbg_data(),
        }
    }
}

#[derive(Debug)]
pub struct PlayerDebugData {
    pub eye_pos: Vec3,
    pub forward_dir: Vec3,
    pub yaw_angle: f32,
    pub y_shearing: f32,
    pub fov: f32,
    pub current_room_id: usize,
    pub physics_state: PhysicsStateDebugData,
}

#[derive(Debug, Default)]
pub struct PlayerInputState {
    pub jump: bool,
    pub fly_up: bool,
    pub fly_down: bool,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub increase_fov: bool,
    pub decrease_fov: bool,
}

impl PlayerInputState {
    pub fn movement(&self) -> Vec2 {
        let x = if self.left { -1.0 } else { 0.0 } + if self.right { 1.0 } else { 0.0 };
        let z =
            if self.forward { 1.0 } else { 0.0 } + if self.backward { -1.0 } else { 0.0 };
        Vec2::new(x, z).try_normalize().unwrap_or_default()
    }

    pub fn fly_direction(&self) -> f32 {
        (if self.fly_up { 1.0 } else { 0.0 } - if self.fly_down { 1.0 } else { 0.0 })
    }

    pub fn fov_change(&self) -> f32 {
        (if self.increase_fov { 1.0 } else { 0.0 }
            + if self.decrease_fov { -1.0 } else { 0.0 })
    }
}
