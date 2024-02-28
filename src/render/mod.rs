pub mod camera;
mod colors;
mod object;
mod platforms;
mod ray;
mod skybox;
mod wall;

use crate::render::camera::Camera;
use crate::render::object::ObjectDrawData;
use crate::world::{SkyboxTextures, Tile, World};
use crate::{player::Player, world::textures::TextureManager};
use glam::Vec3;

use self::ray::Ray;

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

pub fn cast_and_draw<'a, C>(player: &Player, world: &World, column_iter: C)
where
    C: Iterator<Item = &'a mut [u8]>,
{
    let camera = player.get_camera();
    let texture_manager = world.texture_manager();
    let model_manager = world.voxel_model_manager();
    column_iter.enumerate().for_each(|(column_index, column)| {
        let mut room = world.get_room_data(player.get_current_room_id());
        let mut segment = room.segment;
        let mut encountered_objects = Vec::new();

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
            let current_tile_x = params.ray.next_tile.x;
            let current_tile_z = params.ray.next_tile.z;

            // DDA steps
            {
                let ray = &mut params.ray;
                if ray.side_dist_x < ray.side_dist_z {
                    ray.wall_dist = ray.side_dist_x.max(0.0);
                    ray.next_tile.x += ray.step_x;
                    ray.side_dist_x += ray.delta_dist_x;
                    ray.hit_wall_side = Side::Vertical;
                    let wall_offset = ray.origin.z + ray.wall_dist * ray.dir.z;
                    ray.wall_offset = wall_offset - wall_offset.floor();
                } else {
                    ray.wall_dist = ray.side_dist_z.max(0.0);
                    ray.next_tile.z += ray.step_z;
                    ray.side_dist_z += ray.delta_dist_z;
                    ray.hit_wall_side = Side::Horizontal;
                    let wall_offset = ray.origin.x + ray.wall_dist * ray.dir.x;
                    ray.wall_offset = wall_offset - wall_offset.floor();
                }
            }

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

            let mut next_tile =
                match segment.get_tile(params.ray.next_tile.x, params.ray.next_tile.z) {
                    Some(&t) => t,
                    None => break,
                };

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
                params.ray.portal_teleport(src_portal, dest_portal);
                params.skybox = room.data.get_skybox();
                params.ambient_light = room.data.get_ambient_light();
            }

            if let Some(model_id) = next_tile.voxel_model {
                encountered_objects.push(ObjectDrawData {
                    pos_x: next_tile.position.x,
                    pos_z: next_tile.position.z,
                    pos_y: next_tile.ground_level,
                    model_data: model_manager.get_model(model_id),
                    ray: params.ray,
                    ambient_light: params.ambient_light,
                    bottom_draw_bound: params.bottom_draw_bound,
                    top_draw_bound: params.top_draw_bound,
                });
            }

            params.tile = next_tile;
            // Drawing top and bottom walls
            let drawn_to = wall::draw_bottom_wall(params, column);
            params.bottom_draw_bound = drawn_to;
            let drawn_from = wall::draw_top_wall(params, column);
            params.top_draw_bound = drawn_from;

            params.ray.previous_wall_dist = params.ray.wall_dist;
        }
        object::draw_objects(encountered_objects, camera, column);
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
