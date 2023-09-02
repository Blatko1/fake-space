mod floor_ceiling;
mod object;
mod transparent;
mod void;
mod wall;

use glam::{Vec2, Vec3};
use std::f32::consts::TAU;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use crate::{
    map::{Map, Tile, TransparentTile, WallTile},
    object::{ModelManager, Object, ObjectType},
    textures::{
        BLUE_BRICK_TEXTURE, BLUE_GLASS_TEXTURE, FENCE_TEXTURE,
        LIGHT_PLANK_TEXTURE,
    },
};

use self::object::VoxelSide;
// TODO rotation control with mouse and/or keyboard
const MOVEMENT_SPEED: f32 = 0.1;

#[derive(Debug)]
pub struct RayCast {
    /// X-coordinate of a pixel column out of which the ray was casted.
    screen_x: u32,
    /// Direction of the ray which hit the tile (wall).
    dir: Vec3,
    /// Data about the ray's final hit point, ray doesn't continue after the hit.
    hit: RayHit,
    delta_dist_x: f32,
    delta_dist_z: f32,
    /// Data about the ray's hit point through which the ray passes if
    /// the hit tile is transparent (i.e. window, glass, different shapes).
    /// Since the object has transparency, all four sides should be rendered,
    /// meaning that each ray passes through two sides (adjacent or opposite).
    /// First in array is the first hit tile side and second is the other.
    through_hits: Vec<RayHit>,
}

#[derive(Debug)]
pub struct FastRayCast {
    screen_x: u32,
    dir: Vec3,
    hit: FastRayHit,
    delta_dist_x: f32,
    delta_dist_z: f32,
    through_hits: Vec<FastRayHit>,
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
    object: Option<ObjectHit>,
}

#[derive(Debug, Clone, Copy)]
pub struct FastRayHit {
    wall_dist: f32,
    tile: Tile,
    side: Side,
    wall_x: f32,
    map_pos_x: i32,
    map_pos_z: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct ObjectHit {
    obj: ObjectType,
    obj_map_pos_x: i32,
    obj_map_pos_z: i32,
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
    /// FOV in radians.
    fov: f32,
    /// Calculated from FOV, used for the plane Vec2.
    plane_mag: f32,
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

    // Specific use variables with goal to improve performance.
    four_width: usize,
    width_recip: f32,
    height_recip: f32,
    int_half_height: i32,
    float_half_height: f32,

    hits: Vec<RayCast>,

    // Variables for controlling and moving the scene.
    turn_left: f32,
    turn_right: f32,
    strafe_left: f32,
    strafe_right: f32,
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
        fov: f32,
        width: u32,
        height: u32,
    ) -> Self {
        let hits = Vec::with_capacity(width as usize);
        let plane_mag = f32::tan(fov / 2.0);
        let aspect = width as f32 / height as f32;

        // Raycaster position and main direction (is always normalized)
        let pos = Vec3::new(pos_x, pos_y, pos_z);
        let dir_vec2 = Vec2::from_angle(angle);
        let dir = Vec3::new(dir_vec2.x, 0.0, dir_vec2.y);
        let plane_v = Vec3::new(0.0, plane_mag / aspect, 0.0);
        let dir_perp = Vec3::cross(plane_v, dir).normalize();
        let plane_h = dir_perp * plane_mag;

        let f_width = width as f32;
        let f_height = height as f32;

        Self {
            fov,
            plane_mag,
            pos,
            dir,
            plane_h,
            plane_v,
            angle,
            width,
            height,
            aspect,

            four_width: 4 * width as usize,
            width_recip: f_width.recip(),
            height_recip: f_height.recip(),
            int_half_height: height as i32 / 2,
            float_half_height: height as f32 * 0.5,

            hits,

            turn_left: 0.0,
            turn_right: 0.0,
            strafe_left: 0.0,
            strafe_right: 0.0,
            forward: 0.0,
            backward: 0.0,
        }
    }

