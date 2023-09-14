// TODO problem! some textures below the walls 
// are bleeding out when further away
// TODO problem! trying to implement sprite entities
// is difficult due to existence of transparent walls 
// and their fully transparent parts 
// TODO problem! adding unsafe could improve performance
use crate::{
    map::{MapTile, TestMap},
    textures::TextureManager,
};

use super::{blend, Raycaster};

impl Raycaster {
    pub fn draw_top_bottom(
        &self,
        map: &TestMap,
        textures: &TextureManager,
        data: &mut [u8],
    ) {
        let ray_dir = self.dir - self.plane_h;

        // Precalculating for better performance
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        let half_height = self.height as f32 / 2.0;
        let width = self.width as usize;
        let height = self.height as usize;
        let f_height = self.height as f32;

        let mut past_floor_tile = MapTile::VOID;
        let mut floor_tex = textures.get_floor_tex(past_floor_tile.floor_tile);

        let mut past_ceiling_tile = MapTile::VOID;
        let mut ceiling_tex =
            textures.get_ceiling_tex(past_ceiling_tile.ceiling_tile);

        // DRAW FLOOR
        for y in ((height as i32 / 2 - self.y_shearing as i32) as usize)
            ..height
        {
            let p = y as f32 - half_height;

            let floor_row_dist = self.pos.y * f_height
                / (p + self.y_shearing)
                * self.plane_dist;
            let floor_step = tile_step_factor * floor_row_dist;
            let mut floor_pos = self.pos + ray_dir * floor_row_dist;

            let draw_floor_y_offset = y * self.four_width;

            let mut floor_tile_x = i32::MAX;
            let mut floor_tile_z = i32::MAX;

            for x in 0..width {
                //FLOOR
                let index = draw_floor_y_offset + x * 4;
                let rgba = &mut data[index..index + 4];
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
                            floor_tex = textures.get_floor_tex(tile.floor_tile);
                        }
                    }
                    floor_tile_x = current_floor_tile_x;
                    floor_tile_z = current_floor_tile_z;

                    let (texture, tex_width, tex_height) =
                        (floor_tex.texture, floor_tex.width as usize, floor_tex.height as usize);
                    let tx_floor = ((tex_width as f32 * floor_pos.x.fract())
                        as usize)
                        .min(tex_width - 1);
                    let ty_floor = ((tex_height as f32 * floor_pos.z.fract())
                        as usize)
                        .min(tex_height - 1);
                    let i_floor =
                        tex_width * 4 * ty_floor + tx_floor * 4;
                    let color = &texture[i_floor..i_floor + 4];
                    if alpha == 0 {
                        rgba.copy_from_slice(color);
                    } else {
                        rgba.copy_from_slice(&blend(color, rgba));
                    }
                }
                floor_pos.x += floor_step.x;
                floor_pos.z += floor_step.z;
            }
        }

        // DRAW CEILING
        for y in ((self.height as i32 / 2 + self.y_shearing as i32) as usize)
            ..height
        {
            let p = y as f32 - half_height;

            let ceiling_row_dist = (2.0 - self.pos.y) * f_height
                / (p - self.y_shearing)
                * self.plane_dist;
            let ceiling_step = tile_step_factor * ceiling_row_dist;
            let mut ceiling_pos = self.pos + ray_dir * ceiling_row_dist;

            let draw_ceiling_y_offset =
                (height - y - 1) * self.four_width;

            let mut ceiling_tile_x = i32::MAX;
            let mut ceiling_tile_z = i32::MAX;

            for x in 0..width {
                // CEILING
                let index = draw_ceiling_y_offset + x * 4;
                let rgba = &mut data[index..index + 4];
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

                    let tx_ceiling = ((tex_width as f32 * ceiling_pos.x.fract())
                        as usize)
                        .min(tex_width - 1);
                    let ty_ceiling = ((tex_height as f32 * ceiling_pos.z.fract())
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
            }
        }
    }
}
