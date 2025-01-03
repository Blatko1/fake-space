mod platforms;
mod ray;
mod wall;

use crate::map::room::{RoomID, RoomRef};
use crate::map::Map;
use crate::player::Player;
use crate::textures::TextureArray;
use crate::raycaster::camera::Camera;

use self::platforms::PlatformDrawData;
use self::ray::Ray;
use self::wall::WallDrawData;

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

// TODO maybe draw first the floor, then bottom wall, then top wall, then ceiling

struct ColumnRenderer<'a> {
    map: &'a Map,
    textures: &'a TextureArray,
    camera: &'a Camera,

    ray: Ray,
    top_draw_bound: usize,
    bottom_draw_bound: usize,

    current_room: RoomRef<'a>,
    current_room_dimensions: (i64, i64),
}

impl<'a> ColumnRenderer<'a> {
    fn new(
        ray: Ray,
        camera: &'a Camera,
        player: &'a Player,
        map: &'a Map,
        textures: &'a TextureArray,
    ) -> Self {
        let current_room = map.get_room_data(player.current_room_id());
        let current_room_dimensions = current_room.segment.dimensions_i64();

        Self {
            map,
            textures,
            camera,

            ray,
            top_draw_bound: camera.view_height as usize,
            bottom_draw_bound: 0,

            current_room,
            current_room_dimensions,
        }
    }

    fn draw(&mut self, column: &mut [u8]) {
        // DDA loop
        loop {
            let current_tile_x = self.ray.next_tile.x as usize;
            let current_tile_z = self.ray.next_tile.z as usize;

            if self.ray.side_dist_x < self.ray.side_dist_z {
                self.ray.wall_dist = self.ray.side_dist_x.max(0.0);
                self.ray.next_tile.x += self.ray.step_x;
                if self.ray.next_tile.x >= self.current_room_dimensions.0
                    || self.ray.next_tile.x < 0
                {
                    break;
                }
                self.ray.side_dist_x += self.ray.delta_dist_x;
                self.ray.hit_wall_side = Side::Vertical;
                let wall_offset = self.ray.origin.z + self.ray.wall_dist * self.ray.dir.z;
                self.ray.wall_offset = wall_offset - wall_offset.floor();
            } else {
                self.ray.wall_dist = self.ray.side_dist_z.max(0.0);
                self.ray.next_tile.z += self.ray.step_z;
                if self.ray.next_tile.z >= self.current_room_dimensions.1
                    || self.ray.next_tile.z < 0
                {
                    break;
                }
                self.ray.side_dist_z += self.ray.delta_dist_z;
                self.ray.hit_wall_side = Side::Horizontal;
                let wall_offset = self.ray.origin.x + self.ray.wall_dist * self.ray.dir.x;
                self.ray.wall_offset = wall_offset - wall_offset.floor();
            }

            // Tile which the ray just traveled over before hitting a wall
            let current_tile = self
                .current_room
                .segment
                .get_tile_unchecked(current_tile_x, current_tile_z);

            // Draw ground platform
            let ground_platform = PlatformDrawData {
                texture_data: self.textures.get_texture_data(current_tile.ground_tex),
                height_level: current_tile.ground_level,
                draw_from_dist: self.ray.previous_wall_dist,
                draw_to_dist: self.ray.wall_dist,
            };
            let (_, drawn_to) = self.draw_platform(ground_platform, column);
            self.bottom_draw_bound = drawn_to;

            // Draw ceiling platform
            let ceiling_platform = PlatformDrawData {
                texture_data: self.textures.get_texture_data(current_tile.ceiling_tex),
                height_level: current_tile.ceiling_level,
                draw_from_dist: self.ray.wall_dist,
                draw_to_dist: self.ray.previous_wall_dist,
            };
            let (drawn_from, _) = self.draw_platform(ceiling_platform, column);
            self.top_draw_bound = drawn_from;

            // The tile ray hit
            let next_tile = self
                .current_room
                .segment
                .get_tile_unchecked(self.ray.next_tile.x as usize, self.ray.next_tile.z as usize);

            // Draw bottom wall
            let bottom_wall_data = WallDrawData {
                texture_data: self.textures.get_texture_data(next_tile.bottom_wall_tex),
                bottom_wall_level: next_tile.bottom_level,
                top_wall_level: next_tile.ground_level,
            };
            let (_, drawn_to) = self.draw_wall(bottom_wall_data, column);
            self.bottom_draw_bound = drawn_to;

            // Draw top wall
            let top_wall_data = WallDrawData {
                texture_data: self.textures.get_texture_data(next_tile.top_wall_tex),
                bottom_wall_level: next_tile.ceiling_level,
                top_wall_level: next_tile.top_level,
            };
            let (drawn_from, _) = self.draw_wall(top_wall_data, column);
            self.top_draw_bound = drawn_from;

            // Switch to the different room if portal is hit
            if let Some(src_dummy_portal) = next_tile.portal {
                let src_portal = self.current_room.get_portal(src_dummy_portal.id);
                match src_portal.link {
                    Some((room_id, portal_id)) => {
                        let dest_room = self.map.get_room_data(room_id);
                        let dest_portal = dest_room.get_portal(portal_id);
                        self.ray.portal_teleport(src_portal, dest_portal);
                        self.current_room = dest_room;
                        self.current_room_dimensions =
                            self.current_room.segment.dimensions_i64();
                    }
                    None => {
                        fill_black(column, self.bottom_draw_bound, self.top_draw_bound);
                        break;
                    }
                }
            }

            fill_black(column, self.bottom_draw_bound, self.top_draw_bound);

            self.ray.previous_wall_dist = self.ray.wall_dist;
        }
    }
}

pub fn cast_and_draw<'a, C>(
    camera: &Camera,
    player: &Player,
    map: &Map,
    textures: &TextureArray,
    column_iter: C,
) where
    C: Iterator<Item = &'a mut [u8]>,
{
    // TODO ask ChatGPT how to multitrhead this. 
    for (column_index, column) in column_iter.enumerate() {
        let ray = Ray::camera_cast(camera, column_index);

        let mut column_drawer =
            ColumnRenderer::new(ray, camera, player, map, textures);

        column_drawer.draw(column);
    }
}

fn fill_black(column: &mut [u8], bottom_bound: usize, top_bound: usize) {
    column[bottom_bound * 3..top_bound * 3].fill(0)
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}