    pub fn render(&self, models: &ModelManager, data: &mut [u8]) {
        self.draw_floor_and_ceiling(data);

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

            //Draw the hit tile with transparency (walls with holes, objects, transparent textures):
            for through in ray.through_hits.iter().rev() {
                match through.tile {
                    Tile::Transparent(TransparentTile::Object(_)) => {
                        self.draw_object(ray, through, models, data)
                    }
                    _ => self.draw_transparent(ray, through, models, data),
                }
            }
        }
    }

    /// Casts rays from the current position and angle on the provided map.
    /// Stores all [`RayHit`]s in the internal array.
    pub fn cast_rays(&mut self, tile_map: &Map) {
        self.hits.clear();
        // For each pixel column on the screen
        for x in 0..self.width {
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

            let mut through_hits = Vec::new();
            // DDA loop
            // Iterates over all hit sides until it hits a non empty tile.
            // If a transparent tile is hit, continue iterating.
            // If another transparent tile was hit, store it as a final hit.
            loop {
                // Distance to the first hit wall's x/z side if exists
                let side = if side_dist_x < side_dist_z {
                    map_x += step_x;
                    side_dist_x += delta_dist_x;
                    Side::Vertical
                } else {
                    map_z += step_z;
                    side_dist_z += delta_dist_z;
                    Side::Horizontal
                };
                // Get value of the hit tile
                let tile = tile_map.get_value(map_x, map_z);
                // If the hit tile is not Tile::Empty (out of bounds != Tile::Empty) store data
                if tile != Tile::Empty {
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
                    let mut hit = RayHit {
                        wall_dist: perp_wall_dist,
                        tile,
                        side,
                        wall_x,
                        object: None,
                    };
                    // If the hit tile has transparency, also calculate the Hit to the next closest
                    // Vertical or Horizontal side on the ray path and `continue`
                    if let Tile::Transparent(_) = tile {
                        if let Tile::Transparent(TransparentTile::Object(
                            obj,
                        )) = tile
                        {
                            hit.object = Some(ObjectHit {
                                obj,
                                obj_map_pos_x: map_x,
                                obj_map_pos_z: map_z,
                            });
                            through_hits.push(hit);
                            continue;
                        }
                        let (perp_wall_dist, wall_x, side) = if side_dist_x
                            < side_dist_z
                        {
                            let dist = side_dist_x.max(0.0);
                            let wall_x = self.pos.z + dist * ray_dir.z;
                            (dist, wall_x - wall_x.floor(), Side::Vertical)
                        } else {
                            let dist = side_dist_z.max(0.0);
                            let wall_x = self.pos.x + dist * ray_dir.x;
                            (dist, wall_x - wall_x.floor(), Side::Horizontal)
                        };
                        let hit2 = RayHit {
                            wall_dist: perp_wall_dist,
                            tile,
                            side,
                            wall_x,
                            object: None,
                        };
                        through_hits.push(hit);
                        through_hits.push(hit2);
                    } else {
                        self.hits.push(RayCast {
                            screen_x: x,
                            dir: ray_dir,
                            hit,
                            delta_dist_x,
                            delta_dist_z,
                            through_hits,
                        });
                        break;
                    }
                }
            }
        }
    }

    pub fn fast_cast_rays(
        &mut self,
        tile_map: &Map,
        data: &mut [u8],
        models: &ModelManager,
    ) {
        for x in 0..self.width {
            let plane_x = 2.0 * (x as f32 * self.width_recip) - 1.0;
            let ray_dir = self.dir + self.plane_h * plane_x;
            let delta_dist_x = 1.0 / ray_dir.x.abs();
            let delta_dist_z = 1.0 / ray_dir.z.abs();

            let mut side_dist_x = delta_dist_x
                * if ray_dir.x < 0.0 {
                    self.pos.x.fract()
                } else {
                    1.0 - self.pos.x.fract()
                };
            let mut side_dist_z = delta_dist_z
                * if ray_dir.z < 0.0 {
                    self.pos.z.fract()
                } else {
                    1.0 - self.pos.z.fract()
                };

            let mut map_x = self.pos.x as i32;
            let mut map_z = self.pos.z as i32;
            let (step_x, step_z) =
                (ray_dir.x.signum() as i32, ray_dir.z.signum() as i32);

            let mut through_hits = Vec::new();
            let cast = loop {
                let side = if side_dist_x < side_dist_z {
                    map_x += step_x;
                    side_dist_x += delta_dist_x;
                    Side::Vertical
                } else {
                    map_z += step_z;
                    side_dist_z += delta_dist_z;
                    Side::Horizontal
                };
                let tile = tile_map.get_value(map_x, map_z);
                if tile != Tile::Empty {
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
                    let hit = FastRayHit {
                        wall_dist: perp_wall_dist,
                        tile,
                        side,
                        wall_x,
                        map_pos_x: map_x,
                        map_pos_z: map_z,
                    };
                    if let Tile::Transparent(transparent_tile) = tile {
                        if let TransparentTile::Object(_) = transparent_tile {
                            through_hits.push(hit);
                            continue;
                        }
                        let (perp_wall_dist, wall_x, side) = if side_dist_x
                            < side_dist_z
                        {
                            let dist = side_dist_x.max(0.0);
                            let wall_x = self.pos.z + dist * ray_dir.z;
                            (dist, wall_x - wall_x.floor(), Side::Vertical)
                        } else {
                            let dist = side_dist_z.max(0.0);
                            let wall_x = self.pos.x + dist * ray_dir.x;
                            (dist, wall_x - wall_x.floor(), Side::Horizontal)
                        };
                        let hit2 = FastRayHit {
                            wall_dist: perp_wall_dist,
                            tile,
                            side,
                            wall_x,
                            map_pos_x: map_x,
                            map_pos_z: map_z,
                        };
                        through_hits.push(hit);
                        through_hits.push(hit2);
                        continue;
                    }
                    break FastRayCast {
                        screen_x: x,
                        dir: ray_dir,
                        hit,
                        delta_dist_x,
                        delta_dist_z,
                        through_hits,
                    };
                }
            };
            let mut row_data = self.create_row_data(&cast, models);
            let mut color = [0; 4];
            'row: for y in 0..self.height {
                let mut transparent_tex_tile =
                    row_data.transparent_texture_tile.iter_mut();
                let index = (self.height as usize - 1 - y as usize)
                    * self.four_width
                    + x as usize * 4;
                'depth: for (i, through) in
                    cast.through_hits.iter().enumerate()
                {
                    if let Tile::Transparent(TransparentTile::Object(obj)) =
                        through.tile
                    {
                    } else {
                        let hit = transparent_tex_tile.next().unwrap();
                        if y < hit.y_begin || y >= hit.y_end {
                            continue 'depth;
                        }
                        let tex_y_pos =
                            hit.tex_y_start
                                .min(hit.tex_height_minus_one)
                                .round() as u32;

                        let tex_index = ((hit.tex_height - tex_y_pos - 1)
                            * hit.tex_width
                            * 4
                            + hit.four_tex_x)
                            as usize;
                        color.copy_from_slice(
                            &hit.texture[tex_index..tex_index + 4],
                        );
                        match through.side {
                            Side::Vertical => (),
                            Side::Horizontal => {
                                color[0] = color[0].saturating_sub(15);
                                color[1] = color[1].saturating_sub(15);
                                color[2] = color[2].saturating_sub(15);
                            }
                        };
                        hit.tex_y_start += hit.tex_y_step;
                        let a = color[3];
                        if a == 0 {
                            continue 'depth;
                        }
                        let rgba = &mut data[index..index + 4];
                        if a == 255 {
                            rgba.copy_from_slice(&color);
                            continue 'row;
                        }
                        if i == 0 {
                            rgba.copy_from_slice(&color)
                        } else {
                            rgba.copy_from_slice(&blend(rgba, color))
                        }
                    }
                }
                match cast.hit.tile {
                    Tile::Void => data[index..index + 4].fill(200),
                    Tile::Wall(_) => {
                        let wall_data =
                            row_data.wall_row_data.as_mut().unwrap();
                        if y < wall_data.y_begin || y >= wall_data.y_end {
                            continue 'row;
                        }
                        let tex_y_pos = wall_data
                            .tex_y_start
                            .min(wall_data.tex_height_minus_one)
                            .round()
                            as u32;
                        let tex_index = ((wall_data.tex_height - tex_y_pos - 1)
                            * wall_data.tex_width
                            * 4
                            + wall_data.four_tex_x)
                            as usize;
                        color.copy_from_slice(
                            &wall_data.texture[tex_index..tex_index + 4],
                        );
                        match cast.hit.side {
                            Side::Vertical => (),
                            Side::Horizontal => {
                                color[0] = color[0].saturating_sub(15);
                                color[1] = color[1].saturating_sub(15);
                                color[2] = color[2].saturating_sub(15);
                            }
                        };
                        wall_data.tex_y_start += wall_data.tex_y_step;
                        data[index..index + 4].copy_from_slice(&color);
                        continue;
                    }

                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn update(&mut self) {
        // Update rotation and direction
        self.angle =
            norm_rad(self.angle + (self.turn_left - self.turn_right) * 0.035);
        let dir_vec2 = Vec2::from_angle(self.angle);
        self.dir = Vec3::new(dir_vec2.x, 0.0, dir_vec2.y);
        let dir_perp = Vec3::cross(self.plane_v, self.dir).normalize();

        // Rotate raycaster (camera) horizontal plane
        self.plane_h = dir_perp * self.plane_mag;

        // Update position
        self.pos += self.dir * (self.forward - self.backward) * MOVEMENT_SPEED;
        self.pos += self.plane_h.normalize()
            * (self.strafe_right - self.strafe_left)
            * MOVEMENT_SPEED;
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
                // Strafe left:
                VirtualKeyCode::Q => self.strafe_left = value,
                // Strafe right:
                VirtualKeyCode::E => self.strafe_right = value,
                _ => (),
            }
        }
    }

    fn create_row_data<'a>(
        &self,
        ray: &FastRayCast,
        models: &ModelManager,
    ) -> RowData<'a> {
        let objects = ray
            .through_hits
            .iter()
            .filter(|hit| {
                if let Tile::Transparent(TransparentTile::Object(_)) = hit.tile
                {
                    true
                } else {
                    false
                }
            })
            .count();
        let mut object_row_data = Vec::with_capacity(objects);
        let mut transparent_texture_tile =
            Vec::with_capacity(ray.through_hits.len() - objects);
        for through_hit in ray.through_hits.iter().rev() {
            if let Tile::Transparent(TransparentTile::Object(obj)) =
                through_hit.tile
            {
                let object_hit = through_hit;
                let object = obj.get_object(models);
                let dimension = object.dimension() as f32;
                let dimension_i = dimension as i32;
                let ray_origin = self.pos * dimension;
                let obj_x_pos = (object_hit.map_pos_x as f32) * dimension;
                let obj_z_pos = (object_hit.map_pos_z as f32) * dimension;
                // North is in front (positive Z)
                let (top_left_point, top_side, voxel_side) =
                    match object_hit.side {
                        Side::Vertical => {
                            // is east side hit
                            if ray.dir.x > 0.0 {
                                let top_left = Vec3::new(
                                    obj_x_pos,
                                    dimension,
                                    obj_z_pos + dimension,
                                );
                                (
                                    top_left,
                                    Vec3::new(0.0, 0.0, -dimension),
                                    VoxelSide::Left,
                                )
                            }
                            // is west side hit
                            else {
                                let top_left = Vec3::new(
                                    obj_x_pos + dimension,
                                    dimension,
                                    obj_z_pos,
                                );
                                (
                                    top_left,
                                    Vec3::new(0.0, 0.0, dimension),
                                    VoxelSide::Right,
                                )
                            }
                        }
                        Side::Horizontal => {
                            // is north side hit
                            if ray.dir.z > 0.0 {
                                let top_left =
                                    Vec3::new(obj_x_pos, dimension, obj_z_pos);
                                (
                                    top_left,
                                    Vec3::new(dimension, 0.0, 0.0),
                                    VoxelSide::Front,
                                )
                            }
                            // is south side hit
                            else {
                                let top_left = Vec3::new(
                                    obj_x_pos + dimension,
                                    dimension,
                                    obj_z_pos + dimension,
                                );
                                (
                                    top_left,
                                    Vec3::new(-dimension, 0.0, 0.0),
                                    VoxelSide::Back,
                                )
                            }
                        }
                    };
                let left_side = Vec3::new(0.0, -dimension, 0.0);
                let rectangle_normal = top_side.cross(left_side);
                let data = ObjectRowData {
                    object,
                    dimension,
                    dimension_i,
                    ray_origin,
                    pos_x: obj_x_pos,
                    pos_z: obj_z_pos,
                    top_left_point,
                    top_side,
                    left_side,
                    rectangle_normal,
                    voxel_side,
                };
                object_row_data.push(data);
            } else {
                let (
                    texture,
                    tex_width,
                    tex_height,
                    tex_bottom_height,
                    tex_top_height,
                ) = match through_hit.tile {
                    Tile::Transparent(tex) => match tex {
                        TransparentTile::Fence => FENCE_TEXTURE,
                        TransparentTile::BlueGlass => BLUE_GLASS_TEXTURE,
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                };

                // TODO better names
                let full_line_pixel_height = (self.height as f32
                    / (through_hit.wall_dist)
                    / self.aspect)
                    as i32;
                let top_height = ((full_line_pixel_height / 2) as f32
                    * tex_top_height) as i32;
                let bottom_height = ((full_line_pixel_height / 2) as f32
                    * tex_bottom_height)
                    as i32;
                let line_height = top_height.saturating_add(bottom_height);

                let begin =
                    (self.int_half_height - bottom_height).max(0) as u32;
                let end = ((self.int_half_height + top_height).max(0) as u32)
                    .min(self.height - 1);

                let tex_height_minus_one = tex_height as f32 - 1.0;
                let tex_x = match through_hit.side {
                    Side::Vertical if ray.dir.x > 0.0 => {
                        tex_width
                            - (through_hit.wall_x * tex_width as f32) as u32
                            - 1
                    }

                    Side::Horizontal if ray.dir.z < 0.0 => {
                        tex_width
                            - (through_hit.wall_x * tex_width as f32) as u32
                            - 1
                    }
                    _ => (through_hit.wall_x * tex_width as f32) as u32,
                };
                let four_tex_x = tex_x * 4;
                let tex_y_step = tex_height as f32 / line_height as f32;
                let tex_y_start = (begin as f32 + bottom_height as f32
                    - self.float_half_height)
                    * tex_y_step;
                let data = TransparentTextureTile {
                    texture,
                    tex_width,
                    tex_height,
                    tex_bottom_height,
                    tex_top_height,
                    full_line_pixel_height,
                    top_height,
                    bottom_height,
                    line_height,
                    y_begin: begin,
                    y_end: end,
                    tex_height_minus_one,
                    tex_x,
                    four_tex_x,
                    tex_y_step,
                    tex_y_start,
                };
                transparent_texture_tile.push(data);
            }
        }
        let wall_row_data = match ray.hit.tile {
            Tile::Wall(tex) => {
                let (
                    texture,
                    tex_width,
                    tex_height,
                    tex_bottom_height,
                    tex_top_height,
                ) = match tex {
                    WallTile::BlueBrick => BLUE_BRICK_TEXTURE,
                    WallTile::LightPlank => LIGHT_PLANK_TEXTURE,
                };

                // TODO better names
                let full_line_pixel_height =
                    (self.height as f32 / ray.hit.wall_dist / self.aspect)
                        as i32;
                let top_height = ((full_line_pixel_height / 2) as f32
                    * tex_top_height) as i32;
                let bottom_height = ((full_line_pixel_height / 2) as f32
                    * tex_bottom_height)
                    as i32;
                let line_height = top_height.saturating_add(bottom_height);

                let begin =
                    (self.int_half_height - bottom_height).max(0) as u32;
                let end = ((self.int_half_height + top_height).max(0) as u32)
                    .min(self.height - 1);

                let tex_height_minus_one = tex_height as f32 - 1.0;
                let tex_x = match ray.hit.side {
                    Side::Vertical if ray.dir.x > 0.0 => {
                        tex_width
                            - (ray.hit.wall_x * tex_width as f32) as u32
                            - 1
                    }

                    Side::Horizontal if ray.dir.z < 0.0 => {
                        tex_width
                            - (ray.hit.wall_x * tex_width as f32) as u32
                            - 1
                    }
                    _ => (ray.hit.wall_x * tex_width as f32) as u32,
                };
                let four_tex_x = tex_x * 4;
                let tex_y_step = tex_height as f32 / line_height as f32;
                let mut tex_y_start = (begin as f32 + bottom_height as f32
                    - self.float_half_height)
                    * tex_y_step;
                Some(WallRowData {
                    texture,
                    tex_width,
                    tex_height,
                    tex_bottom_height,
                    tex_top_height,
                    full_line_pixel_height,
                    top_height,
                    bottom_height,
                    line_height,
                    y_begin: begin,
                    y_end: end,
                    tex_height_minus_one,
                    tex_x,
                    four_tex_x,
                    tex_y_step,
                    tex_y_start,
                })
            }
            Tile::Void => None,
            _ => unreachable!(),
        };
        RowData {
            object_row_data,
            transparent_texture_tile,
            wall_row_data,
        }
    }
}

