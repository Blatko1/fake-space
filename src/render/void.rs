use crate::textures::{VOID, VOID_HEIGHT, VOID_WIDTH, FENCE, FENCE_WIDTH, FENCE_HEIGHT};

use super::{RayCast, Raycaster, Side};

const PURPLE: [u8; 4] = [200, 0, 220, 255];

impl Raycaster {
    pub fn draw_void(&self, ray: &RayCast, data: &mut [u8]) {
        /*let draw_x_offset = 4 * (self.width - ray.screen_x - 1) as usize;
        for y in 0..self.height - 1 {
            let index = (self.height as usize - 1 - y as usize)
                * self.four_width
                + draw_x_offset;
            data[index..index + 4].copy_from_slice(&PURPLE);
        }*/
        //let begin = (ray.screen_x * 4 * self.height) as usize;
        //let end = (ray.screen_x * 4 * self.height + self.height * 4) as usize;
        //data[begin..end]
        //    .chunks_exact_mut(4)
        //    .for_each(|rgba| rgba.copy_from_slice(&PURPLE));
        
        let mut color = [0, 0, 0, 0];
        let draw_x_offset = 4 * (self.width - ray.screen_x - 1) as usize;
        let half_h_i = self.height as i32 / 2;
        let half_h_f = self.height as f32 * 0.5;

        let hit = ray.hit;

        let (texture, tex_width, tex_height) = (FENCE, FENCE_WIDTH, FENCE_HEIGHT);

        let line_pixel_height = self.height as i32;
        let half_l = line_pixel_height / 2;

        let begin = (half_h_i - half_l).max(0) as u32;
        let end = ((half_h_i + half_l) as u32).min(self.height - 1);

        let tex_height_minus_one = tex_height as f32 - 1.0;
        let tex_x = match hit.side {
            Side::Vertical if ray.dir.x > 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
            }

            Side::Horizontal if ray.dir.y < 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
            }
            _ => (hit.wall_x * tex_width as f32) as u32,
        };
        let four_tex_x = tex_x * 4;
        //assert!(tex_x < 16);
        let tex_y_step = 16.0 / line_pixel_height as f32;
        let mut tex_y = (begin as f32 + line_pixel_height as f32 * 0.5
            - half_h_f)
            * tex_y_step;
        // TODO fix texture mapping.
        //assert!(tex_y >= 0.0);
        for y in begin..end {
            //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
            let y_pos = tex_y.min(tex_height_minus_one).round() as u32;
            let i = ((tex_height - y_pos - 1) * tex_width * 4 + four_tex_x)
                as usize;
            color.copy_from_slice(&texture[i..i + 4]);
            match hit.side {
                Side::Vertical => (),
                Side::Horizontal => {
                    color[0] = color[0] - 15;
                    color[1] = color[1] - 15;
                    color[2] = color[2] - 15;
                    color[3] = color[3] - 15
                }
            };
            let index = (self.height as usize - 1 - y as usize)
                * self.four_width
                + draw_x_offset;
            data[index..index + 4].copy_from_slice(&color);
            tex_y += tex_y_step;
            //assert!(tex_y <= 16.0);
        }
    }
}
