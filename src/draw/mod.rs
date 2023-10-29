mod colors;
mod top_bottom;
mod void;
mod voxel_model;
mod wall;

use glam::Vec3;
use std::f32::consts::{PI, TAU};
use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    map::Map,
    textures::{Texture, TextureManager},
    voxel::{VoxelModelManager, VoxelModelRef},
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

#[derive(Debug)]
pub struct RayHit /*<'a>*/ {
    /// X-coordinate of a pixel column out of which the ray was casted.
    screen_x: u32,
    /// Direction of the ray which hit the tile (wall).
    dir: Vec3,
    /// Perpetual distance from the raycaster to the hit point on tile (wall).
    wall_dist: f32,
    max_height_on_path: usize,
    /// Which side of tile was hit.
    side: Side,
    /// Number in range [0.0, 1.0) which represents the x-coordinate of
    /// the hit tile side (wall).
    /// If the ray hit the left portion of the tile side (wall), the
    /// x-coordinate would be somewhere in range [0.0, 0.5].
    wall_x: f32,
    delta_dist_x: f32,
    delta_dist_z: f32,
}

#[derive(Debug)]
pub struct VoxelModelHit<'a> {
    model: VoxelModelRef<'a>,
    map_pos_x: i32,
    map_pos_z: i32,
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
pub struct Raycaster {
    /// Field of view in degrees.
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
    float_half_height: f32,

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

impl Raycaster {
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
            float_half_height: height as f32 * 0.5,

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

