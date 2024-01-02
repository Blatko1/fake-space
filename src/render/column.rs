use glam::Vec3;

use crate::{
    textures::TextureManager,
    world::world::{RoomDataRef, Tile, World},
};

use super::{Ray, RayCaster, Side};

// TODO maybe remove clone and copy
#[derive(Debug, Clone, Copy)]
pub struct DrawParams {
    pub closer_wall_dist: f32,
    pub further_wall_dist: f32,
    pub tile: Tile,
    pub tile_x: f32,
    pub tile_z: f32,
    pub side: Side,
    pub wall_offset: f32,
    //pub delta_dist_x: f32,
    //pub delta_dist_z: f32,
}

pub struct ColumnRenderer<'a> {
    pub(super) caster: &'a RayCaster,
    pub(super) ray: Ray,
    pub(super) bottom_draw_bound: usize,
    pub(super) top_draw_bound: usize,

    portals_passed: u32,
    //previous_perp_wall_dist: f32,
}
impl<'a> ColumnRenderer<'a> {
    pub fn new(caster: &'a RayCaster, ray: Ray) -> Self {
        let previous_perp_wall_dist = 0.0;
        let bottom_draw_bound = 0;
        let top_draw_bound = caster.height as usize;

        Self {
            caster,
            ray,
            bottom_draw_bound,
            top_draw_bound,

            portals_passed: 0,
            //previous_perp_wall_dist: todo!(),
        }
    }
    pub fn draw(mut self, world: &'a World, column: &mut [u8]) {
        // ====================================================
        //    | LOOP OVER THE RAY PATH AND DRAW HORIZONTAL |
        //    | PLATFORMS AND VERTICAL PLATFORMS (WALLS)   |
        // ====================================================
        let caster = self.caster;
        let ray = self.ray;
        let texture_manager = world.texture_manager();
        let mut next_tile_x = ray.next_tile_x;
        let mut next_tile_z = ray.next_tile_z;
        let mut side_dist_x = ray.side_dist_x;
        let mut side_dist_z = ray.side_dist_z;

        let mut room = world.get_current_room_data();
        let mut segment = room.segment;
        let mut previous_perp_wall_dist = 0.0;
        loop {
            let current_tile_x = next_tile_x;
            let current_tile_z = next_tile_z;
            // DDA loop
            let (side, perp_wall_dist, wall_offset) = if side_dist_x < side_dist_z {
                let dist_to_wall = side_dist_x.max(0.0);
                let wall_offset = caster.origin.z + dist_to_wall * ray.dir.z;
                next_tile_x += ray.step_x;
                side_dist_x += ray.delta_dist_x;
                (
                    Side::Vertical,
                    dist_to_wall,
                    wall_offset - wall_offset.floor(),
                )
            } else {
                let dist_to_wall = side_dist_z.max(0.0);
                let wall_offset = caster.origin.x + dist_to_wall * ray.dir.x;
                next_tile_z += ray.step_z;
                side_dist_z += ray.delta_dist_z;
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
            // Switch to the different room if portal is hit
            if let Some(portal) = current_tile.portal {
                self.portals_passed += 1;
                if self.portals_passed >= 4 {
                    return;
                }
                let Some((dest_room_id, dest_portal_id)) = room.portals[portal.id.0].connection else {
                    break;
                };
                room = world.get_room_data(dest_room_id);
                segment = room.segment;
                let dest_portal = &room.portals[dest_portal_id.0];
                //next_tile_x = dest_portal.
            }
            let mut params = DrawParams {
                closer_wall_dist: previous_perp_wall_dist,
                further_wall_dist: perp_wall_dist,
                tile: current_tile,
                tile_x: current_tile_x,
                tile_z: current_tile_z,
                side,
                wall_offset,
            };

            // Drawing top and bottom platforms
            let drawn_to = self.draw_bottom_platform(params, texture_manager, column);
            self.bottom_draw_bound = drawn_to;
            let drawn_from = self.draw_top_platform(params, texture_manager, column);
            self.top_draw_bound = drawn_from;

            let Some(&next_tile) =
                segment.get_tile(next_tile_x as i32, next_tile_z as i32)
            else {
                break;
            };
            params.tile = next_tile;
            // Drawing top and bottom walls
            let drawn_to = self.draw_bottom_wall(params, texture_manager, column);
            self.bottom_draw_bound = drawn_to;
            let drawn_from = self.draw_top_wall(params, texture_manager, column);
            self.top_draw_bound = drawn_from;

            previous_perp_wall_dist = perp_wall_dist;
        }
    }
}
