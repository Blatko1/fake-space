use glam::Vec3;

use crate::{
    textures::TextureManager,
    world::world::{Tile, World},
};

use super::{RayCaster, Side};

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

impl RayCaster {
    pub fn draw_column(&self, x: usize, world: &World, column: &mut [u8]) {
        // ====================================================
        //    | LOOP OVER THE RAY PATH AND DRAW HORIZONTAL |
        //    | PLATFORMS AND VERTICAL PLATFORMS (WALLS)   |
        // ====================================================
        let mut ray = self.cast_ray(x);
        let texture_manager = world.texture_manager();
        let mut room = world.get_current_room_data();
        let mut segment = room.segment;

        let mut portals_passed = 0;
        let mut previous_perp_wall_dist = 0.0;
        let mut bottom_draw_bound = 0;
        let mut top_draw_bound = self.height as usize;
        // DDA loop
        loop {
            let current_tile_x = ray.next_tile_x;
            let current_tile_z = ray.next_tile_z;

            let (side, perp_wall_dist, wall_offset) = if ray.side_dist_x < ray.side_dist_z
            {
                let dist_to_wall = ray.side_dist_x.max(0.0);
                let wall_offset = self.origin.z + dist_to_wall * ray.dir.z;
                ray.next_tile_x += ray.step_x;
                ray.side_dist_x += ray.delta_dist_x;
                (
                    Side::Vertical,
                    dist_to_wall,
                    wall_offset - wall_offset.floor(),
                )
            } else {
                let dist_to_wall = ray.side_dist_z.max(0.0);
                let wall_offset = self.origin.x + dist_to_wall * ray.dir.x;
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
            let drawn_to = self.draw_bottom_platform(params, column);
            bottom_draw_bound = drawn_to;
            params.bottom_draw_bound = bottom_draw_bound;
            let drawn_from = self.draw_top_platform(params, column);
            top_draw_bound = drawn_from;
            params.top_draw_bound = top_draw_bound;

            let Some(&next_tile) =
                segment.get_tile(ray.next_tile_x as i32, ray.next_tile_z as i32)
            else {
                break;
            };
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
                ray.origin.x += (-old_next_x + ray.next_tile_x) as f32;
                ray.origin.z += (-old_next_z + ray.next_tile_z) as f32;
                params.ray = ray;
            }
            let Some(&next_tile) =
                segment.get_tile(ray.next_tile_x as i32, ray.next_tile_z as i32)
            else {
                break;
            };
            params.tile = next_tile;
            // Drawing top and bottom walls
            let drawn_to = self.draw_bottom_wall(params, column);
            bottom_draw_bound = drawn_to;
            let drawn_from = self.draw_top_wall(params, column);
            top_draw_bound = drawn_from;

            previous_perp_wall_dist = perp_wall_dist;
        }
    }

    fn cast_ray(&self, x: usize) -> Ray {
        Ray::cast_for_column(x, self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub x: usize,
    pub origin: Vec3,
    pub dir: Vec3,
    pub caster_dir: Vec3,
    pub delta_dist_x: f32,
    pub delta_dist_z: f32,
    pub side_dist_x: f32,
    pub side_dist_z: f32,
    pub next_tile_x: i64,
    pub next_tile_z: i64,
    pub step_x: i64,
    pub step_z: i64,
}

impl Ray {
    pub fn cast_for_column(x: usize, caster: &RayCaster) -> Self {
        let origin = caster.origin;
        let caster_dir = caster.dir;

        // X-coordinate on the horizontal camera plane (range [-1.0, 1.0])
        let plane_x = 2.0 * (x as f32 * caster.width_recip) - 1.0;
        // Ray direction for current pixel column
        let ray_dir = caster_dir + caster.plane_h * plane_x;
        // Length of ray from one x/z side to next x/z side on the tile_map
        let delta_dist_x = 1.0 / ray_dir.x.abs();
        let delta_dist_z = 1.0 / ray_dir.z.abs();
        // Distance to nearest x side
        let side_dist_x = delta_dist_x
            * if ray_dir.x < 0.0 {
                origin.x.fract()
            } else {
                1.0 - origin.x.fract()
            };
        // Distance to nearest z side
        let side_dist_z = delta_dist_z
            * if ray_dir.z < 0.0 {
                origin.z.fract()
            } else {
                1.0 - origin.z.fract()
            };

        Self {
            x,
            origin,
            dir: ray_dir,
            caster_dir,
            delta_dist_x,
            delta_dist_z,
            side_dist_x,
            side_dist_z,
            // Coordinates of the map tile the raycaster is in
            next_tile_x: origin.x as i64,
            next_tile_z: origin.z as i64,
            step_x: ray_dir.x.signum() as i64,
            step_z: ray_dir.z.signum() as i64,
        }
    }
}
