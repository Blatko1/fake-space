mod ray;

use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use crate::map::Map;
use crate::player::Player;
use crate::raycaster::camera::Camera;
use crate::textures::{TextureArray, TextureID};

use self::ray::Ray;

#[derive(Debug, Copy, Clone)]
pub struct PointXZ<T> {
    pub x: T,
    pub z: T,
}

impl<T> PointXZ<T> {
    pub fn new(x: T, z: T) -> Self {
        Self { x, z }
    }
}

pub fn cast_and_draw_par<'a>(
    camera: &Camera,
    player: &Player,
    map: &Map,
    textures: &TextureArray,
    column_iter: &mut [u8],
) {
    // TODO is there better multithreading?
    column_iter
        .par_chunks_exact_mut(camera.view_height as usize * 3)
        .enumerate()
        .for_each(|(column_index, column)| {
            cast_and_draw_column(camera, player, map, textures, column_index, column);
        });
}

pub fn cast_and_draw<'a>(
    camera: &Camera,
    player: &Player,
    map: &Map,
    textures: &TextureArray,
    column_iter: &mut [u8],
) {
    for (column_index, column) in column_iter
        .chunks_exact_mut(camera.view_height as usize * 3)
        .enumerate()
    {
        cast_and_draw_column(camera, player, map, textures, column_index, column);
    }
}

