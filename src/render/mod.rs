pub mod camera;
mod ray;
mod colors;
mod platforms;
mod voxel_model;
mod wall;

use glam::Vec3;

use crate::{World, player::{Player}, world::world::{Tile, PortalRotationDifference}, textures::TextureManager};

use self::{ray::Ray, camera::DEFAULT_PLANE_V};

pub fn cast_and_draw<'a, C>(player: &Player, world: &World, column_iter: C) where C: Iterator<Item = &'a mut [u8]> {
    let cam = player.get_camera();
    column_iter.enumerate().for_each(|(x, column)| {
        // ====================================================
        //    | LOOP OVER THE RAY PATH AND DRAW HORIZONTAL |
        //    | PLATFORMS AND VERTICAL PLATFORMS (WALLS)   |
        // ====================================================
        let mut ray = cam.cast_ray_for(x);
        let texture_manager = world.texture_manager();
        let mut room = world.get_room_data(player.get_current_room_id());
        let mut segment = room.segment;

        let mut portals_passed = 0;
        let mut previous_perp_wall_dist = 0.0;
        let mut bottom_draw_bound = 0;
        let mut top_draw_bound = cam.view_height as usize;
        // DDA loop
        loop {
            let current_tile_x = ray.next_tile_x;
            let current_tile_z = ray.next_tile_z;

            let (side, perp_wall_dist, wall_offset) = if ray.side_dist_x < ray.side_dist_z
            {
                let dist_to_wall = ray.side_dist_x.max(0.0);
                let wall_offset = ray.origin.z + dist_to_wall * ray.dir.z;
                ray.next_tile_x += ray.step_x;
                ray.side_dist_x += ray.delta_dist_x;
                (
                    Side::Vertical,
                    dist_to_wall,
                    wall_offset - wall_offset.floor(),
                )
            } else {
                let dist_to_wall = ray.side_dist_z.max(0.0);
                let wall_offset = ray.origin.x + dist_to_wall * ray.dir.x;
                ray.next_tile_z += ray.step_z;
                ray.side_dist_z += ray.delta_dist_z;
                (
                    Side::Horizontal,
                    dist_to_wall,
                    wall_offset - wall_offset.floor(),
                )
            };

            // ====================================================
            //  | DRAW TOP AND BOTTOM PLATFORMS OF CURRENT TILE |
            // ====================================================
            // Tile which the ray just traveled over before hitting a wall.
            let Some(&current_tile) =
                segment.get_tile(current_tile_x as i32, current_tile_z as i32)
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
            let drawn_to = platforms::draw_bottom_platform(&cam, params, column);
            bottom_draw_bound = drawn_to;
            params.bottom_draw_bound = bottom_draw_bound;
            let drawn_from = platforms::draw_top_platform(&cam, params, column);
            top_draw_bound = drawn_from;
            params.top_draw_bound = top_draw_bound;

            let mut next_tile = match segment.get_tile(ray.next_tile_x as i32, ray.next_tile_z as i32) {
                Some(&t) => t,
                None => break,
            };
            params.tile = next_tile;
            // Drawing top and bottom walls
            let drawn_to = wall::draw_bottom_wall(&cam, params, column);
            bottom_draw_bound = drawn_to;
            let drawn_from = wall::draw_top_wall(&cam, params, column);
            top_draw_bound = drawn_from;

            previous_perp_wall_dist = perp_wall_dist;

            // Switch to the different room if portal is hit
            if let Some(portal) = next_tile.portal {
                portals_passed += 1;
                if portals_passed >= 4 {
                    break;
                }
                let Some((dest_room_id, dest_portal_id)) =
                    room.portals[portal.id.0].connection
                else {
                    break;
                };
                room = world.get_room_data(dest_room_id);
                segment = room.segment;
                let dest_portal = &room.portals[dest_portal_id.0];
                let old_next_x = ray.next_tile_x;
                let old_next_z = ray.next_tile_z;
                ray.next_tile_x = dest_portal.local_position.0 as i64;
                ray.next_tile_z = dest_portal.local_position.1 as i64;
                ray.origin.x += (ray.next_tile_x - old_next_x) as f32;
                ray.origin.z += (ray.next_tile_z - old_next_z) as f32;
                let old_tile_ground_level = next_tile.level2;
                let next_tile = match segment.get_tile(ray.next_tile_x as i32, ray.next_tile_z as i32) {
                    Some(&t) => t,
                    None => break,
                };
                ray.origin.y -= old_tile_ground_level - next_tile.level2;

                let src_portal_dir = portal.direction;
                let dest_portal_dir = dest_portal.direction;
                match src_portal_dir.rotation_radian_difference(dest_portal_dir) {
                    PortalRotationDifference::None => (),
                    PortalRotationDifference::LeftDeg90 => todo!(),
                    PortalRotationDifference::RightDeg90 => todo!(),
                    PortalRotationDifference::Deg180 => {
                        ray.origin.x = 2.0*dest_portal.local_position.0 as f32 + 1.0 - ray.origin.x;
                        ray.origin.z = 2.0*dest_portal.local_position.1 as f32 + 1.0 - ray.origin.z;
                        ray.dir = -ray.dir;
                        ray.camera_dir = -ray.camera_dir;
                        ray.horizontal_plane =
                        Vec3::cross(DEFAULT_PLANE_V, ray.camera_dir) * cam.aspect / cam.plane_dist;
                        ray.step_x = -ray.step_x;
                        ray.step_z = -ray.step_z;
                    },
                }
            }
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
    pub texture_manager: &'a TextureManager,
    //pub delta_dist_x: f32,
    //pub delta_dist_z: f32,
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
        ((foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha) as u8),
        ((foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha) as u8),
        ((foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha) as u8),
        (255.0 * alpha + background[3] as f32 * inv_alpha) as u8,
    ]
}
