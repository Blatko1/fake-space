use std::f32::consts::PI;

use glam::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    canvas::Pixel,
    map::{Map, Tile},
    textures::BRICK,
};
// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;

const RED: Pixel = Pixel {
    r: 200,
    g: 10,
    b: 10,
    a: 255,
};

const GRAY: Pixel = Pixel {
    r: 100,
    g: 100,
    b: 100,
    a: 255,
};

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

    hits: Vec<RayHit>,

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

    pub fn render(&self, data: &mut [Pixel]) {
        let width = self.dimensions.0;
        let height = self.dimensions.1;
        for hit in self.hits.iter() {
            let line_pixel_height = (height as f32 / hit.wall_dist) as i32;
            let half_h = height as i32 / 2;
            let half_l = line_pixel_height / 2;

            let y0 = i32::max(half_h - half_l, 0) as u32;
            let y1 = u32::min((half_h + half_l) as u32, height - 1);

            let mut tex_x = (hit.wall_x * 16.0) as u32;

            match hit.side {
                Side::Horizontal if hit.ray_dir.x > 0.0 => {
                    tex_x = 16 - tex_x - 1
                }

                Side::Vertical if hit.ray_dir.y < 0.0 => tex_x = 16 - tex_x - 1,
                _ => (),
            }
            assert!(tex_x < 16);
            verline(hit.screen_x, 0, y0, data, GRAY, width, height);
            verline(hit.screen_x, y1, height - 1, data, RED, width, height);
            let tex_step_y = 16.0 / line_pixel_height as f32;
            let mut tex_y = (y0 as f32 + line_pixel_height as f32 / 2.0
                - height as f32 / 2.0) as f32
                * tex_step_y;
            assert!(tex_y >= 0.0);
            for y in y0..y1 {
                let y_pos = tex_y.round() as u32 & (16 - 1);
                let color = Pixel {
                    r: BRICK[((16 - y_pos - 1) * 64 + tex_x * 4) as usize],
                    g: BRICK[((16 - y_pos - 1) * 64 + tex_x * 4 + 1) as usize],
                    b: BRICK[((16 - y_pos - 1) * 64 + tex_x * 4 + 2) as usize],
                    a: BRICK[((16 - y_pos - 1) * 64 + tex_x * 4 + 3) as usize],
                };
                let color = match hit.side {
                    Side::Vertical => color,
                    Side::Horizontal => Pixel {
                        r: color.r - 15,
                        g: color.g - 15,
                        b: color.b - 15,
                        a: color.a,
                    },
                };
                data[(height as usize - 1 - y as usize) * width as usize
                    + (width - hit.screen_x - 1) as usize] = color;
                tex_y += tex_step_y;
                //assert!(tex_y <= 16.0);
            }
        }
    }

    /// Casts rays from the current position and angle on the provided map.
    /// Stores all [`RayHit`]s in the internal array.
    pub fn cast_rays(&mut self, tile_map: &Map) {
        let width = self.dimensions.0 as f32;
        self.hits.clear();
        // For each pixel in the screen
        for x in 0..width as u32 {
            // X-coordinate on the camera plane (range [-1.0, 1.0])
            let plane_x = 2.0 * (x as f32 / width) - 1.0;
            // Ray direction for current pixel.
            let ray_dir = self.dir + self.plane * plane_x;
            // Length of ray from one x/y side to next x/y
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
            let mut map = self.pos.floor();
            let (step_x, step_y) = (ray_dir.x.signum(), ray_dir.y.signum());

            // DDA loop
            // Iterates over all hit sides until it hits a wall
            loop {
                // Distance to the first hit wall's x/y side if exists
                let side = if side_dist_x < side_dist_y {
                    map.x += step_x;
                    side_dist_x += delta_dist.x;
                    Side::Horizontal
                } else {
                    map.y += step_y;
                    side_dist_y += delta_dist.y;
                    Side::Vertical
                };
                let tile = tile_map.get_value(map.x as usize, map.y as usize);
                if tile != Tile::Empty {
                    let (perp_wall_dist, wall_x) = match side {
                        Side::Horizontal => {
                            let dist = side_dist_x - delta_dist.x;
                            (dist, self.pos.y + dist * ray_dir.y)
                        }
                        Side::Vertical => {
                            let dist = side_dist_y - delta_dist.y;
                            (dist, self.pos.x + dist * ray_dir.x)
                        }
                    };
                    let wall_x = wall_x - wall_x.floor();
                    let wall_dist = perp_wall_dist.max(0.0);
                    self.hits.push(RayHit {
                        screen_x: x,
                        wall_dist,
                        tile,
                        side,
                        wall_x,
                        ray_dir,
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

fn verline(
    x: u32,
    start: u32,
    end: u32,
    data: &mut [Pixel],
    color: Pixel,
    width: u32,
    height: u32,
) {
    for i in start..end {
        data[(height as usize - 1 - i as usize) * width as usize
            + (width - x - 1) as usize] = color
    }
}
// TODO invert the image instead
#[derive(Debug)]
pub struct RayHit {
    /// X-coordinate of a pixel column out of which the ray was casted.
    screen_x: u32,
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
    /// Direction of the ray which hit the tile (wall).
    ray_dir: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

#[inline]
fn norm_rad(angle: f32) -> f32 {
    if angle > 2.0 * PI {
        angle - 2.0 * PI
    } else if angle < 0.0 {
        angle + 2.0 * PI
    } else {
        angle
    }
}
