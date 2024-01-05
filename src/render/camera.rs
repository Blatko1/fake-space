use std::f32::consts::{PI, TAU};

use glam::Vec3;
use winit::event::{ElementState, VirtualKeyCode, KeyboardInput, DeviceEvent};

use super::ray::Ray;

// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;
const ROTATION_SPEED: f32 = 0.035;
const FLY_UP_DOWN_SPEED: f32 = 0.05;
const ONE_DEGREE_RAD: f32 = PI / 180.0;
const MAX_FOV_RAD: f32 = 119.0 * ONE_DEGREE_RAD;
pub(super) const DEFAULT_PLANE_V: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const Y_SHEARING_SENSITIVITY: f32 = 0.8;
const MOUSE_ROTATION_SPEED: f32 = 0.08;
const MAX_Y: f32 = 50.0;
const MIN_Y: f32 = -50.0;

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
    vertical_plane: Vec3,
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
    height_recip: f32,
    pub(super) f_half_height: f32,

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
    in_portal: bool,
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
            in_portal: false,
        }
    }

    /*pub fn cast_and_draw(&self, world: &mut World, data: &mut [u8]) {
        let canvas_column_iterator =
            data.chunks_exact_mut(self.height as usize * 4).enumerate();
        canvas_column_iterator.for_each(|(x, column)| {
            self.draw_column(x, world, column);
        });
    }*/

    pub fn update(&mut self) {
        // Change FOV and vertical FOV
        self.fov = (self.fov + (self.increase_fov - self.decrease_fov) * ONE_DEGREE_RAD)
            .clamp(ONE_DEGREE_RAD, MAX_FOV_RAD);
        self.plane_dist = 1.0 / f32::tan(self.fov * 0.5);

        // Change y_shearing (look up/down)
        self.y_shearing = (self.y_shearing
            + (self.decrease_y_shearing - self.increase_y_shearing) * 2.5)
            .clamp(-self.f_height, self.f_height);

        // Update rotation and direction
        self.yaw_angle = normalize_rad(
            self.yaw_angle + (self.turn_left - self.turn_right) * ROTATION_SPEED,
        );
        self.dir = Vec3::new(self.yaw_angle.cos(), 0.0, self.yaw_angle.sin());

        // Rotate raycaster (camera) planes
        self.vertical_plane = DEFAULT_PLANE_V / self.plane_dist;
        self.horizontal_plane =
            Vec3::cross(DEFAULT_PLANE_V, self.dir) * self.aspect / self.plane_dist;

        // Update origin position
        self.origin.x += self.dir.x * (self.forward - self.backward) * MOVEMENT_SPEED;
        self.origin.z += self.dir.z * (self.forward - self.backward) * MOVEMENT_SPEED;
        self.origin += self.horizontal_plane.normalize()
            * (self.strafe_right - self.strafe_left)
            * MOVEMENT_SPEED;
        self.origin.y = (self.origin.y
            + (self.fly_up - self.fly_down) * FLY_UP_DOWN_SPEED)
            .clamp(MIN_Y, MAX_Y);
    }

    pub fn get_origin(&self) -> Vec3 {
        self.origin
    }

    pub fn set_origin(&mut self, x: f32, y: f32, z: f32) {
        self.origin.x = x;
        self.origin.y = y;
        self.origin.z = z;
    }

    pub fn increase_direction_angle(&mut self, inc_radians: f32) {
        self.yaw_angle += inc_radians;
        self.update();
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

    pub fn process_keyboard_input(&mut self, event: KeyboardInput) {
        if let Some(key) = event.virtual_keycode {
            let value = match event.state {
                ElementState::Pressed => 1.0,
                ElementState::Released => 0.0,
            };
            match key {
                // Turn left:
                VirtualKeyCode::Q => self.turn_left = value,
                // Turn right:
                VirtualKeyCode::E => self.turn_right = value,
                // Move forward:
                VirtualKeyCode::W => self.forward = value,
                // Move backward:
                VirtualKeyCode::S => self.backward = value,
                // Strafe left:
                VirtualKeyCode::A => self.strafe_left = value,
                // Strafe right:
                VirtualKeyCode::D => self.strafe_right = value,
                // Increase FOV:
                VirtualKeyCode::Up => self.increase_fov = value,
                // Increase FOV:
                VirtualKeyCode::Down => self.decrease_fov = value,
                // Look more up (y_shearing):
                VirtualKeyCode::PageUp => self.increase_y_shearing = value,
                // Look more down (y_shearing):
                VirtualKeyCode::PageDown => self.decrease_y_shearing = value,
                // Reset look (y_shearing):
                VirtualKeyCode::Home => self.y_shearing = 0.0,
                // Reset look (y_shearing):
                VirtualKeyCode::Space => self.fly_up = value,
                // Reset look (y_shearing):
                VirtualKeyCode::LShift => self.fly_down = value,
                _ => (),
            }
        }
    }
}

#[inline]
fn normalize_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}