    /// Casts rays from the current position and angle on the provided map.
    /// Stores all [`RayHit`]s in the internal array.
    pub fn cast_rays(
        &mut self,
        tile_map: &Map,
        models: &VoxelModelManager,
        textures: &TextureManager,
        data: &mut [u8],
    ) {
        // For each pixel column on the screen
        data.chunks_exact_mut(self.height as usize * 4)
            .enumerate()
            .for_each(|(x, column)| {
                assert!(x < self.width as usize);
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
                let mut map_x = self.pos.x as i32;
                let mut map_z = self.pos.z as i32;
                let (step_x, step_z) =
                    (ray_dir.x.signum() as i32, ray_dir.z.signum() as i32);

                // DDA loop
                // Iterates over all hit sides until it hits a non empty tile.
                // If a transparent tile is hit, continue iterating.
                // If another transparent tile was hit, store it as a final hit.
                let mut previous_perp_wall_dist = 0.0;
                let mut max_top_height = 0;
                loop {
                    let current_map_x = map_x;
                    let current_map_z = map_z;
                    // Distance to the first hit wall's x/z side if the wall isn't empty
                    let side = if side_dist_x < side_dist_z {
                        map_x += step_x;
                        side_dist_x += delta_dist_x;
                        Side::Vertical
                    } else {
                        map_z += step_z;
                        side_dist_z += delta_dist_z;
                        Side::Horizontal
                    };
                    // Calculate perpetual wall distance from the camera and wall_x.
                    // wall_x represents which part of wall was hit from the left border (0.0)
                    // to the right border (0.99999) and everything in between in range <0.0, 1.0>
                    let (perp_wall_dist, wall_x) = match side {
                        Side::Vertical => {
                            let dist = side_dist_x - delta_dist_x;
                            let wall_x = self.pos.z + dist * ray_dir.z;
                            (dist.max(0.0), wall_x - wall_x.floor())
                        }
                        Side::Horizontal => {
                            let dist = side_dist_z - delta_dist_z;
                            let wall_x = self.pos.x + dist * ray_dir.x;
                            (dist.max(0.0), wall_x - wall_x.floor())
                        }
                    };
                    let tile =
                        match tile_map.get_tile(current_map_x, current_map_z) {
                            Some(t) => t,
                            None => {
                                // draw non moving background
                                break;
                            }
                        };
                    // Draw top part of cube
                    let drawn_to = self.draw_floor(
                        previous_perp_wall_dist,
                        perp_wall_dist,
                        max_top_height,
                        tile.obj_top_height,
                        textures.get(tile.object_top),
                        x as u32,
                        current_map_x as f32,
                        current_map_z as f32,
                        column,
                    );
                    max_top_height = drawn_to.max(max_top_height);
                    let tile = match tile_map.get_tile(map_x, map_z) {
                        Some(t) => t,
                        None => {
                            // draw non moving background
                            break},
                    };
                    let hit = RayHit {
                        screen_x: x as u32,
                        dir: ray_dir,
                        wall_dist: perp_wall_dist,
                        max_height_on_path: max_top_height,
                        side,
                        wall_x,
                        delta_dist_x,
                        delta_dist_z,
                    };
                    let drawn_to = self.draw_wall(hit, textures.get(tile.object), max_top_height, tile.obj_top_height, tile.obj_bottom_height ,column);
                    max_top_height = drawn_to.max(max_top_height);

                    previous_perp_wall_dist = perp_wall_dist;
                }
            });
        /*(0..self.width).for_each(|x| {
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
            let mut map_x = self.pos.x as i32;
            let mut map_z = self.pos.z as i32;
            let (step_x, step_z) =
                (ray_dir.x.signum() as i32, ray_dir.z.signum() as i32);

            // DDA loop
            // Iterates over all hit sides until it hits a non empty tile.
            // If a transparent tile is hit, continue iterating.
            // If another transparent tile was hit, store it as a final hit.
            let mut previous_perp_wall_dist = 0.0;
            let mut max_top_height = 0;
            loop {
                let current_map_x = map_x;
                let current_map_z = map_z;
                // Distance to the first hit wall's x/z side if the wall isn't empty
                let side = if side_dist_x < side_dist_z {
                    map_x += step_x;
                    side_dist_x += delta_dist_x;
                    Side::Vertical
                } else {
                    map_z += step_z;
                    side_dist_z += delta_dist_z;
                    Side::Horizontal
                };
                // Calculate perpetual wall distance from the camera and wall_x.
                // wall_x represents which part of wall was hit from the left border (0.0)
                // to the right border (0.99999) and everything in between in range <0.0, 1.0>
                let (perp_wall_dist, wall_x) = match side {
                    Side::Vertical => {
                        let dist = side_dist_x - delta_dist_x;
                        let wall_x = self.pos.z + dist * ray_dir.z;
                        (dist.max(0.0), wall_x - wall_x.floor())
                    }
                    Side::Horizontal => {
                        let dist = side_dist_z - delta_dist_z;
                        let wall_x = self.pos.x + dist * ray_dir.x;
                        (dist.max(0.0), wall_x - wall_x.floor())
                    }
                };
                let tile = match tile_map.get_tile(current_map_x, current_map_z) {
                    Some(t) => t,
                    None => {
                        // draw non moving background
                        break},
                };
                // Draw top part of cube
                let drawn_to = self.draw_floor(
                    previous_perp_wall_dist,
                    perp_wall_dist,
                    max_top_height,
                    tile.obj_top_height,
                    textures.get(tile.object_top),
                    x,
                    data,
                );
                max_top_height = drawn_to.max(max_top_height);
                let tile = match tile_map.get_tile(map_x, map_z) {
                    Some(t) => t,
                    None => {
                        // draw non moving background
                        break},
                };
                let hit = RayHit {
                    screen_x: x,
                    dir: ray_dir,
                    wall_dist: perp_wall_dist,
                    max_height_on_path: max_top_height,
                    side,
                    wall_x,
                    delta_dist_x,
                    delta_dist_z,
                };
                let drawn_to = self.draw_wall(hit, textures.get(tile.object), max_top_height, tile.obj_top_height, tile.obj_bottom_height ,data);
                max_top_height = drawn_to.max(max_top_height);

                previous_perp_wall_dist = perp_wall_dist;
                //self.draw


                // If the hit tile is not Tile::Empty (out of bounds != Tile::Empty) store data
                /*if tile.object != ObjectType::Empty {
                    let mut hit = RayHit {
                        screen_x: x,
                        dir: ray_dir,
                        wall_dist: perp_wall_dist,
                        last_wall_dist: last_perp_wall_dist,
                        max_height_on_path,
                        side,
                        wall_x,
                        //texture: TextureDataRef::default(),
                        delta_dist_x,
                        delta_dist_z,
                    };
                    match tile.object {
                        // If the hit tile has transparency, also calculate the Hit to the next closest
                        // Vertical or Horizontal side on the ray path and `continue`
                        ObjectType::TransparentWall(tile) => {
                            let (perp_wall_dist_2, wall_x, side) =
                                if side_dist_x < side_dist_z {
                                    let dist = side_dist_x.max(0.0);
                                    let wall_x = self.pos.z + dist * ray_dir.z;
                                    (
                                        dist,
                                        wall_x - wall_x.floor(),
                                        Side::Vertical,
                                    )
                                } else {
                                    let dist = side_dist_z.max(0.0);
                                    let wall_x = self.pos.x + dist * ray_dir.x;
                                    (
                                        dist,
                                        wall_x - wall_x.floor(),
                                        Side::Horizontal,
                                    )
                                };
                            let transparent_tex =
                                textures.get_transparent_wall_tex(tile);
                            let hit_2 = RayHit {
                                screen_x: x,
                                dir: ray_dir,
                                wall_dist: perp_wall_dist_2,
                                last_wall_dist: perp_wall_dist,
                                max_height_on_path,
                                side,
                                wall_x,
                                texture: transparent_tex,
                                delta_dist_x,
                                delta_dist_z,
                            };
                            hit.texture = transparent_tex;
                            // Closer one
                            self.draw_wall(hit, data);
                            // Further one
                            self.draw_wall(hit_2, data);
                            last_perp_wall_dist = perp_wall_dist;
                        }
                        ObjectType::FullWall(tile) => {
                            hit.texture = textures.get_full_wall_tex(tile);
                            self.draw_wall(hit, data);
                            last_perp_wall_dist = perp_wall_dist;
                        }
                        ObjectType::VoxelModel(model) => self.draw_voxel_model(
                            hit,
                            VoxelModelHit {
                                model: models.get_model(model),
                                map_pos_x: map_x,
                                map_pos_z: map_z,
                            },
                            data,
                        ),
                        //ObjectType::Void => {
                        //    self.draw_void(hit, data);
                        //    break;
                        //}
                        ObjectType::Void => {
                            break;
                        }
                        _ => (),
                    }
                } else {
                    self.draw_floor(
                        last_perp_wall_dist,
                        perp_wall_dist,
                        0.41,
                        textures.get_bound_tex(BoundType::MossyStone),
                        x,
                        data,
                    );
                    last_perp_wall_dist = perp_wall_dist;
                }*/
            }
        });
        //self.draw_top_bottom(tile_map, textures, data);*/
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
            .clamp(-self.float_half_height + 1.0, self.float_half_height - 1.0);

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
            .min(5.999999)
            .max(-0.999999);
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