// TODO maybe draw first the floor, then bottom wall, then top wall, then ceiling
fn cast_and_draw_column<'a>(
    camera: &Camera,
    player: &Player,
    map: &Map,
    textures: &TextureArray,
    column_index: usize,
    column: &mut [u8],
) {
    let mut ray = Ray::camera_cast(camera, column_index);
    let mut current_room = map.get_room_data(player.current_room_id());
    let mut current_room_dimensions = current_room.segment.dimensions_i64();

    let mut top_draw_bound = camera.view_height as usize;
    let mut bottom_draw_bound = 0;

    // DDA loop
    loop {
        let current_tile_x = ray.next_tile.x as usize;
        let current_tile_z = ray.next_tile.z as usize;

        /* ==========================================================
         *                         DDA step
         * ========================================================== */
        if ray.side_dist_x < ray.side_dist_z {
            ray.wall_dist = ray.side_dist_x.max(0.0);
            ray.next_tile.x += ray.step_x;
            if ray.next_tile.x >= current_room_dimensions.0 || ray.next_tile.x < 0 {
                break;
            }
            ray.side_dist_x += ray.delta_dist_x;
            ray.hit_wall_side = Side::Vertical;
            let wall_offset = ray.origin.z + ray.wall_dist * ray.dir.z;
            ray.wall_offset = wall_offset - wall_offset.floor();
        } else {
            ray.wall_dist = ray.side_dist_z.max(0.0);
            ray.next_tile.z += ray.step_z;
            if ray.next_tile.z >= current_room_dimensions.1 || ray.next_tile.z < 0 {
                break;
            }
            ray.side_dist_z += ray.delta_dist_z;
            ray.hit_wall_side = Side::Horizontal;
            let wall_offset = ray.origin.x + ray.wall_dist * ray.dir.x;
            ray.wall_offset = wall_offset - wall_offset.floor();
        }

        /* ==========================================================
         *           Drawing platforms (floor and ceiling)
         * ========================================================== */
        // Tile which the ray just traveled over before hitting a wall
        let current_tile = current_room
            .segment
            .get_tile_unchecked(current_tile_x, current_tile_z);

        // Variables used for reducing the amount of calculations and for optimization
        let tile_step_factor = ray.horizontal_plane * 2.0 * camera.width_recip;
        let pos_factor = ray.camera_dir - ray.horizontal_plane
            + tile_step_factor * ray.column_index as f32;
        let mut draw_platform = |draw_from_dist: f32,
                                 draw_to_dist: f32,
                                 height: f32,
                                 texture_id: TextureID|
         -> (usize, usize) {
            let texture = textures.get_texture_data(texture_id);
            // TODO idk if this 'if' is necessary
            if texture.is_empty() {
                return (bottom_draw_bound, top_draw_bound);
            }
            // Draw from (always drawing from bottom to top):
            let half_wall_pixel_height = camera.f_half_height / draw_from_dist;
            let pixels_to_top =
                half_wall_pixel_height * (height - ray.origin.y) + camera.y_shearing;
            let draw_from = ((camera.f_half_height + pixels_to_top) as usize)
                .clamp(bottom_draw_bound, top_draw_bound);

            // Draw to:
            let half_wall_pixel_height = camera.f_half_height / draw_to_dist;
            let pixels_to_top =
                half_wall_pixel_height * (height - ray.origin.y) + camera.y_shearing;
            let draw_to = ((camera.f_half_height + pixels_to_top) as usize)
                .clamp(draw_from, top_draw_bound);

            let (tex_width, tex_height) =
                (texture.width as usize, texture.height as usize);

            let segment = column
                .chunks_exact_mut(3)
                .enumerate()
                .skip(draw_from)
                .take(draw_to - draw_from);

            let denominator = (height - ray.origin.y) * camera.f_half_height;

            for (y, pixel) in segment {
                let row_dist =
                    denominator / (y as f32 - camera.y_shearing - camera.f_half_height);
                let pos = ray.origin + row_dist * pos_factor;

                let tex_x =
                    ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
                let tex_y =
                    ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
                let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
                let color = &texture.data[i..i + 3];

                pixel.copy_from_slice(color);
            }
            (draw_from, draw_to)
        };

        // Draw ground platform
        let (from, drawn_to) = draw_platform(
            ray.previous_wall_dist,
            ray.wall_dist,
            current_tile.ground_level,
            current_tile.ground_tex,
        );
        // Draw ceiling platform
        let (drawn_from, to) = draw_platform(
            ray.wall_dist,
            ray.previous_wall_dist,
            current_tile.ceiling_level,
            current_tile.ceiling_tex,
        );
        if from != bottom_draw_bound {
            fill_color(column, bottom_draw_bound, from, 200);
        }
        if to != top_draw_bound {
            fill_color(column, to, top_draw_bound, 200);
        }

        bottom_draw_bound = drawn_to;
        top_draw_bound = drawn_from;


        /* ==========================================================
         *            Drawing walls (bottom and top wall)
         * ========================================================== */
        // The tile ray hit
        let next_tile = current_room
            .segment
            .get_tile_unchecked(ray.next_tile.x as usize, ray.next_tile.z as usize);

        let half_wall_pixel_height = camera.f_half_height / ray.wall_dist;
        let mut draw_wall = |bottom_level: f32,
                             top_level: f32,
                             texture_id: TextureID|
         -> (usize, usize) {
            let texture = textures.get_texture_data(texture_id);
            if texture.is_empty() {
                return (bottom_draw_bound, top_draw_bound);
            }
            let (texture, tex_width, tex_height) = (
                texture.data,
                texture.width as usize,
                texture.height as usize,
            );

            // Calculate wall pixel height for the parts above and below the middle
            let pixels_to_bottom = half_wall_pixel_height * (ray.origin.y - bottom_level)
                - camera.y_shearing;
            let pixels_to_top =
                half_wall_pixel_height * (top_level - ray.origin.y) + camera.y_shearing;
            let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

            let draw_from = ((camera.f_half_height - pixels_to_bottom) as usize)
                .clamp(bottom_draw_bound, top_draw_bound);
            let draw_to = ((camera.f_half_height + pixels_to_top) as usize)
                .clamp(bottom_draw_bound, top_draw_bound);

            let tex_x = match ray.hit_wall_side {
                Side::Vertical if ray.dir.x > 0.0 => {
                    tex_width - (ray.wall_offset * tex_width as f32) as usize - 1
                }
                Side::Horizontal if ray.dir.z < 0.0 => {
                    tex_width - (ray.wall_offset * tex_width as f32) as usize - 1
                }
                _ => (ray.wall_offset * tex_width as f32) as usize,
            };
            let tex_y_step = (top_level - bottom_level) * tex_height as f32
                / full_wall_pixel_height
                * 0.5;
            let mut tex_y =
                (draw_from as f32 + pixels_to_bottom - camera.f_half_height) * tex_y_step;

            let segment = column
                .chunks_exact_mut(3)
                .enumerate()
                .skip(draw_from)
                .take(draw_to - draw_from);

            for (_y, pixel) in segment {
                // avoids small artefacts let tex_y_pos = tex_y.round() as usize % tex_height;
                let tex_y_pos = tex_y as usize % tex_height;
                tex_y += tex_y_step;
                let i = 4 * ((tex_height - tex_y_pos - 1) * tex_width + tex_x);
                let color = &texture[i..i + 3];

                pixel.copy_from_slice(color);
            }
            (draw_from, draw_to)
        };

        // Draw bottom wall
        let (from, drawn_to) = draw_wall(
            next_tile.bottom_level,
            next_tile.ground_level,
            next_tile.bottom_wall_tex,
        );
        // Draw top wall
        let (drawn_from, to) = draw_wall(
            next_tile.ceiling_level,
            next_tile.top_level,
            next_tile.top_wall_tex,
        );
        if from != bottom_draw_bound {
            fill_color(column, bottom_draw_bound, from, 200);
        }
        if to != top_draw_bound {
            fill_color(column, to, top_draw_bound, 200);
        }

        bottom_draw_bound = drawn_to;
        top_draw_bound = drawn_from;

        /* ==========================================================
         *                      Check for portal
         * ========================================================== */
        // Switch to the different room if portal is hit
        if let Some(src_dummy_portal) = next_tile.portal {
            let src_portal = current_room.get_portal(src_dummy_portal.id);
            match src_portal.link {
                Some((room_id, portal_id)) => {
                    let dest_room = map.get_room_data(room_id);
                    let dest_portal = dest_room.get_portal(portal_id);
                    ray.portal_teleport(src_portal, dest_portal);
                    current_room = dest_room;
                    current_room_dimensions = current_room.segment.dimensions_i64();
                }
                None => {
                    fill_color(column, bottom_draw_bound, top_draw_bound, 0);
                    break;
                }
            }
        }

        fill_color(column, bottom_draw_bound, top_draw_bound, 100);

        ray.previous_wall_dist = ray.wall_dist;
    }
}

fn fill_color(column: &mut [u8], bottom_bound: usize, top_bound: usize, color: u8) {
    column[bottom_bound * 3..top_bound * 3].fill(color)
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}
