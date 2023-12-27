mod colors;
mod top_bottom;
mod voxel_model;
mod wall;

use glam::Vec3;
use std::f32::consts::{PI, TAU};
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    voxel::VoxelModelManager,
    world::map::{Map, Tile},
};

// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;
const ROTATION_SPEED: f32 = 0.035;
const FLY_UP_DOWN_SPEED: f32 = 0.05;
const ONE_DEGREE_RAD: f32 = PI / 180.0;
const MAX_FOV_RAD: f32 = 119.0 * ONE_DEGREE_RAD;
const DEFAULT_PLANE_V: Vec3 = Vec3::new(0.0, 0.5, 0.0);
const Y_SHEARING_SENSITIVITY: f32 = 0.8;
const MOUSE_ROTATION_SPEED: f32 = 0.08;
const MAX_Y: f32 = 50.0;
const MIN_Y: f32 = -50.0;

#[derive(Debug, Clone, Copy)]
struct DrawParams {
    pub closer_wall_dist: f32,
    pub further_wall_dist: f32,
    pub bottom_draw_bound: usize,
    pub top_draw_bound: usize,
    pub draw_x: u32,
    pub current_tile: Tile,
    pub next_tile: Tile,
    pub tile_x: f32,
    pub tile_z: f32,
    pub ray_dir: Vec3,
    pub side: Side,
    pub wall_offset: f32,
    pub delta_dist_x: f32,
    pub delta_dist_z: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

#[derive(Debug)]
/// Draws the player view on the screen framebuffer.
/// Uses a coordinate system where y-axis points upwards,
/// z-axis forwards and x-axis to the right.
pub struct RayCaster {
    /// Field of view in radians.
    fov: f32,
    /// Distance from the raycaster position to the camera plane.
    plane_dist: f32,
    /// Position of the raycaster. Whole number represents the tile and
    /// fraction represents the offset in the tile. Each tile has width and
    /// height of `1.0`.
    pos: Vec3,
    /// Direction of the raycaster. Raycaster game engines can't make the player
    /// look up the 'normal' way and instead uses y-shearing.
    /// y-coord is always 0.
    dir: Vec3,
    /// Raycaster (camera) horizontal plane.
    /// y-coord is always 0.
    plane_h: Vec3,
    /// Raycaster (camera) vertical plane.
    plane_v: Vec3,
    /// Angle in radians.
    angle: f32,
    /// Width of the output screen/texture.
    width: u32,
    /// Height of the output screen/texture.
    height: u32,
    /// Output screen dimension aspect (width/height)
    aspect: f32,
    /// Creates an illusion that the camera is looking up or down.
    /// In interval of [-self.height/2.0, self.height/2.0]
    y_shearing: f32,

