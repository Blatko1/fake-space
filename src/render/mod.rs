pub mod camera;
mod colors;
mod platforms;
mod ray;
mod skybox;
mod voxel_model;
mod wall;

use glam::Vec3;
use crate::render::camera::Camera;
use crate::world::{SkyboxTextures, Tile, World};
use crate::{player::Player, world::textures::TextureManager};
use crate::voxel::VoxelModel;

use self::ray::Ray;

const SPOTLIGHT_DISTANCE: f32 = 2.5;
const FLASHLIGHT_INTENSITY: f32 = 3.0;
const FLASHLIGHT_RADIUS: f32 = 0.65;
const FLASHLIGHT_DISTANCE: f32 = 21.0;

#[derive(Debug, Copy, Clone)]
pub struct PointXZ<T> {
    pub x: T,
    pub z: T,
}

pub fn cast_and_draw<'a, C>(player: &Player, world: &World, column_iter: C)
where
    C: Iterator<Item = &'a mut [u8]>,
{
    let dimension = 8;
    let mut damaged_data = vec![vec![vec![1; dimension]; dimension]; dimension];
    for x in 0..4 {
        for z in 0..3 {
            for y in 5..8 {
                damaged_data[y][z][x] = 0u8;
            }
        }
    }
    for y in 0..dimension {
        damaged_data[y][0][0] = 0u8;
        damaged_data[y][5][3] = y as u8;
    }
    for z in 0..dimension {
        damaged_data[4][z][3] = 0u8;
        damaged_data[4][z][4] = 0u8;
        damaged_data[4][z][5] = 0u8;
    }
    damaged_data[5][1][3] = 10u8;
    damaged_data[5][2][3] = 20u8;
    let damaged_data = damaged_data.into_iter().flatten().flatten().collect();
    let damaged = VoxelModel::new(damaged_data, dimension);
    let model = damaged.as_ref();

    let camera = player.get_camera();
    let texture_manager = world.texture_manager();
    column_iter.enumerate().for_each(|(column_index, column)| {
        let mut room = world.get_room_data(player.get_current_room_id());
        let mut segment = room.segment;
        let mut encountered_models = Vec::new();

        let starting_ray = Ray::cast_with_camera(column_index, camera);
        let mut params = DrawParams {
            bottom_draw_bound: 0,
            top_draw_bound: camera.view_height as usize,
            outer_bottom_draw_bound: camera.view_height as usize,
            outer_top_draw_bound: 0,
            tile: Tile::EMPTY,
            ray: starting_ray,
            skybox: room.data.get_skybox(),
            ambient_light: room.data.get_ambient_light(),
            texture_manager,
            camera,
        };
        skybox::draw_skybox(params, column);
        // DDA loop
        loop {
            let mut ray = params.ray;
            let current_tile_x = ray.next_tile.x;
            let current_tile_z = ray.next_tile.z;

            // DDA step
            if ray.side_dist_x < ray.side_dist_z {
                ray.wall_dist = ray.side_dist_x.max(0.0);
                ray.next_tile.x += ray.step_x;
                ray.side_dist_x += ray.delta_dist_x;
                ray.wall_side_hit = Side::Vertical;
                let wall_offset = ray.origin.z + ray.wall_dist * ray.dir.z;
                ray.wall_offset = wall_offset - wall_offset.floor();
            } else {
                ray.wall_dist = ray.side_dist_z.max(0.0);
                ray.next_tile.z += ray.step_z;
                ray.side_dist_z += ray.delta_dist_z;
                ray.wall_side_hit = Side::Horizontal;
                let wall_offset = ray.origin.x + ray.wall_dist * ray.dir.x;
                ray.wall_offset = wall_offset - wall_offset.floor();
            };
            params.ray = ray;

            // Tile which the ray just traveled over before hitting a wall.
            match segment.get_tile(current_tile_x, current_tile_z) {
                Some(&current_tile) => params.tile = current_tile,
                None => break,
            };

            // Drawing top and bottom platforms
            let drawn_to = platforms::draw_bottom_platform(params, column);
            params.bottom_draw_bound = drawn_to;
            let drawn_from = platforms::draw_top_platform(params, column);
            params.top_draw_bound = drawn_from;

            let mut next_tile = match segment.get_tile(ray.next_tile.x, ray.next_tile.z) {
                Some(&t) => t,
                None => break,
            };

            if let Some(model) = next_tile.voxel_object.clone() {
                encountered_models.push((model, params.bottom_draw_bound, params.top_draw_bound, ray.wall_side_hit));
            }

            // Switch to the different room if portal is hit
            if let Some(src_dummy_portal) = next_tile.portal {
                let src_portal = room.get_portal(src_dummy_portal.id);
                let (dest_room, dest_portal) = match src_portal.link {
                    Some((room_id, portal_id)) => {
                        let dest_room = world.get_room_data(room_id);
                        let dest_portal = dest_room.get_portal(portal_id);
                        (dest_room, dest_portal)
                    }
                    None => break,
                };
                room = dest_room;
                segment = room.segment;
                next_tile = match segment.get_tile(
                    dest_portal.position.x as i64,
                    dest_portal.position.z as i64,
                ) {
                    Some(&t) => t,
                    None => break,
                };
                ray.portal_teleport(src_portal, dest_portal);
                params.ray = ray;
                params.skybox = room.data.get_skybox();
                params.ambient_light = room.data.get_ambient_light();
            }

            params.tile = next_tile;
            // Drawing top and bottom walls
            let drawn_to = wall::draw_bottom_wall(params, column);
            params.bottom_draw_bound = drawn_to;
            let drawn_from = wall::draw_top_wall(params, column);
            params.top_draw_bound = drawn_from;

            ray.previous_wall_dist = ray.wall_dist;
            params.ray = ray;
        }
        let ray = starting_ray;
        if !encountered_models.is_empty() {
            column
                .chunks_exact_mut(4)
                .enumerate()
                .for_each(|(y, pixel)| {
                    let screen_y = y;
                    let y = y as f32 - camera.y_shearing;
                    // Filter models which are covered by walls or platforms
                    for &(object, bottom_bound, top_bound, hit_side) in encountered_models.iter().rev().filter(|&&(_, bottom_bound, top_bound, _)| {
                        bottom_bound <= screen_y && top_bound > screen_y
                    }) {
                        //TODO TEMP
                        let dimension = 8.0;

                        // TODO why do y positions have to be divided by 2
                        let mut ray_origin = camera.origin * dimension;
                        ray_origin.y *= 0.5;
                        let obj_x_pos = object.pos_x as f32 * dimension;
                        let obj_y_pos = object.pos_y * dimension * 0.5;
                        let obj_z_pos = object.pos_z as f32 * dimension;
                        let (top_left_point, top_side, voxel_side) = match hit_side {
                            Side::Vertical => {
                                // is east side hit
                                if ray.dir.x > 0.0 {
                                    let top_left =
                                        Vec3::new(obj_x_pos, obj_y_pos + dimension, obj_z_pos + dimension);
                                    (top_left, Vec3::new(0.0, 0.0, -dimension), VoxelSide::Left)
                                }
                                // is west side hit
                                else {
                                    let top_left =
                                        Vec3::new(obj_x_pos + dimension, obj_y_pos + dimension, obj_z_pos);
                                    (top_left, Vec3::new(0.0, 0.0, dimension), VoxelSide::Right)
                                }
                            }
                            Side::Horizontal => {
                                // is north side hit
                                if ray.dir.z > 0.0 {
                                    let top_left = Vec3::new(obj_x_pos, obj_y_pos + dimension, obj_z_pos);
                                    (top_left, Vec3::new(dimension, 0.0, 0.0), VoxelSide::Front)
                                }
                                // is south side hit
                                else {
                                    let top_left = Vec3::new(
                                        obj_x_pos + dimension,
                                        obj_y_pos + dimension,
                                        obj_z_pos + dimension,
                                    );
                                    (top_left, Vec3::new(-dimension, 0.0, 0.0), VoxelSide::Back)
                                }
                            }
                        };
                        let left_side = Vec3::new(0.0, -dimension, 0.0);
                        // Calculate the normal vector (N) of the rectangle's surface.
                        let rectangle_normal = top_side.cross(left_side);

                        // Y-coordinate on the vertical camera plane (range [-1.0, 1.0])
                        let plane_y = y * camera.height_recip * 2.0 - 1.0;
                        // Ray direction for current pixel column
                        let ray_dir = ray.dir + camera.vertical_plane * plane_y;
                        // Length of ray from one x/y/z side to next x/y/z side on the tile_map
                        let delta_dist = Vec3::new(
                            ray.delta_dist_x,
                            1.0 / ray_dir.y.abs(),
                            ray.delta_dist_z,
                        );

                        // Somehow I don't need to normalize ray_dir since I am getting the same result
                        // without normalization
                        if let Some(intersection) = rectangle_vector_intersection(
                            top_left_point,
                            left_side,
                            rectangle_normal,
                            ray_dir,
                            ray_origin,
                            voxel_side,
                        ) {


                            let mut side_dist_x = delta_dist.x
                                * if ray_dir.x < 0.0 {
                                intersection.x.fract()
                            } else {
                                1.0 - intersection.x.fract()
                            };
                            let mut side_dist_y = delta_dist.y
                                * if ray_dir.y < 0.0 {
                                (intersection.y - obj_y_pos).fract()
                            } else {
                                1.0 - (intersection.y - obj_y_pos).fract()
                            };
                            let mut side_dist_z = delta_dist.z
                                * if ray_dir.z < 0.0 {
                                intersection.z.fract()
                            } else {
                                1.0 - intersection.z.fract()
                            };

                            let mut grid_x = (intersection.x - obj_x_pos).max(0.0) as i32;
                            let mut grid_z = (intersection.z - obj_z_pos).max(0.0) as i32;
                            let mut grid_y = (intersection.y - obj_y_pos).max(0.0) as i32;
                            let (step_x, step_y, step_z) = (
                                ray_dir.x.signum() as i32,
                                ray_dir.y.signum() as i32,
                                ray_dir.z.signum() as i32,
                            );

                            let mut side = voxel_side;
                            match side {
                                VoxelSide::Top | VoxelSide::Right | VoxelSide::Back => {
                                    if side_dist_x < side_dist_y {
                                        if side_dist_x < side_dist_z {
                                            grid_x += step_x;
                                            if grid_x < 0 {
                                                continue;
                                            }
                                            side_dist_x += delta_dist.x;
                                        } else {
                                            grid_z += step_z;
                                            if grid_z < 0 {
                                                continue;
                                            }
                                            side_dist_z += delta_dist.z;
                                        }
                                    } else if side_dist_y < side_dist_z {
                                        grid_y += step_y;
                                        if grid_y < 0 {
                                            continue;
                                        }
                                        side_dist_y += delta_dist.y;
                                    } else {
                                        grid_z += step_z;
                                        if grid_z < 0 {
                                            continue;
                                        }
                                        side_dist_z += delta_dist.z;
                                    }
                                }
                                _ => (),
                            }
                            loop {
                                let voxel = model.get_voxel(
                                    grid_x as usize,
                                    grid_y as usize,
                                    grid_z as usize,
                                );
                                match voxel {
                                    Some(0) => (),
                                    Some(v) => {
                                        let x = (grid_x as f32 + obj_x_pos - ray_origin.x)/ dimension;
                                        let y = (grid_y as f32 + obj_y_pos - ray_origin.y) * 2.0/ dimension;
                                        let z = (grid_z as f32 + obj_z_pos - ray_origin.z)/ dimension;
                                        let distance = ((x*x + y*y + z*z)).sqrt();
                                        if column_index as u32 == camera.view_width / 2 && screen_y as u32 == camera.view_height / 2 {
                                            println!("d: {}", distance);
                                        }
                                        let flashlight_x = (2.0 * ray.column_index as f32 * camera.width_recip - 1.0) * camera.aspect;
                                        let t = 1.0 - (distance / SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
                                        let spotlight = t * t * (3.0 - t * 2.0);
                                        let flashlight_intensity_factor = (1.0 - (distance / FLASHLIGHT_DISTANCE).clamp(0.0, 1.0)) * FLASHLIGHT_INTENSITY;
                                        pixel.copy_from_slice(&[100, 200, 240, 255]);
                                        let flashlight_y = 2.0 * screen_y as f32 * camera.height_recip - 1.0;
                                        for color in &mut pixel[0..3] {
                                            let flashlight_intensity = (FLASHLIGHT_RADIUS - (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt()) * flashlight_intensity_factor;
                                            let intensity = flashlight_intensity.max(0.0) + 0.03;
                                            *color = (*color as f32 * intensity) as u8;
                                        }
                                        darken_side(side, pixel);
                                        break;
                                    }

                                    None => (),
                                }
                                //if column_index as u32 == camera.view_width / 2 && screen_y as u32 == camera.view_height / 2 {
                                //    println!("inter: {}, x: {}, y: {}, z: {}, int: {}, y_pos: {}", intersection, grid_x, grid_y, grid_z, intersection.y, obj_y_pos);
                                //}
                                if side_dist_x < side_dist_y {
                                    if side_dist_x < side_dist_z {
                                        grid_x += step_x;
                                        if grid_x < 0 || grid_x >= dimension as i32 {
                                            break;
                                        }
                                        side = if step_x.is_positive() {
                                            VoxelSide::Left
                                        } else {
                                            VoxelSide::Right
                                        };
                                        side_dist_x += delta_dist.x;
                                    } else {
                                        grid_z += step_z;
                                        if grid_z < 0 || grid_z >= dimension as i32 {
                                            break;
                                        }
                                        side = if step_z.is_positive() {
                                            VoxelSide::Front
                                        } else {
                                            VoxelSide::Back
                                        };
                                        side_dist_z += delta_dist.z;
                                    }
                                } else if side_dist_y < side_dist_z {
                                    grid_y += step_y;
                                    if grid_y < 0 {
                                        break;
                                    }
                                    side = if step_y.is_positive() {
                                        VoxelSide::Bottom
                                    } else {
                                        VoxelSide::Top
                                    };
                                    side_dist_y += delta_dist.y;
                                } else {
                                    grid_z += step_z;
                                    if grid_z < 0 || grid_z >= dimension as i32 {
                                        break;
                                    }
                                    side = if step_z.is_positive() {
                                        VoxelSide::Front
                                    } else {
                                        VoxelSide::Back
                                    };
                                    side_dist_z += delta_dist.z;
                                }
                            }
                            //pixel.copy_from_slice(&[100, 200, 240, 255]);
                        }
                    }
                });
        }
    })
}

#[derive(Clone, Copy)]
pub struct DrawParams<'a> {
    pub bottom_draw_bound: usize,
    pub top_draw_bound: usize,
    pub outer_bottom_draw_bound: usize,
    pub outer_top_draw_bound: usize,
    pub tile: Tile,
    pub ray: Ray,
    pub skybox: SkyboxTextures,
    pub ambient_light: f32,
    pub texture_manager: &'a TextureManager,
    pub camera: &'a Camera,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

// TODO switch to unsafe for speed
#[inline(always)]
fn blend(background: &[u8], foreground: &[u8]) -> [u8; 4] {
    let alpha = foreground[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    [
        (foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha) as u8,
        (foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha) as u8,
        (foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha) as u8,
        (255.0 * alpha + background[3] as f32 * inv_alpha) as u8,
    ]
}


#[derive(Debug, Clone, Copy)]
pub enum VoxelSide {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

/// The rectangle-vector intersection solution was taken directly
/// from [`StackOverflow`](https://stackoverflow.com/questions/8812073).
#[inline]
fn rectangle_vector_intersection(
    corner: Vec3,
    left_side: Vec3,
    rectangle_normal: Vec3,
    ray_dir: Vec3,
    ray_origin: Vec3,
    side: VoxelSide,
) -> Option<Vec3> {
    let a = rectangle_normal.dot(corner - ray_origin)
        / ray_dir.dot(rectangle_normal);

    let mut intersection_point = ray_origin + a * ray_dir;

    match side {
        VoxelSide::Top | VoxelSide::Bottom => intersection_point.y = corner.y,
        VoxelSide::Left | VoxelSide::Right => intersection_point.x = corner.x,
        VoxelSide::Front | VoxelSide::Back => intersection_point.z = corner.z,
    }

    let left_side_len = left_side.length();

    let p0p = intersection_point - corner;
    //let q1: f32 = p0p.dot(top_side) / top_side_len;
    let q2 = p0p.dot(left_side) / left_side_len;

    // Check if the intersection point is inside the rectangle.
    // Only check if the intersection point is too high or too low.
    if q2 <= left_side_len {
        Some(intersection_point)
    } else {
        None
    }
}

#[inline]
fn darken_side(side: VoxelSide, color: &mut [u8]) {
    match side {
        VoxelSide::Top => (),
        VoxelSide::Bottom => {
            color[0] = color[0].saturating_sub(55);
            color[1] = color[1].saturating_sub(55);
            color[2] = color[2].saturating_sub(55);
        }
        VoxelSide::Left => {
            color[0] = color[0].saturating_sub(15);
            color[1] = color[1].saturating_sub(15);
            color[2] = color[2].saturating_sub(15);
        }
        VoxelSide::Right => {
            color[0] = color[0].saturating_sub(35);
            color[1] = color[1].saturating_sub(35);
            color[2] = color[2].saturating_sub(35);
        }
        VoxelSide::Front => {
            color[0] = color[0].saturating_sub(5);
            color[1] = color[1].saturating_sub(5);
            color[2] = color[2].saturating_sub(5);
        }
        VoxelSide::Back => {
            color[0] = color[0].saturating_sub(45);
            color[1] = color[1].saturating_sub(45);
            color[2] = color[2].saturating_sub(45);
        }
    }
}
