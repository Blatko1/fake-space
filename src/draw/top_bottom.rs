use crate::{
    map::{TestMap, Tile},
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
        let ray_dir_x0 = self.dir.x - self.plane_h.x;
        let ray_dir_z0 = self.dir.z - self.plane_h.z;
        let ray_dir_x1 = self.dir.x + self.plane_h.x;
        let ray_dir_z1 = self.dir.z + self.plane_h.z;
        let pos_y = self.pos.y * self.height as f32;

        // Precalculating for better performance
        let ray_dir_x1_minus_x0 = ray_dir_x1 - ray_dir_x0;
        let ray_dir_z1_minus_z0 = ray_dir_z1 - ray_dir_z0;
        let floor_step_x_factor = ray_dir_x1_minus_x0 * self.width_recip;
        let floor_step_z_factor = ray_dir_z1_minus_z0 * self.width_recip;
        let half_height = self.height as f32 / 2.0;

        let mut past_floor_tile = Tile::Empty;
        let mut floor_tex = textures.get_floor_tex(past_floor_tile);

        let mut past_ceiling_tile = Tile::Empty;
        let mut ceiling_tex = textures.get_ceiling_tex(past_ceiling_tile);

        // DRAW FLOOR
        for y in ((self.height as i32 / 2 - self.y_shearing as i32) as u32)
            ..self.height
        {
            let p = y as f32 - half_height;

            let floor_row_dist =
                pos_y / (p + self.y_shearing) * self.plane_dist;
            let floor_step_x = floor_row_dist * floor_step_x_factor;
            let floor_step_z = floor_row_dist * floor_step_z_factor;
            let mut floor_x = self.pos.x + floor_row_dist * ray_dir_x0;
            let mut floor_z = self.pos.z + floor_row_dist * ray_dir_z0;

            let draw_floor_y_offset = y * self.four_width as u32;

            let mut floor_tile_x = i32::MAX;
            let mut floor_tile_z = i32::MAX;

            for x in 0..self.width {
                //FLOOR
                let index = (draw_floor_y_offset + x * 4) as usize;
                let rgba = &mut data[index..index + 4];
                let alpha = rgba[3];
                if alpha != 255 {
                    let current_floor_tile_x = floor_x as i32;
                    let current_floor_tile_z = floor_z as i32;
                    if floor_tile_x != current_floor_tile_x
                        || floor_tile_z != current_floor_tile_z
                    {
                        let tile = map.get_top_bottom_tile(
                            current_floor_tile_x as usize,
                            current_floor_tile_z as usize,
                        );
                        if past_floor_tile != tile {
                            past_floor_tile = tile;
                            floor_tex = textures.get_floor_tex(tile);
                        }
                    }
                    floor_tile_x = current_floor_tile_x;
                    floor_tile_z = current_floor_tile_z;

                    let (texture, tex_width, tex_height) =
                        (floor_tex.texture, floor_tex.width, floor_tex.height);
                    let tx_floor = ((tex_width as f32 * floor_x.fract())
                        as u32)
                        .min(tex_width - 1);
                    let ty_floor = ((tex_height as f32 * floor_z.fract())
                        as u32)
                        .min(tex_height - 1);
                    let i_floor =
                        (tex_width * 4 * ty_floor + tx_floor * 4) as usize;
                    let color = &texture[i_floor..i_floor + 4];
                    if alpha == 0 {
                        rgba.copy_from_slice(color);
                    } else {
                        rgba.copy_from_slice(&blend(color, rgba));
                    }
                }
                floor_x += floor_step_x;
                floor_z += floor_step_z;
            }
        }

        // DRAW CEILING
        for y in ((self.height as i32 / 2 + self.y_shearing as i32) as u32)
            ..self.height
        {
            let p = y as f32 - half_height;

            let ceiling_row_dist =
                pos_y / (p - self.y_shearing) * self.plane_dist * 2.0;
            let ceiling_step_x = ceiling_row_dist * floor_step_x_factor;
            let ceiling_step_z = ceiling_row_dist * floor_step_z_factor;
            let mut ceiling_x = self.pos.x + ceiling_row_dist * ray_dir_x0;
            let mut ceiling_z = self.pos.z + ceiling_row_dist * ray_dir_z0;

            let draw_ceiling_y_offset =
                (self.height as u32 - y - 1) * self.four_width as u32;

            let mut ceiling_tile_x = i32::MAX;
            let mut ceiling_tile_z = i32::MAX;

            for x in 0..self.width {
                // CEILING
                let index = (draw_ceiling_y_offset + x * 4) as usize;
                let rgba = &mut data[index..index + 4];
                let alpha = rgba[3];
                if alpha != 255 {
                    let current_ceiling_tile_x = ceiling_x as i32;
                    let current_ceiling_tile_z = ceiling_z as i32;
                    if ceiling_tile_x != current_ceiling_tile_x
                        || ceiling_tile_z != current_ceiling_tile_z
                    {
                        let tile = map.get_top_bottom_tile(
                            current_ceiling_tile_x as usize,
                            current_ceiling_tile_z as usize,
                        );
                        if past_ceiling_tile != tile {
                            past_ceiling_tile = tile;
                            ceiling_tex = textures.get_ceiling_tex(tile);
                        }
                    }
                    ceiling_tile_x = current_ceiling_tile_x;
                    ceiling_tile_z = current_ceiling_tile_z;
                    let (texture, tex_width, tex_height) = (
                        ceiling_tex.texture,
                        ceiling_tex.width,
                        ceiling_tex.height,
                    );

                    let tx_ceiling = ((tex_width as f32 * ceiling_x.fract())
                        as u32)
                        .min(tex_width - 1);
                    let ty_ceiling = ((tex_height as f32 * ceiling_z.fract())
                        as u32)
                        .min(tex_height - 1);
                    let i_ceiling =
                        (tex_width * 4 * ty_ceiling + tx_ceiling * 4) as usize;
                    let color = &texture[i_ceiling..i_ceiling + 4];
                    if alpha == 0 {
                        rgba.copy_from_slice(color);
                    } else {
                        rgba.copy_from_slice(&blend(color, rgba));
                    }
                }
                ceiling_x += ceiling_step_x;
                ceiling_z += ceiling_step_z;
            }
        }
    }
}
