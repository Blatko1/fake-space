use crate::{
    backend::Canvas,
    player::{camera::Camera, Player},
    world::{RoomID, World},
};
use winit::event::DeviceEvent;
use winit::event::KeyEvent;

pub struct State {
    world: World,
    player: Player,
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
        self.player.process_keyboard_input(event);
    }

    #[inline]
    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        self.player.process_mouse_input(event);
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn world(&self) -> &World {
        &self.world
    }
}
