use crate::{
    map::Map,
    textures::TextureManager,
};

use super::{blend, Raycaster};

impl Raycaster {
    pub fn draw_floor_and_ceiling(
        &self,
        map: &Map,
        textures: &TextureManager,
        data: &mut [u8],
    ) {
        let ray_dir_x0 = self.dir.x - self.plane_h.x;
        let ray_dir_z0 = self.dir.z - self.plane_h.z;
        let ray_dir_x1 = self.dir.x + self.plane_h.x;
        let ray_dir_z1 = self.dir.z + self.plane_h.z;
        let pos_y = self.pos.y * self.height as f32;

        // For better performance
        let pos_y_div_aspect = pos_y / self.aspect;
        let ray_dir_x1_minus_x0 = ray_dir_x1 - ray_dir_x0;
        let ray_dir_z1_minus_z0 = ray_dir_z1 - ray_dir_z0;

        let mut color = [0; 4];

        for y in self.height / 2..self.height {
            let p = y as f32 - self.height as f32 / 2.0;

            let floor_row_dist = pos_y_div_aspect / p;
            let floor_step_x =
                floor_row_dist * ray_dir_x1_minus_x0 * self.width_recip;
            let floor_step_z =
                floor_row_dist * ray_dir_z1_minus_z0 * self.width_recip;
            let mut floor_x = self.pos.x + floor_row_dist * ray_dir_x0;
            let mut floor_z = self.pos.z + floor_row_dist * ray_dir_z0;

            let ceil_row_dist = floor_row_dist * 2.0;
            let ceil_step_x =
                ceil_row_dist * ray_dir_x1_minus_x0 * self.width_recip;
            let ceil_step_z =
                ceil_row_dist * ray_dir_z1_minus_z0 * self.width_recip;
            let mut ceil_x = self.pos.x + ceil_row_dist * ray_dir_x0;
            let mut ceil_z = self.pos.z + ceil_row_dist * ray_dir_z0;

            let draw_ceiling_y_offset = (self.height - y - 1) * 4 * self.width;
            let draw_floor_y_offset = y * 4 * self.width;

            for x in 0..self.width {
                //FLOOR
                {
                    let floor_tile_x = floor_x as i32;
                    let floor_tile_z = floor_z as i32;
                    let tile = map.get_value(floor_tile_x, floor_tile_z);
                    let tex = textures.get_floor_tex(tile);
                    let (texture, tex_width, tex_height) =
                        (tex.texture, tex.width, tex.height);
                    let index = (draw_floor_y_offset + x * 4) as usize;
                    let rgba = &mut data[index..index + 4];
                    let alpha = rgba[3];
                    if alpha != 255 {
                        let tx_floor = (tex_width as f32 * floor_x.fract())
                            as u32
                            & (tex_width - 1);
                        let ty_floor = (tex_height as f32 * floor_z.fract())
                            as u32
                            & (tex_height - 1);
                        let i_floor =
                            (tex_width * 4 * ty_floor + tx_floor * 4) as usize;
                        color.copy_from_slice(&texture[i_floor..i_floor + 4]);
                        if alpha == 0 {
                            rgba.copy_from_slice(&color);
                        } else {
                            rgba.copy_from_slice(&blend(&color, rgba));
                        }
                    }
                    floor_x += floor_step_x;
                    floor_z += floor_step_z;
                }

                // CEILING
                {
                    let ceil_tile_x = ceil_x as i32;
                    let ceil_tile_z = ceil_z as i32;
                    let tile = map.get_value(ceil_tile_x, ceil_tile_z);
                    let tex = textures.get_ceiling_tex(tile);
                    let (texture, tex_width, tex_height) =
                        (tex.texture, tex.width, tex.height);
                    let index = (draw_ceiling_y_offset + x * 4) as usize;
                    let rgba = &mut data[index..index + 4];
                    let alpha = rgba[3];
                    if alpha != 255 {
                        let tx_ceiling = (tex_width as f32 * ceil_x.fract())
                            as u32
                            & (tex_width - 1);
                        let ty_ceiling = (tex_height as f32 * ceil_z.fract())
                            as u32
                            & (tex_height - 1);

                        let i_ceiling = (tex_width * 4 * ty_ceiling
                            + tx_ceiling * 4)
                            as usize;

                        // CEILING
                        color.copy_from_slice(
                            &texture[i_ceiling..i_ceiling + 4],
                        );
                        if alpha == 0 {
                            rgba.copy_from_slice(&color);
                        } else {
                            rgba.copy_from_slice(&blend(&color, rgba));
                        }
                    }
                    ceil_x += ceil_step_x;
                    ceil_z += ceil_step_z;
                }
            }
        }
    }
}