    // Specific use variables with goal to improve performance.
    four_width: usize,
    f_height: f32,
    width_recip: f32,
    height_recip: f32,
    f_half_height: f32,

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

impl RayCaster {
    /// `pos_y` - height of the raycaster (camera)
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        angle: f32,
        width: u32,
        height: u32,
    ) -> Self {
        let fov = 80f32.to_radians();
        let f_width = width as f32;
        let f_height = height as f32;

        let plane_dist = 1.0 / f32::tan(fov / 2.0);
        let aspect = f_width / f_height;

        let pos = Vec3::new(pos_x, pos_y, pos_z);
        let dir = Vec3::new(angle.cos(), 0.0, angle.sin());

        let plane_v = DEFAULT_PLANE_V / plane_dist;
        let plane_h = Vec3::cross(DEFAULT_PLANE_V, dir) * aspect / plane_dist;

        Self {
            fov,
            plane_dist,
            pos,
            dir,
            plane_h,
            plane_v,
            angle,
            width,
            height,
            aspect,
            y_shearing: 0.0,

            four_width: 4 * width as usize,
            f_height,
            width_recip: f_width.recip(),
            height_recip: f_height.recip(),
            f_half_height: height as f32 * 0.5,

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

    pub fn cast_and_draw(
        &self,
        tile_map: &Map,
        models: &VoxelModelManager,
        data: &mut [u8],
    ) {
        let texture_manager = tile_map.texture_manager();
        let canvas_column_iterator =
            data.chunks_exact_mut(self.height as usize * 4).enumerate();
        canvas_column_iterator.for_each(|(x, column)| {
            // ========================================================
            //           | CALCULATE RAY DISTANCES AND INFO |
            // ========================================================
            // X-coordinate on the horizontal camera plane (range [-1.0, 1.0])
            let plane_x = 2.0 * (x as f32 * self.width_recip) - 1.0;
            // Ray direction for current pixel column
            let ray_dir = self.dir + self.plane_h * plane_x;
            // Length of ray from one x/z side to next x/z side on the tile_map
            let delta_dist_x = 1.0 / ray_dir.x.abs();
            let delta_dist_z = 1.0 / ray_dir.z.abs();
            // Distance to nearest x side
            let mut side_dist_x = delta_dist_x
                * if ray_dir.x < 0.0 {
                    self.pos.x.fract()
                } else {
                    1.0 - self.pos.x.fract()
                };
            // Distance to nearest z side
            let mut side_dist_z = delta_dist_z
                * if ray_dir.z < 0.0 {
                    self.pos.z.fract()
                } else {
                    1.0 - self.pos.z.fract()
                };
            // Coordinates of the map tile the raycaster is in
            let mut next_tile_x = self.pos.x.trunc();
            let mut next_tile_z = self.pos.z.trunc();
            let (step_x, step_z) = (ray_dir.x.signum(), ray_dir.z.signum());

            // ====================================================
            //    | LOOP OVER THE RAY PATH AND DRAW HORIZONTAL |
            //    | PLATFORMS AND VERTICAL PLATFORMS (WALLS)   |
            // ====================================================
            let mut previous_perp_wall_dist = 0.0;
            let mut bottom_draw_bound = 0usize;
            let mut top_draw_bound = self.height as usize;
            loop {
                let current_tile_x = next_tile_x;
                let current_tile_z = next_tile_z;
                // DDA loop
                let (side, perp_wall_dist, wall_offset) =
                    if side_dist_x < side_dist_z {
                        let dist_to_wall = side_dist_x.max(0.0);
                        let wall_offset = self.pos.z + dist_to_wall * ray_dir.z;
                        next_tile_x += step_x;
                        side_dist_x += delta_dist_x;
                        (
                            Side::Vertical,
                            dist_to_wall,
                            wall_offset - wall_offset.floor(),
                        )
                    } else {
                        let dist_to_wall = side_dist_z.max(0.0);
                        let wall_offset = self.pos.x + dist_to_wall * ray_dir.x;
                        next_tile_z += step_z;
                        side_dist_z += delta_dist_z;
                        (
                            Side::Horizontal,
                            dist_to_wall,
                            wall_offset - wall_offset.floor(),
                        )
                    };

                // ====================================================
                //  | DRAW TOP AND BOTTOM PLATFORMS OF CURRENT TILE |
                // ====================================================

                // Tile which the ray just traveled over before hitting a wall.
                let current_tile = match tile_map
                    .get_tile(current_tile_x as i32, current_tile_z as i32)
                {
                    Some(t) => *t,
                    None => {
                        break;
                    }
                };
                let next_tile = match tile_map
                    .get_tile(next_tile_x as i32, next_tile_z as i32)
                {
                    Some(t) => *t,
                    None => {
                        break;
                    }
                };
                let mut params = DrawParams {
                    closer_wall_dist: previous_perp_wall_dist,
                    further_wall_dist: perp_wall_dist,
                    bottom_draw_bound,
                    top_draw_bound,
                    draw_x: x as u32,
                    current_tile,
                    next_tile,
                    tile_x: current_tile_x,
                    tile_z: current_tile_z,
                    ray_dir,
                    side,
                    wall_offset,
                    delta_dist_x,
                    delta_dist_z,
                };

                // Drawing top and bottom platforms
                let drawn_to =
                    self.draw_bottom_platform(params, texture_manager, column);
                bottom_draw_bound = drawn_to;
                params.bottom_draw_bound = bottom_draw_bound;

                let drawn_from =
                    self.draw_top_platform(params, texture_manager, column);
                top_draw_bound = drawn_from;
                params.top_draw_bound = top_draw_bound;

                // Drawing top and bottom walls
                let drawn_to =
                    self.draw_bottom_wall(params, texture_manager, column);
                bottom_draw_bound = drawn_to;

                let drawn_from =
                    self.draw_top_wall(params, texture_manager, column);
                top_draw_bound = drawn_from;

                previous_perp_wall_dist = perp_wall_dist;
            }
        });
    }

    pub fn update(&mut self) {
        // Change FOV and vertical FOV
        self.fov = (self.fov
            + (self.increase_fov - self.decrease_fov) * ONE_DEGREE_RAD)
            .clamp(ONE_DEGREE_RAD, MAX_FOV_RAD);
        self.plane_dist = 1.0 / f32::tan(self.fov * 0.5);

        // Change y_shearing (look up/down)
        self.y_shearing = (self.y_shearing
            + (self.decrease_y_shearing - self.increase_y_shearing) * 2.5)
            .clamp(-self.f_height, self.f_height);

        // Update rotation and direction
        self.angle = normalize_rad(
            self.angle + (self.turn_left - self.turn_right) * ROTATION_SPEED,
        );
        self.dir = Vec3::new(self.angle.cos(), 0.0, self.angle.sin());

        // Rotate raycaster (camera) planes
        self.plane_v = DEFAULT_PLANE_V / self.plane_dist;
        self.plane_h = Vec3::cross(DEFAULT_PLANE_V, self.dir) * self.aspect
            / self.plane_dist;

        // Update position
        self.pos.x +=
            self.dir.x * (self.forward - self.backward) * MOVEMENT_SPEED;
        self.pos.z +=
            self.dir.z * (self.forward - self.backward) * MOVEMENT_SPEED;
        self.pos += self.plane_h.normalize()
            * (self.strafe_right - self.strafe_left)
            * MOVEMENT_SPEED;
        self.pos.y = (self.pos.y
            + (self.fly_up - self.fly_down) * FLY_UP_DOWN_SPEED)
            .clamp(MIN_Y, MAX_Y);
    }

    pub fn process_mouse_input(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.y_shearing += delta.1 as f32 * Y_SHEARING_SENSITIVITY;

                self.angle -=
                    delta.0 as f32 * ONE_DEGREE_RAD * MOUSE_ROTATION_SPEED;
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

// TODO convert to unsafe for speed
#[inline(always)]
fn blend(background: &[u8], foreground: &[u8]) -> [u8; 4] {
    let alpha = foreground[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    [
        ((foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha)
            as u8),
        ((foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha)
            as u8),
        ((foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha)
            as u8),
        (255.0 * alpha + background[3] as f32 * inv_alpha) as u8,
    ]
}

#[inline]
fn normalize_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}
