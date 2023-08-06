use std::f32::consts::PI;

use glam::Vec2;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    canvas::Pixel,
    raycaster::Raycaster,
    state::{MAP, MAP_HEIGHT, MAP_WIDTH},
    HEIGHT, WIDTH,
};

const MOVEMENT_SPEED: f32 = 0.05;

#[derive(Debug)]
pub struct Player {
    raycaster: Raycaster,

    movement: f32,
    rotation: f32,
}

impl Player {
    /// - angle - in degrees
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        let fov = 80f32.to_radians();
        let raycaster = Raycaster::new(x, y, angle.to_radians(), fov);

        Self {
            raycaster,

            movement: 0.0,
            rotation: 0.0,
        }
    }

    pub fn update(&mut self) {}

    pub fn process_input(&mut self, keyboard: KeyboardInput) {
        if let Some(key) = keyboard.virtual_keycode {
            let value = match keyboard.state {
                ElementState::Pressed => 1.0,
                ElementState::Released => 0.0,
            };

            match key {
                // Turn left:
                VirtualKeyCode::A => self.rotation = value,
                // Turn right:
                VirtualKeyCode::D => self.rotation = -value,
                // Move forward:
                VirtualKeyCode::W => self.movement = value,
                // Move backward:
                VirtualKeyCode::S => self.movement = -value,
                _ => (),
            }
        }
    }
}

#[derive(Debug)]
pub struct Hit {
    pub value: u32,
    pub side: Side,
    pub pos: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

fn verline(x: u32, start: u32, end: u32, data: &mut [Pixel], color: u8) {
    for i in start..end {
        data[(HEIGHT as usize - 1 - i as usize) * WIDTH as usize + x as usize] =
            Pixel {
                r: color,
                g: color,
                b: color,
                a: 255,
            }
    }
}

#[inline]
fn norm_rad(angle: f32) -> f32 {
    if angle > 2.0 * PI {
        angle - 2.0 * PI
    } else if angle < 0.0 {
        angle + 2.0 * PI
    } else {
        angle
    }
}
