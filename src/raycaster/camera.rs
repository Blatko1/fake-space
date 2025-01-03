use glam::{Vec2, Vec3};
use std::f32::consts::{PI, TAU};
use winit::event::MouseScrollDelta;

use crate::control::GameInput;

const DEFAULT_PLANE_V: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const Y_SHEARING_SENSITIVITY: f32 = 0.8;

// TODO revert to pub(super) later

#[derive(Debug, Default)]
/// Draws the player view on the screen framebuffer.
/// Uses a coordinate system where y-axis points upwards,
/// z-axis forwards and x-axis to the right.
pub struct Camera {
    /// Position of the raycaster. Whole number represents the tile and
    /// fraction represents the offset in the tile. Each tile has width and
    /// height of `1.0`.
    pub origin: Vec3,
    /// Direction of the raycaster. Raycaster game engines can't make the player
    /// look up the 'normal' way and instead uses y-shearing.
    /// y-coord is always 0.
    pub forward_dir: Vec3,
    pub right_dir: Vec3,
    /// Raycaster (camera) horizontal plane.
    /// y-coord is always 0.
    pub horizontal_plane: Vec3,
    /// Raycaster (camera) vertical plane.
    pub vertical_plane: Vec3,
    /// Angle in radians.
    //pub(super) yaw: f32,
    /// Width of the output screen/texture.
    pub view_width: u32,
    /// Height of the output screen/texture.
    pub view_height: u32,
    /// Output screen dimension aspect (width/height)
    pub view_aspect: f32,
    /// Creates an illusion that the camera is looking up or down.
    /// In interval of [-self.height/2.0, self.height/2.0]
    pub y_shearing: f32,

    // Specific use variables with goal to improve performance.
    // TODO rename to view_width and view_height
    pub f_height: f32,
    pub width_recip: f32,
    pub height_recip: f32,
    pub f_half_height: f32,
}

impl Camera {
    pub fn new(view_width: u32, view_height: u32) -> Self {
        let f_width = view_width as f32;
        let f_height = view_height as f32;

        let view_aspect = f_width / f_height;

        Self {
            vertical_plane: DEFAULT_PLANE_V,

            view_width,
            view_height,
            view_aspect,

            f_height,
            width_recip: f_width.recip(),
            height_recip: f_height.recip(),
            f_half_height: view_height as f32 * 0.5,

            ..Default::default()
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, _) => println!("its scrol"),
            MouseScrollDelta::PixelDelta(physical_position) => println!("its pixel"),
        }
    }

    pub fn follow<T: CameraTarget>(&mut self, target: &T) {
        let data = target.get_target_data();
        self.origin = data.origin;
        //self.yaw = data.yaw;
        self.forward_dir = data.forward_dir;
        self.horizontal_plane =
            Vec3::cross(DEFAULT_PLANE_V, data.forward_dir) * self.view_aspect;
        self.y_shearing = data.pitch.sin() * self.f_height;
    }
}

#[inline]
pub fn normalize_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}

pub trait CameraTarget {
    fn get_target_data(&self) -> CameraTargetData;
}

// TODO instead store precalculated y_shearing variable
pub struct CameraTargetData {
    pub origin: Vec3,
    pub forward_dir: Vec3,
    pub right_dir: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}
