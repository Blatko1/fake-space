pub mod camera;
mod platform;
mod ray;
mod wall;

use glam::{Vec2, Vec3};
use platform::{PlatformRenderParams, PlatformType};
use ray::WallSide;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use rayon::slice::ParallelSliceMut;
use wall::WallRenderParams;

use crate::map::portal::Orientation;
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

    // Frequently used values
    y_shearing: f32,
    view_height: u32,
    half_view_height: f32,
    width_recip: f32,
}

impl<'a> FrameRenderer<'a> {
    pub fn new(
        camera: &'a Camera,
        player: &'a Player,
        map: &'a Map,
        textures: &'a TextureArray,
    ) -> Self {
        Self {
            camera,
            player,
            map,
            textures,

            y_shearing: camera.y_shearing,
            view_height: camera.view_height,
            half_view_height: camera.view_height as f32 * 0.5,
            width_recip: 1.0 / camera.view_width as f32,
        }
    }

    pub fn render_par(&mut self, pixel_buffer: &'a mut [u8]) {
        // TODO is there better multithreading?
        pixel_buffer
            .par_chunks_exact_mut(self.camera.view_height as usize * 3)
            .enumerate()
            .for_each(|(column_index, column)| {
                self.render_column(column_index, column);
            });
    }

    pub fn render(&mut self, pixel_buffer: &'a mut [u8]) {
        for (column_index, column) in pixel_buffer
            .chunks_exact_mut(self.camera.view_height as usize * 3)
            .enumerate()
        {
            self.render_column(column_index, column);
        }
    }

    // TODO maybe draw first the floor, then bottom wall, then top wall, then ceiling
    fn render_column(&self, column_index: usize, column: &mut [u8]) {
        let mut ray = Ray::new(self.camera, self.camera.origin, column_index);
        let static_ray = Ray::new_one_step(self.camera, Vec3::splat(0.5), column_index);

        let mut current_room = self.map.get_room_data(self.player.current_room_id());
        let mut current_room_dimensions = current_room.tilemap.dimensions_i64();

        let room_direction = current_room.data.direction;

        let mut skybox_textures = self
            .textures
            .get_skybox_textures(current_room.data.skybox());

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
                .tilemap
                .get_tile_unchecked(current_tile_x, current_tile_z);

            // Draw ground platform
            let params = PlatformRenderParams {
                ray,
                bottom_draw_bound,
                top_draw_bound,
                height: current_tile.ground_height,
                platform_type: PlatformType::Floor,
                texture: self.textures.get_texture_data(current_tile.ground_tex),
            };

            let (from, drawn_to) = self.render_platform(params, column);

            // Draw ceiling platform
            let params = PlatformRenderParams {
                ray,
                bottom_draw_bound,
                top_draw_bound,
                height: current_tile.ceiling_height,
                platform_type: PlatformType::Ceiling,
                texture: self.textures.get_texture_data(current_tile.ceiling_tex),
            };

            let (drawn_from, to) = self.render_platform(params, column);
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
                .tilemap
                .get_tile_unchecked(ray.next_tile.x as usize, ray.next_tile.z as usize);

            let params = WallRenderParams {
                ray,
                bottom_draw_bound,
                top_draw_bound,
                bottom_level: next_tile.bottom_height,
                top_level: next_tile.ground_height,
                texture: self.textures.get_texture_data(next_tile.bottom_wall_tex),
            };

            // Draw bottom wall
            let (from, drawn_to) = self.render_wall(params, column);
            let params = WallRenderParams {
                ray,
                bottom_draw_bound,
                top_draw_bound,
                bottom_level: next_tile.ceiling_height,
                top_level: next_tile.top_height,
                texture: self.textures.get_texture_data(next_tile.bottom_wall_tex),
            };
            // Draw top wall
            let (drawn_from, to) = self.render_wall(params, column);
            if from != bottom_draw_bound {
                //println!("wall bottom skiped!");
                self.render_skybox(
                    static_ray,
                    room_direction,
                    skybox_textures,
                    bottom_draw_bound,
                    from,
                    column,
                );
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
            if let Some(id) = next_tile.portal_id {
                let src_portal = current_room.get_portal(id);
                match src_portal.destination {
                    Some((room_id, dest_id)) => {
                        let dest_room = self.map.get_room_data(room_id);
                        //dest_room.data.direction
                        let dest_portal = dest_room.get_portal(dest_id);
                        let src_angle = f32::atan2(src_portal.direction.y, src_portal.direction.x);
                        let dest_angle = f32::atan2(-dest_portal.direction.y, -dest_portal.direction.x);
                        let diff = dest_angle - src_angle;
                        //panic!("diff: {}", diff);

                        //let rotation = src_portal.direction_difference(&dest_portal);
                        ray.rotate(diff);

                        let offset = Vec2::new(ray.origin.x, ray.origin.z) - src_portal.center;
                        let rotation = glam::mat2(Vec2::new(diff.cos(), diff.sin()), Vec2::new(-diff.sin(), diff.cos()));
                        let rotated_offset = rotation * offset;
                        let new_position = dest_portal.center + rotated_offset + (-dest_portal.direction);
                        ray.origin = Vec3::new(new_position.x, ray.origin.y + dest_portal.ground_height - src_portal.ground_height, new_position.y);

                        current_room = dest_room;
                        current_room_dimensions = current_room.tilemap.dimensions_i64();
                        skybox_textures = self
                            .textures
                            .get_skybox_textures(current_room.data.skybox());
                    }
                    None => {
                        fill_color(column, bottom_draw_bound, top_draw_bound, 0);
                        return;
                    }
                }
            }

            ray.previous_wall_dist = ray.wall_dist;
        }
        self.render_skybox(
            static_ray,
            room_direction,
            skybox_textures,
            bottom_draw_bound,
            top_draw_bound,
            column,
        );
    }

    fn render_skybox(
        &self,
        ray: Ray,
        room_direction: Vec2,
        skybox_textures: SkyboxTexturesRef,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        column: &mut [u8],
    ) {
        let wall_texture = match ray.wall_side {
            WallSide::North => skybox_textures.north,
            WallSide::East => skybox_textures.east,
            WallSide::South => skybox_textures.south,
            WallSide::West => skybox_textures.west,
        };

        let params = WallRenderParams {
            ray,
            bottom_draw_bound,
            top_draw_bound,
            bottom_level: -0.5,
            top_level: 1.5,
            texture: wall_texture,
        };

        self.render_wall(params, column);

        let params = PlatformRenderParams {
            ray,
            bottom_draw_bound,
            top_draw_bound,
            height: -0.5,
            platform_type: PlatformType::Floor,
            texture: skybox_textures.bottom,
        };

        // Draw ground platform
        self.render_platform(params, column);

        let params = PlatformRenderParams {
            ray,
            bottom_draw_bound,
            top_draw_bound,
            height: 1.5,
            platform_type: PlatformType::Ceiling,
            texture: skybox_textures.top,
        };

        // Draw ceiling platform
        self.render_platform(params, column);
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

pub struct blueprint<'a> {
    data: &'a mut [u8],
    start_offset: usize,
}

impl<'a> blueprint<'a> {
    pub fn new(from: usize, to: usize, column: &'a mut [u8]) -> Self {
        Self {
            data: &mut column[from..to],
            start_offset: from,
        }
    }
}
