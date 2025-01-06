mod ray;

use ray::SkyboxSide;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;

use crate::map::Map;
use crate::player::Player;
use crate::raycaster::camera::Camera;
use crate::textures::{SkyboxTexturesRef, TextureArray, TextureDataRef, TextureID};

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

pub struct FrameRenderer<'a> {
    camera: &'a Camera,
    player: &'a Player,
    map: &'a Map,
    textures: &'a TextureArray,

    // 
    y_shearing: f32,
    view_height: u32,
    half_view_height: f32,
    width_recip: f32
}

impl<'a> FrameRenderer<'a> {
    pub fn new(camera: &'a Camera, player: &'a Player, map: &'a Map, textures: &'a TextureArray) -> Self {
        Self {
            camera,
            player,
            map,
            textures,

            y_shearing: camera.y_shearing,
            view_height: camera.view_height,
            half_view_height: camera.view_height as f32 * 0.5,
            width_recip: 1.0 / camera.view_width as f32
        }
    }

    pub fn render_par(&mut self, pixel_buffer: &'a mut [u8]
    ) {
        // TODO is there better multithreading?
        pixel_buffer
            .par_chunks_exact_mut(self.camera.view_height as usize * 3)
            .enumerate()
            .for_each(|(column_index, column)| {
                self.render_column(column_index, column);
            });
    }
    
    pub fn render(&mut self, pixel_buffer: &'a mut [u8]
    ) {
        for (column_index, column) in pixel_buffer
            .chunks_exact_mut(self.camera.view_height as usize * 3)
            .enumerate()
        {
            self.render_column(column_index, column);
        }
    }


// TODO maybe draw first the floor, then bottom wall, then top wall, then ceiling
fn render_column(&self,
    column_index: usize,
    column: &mut [u8],
) {
    let mut ray = Ray::camera_cast(self.camera, column_index);
    let mut current_room = self.map.get_room_data(self.player.current_room_id());
    let mut current_room_dimensions = current_room.segment.dimensions_i64();

    let skybox_textures = self.textures.get_skybox_textures(current_room.data.skybox());

    let mut top_draw_bound = self.view_height as usize;
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
        let tile_step_factor = ray.horizontal_plane * 2.0 * self.width_recip;
        let pos_factor = ray.camera_dir - ray.horizontal_plane
            + tile_step_factor * ray.column_index as f32;
        let mut draw_platform = |draw_from_dist: f32,
                                 draw_to_dist: f32,
                                 height: f32,
                                 texture_id: TextureID|
         -> (usize, usize) {
            let texture = self.textures.get_texture_data(texture_id);
            // TODO idk if this 'if' is necessary
            if texture.is_empty() {
                return (bottom_draw_bound, top_draw_bound);
            }
            // Draw from (always drawing from bottom to top):
            let half_wall_pixel_height = self.half_view_height / draw_from_dist;
            let pixels_to_top =
                half_wall_pixel_height * (height - ray.origin.y) + self.y_shearing;
            let draw_from = ((self.half_view_height + pixels_to_top) as usize)
                .clamp(bottom_draw_bound, top_draw_bound);

            // Draw to:
            let half_wall_pixel_height = self.half_view_height / draw_to_dist;
            let pixels_to_top =
                half_wall_pixel_height * (height - ray.origin.y) + self.y_shearing;
            let draw_to = ((self.half_view_height + pixels_to_top) as usize)
                .clamp(draw_from, top_draw_bound);

            let (tex_width, tex_height) =
                (texture.width as usize, texture.height as usize);

            let segment = column
                .chunks_exact_mut(3)
                .enumerate()
                .skip(draw_from)
                .take(draw_to - draw_from);

            let denominator = (height - ray.origin.y) * self.half_view_height;

            for (y, pixel) in segment {
                let row_dist =
                    denominator / (y as f32 - self.y_shearing - self.half_view_height);
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
            //println!("floor skiped!");
            fill_color(column, bottom_draw_bound, from, 200);
        }
        if to != top_draw_bound {
            //println!("ceiling skiped!");
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

        let half_wall_pixel_height = self.half_view_height / ray.wall_dist;
        let mut draw_wall = |bottom_level: f32,
                             top_level: f32,
                             texture_id: TextureID|
         -> (usize, usize) {
            let texture = self.textures.get_texture_data(texture_id);
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
                - self.y_shearing;
            let pixels_to_top =
                half_wall_pixel_height * (top_level - ray.origin.y) + self.y_shearing;
            let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

            let draw_from = ((self.half_view_height - pixels_to_bottom) as usize)
                .clamp(bottom_draw_bound, top_draw_bound);
            let draw_to = ((self.half_view_height + pixels_to_top) as usize)
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
                (draw_from as f32 + pixels_to_bottom - self.half_view_height) * tex_y_step;

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
            //println!("wall bottom skiped!");
            fill_color(column, bottom_draw_bound, from, 200);
        }
        if to != top_draw_bound {
            //println!("wall top skiped!");
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
                    let dest_room = self.map.get_room_data(room_id);
                    let dest_portal = dest_room.get_portal(portal_id);
                    ray.portal_teleport(src_portal, dest_portal);
                    current_room = dest_room;
                    current_room_dimensions = current_room.segment.dimensions_i64();
                }
                None => {
                    fill_color(column, bottom_draw_bound, top_draw_bound, 0);
                    return;
                }
            }
        }

        ray.previous_wall_dist = ray.wall_dist;
    }
    self.fill_skybox(&ray, skybox_textures, column, bottom_draw_bound, top_draw_bound);
}

