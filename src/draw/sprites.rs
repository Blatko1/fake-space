use crate::{world::World, map::Tile, textures::TextureManager};

use super::{Raycaster, blend};

impl Raycaster {
    pub fn draw_sprites(&self, world: &World, textures: &TextureManager, data: &mut[u8]) {
        let entity_count = world.entity_iter().count();
        let mut distance = Vec::with_capacity(entity_count);
        for (i, entity) in world.entity_iter().enumerate() {
            distance.push(((self.pos - entity.pos()).length_squared(), i));
        }
        distance.sort_by(|(a, _), (b, _)| a.total_cmp(b));

        let entities = world.entities();
        for (_, index) in distance {
            let entity = &entities[index];
            let tex = textures.get_entity_texture(entity.texture());
            let (texture, tex_width, tex_height, bottom_height, top_height) =
            (tex.texture, tex.width, tex.height, tex.bottom_height, tex.top_height);

            let sprite_pos = entity.pos() - self.pos;

            let inv_det = 1.0
                / (self.plane_h.x * self.dir.z - self.dir.x * self.plane_h.z);

            let transform_x = inv_det
                * (self.dir.z * sprite_pos.x - self.dir.x * sprite_pos.z);
            let transform_z = inv_det
                * (self.plane_h.x * sprite_pos.z
                    - self.plane_h.z * sprite_pos.x);

            let sprite_screen_x = ((self.width as f32 / 2.0)
                * (1.0 + transform_x / transform_z))
                as i32;
            let sprite_dimension =
                (self.height as f32 / transform_z).abs() as i32;

            let half_sprite_dimension = sprite_dimension / 2;
            let begin_y =
                (self.int_half_height - half_sprite_dimension).max(0) as u32;
            let end_y = ((self.int_half_height + half_sprite_dimension).max(0)
                as u32)
                .min(self.height - 1);

            let begin_x =
                (sprite_screen_x - half_sprite_dimension).max(0) as u32;
            let end_x = ((sprite_screen_x + half_sprite_dimension).max(0)
                as u32)
                .min(self.width - 1);

            for x in begin_x..end_x {
                // TODO test for behind objects with opacity
                let tex_x = ((x as f32
                    - (sprite_screen_x as f32 - half_sprite_dimension as f32))
                    * 256.0
                    * tex_width as f32
                    / sprite_dimension as f32)
                    as i32
                    / 256;
                let z_value = self.z_buffer[x as usize];
                if transform_z > 0.0
                    //&& x > 0
                    //&& x < self.width
                    
                {
                    if transform_z < z_value.distance {
                    for y in begin_y..end_y {
                        let index = (self.height as usize - 1 - y as usize)
                            * self.four_width
                            + x as usize * 4;
                        let rgba = &mut data[index..index + 4];

                        let d = 256.0 * y as f32 - self.height as f32 * 128.0
                            + sprite_dimension as f32 * 128.0;
                        let tex_y = (d * tex_height as f32) as i32 / sprite_dimension / 256;
                        let i = (tex_width as i32 * tex_y * 4 + tex_x * 4) as usize;
                        let color = &texture[i..i + 4];
                        rgba.copy_from_slice(color);
                    }
                } else {
                    match z_value.tile {
                        Tile::Wall(_) => continue,
                        _ => ()
                    }
                    for y in begin_y..end_y {
                        let index = (self.height as usize - 1 - y as usize)
                            * self.four_width
                            + x as usize * 4;
                        let rgba = &mut data[index..index + 4];
                        let alpha = rgba[3];
                        if alpha == 255 {
                            continue;
                        }

                        let d = 256.0 * y as f32 - self.height as f32 * 128.0
                            + sprite_dimension as f32 * 128.0;
                        let tex_y = (d * tex_height as f32) as i32 / sprite_dimension / 256;
                        let i = (tex_width as i32 * tex_y * 4 + tex_x * 4) as usize;
                        let color = &texture[i..i + 4];
                        if alpha == 0 {
                            rgba.copy_from_slice(color);
                        } else {
                            rgba.copy_from_slice(&blend(color, rgba));
                        }
                    }
                }
                }
            }
        }
    }
}