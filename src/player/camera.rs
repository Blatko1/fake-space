use glam::{Vec2, Vec3};
use std::f32::consts::{PI, TAU};

use super::PlayerInputState;

const ONE_DEGREE_RAD: f32 = PI / 180.0;
const FOV_CHANGE_SPEED: f32 = ONE_DEGREE_RAD * 50.0;
const MAX_FOV_RAD: f32 = 119.0 * ONE_DEGREE_RAD;
pub(super) const DEFAULT_PLANE_V: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const Y_SHEARING_SENSITIVITY: f32 = 0.8;
const MOUSE_DRAG_SPEED: f32 = 0.08;
const YAW_CHANGE_FACTOR: f32 = ONE_DEGREE_RAD * MOUSE_DRAG_SPEED;

#[derive(Debug)]
/// Draws the player view on the screen framebuffer.
/// Uses a coordinate system where y-axis points upwards,
/// z-axis forwards and x-axis to the right.
pub struct Camera {
    /// Field of view in radians.
    pub(super) fov: f32,
    /// Distance from the raycaster position to the camera plane.
    pub(super) plane_dist: f32,
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
    pub(super) yaw_angle: f32,
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
    four_width: usize,
    pub(super) f_height: f32,
    pub(super) width_recip: f32,
    pub(super) height_recip: f32,
    pub(super) f_half_height: f32,
    pub(super) f_half_width: f32,
}

impl Camera {
    /// `pos_y` - height of the raycaster (camera)
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        yaw_angle_radian: f32,
        fov_radian: f32,
        view_width: u32,
        view_height: u32,
    ) -> Self {
        let fov = fov_radian;
        let yaw_angle = yaw_angle_radian;
        let f_width = view_width as f32;
        let f_height = view_height as f32;

        let plane_dist = 1.0 / f32::tan(fov / 2.0);
        let view_aspect = f_width / f_height;

        let origin = Vec3::new(pos_x, pos_y, pos_z);
        let forward_dir = Vec3::new(yaw_angle.cos(), 0.0, yaw_angle.sin());

        let vertical_plane = DEFAULT_PLANE_V / plane_dist;
        let horizontal_plane =
            Vec3::cross(DEFAULT_PLANE_V, forward_dir) * view_aspect / plane_dist;

        Self {
            fov,
            plane_dist,
            origin,
            forward_dir,
            right_dir: Vec3::new(forward_dir.z, forward_dir.y, -forward_dir.x),
            horizontal_plane,
            vertical_plane,
            yaw_angle,
            view_width,
            view_height,
            view_aspect,
            y_shearing: 0.0,

            four_width: 4 * view_width as usize,
            f_height,
            width_recip: f_width.recip(),
            height_recip: f_height.recip(),
            f_half_height: view_height as f32 * 0.5,
            f_half_width: view_width as f32 * 0.5,
        }
    }

    pub fn update(&mut self, input_state: &PlayerInputState, delta: f32) {
        // Changing FOV
        let fov_change = input_state.fov_change();
        self.fov = (self.fov + fov_change * FOV_CHANGE_SPEED * delta)
            .clamp(ONE_DEGREE_RAD, MAX_FOV_RAD);

        // Reposition camera planes
        self.plane_dist = 1.0 / f32::tan(self.fov * 0.5);
        self.vertical_plane = DEFAULT_PLANE_V / self.plane_dist;
        self.horizontal_plane = Vec3::cross(DEFAULT_PLANE_V, self.forward_dir)
            * self.view_aspect
            / self.plane_dist;
    }

    pub fn on_mouse_move(&mut self, delta: Vec2) {
        self.y_shearing = (self.y_shearing + delta.y * Y_SHEARING_SENSITIVITY)
            .clamp(-self.f_height, self.f_height);

        self.add_yaw_angle(-delta.x * YAW_CHANGE_FACTOR);
    }

    pub fn set_yaw(&mut self, yaw_angle_rad: f32) {
        self.yaw_angle = normalize_rad(yaw_angle_rad);
        self.forward_dir = Vec3::new(self.yaw_angle.cos(), 0.0, self.yaw_angle.sin());
        self.right_dir =
            Vec3::new(self.forward_dir.z, self.forward_dir.y, -self.forward_dir.x);
        self.horizontal_plane = Vec3::cross(DEFAULT_PLANE_V, self.forward_dir)
            * self.view_aspect
            / self.plane_dist;
    }

    pub fn add_yaw_angle(&mut self, rad: f32) {
        self.set_yaw(self.yaw_angle + rad);
    }
}

#[inline]
fn normalize_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}
