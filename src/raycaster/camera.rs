use glam::{Vec2, Vec3};
use winit::event::MouseScrollDelta;
use std::f32::consts::{PI, TAU};

use crate::control::GameInput;

const DEFAULT_PLANE_V: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const Y_SHEARING_SENSITIVITY: f32 = 0.8;

#[derive(Debug)]
/// Draws the player view on the screen framebuffer.
/// Uses a coordinate system where y-axis points upwards,
/// z-axis forwards and x-axis to the right.
pub struct Camera {
    /// Position of the raycaster. Whole number represents the tile and
    /// fraction represents the offset in the tile. Each tile has width and
    /// height of `1.0`.
    pub(super) origin: Vec3,
    /// Direction of the raycaster. Raycaster game engines can't make the player
    /// look up the 'normal' way and instead uses y-shearing.
    /// y-coord is always 0.
    pub(super) forward_dir: Vec3,
    pub(super) right_dir: Vec3,
    /// Raycaster (camera) horizontal plane.
    /// y-coord is always 0.
    pub(super) horizontal_plane: Vec3,
    /// Raycaster (camera) vertical plane.
    pub(super) vertical_plane: Vec3,
    /// Angle in radians.
    pub(super) yaw: f32,
    /// Width of the output screen/texture.
    pub(super) view_width: u32,
    /// Height of the output screen/texture.
    pub(super) view_height: u32,
    /// Output screen dimension aspect (width/height)
    pub(super) view_aspect: f32,
    /// Creates an illusion that the camera is looking up or down.
    /// In interval of [-self.height/2.0, self.height/2.0]
    pub(super) y_shearing: f32,

    // Specific use variables with goal to improve performance.
    // TODO rename to view_width and view_height
    pub(super) f_height: f32,
    pub(super) width_recip: f32,
    pub(super) height_recip: f32,
    pub(super) f_half_height: f32,
}

impl Camera {
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        yaw_radian: f32,
        view_width: u32,
        view_height: u32,
    ) -> Self {
        let yaw = yaw_radian;
        let f_width = view_width as f32;
        let f_height = view_height as f32;

        let view_aspect = f_width / f_height;
        let forward_dir = Vec3::new(yaw.cos(), 0.0, yaw.sin());

        Self {
            origin: Vec3::new(pos_x, pos_y, pos_z),
            forward_dir,
            right_dir: Vec3::new(forward_dir.z, forward_dir.y, -forward_dir.x),
            horizontal_plane: Vec3::cross(DEFAULT_PLANE_V, forward_dir) * view_aspect,
            vertical_plane: DEFAULT_PLANE_V,
            yaw,
            view_width,
            view_height,
            view_aspect,
            y_shearing: 0.0,

            f_height,
            width_recip: f_width.recip(),
            height_recip: f_height.recip(),
            f_half_height: view_height as f32 * 0.5,
        }
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        match delta {
            MouseScrollDelta::LineDelta(_, _) => println!("its scrol"),
            MouseScrollDelta::PixelDelta(physical_position) => println!("its pixel"),
        }
    }

    pub fn set_yaw(&mut self, yaw_rad: f32) {
        self.yaw = normalize_rad(yaw_rad);
        self.forward_dir = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        self.right_dir =
            Vec3::new(self.forward_dir.z, self.forward_dir.y, -self.forward_dir.x);
        self.horizontal_plane =
            Vec3::cross(DEFAULT_PLANE_V, self.forward_dir) * self.view_aspect;
    }
}

#[inline]
pub fn normalize_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}

pub trait CameraManipulator {
    fn get_yaw(&self) -> f32;
    fn get_pitch(&self) -> f32;
    fn get_camera_origin(&self) -> Vec3;
    fn get_forward_dir(&self) -> Vec3;
    fn get_right_dir(&self) -> Vec3;
}
