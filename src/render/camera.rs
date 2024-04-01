use std::f32::consts::{PI, TAU};

use crate::world::portal::{Portal, PortalRotationDifference};
use glam::Vec3;
use winit::event::{DeviceEvent, ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

const MOVEMENT_SPEED: f32 = 4.0;
const ROTATION_SPEED: f32 = 1.8;
const FLY_UP_DOWN_SPEED: f32 = 6.0;
const ONE_DEGREE_RAD: f32 = PI / 180.0;
const FOV_CHANGE_SPEED: f32 = ONE_DEGREE_RAD * 50.0;
const MAX_FOV_RAD: f32 = 119.0 * ONE_DEGREE_RAD;
const Y_SHEARING_SPEED: f32 = 200.0;
pub(super) const DEFAULT_PLANE_V: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const Y_SHEARING_SENSITIVITY: f32 = 0.8;
const MOUSE_ROTATION_SPEED: f32 = 0.08;

#[derive(Debug)]
/// Draws the player view on the screen framebuffer.
/// Uses a coordinate system where y-axis points upwards,
/// z-axis forwards and x-axis to the right.
pub struct Camera {
    /// Field of view in radians.
    fov: f32,
    /// Distance from the raycaster position to the camera plane.
    pub(super) plane_dist: f32,
    /// Position of the raycaster. Whole number represents the tile and
    /// fraction represents the offset in the tile. Each tile has width and
    /// height of `1.0`.
    pub(super) origin: Vec3,
    /// Direction of the raycaster. Raycaster game engines can't make the player
    /// look up the 'normal' way and instead uses y-shearing.
    /// y-coord is always 0.
    pub(super) dir: Vec3,
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
    pub(super) aspect: f32,
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

    // Variables for controlling and moving the scene.
    turn_left: f32,
    turn_right: f32,
    strafe_left: f32,
    strafe_right: f32,
    increase_fov: f32,
    decrease_fov: f32,
    increase_y_shearing: f32,
    decrease_y_shearing: f32,
    fly_up: f32,
    fly_down: f32,
    forward: f32,
    backward: f32,
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
        let aspect = f_width / f_height;

        let origin = Vec3::new(pos_x, pos_y, pos_z);
        let dir = Vec3::new(yaw_angle.cos(), 0.0, yaw_angle.sin());

        let vertical_plane = DEFAULT_PLANE_V / plane_dist;
        let horizontal_plane = Vec3::cross(DEFAULT_PLANE_V, dir) * aspect / plane_dist;

        Self {
            fov,
            plane_dist,
            origin,
            dir,
            horizontal_plane,
            vertical_plane,
            yaw_angle,
            view_width,
            view_height,
            aspect,
            y_shearing: 0.0,

            four_width: 4 * view_width as usize,
            f_height,
            width_recip: f_width.recip(),
            height_recip: f_height.recip(),
            f_half_height: view_height as f32 * 0.5,
            f_half_width: view_width as f32 * 0.5,

            turn_left: 0.0,
            turn_right: 0.0,
            strafe_left: 0.0,
            strafe_right: 0.0,
            increase_fov: 0.0,
            decrease_fov: 0.0,
            increase_y_shearing: 0.0,
            decrease_y_shearing: 0.0,
            fly_up: 0.0,
            fly_down: 0.0,
            forward: 0.0,
            backward: 0.0,
        }
    }

    pub fn update(&mut self, frame_time: f32) {
        // Change FOV and vertical FOV
        self.fov = (self.fov
            + (self.increase_fov - self.decrease_fov) * FOV_CHANGE_SPEED * frame_time)
            .clamp(ONE_DEGREE_RAD, MAX_FOV_RAD);
        self.plane_dist = 1.0 / f32::tan(self.fov * 0.5);

        // Change y_shearing (look up/down)
        self.y_shearing = (self.y_shearing
            + (self.decrease_y_shearing - self.increase_y_shearing)
                * Y_SHEARING_SPEED
                * frame_time)
            .clamp(-self.f_height, self.f_height);

        // Update rotation and direction
        self.yaw_angle = normalize_rad(
            self.yaw_angle
                + (self.turn_left - self.turn_right) * ROTATION_SPEED * frame_time,
        );
        self.dir = Vec3::new(self.yaw_angle.cos(), 0.0, self.yaw_angle.sin());

        // Rotate camera planes
        self.vertical_plane = DEFAULT_PLANE_V / self.plane_dist;
        self.horizontal_plane =
            Vec3::cross(DEFAULT_PLANE_V, self.dir) * self.aspect / self.plane_dist;

        // Update origin position
        self.origin.x +=
            self.dir.x * (self.forward - self.backward) * MOVEMENT_SPEED * frame_time;
        self.origin.z +=
            self.dir.z * (self.forward - self.backward) * MOVEMENT_SPEED * frame_time;
        let horizontal_plane_norm = self.horizontal_plane.normalize();
        self.origin.x += horizontal_plane_norm.x
            * (self.strafe_right - self.strafe_left)
            * MOVEMENT_SPEED
            * frame_time;
        self.origin.z += horizontal_plane_norm.z
            * (self.strafe_right - self.strafe_left)
            * MOVEMENT_SPEED
            * frame_time;
        self.origin.y += (self.fly_up - self.fly_down) * FLY_UP_DOWN_SPEED * frame_time;
    }

    pub fn portal_teleport(&mut self, src: Portal, dest: Portal) {
        let x;
        let y = self.origin.y + dest.ground_level - src.ground_level;
        let z;
        match src.direction.rotation_difference(dest.direction) {
            PortalRotationDifference::None => {
                x = dest.position.x as f32 + self.origin.x.fract();
                z = dest.position.z as f32 + self.origin.z.fract();
            }
            PortalRotationDifference::ClockwiseDeg90 => {
                self.yaw_angle -= PI * 0.5;
                x = dest.center.x - (src.center.z - self.origin.z);
                z = dest.center.z + (src.center.x - self.origin.x);
            }
            PortalRotationDifference::AnticlockwiseDeg90 => {
                self.yaw_angle += PI * 0.5;
                x = dest.center.x + (src.center.z - self.origin.z);
                z = dest.center.z - (src.center.x - self.origin.x);
            }
            PortalRotationDifference::Deg180 => {
                self.yaw_angle += PI;
                x = dest.center.x + (src.center.x) - self.origin.x;
                z = dest.center.z + (src.center.z) - self.origin.z;
            }
        }
        self.origin = Vec3::new(x, y, z);
        self.update(0.0);
    }

    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.y_shearing += delta.1 as f32 * Y_SHEARING_SENSITIVITY;

                self.yaw_angle -= delta.0 as f32 * ONE_DEGREE_RAD * MOUSE_ROTATION_SPEED;
            }
            _ => (),
        }
    }

    pub fn process_keyboard_input(&mut self, event: KeyEvent) {
        let value = match event.state {
            ElementState::Pressed => 1.0,
            ElementState::Released => 0.0,
        };
        if let PhysicalKey::Code(key) = event.physical_key {
            match key {
                // Turn left:
                KeyCode::KeyQ => self.turn_left = value,
                // Turn right:
                KeyCode::KeyE => self.turn_right = value,
                // Move forward:
                KeyCode::KeyW => self.forward = value,
                // Move backward:
                KeyCode::KeyS => self.backward = value,
                // Strafe left:
                KeyCode::KeyA => self.strafe_left = value,
                // Strafe right:
                KeyCode::KeyD => self.strafe_right = value,
                // Increase FOV:
                KeyCode::ArrowUp => self.increase_fov = value,
                // Increase FOV:
                KeyCode::ArrowDown => self.decrease_fov = value,
                // Look more up (y_shearing):
                KeyCode::PageUp => self.increase_y_shearing = value,
                // Look more down (y_shearing):
                KeyCode::PageDown => self.decrease_y_shearing = value,
                // Reset look (y_shearing):
                KeyCode::Home => self.y_shearing = 0.0,
                // Reset look (y_shearing):
                KeyCode::Space => self.fly_up = value,
                // Reset look (y_shearing):
                KeyCode::ShiftLeft => self.fly_down = value,
                _ => (),
            }
        }
    }

    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.dir
    }

    pub fn yaw_angle(&self) -> f32 {
        self.yaw_angle
    }

    pub fn y_shearing(&self) -> f32 {
        self.y_shearing
    }
}

#[inline]
fn normalize_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}
