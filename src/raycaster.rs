use std::f32::consts::TAU;

use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    map::{Map, Tile},
    textures::BRICK,
};
// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;

const RED: [u8; 4] = [200, 10, 10, 255];

const GRAY: [u8; 4] = [100, 100, 100, 255];

#[derive(Debug)]
pub struct Raycaster {
    /// FOV in radians.
    fov: f32,
    /// Calculated from FOV, used for the plane Vec2.
    plane_mag: f32,
    /// Position of the raycaster. Whole number represents the tile and
    /// fraction represents the offset in the tile. Each tile has width and
    /// height of `1.0`.
    pos: Vec2,
    /// Direction of the raycaster.
    dir: Vec2,
    /// Raycaster (camera) 2D plane.
    plane: Vec2,
    /// Angle in radians.
    angle: f32,
    /// Width and height of the output screen/texture.
    dimensions: (u32, u32),

    hits: Vec<RayCast>,

    turn_left: f32,
    turn_right: f32,
    forward: f32,
    backward: f32,
}

impl Raycaster {
    pub fn new(
        pos_x: f32,
        pos_y: f32,
        angle: f32,
        fov: f32,
        width: u32,
        height: u32,
    ) -> Self {
        let hits = Vec::with_capacity(width as usize);
        let plane_mag = f32::tan(fov / 2.0);

        // Raycaster position and main direction (is always normalized)
        let pos = Vec2::new(pos_x, pos_y);
        let dir = Vec2::from_angle(angle);
        // Raycaster's (camera's) 2D plane
        let dir_perpendicular = dir.perp();
        let plane = dir_perpendicular * plane_mag;

        Self {
            fov,
            plane_mag,
            pos,
            dir,
            plane,
            angle,
            dimensions: (width, height),
            hits,

            turn_left: 0.0,
            turn_right: 0.0,
            forward: 0.0,
            backward: 0.0,
        }
    }

    pub fn render(&self, data: &mut [u8]) {
        let width = self.dimensions.0;
        let height = self.dimensions.1;
        for ray in self.hits.iter() {
            let hit = ray.hit;
            let line_pixel_height = (height as f32 / hit.wall_dist) as i32;
            let half_h = height as i32 / 2;
            let half_l = line_pixel_height / 2;

            let y0 = (half_h - half_l).max(0) as u32;
            let y1 = ((half_h + half_l) as u32).min(height - 1);

            let mut tex_x = (hit.wall_x * 16.0) as u32;

            match hit.side {
                Side::Horizontal if ray.dir.x > 0.0 => {
                    tex_x = 16 - tex_x - 1
                }

                Side::Vertical if ray.dir.y < 0.0 => tex_x = 16 - tex_x - 1,
                _ => (),
            }
            //assert!(tex_x < 16);
            verline(ray.screen_x, 0, y0, data, GRAY, width, height);
            verline(ray.screen_x, y1, height - 1, data, RED, width, height);
            let tex_step_y = 16.0 / line_pixel_height as f32;
            let mut tex_y = (y0 as f32 + line_pixel_height as f32 * 0.5
                - height as f32 * 0.5) as f32
                * tex_step_y;
            // TODO fix texture mapping.
            //assert!(tex_y >= 0.0);
            for y in y0..y1 {
                //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
                let y_pos = tex_y.min(15.0).round() as u32;
                let i = ((16 - y_pos - 1) * 64 + tex_x * 4) as usize;
                let rgba = &mut BRICK[i..i + 4];
                match hit.side {
                    Side::Vertical => (),
                    Side::Horizontal => {rgba[0] = rgba[0] - 15; rgba[1] = rgba[1] - 15; rgba[2] = rgba[2] - 15; rgba[3] = rgba[3] - 15;},
                };
                let index = (height as usize - 1 - y as usize) * 4 * width as usize
                    + 4 * (width - ray.screen_x - 1) as usize;
                data[index..index+4].copy_from_slice(rgba);
                tex_y += tex_step_y;
                //assert!(tex_y <= 16.0);
            }
        }
    }