// TODO BETTER NAMES
struct RowData<'a> {
    object_row_data: Vec<ObjectRowData>,
    transparent_texture_tile: Vec<TransparentTextureTile<'a>>,
    wall_row_data: Option<WallRowData<'a>>,
}

struct ObjectRowData {
    object: Object,
    dimension: f32,
    dimension_i: i32,
    ray_origin: Vec3,
    pos_x: f32,
    pos_z: f32,
    top_left_point: Vec3,
    top_side: Vec3,
    left_side: Vec3,
    rectangle_normal: Vec3,
    voxel_side: VoxelSide,
}

struct TransparentTextureTile<'a> {
    texture: &'a [u8],
    tex_width: u32,
    tex_height: u32,
    tex_bottom_height: f32,
    tex_top_height: f32,
    full_line_pixel_height: i32,
    top_height: i32,
    bottom_height: i32,
    line_height: i32,
    y_begin: u32,
    y_end: u32,
    tex_height_minus_one: f32,
    tex_x: u32,
    four_tex_x: u32,
    tex_y_step: f32,
    tex_y_start: f32,
}

struct WallRowData<'a> {
    texture: &'a [u8],
    tex_width: u32,
    tex_height: u32,
    tex_bottom_height: f32,
    tex_top_height: f32,
    full_line_pixel_height: i32,
    top_height: i32,
    bottom_height: i32,
    line_height: i32,
    y_begin: u32,
    y_end: u32,
    tex_height_minus_one: f32,
    tex_x: u32,
    four_tex_x: u32,
    tex_y_step: f32,
    tex_y_start: f32,
}

#[inline]
fn norm_rad(angle: f32) -> f32 {
    angle - (angle / TAU).floor() * TAU
}

#[inline(always)]
fn blend(background: &[u8], foreground: [u8; 4]) -> [u8; 4] {
    if foreground[3] == 255 {
        return foreground;
    }
    let alpha = foreground[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let blended_r =
        (foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha) as u8;
    let blended_g =
        (foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha) as u8;
    let blended_b =
        (foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha) as u8;
    let blended_a = (255.0 * alpha + background[3] as f32 * inv_alpha) as u8;

    [blended_r, blended_g, blended_b, blended_a]
}
