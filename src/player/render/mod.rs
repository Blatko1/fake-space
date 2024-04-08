mod colors;
mod object;
mod platforms;
mod ray;
mod skybox;
mod wall;

use super::Player;
use crate::player::camera::Camera;
use crate::world::{RoomRef, World};
use glam::Vec3;

use self::object::ObjectDrawData;
use self::platforms::PlatformDrawData;
use self::ray::Ray;
use self::skybox::SkyboxSegment;
use self::wall::WallDrawData;

// Distance in tiles
const SPOTLIGHT_DISTANCE: f32 = 1.3;
const SPOTLIGHT_STRENGTH: f32 = 0.65;
const FLASHLIGHT_INTENSITY: f32 = 1.35;
const FLASHLIGHT_OUTER_RADIUS: f32 = 1.1;
const FLASHLIGHT_INNER_RADIUS: f32 = 0.65;
const FLASHLIGHT_DISTANCE: f32 = 18.0;

const NORMAL_Y_POSITIVE: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const NORMAL_Y_NEGATIVE: Vec3 = Vec3::new(0.0, -1.0, 0.0);
const NORMAL_X_POSITIVE: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const NORMAL_X_NEGATIVE: Vec3 = Vec3::new(-1.0, 0.0, 0.0);
const NORMAL_Z_POSITIVE: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const NORMAL_Z_NEGATIVE: Vec3 = Vec3::new(0.0, 0.0, -1.0);

#[derive(Debug, Copy, Clone)]
pub struct PointXZ<T> {
    pub x: T,
    pub z: T,
}

struct ColumnDrawer<'a> {
    world: &'a World,
    camera: &'a Camera,

    ray: Ray,
    top_draw_bound: usize,
    bottom_draw_bound: usize,

    current_room: RoomRef<'a>,
}

impl<'a> ColumnDrawer<'a> {
    fn new(ray: Ray, player: &'a Player, world: &'a World) -> Self {
        let camera = player.camera();

        Self {
            world,
            camera,

            ray,
            top_draw_bound: camera.view_width as usize,
            bottom_draw_bound: 0,

            current_room: world.get_room_data(player.current_room_id()),
        }
    }

