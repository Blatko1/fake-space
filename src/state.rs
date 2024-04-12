use crate::{
    backend::Canvas, control::ControllerSettings, dbg::DebugData, player::{camera::Camera, Player}, world::{RoomID, World}
};
use glam::Vec2;
use winit::{event::DeviceEvent, keyboard::PhysicalKey};
use winit::event::KeyEvent;

pub struct State {
    world: World,
    player: Player,

    controls: ControllerSettings
}

impl State {
    pub fn new(canvas: &Canvas, world: World) -> Self {
        let camera = Camera::new(
            10.5,
            1.0,
            14.5,
            90f32.to_radians(),
            90f32.to_radians(),
            canvas.width(),
            canvas.height(),
        );

        Self {
            world,
            player: Player::new(camera, RoomID(0)),

            controls: ControllerSettings::init()
        }
    }

    pub fn update(&mut self, frame_time: f32) {
        self.player.update(&self.world, frame_time);
        self.world.update(self.player.current_room_id());
    }

    pub fn draw<'a, C>(&mut self, canvas_column_iter: C)
    where
        C: Iterator<Item = &'a mut [u8]>,
    {
        self.player.cast_and_draw(&self.world, canvas_column_iter);
    }

    #[inline]
    pub fn process_keyboard_input(&mut self, event: KeyEvent) {
        match event.physical_key {
            PhysicalKey::Code(key) => if let Some(game_input) = self.controls.get_input_binding(&key) {
                let is_pressed = event.state.is_pressed();
                for input in game_input.iter() {
                    self.player.process_input(*input, is_pressed)
                }
            },
            _ => ()
        }
    }

    #[inline]
    pub fn on_mouse_move(&mut self, delta: Vec2) {
        self.player.on_mouse_move(delta);
    }

    /*pub fn collect_dbg_data(&self, avg_fps_time: f64, current_fps: i32) -> DebugData {
        let player_dbg_data = self.player.collect_dbg_data();
        let world_dbg_data = self.world.collect_dbg_data();

        DebugData {
            current_fps,
            avg_fps_time,
            
            player_data: player_dbg_data,
            world_data: world_dbg_data
        }
    }*/
}
