mod floor_ceiling;
mod transparent;
mod void;
mod wall;

use glam::Vec2;
use std::f32::consts::TAU;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::map::{Map, Tile};
// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;

#[derive(Debug)]
pub struct RayCast {
    /// X-coordinate of a pixel column out of which the ray was casted.
    screen_x: u32,
    /// Direction of the ray which hit the tile (wall).
    dir: Vec2,
    /// Data about the ray's final hit point, ray doesn't continue after the hit.
    hit: RayHit,
    /// Data about the ray's hit point through which the ray passes if
    /// the hit tile is transparent (i.e. window, glass, different shapes).
    /// Since the object has transparency, all four sides should be rendered,
    /// meaning that each ray passes through two sides (adjacent or opposite).
    /// First in array is the first hit tile side and second is the other.
    through_hits: Vec<RayHit>,
    /// Precomputed specific use variable for improving performance.
    draw_x_offset: usize
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
    wall_x: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

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
    /// Width of the output screen/texture.
    width: u32,
    /// Height of the output screen/texture.
    height: u32,

    // Specific use variables with goal to improve performance.
    four_width: usize,
    width_recip: f32,
    int_half_height: i32,
    float_half_height: f32,

    hits: Vec<RayCast>,

    // Variables for controlling and moving the scene.
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
        let f_width = width as f32;

        Self {
            fov,
            plane_mag,
            pos,
            dir,
            plane,
            angle,
            width,
            height,

            four_width: 4 * width as usize,
            width_recip: f_width.recip(),
            int_half_height: height as i32 / 2,
            float_half_height: height as f32 * 0.5,

            hits,

            turn_left: 0.0,
            turn_right: 0.0,
            forward: 0.0,
            backward: 0.0,
        }
    }

    pub fn render(&self, data: &mut [u8]) {
        // TODO don't forget to remove temp '&'
        self.draw_floor_and_ceiling(data);
        //ceiling::fill(self, data);

        for ray in self.hits.iter() {
            let hit = ray.hit;

            // Draw the void (non-tile; out of map bounds):
            if let Tile::Void = hit.tile {
                self.draw_void(ray, data);
            }

            // Draw the hit impassable wall tile:
            if let Tile::Wall(_) = hit.tile {
                self.draw_wall(ray, data);
            }

            //Draw the hit tile with transparency:
            if !ray.through_hits.is_empty() {
                self.draw_transparent(ray, data);
            }
        }
    }

    /// Casts rays from the current position and angle on the provided map.
    /// Stores all [`RayHit`]s in the internal array.
    pub fn cast_rays(&mut self, tile_map: &Map) {
        self.hits.clear();
        // For each pixel column on the screen
        for x in 0..self.width {
            // X-coordinate on the camera plane (range [-1.0, 1.0])
            let plane_x = 2.0 * (x as f32 * self.width_recip) - 1.0;
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

            let mut through_hits = Vec::new();
            // DDA loop
            // Iterates over all hit sides until it hits a non empty tile.
            // If a transparent tile is hit, continue iterating.
            // If another transparent tile was hit, store it as a final hit.
            loop {
                // Distance to the first hit wall's x/y side if exists
                let side = if side_dist_x < side_dist_y {
                    map_x += step_x;
                    side_dist_x += delta_dist.x;
                    Side::Vertical
                } else {
                    map_y += step_y;
                    side_dist_y += delta_dist.y;
                    Side::Horizontal
                };
                // Get value of the hit tile
                let tile = tile_map.get_value(map_x, map_y);
                // If the hit tile is not Tile::Empty (out of bounds != Tile::Empty) store data
                if tile != Tile::Empty {
                    // Calculate perpetual wall distance from the camera and wall_x.
                    // wall_x represents which part of wall was hit from the left border (0.0)
                    // to the right border (0.99999) and everything in between in range <0.0, 1.0>
                    let (perp_wall_dist, wall_x) = match side {
                        Side::Vertical => {
                            let dist = side_dist_x - delta_dist.x;
                            let wall_x = self.pos.y + dist * ray_dir.y;
                            (dist.max(0.0), wall_x - wall_x.floor())
                        }
                        Side::Horizontal => {
                            let dist = side_dist_y - delta_dist.y;
                            let wall_x = self.pos.x + dist * ray_dir.x;
                            (dist.max(0.0), wall_x - wall_x.floor())
                        }
                    };
                    let hit = RayHit {
                        wall_dist: perp_wall_dist,
                        tile,
                        side,
                        wall_x,
                    };
                    // If the hit tile has transparency, also calculate the Hit to the next closest
                    // Vertical or Horizontal side on the ray path
                    if let Tile::Transparent(_) = tile {
                        let (perp_wall_dist, wall_x, side) = if side_dist_x < side_dist_y {
                            let dist = side_dist_x.max(0.0);
                                let wall_x = self.pos.y + dist * ray_dir.y;
                                (dist, wall_x - wall_x.floor(), Side::Vertical)
                            
                        } else {
                            let dist = side_dist_y.max(0.0);
                                let wall_x = self.pos.x + dist * ray_dir.x;
                                (dist, wall_x - wall_x.floor(), Side::Horizontal)
                        };
                        let hit2 = RayHit {
                            wall_dist: perp_wall_dist,
                            tile,
                            side,
                            wall_x,
                        };
                        through_hits.push(hit);
                        through_hits.push(hit2);
                        continue;
                    }
                    self.hits.push(RayCast {
                        screen_x: x,
                        dir: ray_dir,
                        hit,
                        through_hits,
                        draw_x_offset: 4 * (self.width - x - 1) as usize
                    });
                    break;
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
fn norm_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}