    fn draw(&mut self, column: &mut [u8]) -> Vec<ObjectDrawData<'a>> {
        let mut encountered_objects = Vec::new();
        // DDA loop
        loop {
            let current_tile_x = self.ray.next_tile.x;
            let current_tile_z = self.ray.next_tile.z;

            // DDA steps
            {
                if self.ray.side_dist_x < self.ray.side_dist_z {
                    self.ray.wall_dist = self.ray.side_dist_x.max(0.0);
                    self.ray.next_tile.x += self.ray.step_x;
                    self.ray.side_dist_x += self.ray.delta_dist_x;
                    self.ray.hit_wall_side = Side::Vertical;
                    let wall_offset =
                        self.ray.origin.z + self.ray.wall_dist * self.ray.dir.z;
                    self.ray.wall_offset = wall_offset - wall_offset.floor();
                } else {
                    self.ray.wall_dist = self.ray.side_dist_z.max(0.0);
                    self.ray.next_tile.z += self.ray.step_z;
                    self.ray.side_dist_z += self.ray.delta_dist_z;
                    self.ray.hit_wall_side = Side::Horizontal;
                    let wall_offset =
                        self.ray.origin.x + self.ray.wall_dist * self.ray.dir.x;
                    self.ray.wall_offset = wall_offset - wall_offset.floor();
                }
            }

            // Tile which the ray just traveled over before hitting a wall.
            let current_tile = match self
                .current_room
                .segment
                .get_tile(current_tile_x, current_tile_z)
            {
                Some(&current_tile) => current_tile,
                None => break,
            };

            // Drawing top and bottom platforms
            let ground_platform = PlatformDrawData {
                texture_data: self.world.get_texture(current_tile.ground_tex),
                height_level: current_tile.ground_level,
                normal: NORMAL_Y_POSITIVE,
                draw_from_dist: self.ray.previous_wall_dist,
                draw_to_dist: self.ray.wall_dist,
            };
            let (_, drawn_to) = self.draw_platform(ground_platform, column);
            self.bottom_draw_bound = drawn_to;

            let ceiling_platform = PlatformDrawData {
                texture_data: self.world.get_texture(current_tile.ceiling_tex),
                height_level: current_tile.ceiling_level,
                normal: NORMAL_Y_NEGATIVE,
                draw_from_dist: self.ray.wall_dist,
                draw_to_dist: self.ray.previous_wall_dist,
            };
            let (drawn_from, _) = self.draw_platform(ceiling_platform, column);
            self.top_draw_bound = drawn_from;

            let mut next_tile = match self
                .current_room
                .segment
                .get_tile(self.ray.next_tile.x, self.ray.next_tile.z)
            {
                Some(&t) => t,
                None => break,
            };

            // Switch to the different room if portal is hit
            if let Some(src_dummy_portal) = next_tile.portal {
                let src_portal = self.current_room.get_portal(src_dummy_portal.id);
                let (dest_room, dest_portal) = match src_portal.link {
                    Some((room_id, portal_id)) => {
                        let dest_room = self.world.get_room_data(room_id);
                        let dest_portal = dest_room.get_portal(portal_id);
                        (dest_room, dest_portal)
                    }
                    None => break,
                };
                self.ray.portal_teleport(src_portal, dest_portal);

                self.current_room = dest_room;
                next_tile = match self.current_room.segment.get_tile(
                    dest_portal.position.x as i64,
                    dest_portal.position.z as i64,
                ) {
                    Some(&t) => t,
                    None => break,
                };
            }

            if let Some(model_id) = next_tile.voxel_model {
                let model_data = self.world.get_voxel_model(model_id);
                let dimensions = model_data.dimension as f32;
                let pos = Vec3::new(
                    next_tile.position.x as f32 * dimensions,
                    next_tile.ground_level * dimensions * 0.5,
                    next_tile.position.z as f32 * dimensions,
                );
                encountered_objects.push(ObjectDrawData {
                    pos,
                    model_data,
                    ray: self.ray,
                    ambient_light_intensity: self
                        .current_room
                        .data
                        .ambient_light_intensity(),
                    bottom_draw_bound: self.bottom_draw_bound,
                    top_draw_bound: self.top_draw_bound,
                });
            }

            // Draw bottom wall
            let bottom_wall_data = WallDrawData {
                texture_data: self.world.get_texture(next_tile.bottom_wall_tex),
                bottom_wall_level: next_tile.bottom_level,
                top_wall_level: next_tile.ground_level,
            };
            let (_, drawn_to) = self.draw_wall(bottom_wall_data, column);
            self.bottom_draw_bound = drawn_to;

            // Draw top wall
            let top_wall_data = WallDrawData {
                texture_data: self.world.get_texture(next_tile.top_wall_tex),
                bottom_wall_level: next_tile.ceiling_level,
                top_wall_level: next_tile.top_level,
            };
            let (drawn_from, _) = self.draw_wall(top_wall_data, column);
            self.top_draw_bound = drawn_from;

            self.ray.previous_wall_dist = self.ray.wall_dist;
        }

        encountered_objects
    }
}

pub fn cast_and_draw<'a, C>(player: &Player, world: &World, column_iter: C)
where
    C: Iterator<Item = &'a mut [u8]>,
{
    let camera = player.camera();
    let player_room = world.get_room_data(player.current_room_id());
    column_iter.enumerate().for_each(|(column_index, column)| {
        let ray = Ray::cast_with_camera(column_index, camera);

        let skybox = SkyboxSegment::new(
            camera,
            ray,
            world.get_skybox_textures(player_room.data.skybox()),
        );
        skybox.draw_skybox(column);

        let mut column_drawer = ColumnDrawer::new(ray, player, world);
        let encountered_objects = column_drawer.draw(column);

        if !encountered_objects.is_empty() {
            object::draw_objects(encountered_objects, camera, column);
        }
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}