fn fill_skybox(&self, ray: &Ray, skybox_textures: SkyboxTexturesRef, column: &mut [u8], draw_from: usize, draw_to: usize) {
    let wall_texture = match ray.skybox_wall {
        SkyboxSide::North => skybox_textures.north,
        SkyboxSide::East => skybox_textures.east,
        SkyboxSide::South => skybox_textures.south,
        SkyboxSide::West => skybox_textures.west,
    };
    if wall_texture.is_empty() {
        column.fill(0);
        return;
    }
    let (texture, tex_width, tex_height) = (
        wall_texture.data,
        wall_texture.width as usize,
        wall_texture.height as usize,
    );

    let pixels_to_bottom = ray.half_skybox_wall_pixel_height - self.y_shearing;
    let pixels_to_top = ray.half_skybox_wall_pixel_height + self.y_shearing;
    let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

    // From which pixel to begin drawing and on which to end
    let draw_wall_from = ((self.half_view_height - pixels_to_bottom) as usize)
        .clamp(draw_from, draw_to);
    let draw_wall_to = ((self.half_view_height + pixels_to_top) as usize)
        .clamp(draw_wall_from, draw_to);

    let tex_x = (ray.skybox_wall_offset * tex_width as f32) as usize;
    let tex_y_step = tex_height as f32 / full_wall_pixel_height;
    let mut tex_y =
        (draw_wall_from as f32 + pixels_to_bottom - self.half_view_height) * tex_y_step;
    // Precomputed variables for performance increase
    let four_tex_width = tex_width * 4;
    let four_tex_x = tex_x * 4;
    column
        .chunks_exact_mut(3)
        .skip(draw_wall_from)
        .take(draw_wall_to - draw_wall_from)
        .for_each(|dest| {
            let tex_y_pos = tex_y.round() as usize % tex_height;
            let i = (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
            let src = &texture[i..i + 3];

            // Draw the pixel:
            dest.copy_from_slice(src);
            // TODO maybe make it so `tex_y_step` is being subtracted.
            tex_y += tex_y_step;
        });



        let bottom_texture = skybox_textures.bottom;
        if bottom_texture.is_empty() {
            column.fill(0);
            return;
        }

        let (texture, tex_width, tex_height) = (
            bottom_texture.data,
            bottom_texture.width as usize,
            bottom_texture.height as usize,
        );

        // TODO wrong names
        // Draw from:
        let draw_ground_from = draw_from;

        // Draw to:
        let pixels_to_bottom = ray.half_skybox_wall_pixel_height * 0.5 + self.y_shearing;
        let draw_ground_to = ((self.half_view_height + pixels_to_bottom) as usize)
            .clamp(draw_from, draw_wall_from);

        // Variables used for reducing the amount of calculations and for optimization
        let tile_step_factor = self.camera.horizontal_plane * 2.0 * self.width_recip;
        let pos_factor = self.camera.forward_dir - self.camera.horizontal_plane
            + tile_step_factor * ray.column_index as f32;
        column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_ground_from)
            .take(draw_ground_to - draw_ground_from)
            .for_each(|(y, rgba)| {
                let row_dist =
                    self.half_view_height / (y as f32 - self.half_view_height - self.y_shearing);
                let pos = glam::Vec3::new(0.5, 0.5, 0.5) + row_dist * pos_factor;
                let tex_x =
                    ((tex_width as f32 * (1.0 - pos.x)) as usize).min(tex_width - 1);
                let tex_y = ((tex_height as f32 * pos.z) as usize).min(tex_height - 1);
                let i = 4 * (tex_width * tex_y + tex_x);
                let color = &texture[i..i + 3];

                rgba.copy_from_slice(color);
            });


        let top_texture = skybox_textures.top;
        if top_texture.is_empty() {
            column.fill(0);
            return;
        }

        let (texture, tex_width, tex_height) = (
            top_texture.data,
            top_texture.width as usize,
            top_texture.height as usize,
        );

        // TODO wrong names
        // Draw from:
        let pixels_to_bottom = ray.half_skybox_wall_pixel_height * 0.5 - self.y_shearing;
        let draw_ceiling_from = ((self.half_view_height - pixels_to_bottom) as usize)
            .clamp(draw_wall_to, draw_to);

        // Draw to:
        let draw_ceiling_to = draw_to;

        // Variables used for reducing the amount of calculations and for optimization
        let ray_dir = self.camera.forward_dir - self.camera.horizontal_plane;
        let tile_step_factor = self.camera.horizontal_plane * 2.0 * self.width_recip;
        let pos_factor = ray_dir + tile_step_factor * ray.column_index as f32;
        let half_h_plus_shearing = self.half_view_height + self.y_shearing;
        column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_ceiling_from)
            .take(draw_ceiling_to - draw_ceiling_from)
            .for_each(|(y, rgba)| {
                let row_dist = self.half_view_height / (y as f32 - half_h_plus_shearing);
                let pos = glam::Vec3::new(0.5, 0.5, 0.5) + row_dist * pos_factor;
                let tex_x = ((tex_width as f32 * pos.x) as usize).min(tex_width - 1);
                let tex_y = ((tex_height as f32 * pos.z) as usize).min(tex_height - 1);
                let i = 4 * (tex_width * tex_y + tex_x);
                let color = &texture[i..i + 3];

                rgba.copy_from_slice(color);
            });
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
