use std::f32::consts::PI;

use winit::event::{KeyboardInput, VirtualKeyCode};

#[derive(Debug, Default)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub angle: f32, // TODO maybe convert to u32
}

impl Player {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            x,
            y,
            angle: normalize_angle(angle),
        }
    }

    pub fn process_input(&mut self, input: KeyboardInput) {
        if let Some(code) = input.virtual_keycode {
            match code {
                // Turn left:
                VirtualKeyCode::A => {
                    self.angle = normalize_angle(self.angle + 1.0)
                }
                // Turn right:
                VirtualKeyCode::D => {
                    self.angle = normalize_angle(self.angle - 1.0)
                }
                // Move forward:
                VirtualKeyCode::W => {
                    self.x += self.angle.cos() * 2.0;
                    self.y += self.angle.sin() * 2.0;
                }
                // Move backward:
                VirtualKeyCode::S => {
                    self.x -= self.angle.cos() * 2.0;
                    self.y -= self.angle.sin() * 2.0;
                }
                _ => (),
            }
        }
    }
}

#[inline]
fn normalize_angle(angle: f32) -> f32 {
    if angle > 360.0 {
        0.0
    } else if angle < 0.0 {
        360.0
    } else {
        angle
    }
}
