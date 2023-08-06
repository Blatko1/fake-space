use std::f32::consts::PI;

use glam::Vec2;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    canvas::Pixel,
    state::{MAP, MAP_HEIGHT, MAP_WIDTH},
    HEIGHT, WIDTH,
};

const MOVEMENT_SPEED: f32 = 0.05;

#[derive(Debug)]
pub struct Raycaster {
    /// FOV in radians.
    fov: f32,

    /// Calculated from FOV, used for the plane Vec2.
    plane_mag: f32,

    /// Position of the raycaster. Whole number represents the tile and
    /// fraction represents the offset in the tile.
    pub pos: Vec2,

    /// Direction of the raycaster.
    dir: Vec2,

    /// Raycaster (camera) 2D plane.
    plane: Vec2,

    /// Angle in radians.
    angle: f32,
}

impl Raycaster {
    pub fn new(x: f32, y: f32, angle: f32, fov: f32) -> Self {
        let plane_mag = f32::tan(fov / 2.0);

        // Raycaster position and main direction (is always normalized)
        let pos = Vec2::new(x, y);
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
        }
    }

    pub fn cast_rays(
        &self,
        screen_w: u32,
        screen_h: u32,
        data: &mut [Pixel],
    ) -> Vec<(u32, u32)> {
        let mut result = Vec::with_capacity(screen_w as usize);
        // For each pixel in the screen
        for x in 0..screen_w {
            // X-coordinate on the camera plane (range [-1.0, 1.0])
            let plane_x = 2.0 * (x as f32 / screen_w as f32) - 1.0;
            // Ray direction for current pixel.
            let ray_dir = self.dir + self.plane * plane_x;

            // Coordinates of the map tile the raycaster is in
            let mut map_x = self.pos.x.floor();
            let mut map_y = self.pos.y.floor();

            // Length of ray from one x/y side to next x/y
            let delta_dist = Vec2::new(1.0 / ray_dir.x, 1.0 / ray_dir.y).abs();

            // Distance to nearest x side
            let mut side_dist_x = delta_dist.x;
            if ray_dir.x < 0.0 {
                side_dist_x *= self.pos.x - map_x
            } else {
                side_dist_x *= map_x + 1.0 - self.pos.x
            };
            // Distance to nearest x side
            let mut side_dist_y = delta_dist.y;
            if ray_dir.y < 0.0 {
                side_dist_y *= self.pos.y - map_y
            } else {
                side_dist_y *= map_y + 1.0 - self.pos.y
            };
            let (step_x, step_y) = (ray_dir.x.signum(), ray_dir.y.signum());

            let mut side;
            let mut value = 0;
            // DDA loop
            // iterates over all hit sides until it hits a wall
            loop {
                if side_dist_x < side_dist_y {
                    side = Side::Horizontal;
                    map_x += step_x;
                    side_dist_x += delta_dist.x;
                } else {
                    side = Side::Vertical;
                    map_y += step_y;
                    side_dist_y += delta_dist.y;
                };
                if map_x >= MAP_WIDTH as f32 || map_y >= MAP_HEIGHT as f32 {
                    break;
                }
                let val = MAP[map_y as usize][map_x as usize];
                if val != 0 {
                    value = val;
                    break;
                }
            }
            match side {
                Side::Vertical => {
                    let pos_x = (((self.pos.x + ray_dir.x * side_dist_y)
                        / MAP_WIDTH as f32)
                        * WIDTH as f32) as u32;
                    let pos_y = (((MAP_HEIGHT as f32
                        - (self.pos.y + ray_dir.y * side_dist_y))
                        / MAP_HEIGHT as f32)
                        * HEIGHT as f32) as u32;
                    if ((pos_x + pos_y * WIDTH) as usize) < (data.len()) {
                        data[(pos_x + pos_y * WIDTH) as usize] = Pixel {
                            r: 255,
                            g: 100,
                            b: 0,
                            a: 255,
                        }
                    }
                }
                Side::Horizontal => {
                    let pos_x = (((self.pos.x + ray_dir.x * side_dist_x)
                        / MAP_WIDTH as f32)
                        * WIDTH as f32) as u32;
                    let pos_y = (((MAP_HEIGHT as f32
                        - (self.pos.y + ray_dir.y * side_dist_x))
                        / MAP_HEIGHT as f32)
                        * HEIGHT as f32) as u32;
                    if ((pos_x + pos_y * WIDTH) as usize) < (data.len()) {
                        data[(pos_x + pos_y * WIDTH) as usize] = Pixel {
                            r: 255,
                            g: 100,
                            b: 0,
                            a: 255,
                        }
                    }
                }
            }
            let pos_x = (((self.pos.x + ray_dir.x) / MAP_WIDTH as f32)
                * WIDTH as f32) as u32;
            let pos_y = (((MAP_HEIGHT as f32 - (self.pos.y + ray_dir.y))
                / MAP_HEIGHT as f32)
                * HEIGHT as f32) as u32;
            if ((pos_x + pos_y * WIDTH) as usize) < (data.len()) {
                data[(pos_x + pos_y * WIDTH) as usize] = Pixel {
                    r: 255,
                    g: 100,
                    b: 0,
                    a: 255,
                }
            }

            let perp_wall_dist = match side {
                Side::Horizontal => side_dist_x - delta_dist.x,
                Side::Vertical => side_dist_y - delta_dist.y,
            };
            assert!(perp_wall_dist < MAP_WIDTH as f32);
            assert!(perp_wall_dist >= 0.0);
            let line_h =
                (screen_h as f32 / perp_wall_dist.max(f32::EPSILON)) as i32;
            let half_h = screen_h as i32 / 2;
            let half_l = line_h / 2;

            let y0 = i32::max(half_h - half_l, 0) as u32;
            let y1 = u32::min((half_h + half_l) as u32, HEIGHT - 1);
            //println!("y0: {}, y1: {}", y0, y1);
            //result.push((y0, y1));
            //let plane = Hit {
            //    value: 0,
            //    side: Side::Horizontal,
            //    pos: self.pos + ray_dir,
            //};
            //result.push(plane);
            verline(x, 0, y0, data, 20);
            verline(x, y1, HEIGHT - 1, data, 50);
            match side {
                Side::Vertical => verline(x, y0, y1, data, 200),
                Side::Horizontal => verline(x, y0, y1, data, 150),
            }
        }
        result
    }

    pub fn update(&mut self, rotation: f32, movement: f32) {
        // Update rotation and direction
        self.angle = norm_rad(self.angle + rotation * 0.02);
        self.dir = Vec2::from_angle(self.angle);

        // Rotate raycaster (camera) 2D plane
        self.plane = self.dir.perp() * self.plane_mag;

        // Update position
        // (0, 0) is at bottom-left
        self.pos += self.dir * movement * MOVEMENT_SPEED;
    }
}

#[derive(Debug)]
pub struct Hit {
    pub value: u32,
    pub side: Side,
    pub pos: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

fn verline(x: u32, start: u32, end: u32, data: &mut [Pixel], color: u8) {
    for i in start..end {
        data[(HEIGHT as usize - 1 - i as usize) * WIDTH as usize + x as usize] =
            Pixel {
                r: color,
                g: color,
                b: color,
                a: 255,
            }
    }
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
