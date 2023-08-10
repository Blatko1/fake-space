use std::f32::consts::{TAU, PI};

use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    map::{Map, Tile, TransparentTexture, WallTexture},
    textures::{
        BLUE_BRICK, BLUE_BRICK_HEIGHT, BLUE_BRICK_WIDTH, FENCE, FENCE_HEIGHT,
        FENCE_WIDTH, LIGHT_PLANK, LIGHT_PLANK_HEIGHT, LIGHT_PLANK_WIDTH,
    },
};
// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;

const RED: [u8; 4] = [200, 10, 10, 255];

const GRAY: [u8; 4] = [100, 100, 100, 255];

const PURPLE: [u8; 4] = [200, 0, 220, 255];

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

        let index = (height / 2 * width * 4) as usize;
        data[0..index]
            .chunks_exact_mut(4)
            .for_each(|c| c.copy_from_slice(&RED));
        data[index..]
            .chunks_exact_mut(4)
            .for_each(|c| c.copy_from_slice(&GRAY));

        let four_width = 4 * width as usize;
        let half_h_i = height as i32 / 2;
        let half_h_f = height as f32 * 0.5;
        let mut color = [0, 0, 0, 0];
        for ray in self.hits.iter() {
            let hit = ray.hit;
            let draw_x_offset = 4 * (width - ray.screen_x - 1) as usize;
            
            // Draw the void (non-tile; out of bounds)
            if let Tile::Void = hit.tile {
                for y in 0..height - 1 {
                    let index = (height as usize - 1 - y as usize) * four_width
                        + draw_x_offset;
                    data[index..index + 4].copy_from_slice(&PURPLE);
                }
                continue;
            }

            // Draw the hit impassable wall tile:
            if let Tile::Wall(tex) = hit.tile {
                let (texture, tex_width, tex_height) = match tex {

                        WallTexture::BlueBrick => {
                            (BLUE_BRICK, BLUE_BRICK_WIDTH, BLUE_BRICK_HEIGHT)
                        }
                        WallTexture::LightPlank => {
                            (LIGHT_PLANK, LIGHT_PLANK_WIDTH, LIGHT_PLANK_HEIGHT)
                        }
                };

                let line_pixel_height = (height as f32 / hit.wall_dist) as i32;
                let half_l = line_pixel_height / 2;

                let begin = (half_h_i - half_l).max(0) as u32;
                let end = ((half_h_i + half_l) as u32).min(height - 1);

                let tex_height_minus_one = tex_height as f32 - 1.0;
                let tex_x = match hit.side {
                    Side::Vertical if ray.dir.x > 0.0 => {
                        tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
                    }

                    Side::Horizontal if ray.dir.y < 0.0 => {
                        tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
                    }
                    _ => (hit.wall_x * tex_width as f32) as u32,
                };
                let four_tex_x = tex_x * 4;
                //assert!(tex_x < 16);
                let tex_y_step = 16.0 / line_pixel_height as f32;
                let mut tex_y = (begin as f32 + line_pixel_height as f32 * 0.5
                    - half_h_f)
                    * tex_y_step;
                // TODO fix texture mapping.
                //assert!(tex_y >= 0.0);
                for y in begin..end {
                    //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
                    let y_pos = tex_y.min(tex_height_minus_one).round() as u32;
                    let i = ((tex_height - y_pos - 1) * tex_width * 4
                        + four_tex_x) as usize;
                    color.copy_from_slice(&texture[i..i + 4]);
                    match hit.side {
                        Side::Vertical => (),
                        Side::Horizontal => {
                            color[0] = color[0] - 15;
                            color[1] = color[1] - 15;
                            color[2] = color[2] - 15;
                            color[3] = color[3] - 15
                        }
                    };
                    let index = (height as usize - 1 - y as usize) * four_width
                        + draw_x_offset;
                    data[index..index + 4].copy_from_slice(&color);
                    tex_y += tex_y_step;
                    //assert!(tex_y <= 16.0);
                }
            }

            // Draw the hit tile with transparency:
            if let Some(hit) = ray.through_hit {
                let (texture, tex_width, tex_height) = match hit.tile {
                    Tile::Transparent(tex) => match tex {
                        TransparentTexture::Fence => {
                            //(FENCE, FENCE_WIDTH, FENCE_HEIGHT)
                            (BLUE_BRICK, BLUE_BRICK_WIDTH, BLUE_BRICK_HEIGHT)
                        }
                    },
                    _ => unreachable!(),
                };
                //let wall_x = (hit.wall_x + 0.45/f32::atan2(ray.dir.y, ray.dir.x).tan()).clamp(0.0, 1.0 - f32::EPSILON);
                //let offset_dist = 0.45 / f32::atan2(ray.dir.y, ray.dir.x).sin();
                let line_pixel_height = (height as f32 / (hit.wall_dist)) as i32;
                let half_l = line_pixel_height / 2;

                let begin = (half_h_i - half_l).max(0) as u32;
                let end = ((half_h_i + half_l) as u32).min(height - 1);

                let tex_height_minus_one = tex_height as f32 - 1.0;
                let tex_x = match hit.side {
                    Side::Vertical if ray.dir.x > 0.0 => {
                        tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
                    }

                    Side::Horizontal if ray.dir.y < 0.0 => {
                        tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
                    }
                    _ => (hit.wall_x * tex_width as f32) as u32,
                };
                let four_tex_x = tex_x * 4;
                //assert!(tex_x < 16);
                let tex_y_step = 16.0 / line_pixel_height as f32;
                let mut tex_y = (begin as f32 + line_pixel_height as f32 * 0.5
                    - half_h_f)
                    * tex_y_step;
                // TODO fix texture mapping.
                //assert!(tex_y >= 0.0);
                for y in begin..end {
                    //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
                    let y_pos = tex_y.min(tex_height_minus_one).round() as u32;
                    let i = ((tex_height - y_pos - 1) * tex_width * 4
                        + four_tex_x) as usize;
                    color.copy_from_slice(&texture[i..i + 4]);
                    match hit.side {
                        Side::Vertical => (),
                        Side::Horizontal => {
                            color[0] = color[0] - 15;
                            color[1] = color[1] - 15;
                            color[2] = color[2] - 15;
                            color[3] = color[3] - 15
                        }
                    };
                    if color[3] == 0 {
                        continue;
                    }
                    let index = (height as usize - 1 - y as usize) * four_width
                        + draw_x_offset;
                    data[index..index + 4].copy_from_slice(&color);
                    tex_y += tex_y_step;
                    //assert!(tex_y <= 16.0);
                }
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
                    Side::Vertical
                } else {
                    map_y += step_y;
                    side_dist_y += delta_dist.y;
                    Side::Horizontal
                };
                let tile = tile_map.get_value(map_x, map_y);
                if tile != Tile::Empty {
                    //TODO temp:
                    let mut off = 0.0;
                    if let Tile::Transparent(_) = tile {
                        let angle = ray_dir.y.atan2(ray_dir.x);
                        off = match side {
                            Side::Vertical => 0.45 / angle.cos(),
                            Side::Horizontal => 0.45 / angle.sin(),
                        }.abs()
                    };
                    let (perp_wall_dist, wall_x) = match side {
                        Side::Vertical => {
                            let dist = side_dist_x - delta_dist.x;
                            let wall_x = self.pos.y + dist * ray_dir.y;
                            //let wall_x_in = self.pos.y + (dist + off) * ray_dir.y;
                            //if wall_x.ceil() < wall_x_in || wall_x.floor() > wall_x_in {
                            //    continue;
                            //}
                            if off != 0.0 {
                                continue;
                            }
                            ((dist+off).max(0.0), wall_x)
                        }
                        Side::Horizontal => {
                            let dist = side_dist_y - delta_dist.y;
                            let wall_x = self.pos.x + dist * ray_dir.x;
                            let wall_x_in = self.pos.x + (dist + off) * ray_dir.x;
                            if wall_x.ceil() < wall_x_in || wall_x.floor() > wall_x_in {
                                continue;
                            }
                            ((dist+off).max(0.0), wall_x_in)
                        }
                    };
                    let wall_x = wall_x - wall_x.floor();
                    let hit = RayHit {
                        wall_dist: perp_wall_dist,
                        tile,
                        side,
                        wall_x,
                    };
                    if let Tile::Transparent(_) = tile {
                        if through_hit.is_none() {
                            through_hit = Some(hit);
                            continue;
                        }
                    }
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
    wall_x: f32,
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
