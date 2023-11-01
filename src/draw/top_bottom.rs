use glam::Vec3;

// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! trying to implement sprite entities
// is difficult due to existence of transparent walls
// and their fully transparent parts
// TODO problem! adding unsafe could improve performance
use crate::{map::MapTile, textures::TextureDataRef};

use super::{blend, Raycaster};

impl Raycaster {
    pub fn draw_bottom_platform(
        &self,
        draw_from_wall_dist: f32,
        draw_to_wall_dist: f32,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        y_level: f32,
        texture_data: TextureDataRef<'_>,
        draw_x: u32,
        position_x: f32,
        position_z: f32,
        column: &mut [u8],
    ) -> usize {
        let (texture, tex_width, tex_height) = (
            texture_data.data,
            texture_data.width as usize,
            texture_data.height as usize,
        );
        if texture.is_empty() {
            return bottom_draw_bound;
        }

        // Draw from (alway drawing from bottom to top):
        let half_wall_pixel_height =
            self.f_half_height / draw_from_wall_dist * self.plane_dist;
        let pixels_to_top =
            half_wall_pixel_height * (y_level - self.pos.y) + self.y_shearing;
        let draw_from = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            self.f_half_height / draw_to_wall_dist * self.plane_dist;

        let pixels_to_top =
            half_wall_pixel_height * (y_level - self.pos.y) + self.y_shearing;
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(draw_from, top_draw_bound);

        let ray_dir = self.dir - self.plane_h;
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        column
            .chunks_exact_mut(4)
            .rev()
            .enumerate()
            .skip(self.height as usize - draw_to)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = ((self.pos.y - y_level) / 2.0) * self.f_height
                    / (y as f32 - self.f_height / 2.0 + self.y_shearing)
                    * self.plane_dist;
                let step = tile_step_factor * row_dist;
                let pos = self.pos + ray_dir * row_dist + step * draw_x as f32;
                let tex_x = ((tex_width as f32 * (pos.x - position_x))
                    as usize)
                    .min(tex_width - 1);
                let tex_y = ((tex_height as f32 * (pos.z - position_z))
                    as usize)
                    .min(tex_height - 1);
                let i = tex_width * 4 * tex_y + tex_x * 4;
                let color = &texture[i..i + 4];
                rgba.copy_from_slice(color);
            });
        /*if let Some(first) = column.chunks_exact_mut(4).nth(draw_to) {
            first.copy_from_slice(&[255, 255, 255, 255]);
        };
        if let Some(first) = column.chunks_exact_mut(4).nth(draw_from) {
            first.copy_from_slice(&[255, 0, 0, 255]);
        };*/

        draw_to
    }

    pub fn draw_top_platform(
        &self,
        draw_from_wall_dist: f32,
        draw_to_wall_dist: f32,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        y_level: f32,
        texture_data: TextureDataRef<'_>,
        draw_x: u32,
        position_x: f32,
        position_z: f32,
        column: &mut [u8],
    ) -> usize {
        let (texture, tex_width, tex_height) = (
            texture_data.data,
            texture_data.width as usize,
            texture_data.height as usize,
        );
        if texture.is_empty() {
            return top_draw_bound;
        }

        // Draw from:
        let half_wall_pixel_height =
            self.f_half_height / draw_to_wall_dist * self.plane_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (-y_level + self.pos.y) - self.y_shearing;
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            self.f_half_height / draw_from_wall_dist * self.plane_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (-y_level + self.pos.y) - self.y_shearing;
        let draw_to = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(draw_from, top_draw_bound);

        let ray_dir = self.dir - self.plane_h;
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        column
            .chunks_exact_mut(4)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = ((-self.pos.y + y_level) / 2.0) * self.f_height
                    / (y as f32 - self.f_height / 2.0 - self.y_shearing)
                    * self.plane_dist;
                let step = tile_step_factor * row_dist;
                let pos = self.pos + ray_dir * row_dist + step * draw_x as f32;
                let tex_x = ((tex_width as f32 * (pos.x - position_x))
                    as usize)
                    .min(tex_width - 1);
                let tex_y = ((tex_height as f32 * (pos.z - position_z))
                    as usize)
                    .min(tex_height - 1);
                let i = tex_width * 4 * tex_y + tex_x * 4;
                let color = &texture[i..i + 4];
                rgba.copy_from_slice(color);
            });
        /*if let Some(first) = column.chunks_exact_mut(4).nth(draw_to) {
            first.copy_from_slice(&[255, 255, 255, 255]);
        };
        if let Some(first) = column.chunks_exact_mut(4).nth(draw_from) {
            first.copy_from_slice(&[255, 0, 0, 255]);
        };*/

        draw_from
    }

    /*pub fn draw_top_bottom(
        &self,
        map: &TestMap,
        textures: &TextureManager,
        data: &mut [u8],
    ) {
        let ray_dir = self.dir - self.plane_h;

        // Precalculating for better performance
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        let width = self.width as usize;
        let height = self.height as usize;

        let mut past_floor_tile = MapTile::VOID;
        let mut floor_tex = textures.get_floor_tex(past_floor_tile.floor_tile);

        let mut past_ceiling_tile = MapTile::VOID;
        let mut ceiling_tex =
            textures.get_ceiling_tex(past_ceiling_tile.ceiling_tile);

        // DRAW FLOOR
        data.chunks_exact_mut(width * 4)
            .skip((height as i32 / 2 - self.y_shearing as i32) as usize)
            .enumerate()
            .for_each(|(y, row)| {
                let floor_row_dist =
                    self.pos.y * self.f_height / (y as f32) * self.plane_dist;
                let floor_step = tile_step_factor * floor_row_dist;
                let mut floor_pos = self.pos + ray_dir * floor_row_dist;

                let mut floor_tile_x = i32::MAX;
                let mut floor_tile_z = i32::MAX;
                row.chunks_exact_mut(4).for_each(|rgba| {
                    let alpha = rgba[3];
                    if alpha != 255 {
                        let current_floor_tile_x = floor_pos.x as i32;
                        let current_floor_tile_z = floor_pos.z as i32;
                        if floor_tile_x != current_floor_tile_x
                            || floor_tile_z != current_floor_tile_z
                        {
                            let tile = map.get_tile(
                                current_floor_tile_x as usize,
                                current_floor_tile_z as usize,
                            );
                            if past_floor_tile != tile {
                                past_floor_tile = tile;
                                floor_tex =
                                    textures.get_floor_tex(tile.floor_tile);
                            }
                        }
                        floor_tile_x = current_floor_tile_x;
                        floor_tile_z = current_floor_tile_z;

                        let (texture, tex_width, tex_height) = (
                            floor_tex.texture,
                            floor_tex.width as usize,
                            floor_tex.height as usize,
                        );
                        let tx_floor =
                            ((tex_width as f32 * floor_pos.x.fract()) as usize)
                                .min(tex_width - 1);
                        let ty_floor = ((tex_height as f32
                            * floor_pos.z.fract())
                            as usize)
                            .min(tex_height - 1);
                        let i_floor = tex_width * 4 * ty_floor + tx_floor * 4;
                        let color = &texture[i_floor..i_floor + 4];
                        if alpha == 0 {
                            rgba.copy_from_slice(color);
                        } else {
                            rgba.copy_from_slice(&blend(color, rgba));
                        }
                    }
                    floor_pos.x += floor_step.x;
                    floor_pos.z += floor_step.z;
                });
            });

        // DRAW CEILING
        data.chunks_exact_mut(width * 4)
            .take((height as i32 / 2 - self.y_shearing as i32) as usize)
            .rev()
            .enumerate()
            .for_each(|(y, row)| {
                let ceiling_row_dist = (2.0 - self.pos.y) * self.f_height
                    / (y as f32)
                    * self.plane_dist;
                let ceiling_step = tile_step_factor * ceiling_row_dist;
                let mut ceiling_pos = self.pos + ray_dir * ceiling_row_dist;

                let mut ceiling_tile_x = i32::MAX;
                let mut ceiling_tile_z = i32::MAX;
                row.chunks_exact_mut(4).for_each(|rgba| {
                    let alpha = rgba[3];
                    if alpha != 255 {
                        let current_ceiling_tile_x = ceiling_pos.x as i32;
                        let current_ceiling_tile_z = ceiling_pos.z as i32;
                        if ceiling_tile_x != current_ceiling_tile_x
                            || ceiling_tile_z != current_ceiling_tile_z
                        {
                            let tile = map.get_tile(
                                current_ceiling_tile_x as usize,
                                current_ceiling_tile_z as usize,
                            );
                            if past_ceiling_tile != tile {
                                past_ceiling_tile = tile;
                                ceiling_tex =
                                    textures.get_ceiling_tex(tile.ceiling_tile);
                            }
                        }
                        ceiling_tile_x = current_ceiling_tile_x;
                        ceiling_tile_z = current_ceiling_tile_z;
                        let (texture, tex_width, tex_height) = (
                            ceiling_tex.texture,
                            ceiling_tex.width as usize,
                            ceiling_tex.height as usize,
                        );

                        let tx_ceiling = ((tex_width as f32
                            * ceiling_pos.x.fract())
                            as usize)
                            .min(tex_width - 1);
                        let ty_ceiling = ((tex_height as f32
                            * ceiling_pos.z.fract())
                            as usize)
                            .min(tex_height - 1);
                        let i_ceiling =
                            tex_width * 4 * ty_ceiling + tx_ceiling * 4;
                        let color = &texture[i_ceiling..i_ceiling + 4];
                        if alpha == 0 {
                            rgba.copy_from_slice(color);
                        } else {
                            rgba.copy_from_slice(&blend(color, rgba));
                        }
                    }
                    ceiling_pos.x += ceiling_step.x;
                    ceiling_pos.z += ceiling_step.z;
                });
            });
    }*/
}
