use std::f32::consts::PI;

use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{canvas::Pixel, raycaster::Raycaster};

const MOVEMENT_SPEED: f32 = 0.05;

thread_local! {static ALA: u32 = 1;}

#[derive(Debug)]
pub struct Player {
    raycaster: Raycaster,

    movement: f32,
    rotation: f32,
}

impl Player {
    /// - angle - in degrees
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        angle: f32,
        screen_w: u32,
        screen_h: u32,
    ) -> Self {
        let fov = 80f32.to_radians();
        let raycaster = Raycaster::new(
            pos_x,
            pos_y,
            angle.to_radians(),
            fov,
            screen_w,
            screen_h,
        );

        Self {
            raycaster,

            movement: 0.0,
            rotation: 0.0,
        }
    }

    pub fn update(&mut self, texture: [Pixel; 64 * 64], data: &mut [Pixel]) {
        self.raycaster.update();
        self.raycaster.cast_rays(data);
    }

    pub fn process_input(&mut self, keyboard: KeyboardInput) {
        self.raycaster.process_input(keyboard);
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