    /// Casts rays from the current position and angle on the provided map.
    /// Stores all [`RayHit`]s in the internal array.
    pub fn cast_rays(&mut self, tile_map: &Map) {
        let width = self.dimensions.0 as f32;
        let width_recip = width.recip();
        self.hits.clear();
        // For each pixel column on the screen
        for x in 0..width as u32 {
            // X-coordinate on the camera plane (range [-1.0, 1.0])
            let plane_x = 2.0 * (x as f32 * width_recip) - 1.0;
            // Ray direction for current pixel column
            let ray_dir = self.dir + self.plane * plane_x;
            // Length of ray from one x/y side to next x/y side on the tile_map
            let delta_dist = Vec2::new(1.0 / ray_dir.x, 1.0 / ray_dir.y).abs();

            // Distance to nearest x side
            let mut side_dist_x = delta_dist.x
                * if ray_dir.x < 0.0 {
                    self.pos.x.fract()
                } else {
                    1.0 - self.pos.x.fract()
                };
            // Distance to nearest y side
            let mut side_dist_y = delta_dist.y
                * if ray_dir.y < 0.0 {
                    self.pos.y.fract()
                } else {
                    1.0 - self.pos.y.fract()
                };

            // Coordinates of the map tile the raycaster is in
            let mut map_x = self.pos.x as i32;
            let mut map_y = self.pos.y as i32;
            let (step_x, step_y) =
                (ray_dir.x.signum() as i32, ray_dir.y.signum() as i32);

            let mut through_hit = None;
            // DDA loop
            // Iterates over all hit sides until it hits a non empty tile.
            // If a transparent tile is hit, continue iterating.
            // If another transparent tile was hit, store it as a final hit.
            loop {
                // Distance to the first hit wall's x/y side if exists
                let side = if side_dist_x < side_dist_y {
                    map_x += step_x;
                    side_dist_x += delta_dist.x;
                    Side::Horizontal
                } else {
                    map_y += step_y;
                    side_dist_y += delta_dist.y;
                    Side::Vertical
                };
                let tile = tile_map.get_value(map_x, map_y);
                if tile != Tile::Empty {
                    let (perp_wall_dist, wall_x) = match side {
                        Side::Horizontal => {
                            let dist = side_dist_x - delta_dist.x;
                            (dist.max(0.0), self.pos.y + dist * ray_dir.y)
                        }
                        Side::Vertical => {
                            let dist = side_dist_y - delta_dist.y;
                            (dist.max(0.0), self.pos.x + dist * ray_dir.x)
                        }
                    };
                    let wall_x = wall_x - wall_x.floor();
                    let hit = RayHit {
                        wall_dist: perp_wall_dist,
                        tile,
                        side,
                        wall_x,
                    };
                    if tile == Tile::Transparent && through_hit.is_none() {
                        through_hit = Some(hit);
                    } else {
                        self.hits.push(RayCast {
                            screen_x: x,
                            dir: ray_dir,
                            hit,
                            through_hit,
                        });
                        break;
                    }
                }
            }
        }
    }

    pub fn update(&mut self) {
        // Update rotation and direction
        self.angle =
            norm_rad(self.angle + (self.turn_left - self.turn_right) * 0.035);
        self.dir = Vec2::from_angle(self.angle);

        // Rotate raycaster (camera) 2D plane
        self.plane = self.dir.perp() * self.plane_mag;

        // Update position
        self.pos += self.dir * (self.forward - self.backward) * MOVEMENT_SPEED;
    }

    pub fn process_input(&mut self, keyboard: KeyboardInput) {
        if let Some(key) = keyboard.virtual_keycode {
            let value = match keyboard.state {
                ElementState::Pressed => 1.0,
                ElementState::Released => 0.0,
            };

            match key {
                // Turn left:
                VirtualKeyCode::A => self.turn_left = value,
                // Turn right:
                VirtualKeyCode::D => self.turn_right = value,
                // Move forward:
                VirtualKeyCode::W => self.forward = value,
                // Move backward:
                VirtualKeyCode::S => self.backward = value,
                _ => (),
            }
        }
    }
}

#[inline]
fn verline(
    x: u32,
    start: u32,
    end: u32,
    data: &mut [u8],
    color: [u8; 4],
    width: u32,
    height: u32,
) {
    // TODO invert the image instead
    for i in start..end {
        let index = (height as usize - 1 - i as usize) * 4 * width as usize
        + 4 * (width - x - 1) as usize;
        data[index..index+4].copy_from_slice(&color);
    }
}

#[derive(Debug)]
pub struct RayCast {
    /// X-coordinate of a pixel column out of which the ray was casted.
    screen_x: u32,
    /// Direction of the ray which hit the tile (wall).
    dir: Vec2,
    /// Data about the ray's final hit point, ray doesn't continue.
    hit: RayHit,
    /// Data about the ray's hit point through which the ray passes if
    /// the hit tile is transparent (i.e. window, glass, different shapes).
    through_hit: Option<RayHit>,
}

#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    /// Perpetual distance from the raycaster to the hit point on tile (wall).
    wall_dist: f32,
    /// Data of the hit tile.
    tile: Tile,
    /// Which side of tile was hit.
    side: Side,
    /// Number in range [0.0, 1.0) which represents the x-coordinate of
    /// the hit tile side (wall).
    /// If the ray hit the left portion of the tile side (wall), the
    /// x-coordinate would be somewhere in range [0.0, 0.5].
    wall_x: f32
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

#[inline]
fn norm_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}
