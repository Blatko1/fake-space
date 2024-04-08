use glam::Vec3;
use winit::{event::KeyEvent, keyboard::{KeyCode, PhysicalKey}};

pub struct CylinderBody {
    position: Vec3,
    width_radius: f32,
    height: f32,
    eye_height: f32,

    velocity: Vec3,
    gravity_accel: f32,
    friction: f32,
    on_ground: bool,
}

impl CylinderBody {
    pub fn new(width_radius: f32, height: f32, eye_height_factor: f32, gravity_accel: f32, friction: f32) -> Self {
        assert!(eye_height_factor <= 1.0 && eye_height_factor >= 0.0, "Eye height not in range [0, 1]!");
        Self {
            // TODO temp
            position: Vec3::ZERO,
            width_radius,
            height,
            eye_height: eye_height_factor * height,

            velocity: Vec3::ZERO,
            gravity_accel,
            friction,
            on_ground: false
        }
    }
}