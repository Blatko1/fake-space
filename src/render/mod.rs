pub mod camera;
mod colors;
mod platforms;
mod ray;
mod voxel_model;
mod wall;

use crate::world::{Tile, World};
use crate::{player::Player, world::textures::TextureManager};

use self::ray::Ray;

#[derive(Debug, Copy, Clone)]
pub struct PointXZ<T> {
    pub x: T,
    pub z: T
}

pub fn cast_and_draw<'a, C>(player: &Player, world: &World, column_iter: C)
where
    C: Iterator<Item = &'a mut [u8]>,
{
    let cam = player.get_camera();
    column_iter.enumerate().for_each(|(x, column)| {
        let mut ray = Ray::cast_with_camera(x, cam);
        let texture_manager = world.texture_manager();
        let mut room = world.get_room_data(player.get_current_room_id());
        let mut segment = room.segment;

        let mut previous_perp_wall_dist = 0.0;
        let mut bottom_draw_bound = 0;
        let mut top_draw_bound = cam.view_height as usize;
        // DDA loop
        loop {
            let current_tile_x = ray.next_tile.x;
            let current_tile_z = ray.next_tile.z;

            let (side, perp_wall_dist, wall_offset) = if ray.side_dist_x < ray.side_dist_z
            {
                let dist_to_wall = ray.side_dist_x.max(0.0);
                let wall_offset = ray.origin.z + dist_to_wall * ray.dir.z;
                ray.next_tile.x += ray.step_x;
                ray.side_dist_x += ray.delta_dist_x;
                (
                    Side::Vertical,
                    dist_to_wall,
                    wall_offset - wall_offset.floor(),
                )
            } else {
                let dist_to_wall = ray.side_dist_z.max(0.0);
                let wall_offset = ray.origin.x + dist_to_wall * ray.dir.x;
                ray.next_tile.z += ray.step_z;
                ray.side_dist_z += ray.delta_dist_z;
                (
                    Side::Horizontal,
                    dist_to_wall,
                    wall_offset - wall_offset.floor(),
                )
            };

            // Tile which the ray just traveled over before hitting a wall.
            let Some(&current_tile) = segment.get_tile(current_tile_x, current_tile_z)
            else {
                break;
            };

            let mut params = DrawParams {
                closer_wall_dist: previous_perp_wall_dist,
                further_wall_dist: perp_wall_dist,
                bottom_draw_bound,
                top_draw_bound,
                tile: current_tile,
                tile_x: current_tile_x,
                tile_z: current_tile_z,
                side,
                wall_offset,
                ray,
                texture_manager,
            };

            // Drawing top and bottom platforms
            let drawn_to = platforms::draw_bottom_platform(cam, params, column);
            bottom_draw_bound = drawn_to;
            params.bottom_draw_bound = bottom_draw_bound;
            let drawn_from = platforms::draw_top_platform(cam, params, column);
            top_draw_bound = drawn_from;
            params.top_draw_bound = top_draw_bound;

            let mut next_tile = match segment.get_tile(ray.next_tile.x, ray.next_tile.z) {
                Some(&t) => t,
                None => break,
            };

            // Switch to the different room if portal is hit
            if let Some(src_dummy_portal) = next_tile.portal {
                let src_portal = room.get_portal(src_dummy_portal.id);
                let Some((dest_room_id, dest_portal_id)) = src_portal.link else {
                    break;
                };
                room = world.get_room_data(dest_room_id);
                segment = room.segment;
                let dest_portal = room.get_portal(dest_portal_id);
                next_tile = match segment.get_tile(dest_portal.position.x as i64, dest_portal.position.z as i64) {
                    Some(&t) => t,
                    None => break,
                };

                ray.portal_teleport(src_portal, dest_portal);

                params.ray = ray;
            }

            params.tile = next_tile;
            // Drawing top and bottom walls
            let drawn_to = wall::draw_bottom_wall(cam, params, column);
            bottom_draw_bound = drawn_to;
            let drawn_from = wall::draw_top_wall(cam, params, column);
            top_draw_bound = drawn_from;

            previous_perp_wall_dist = perp_wall_dist;
        }
    })
}

// TODO maybe remove clone and copy
#[derive(Clone, Copy)]
pub struct DrawParams<'a> {
    pub closer_wall_dist: f32,
    pub further_wall_dist: f32,
    pub bottom_draw_bound: usize,
    pub top_draw_bound: usize,
    pub tile: Tile,
    pub tile_x: i64,
    pub tile_z: i64,
    pub side: Side,
    pub wall_offset: f32,
    pub ray: Ray,
    pub texture_manager: &'a TextureManager
}

// TODO maybe remove clone and copy
#[derive(Debug, Clone, Copy)]
pub enum Side {
    Vertical,
    Horizontal,
}

// TODO convert to unsafe for speed
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
