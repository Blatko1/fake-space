use super::{RayCast, Raycaster, Side};
use crate::{
    map::{Tile, TransparentTexture},
    textures::{
        FENCE, FENCE_BOTTOM_HEIGHT, FENCE_HEIGHT, FENCE_TOP_HEIGHT, FENCE_WIDTH,
    },
};
// TODO add blue glass texture
impl Raycaster {
    pub fn draw_transparent(&self, ray: &RayCast, data: &mut [u8]) {
        let mut color = [0, 0, 0, 0];

        for through_hit in ray.through_hits.iter().rev() {
            let (texture, tex_width, tex_height, bottom_height, top_height) =
                match through_hit.tile {
                    Tile::Transparent(tex) => match tex {
                        TransparentTexture::Fence => (
                            FENCE,
                            FENCE_WIDTH,
                            FENCE_HEIGHT,
                            FENCE_BOTTOM_HEIGHT,
                            FENCE_TOP_HEIGHT,
                        ),
                    },
                    _ => unreachable!(),
                };
            let full_line_pixel_height =
                (self.height as f32 / (through_hit.wall_dist)) as i32;
            let top_height =
                ((full_line_pixel_height / 2) as f32 * top_height) as i32;
            let bottom_height =
                ((full_line_pixel_height / 2) as f32 * bottom_height) as i32;
            let line_height = top_height + bottom_height;

            let begin = (self.int_half_height - bottom_height).max(0) as u32;
            let end =
                ((self.int_half_height + top_height).max(0) as u32).min(self.height - 1);

            let tex_height_minus_one = tex_height as f32 - 1.0;
            let tex_x = match through_hit.side {
                Side::Vertical if ray.dir.x > 0.0 => {
                    tex_width
                        - (through_hit.wall_x * tex_width as f32) as u32
                        - 1
                }

                Side::Horizontal if ray.dir.y < 0.0 => {
                    tex_width
                        - (through_hit.wall_x * tex_width as f32) as u32
                        - 1
                }
                _ => (through_hit.wall_x * tex_width as f32) as u32,
            };
            let four_tex_x = tex_x * 4;
            //assert!(tex_x < 16);
            let tex_y_step = tex_height as f32 / line_height as f32;
            let mut tex_y =
                (begin as f32 + bottom_height as f32 - self.float_half_height) * tex_y_step;
            // TODO fix texture mapping.
            //assert!(tex_y >= 0.0);
            for y in begin..end {
                //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
                let y_pos = tex_y.min(tex_height_minus_one).round() as u32;

                let i = ((tex_height - y_pos - 1) * tex_width * 4 + four_tex_x)
                    as usize;
                color.copy_from_slice(&texture[i..i + 4]);
                match through_hit.side {
                    Side::Vertical => (),
                    Side::Horizontal => {
                        color[0] = color[0].saturating_sub(15);
                        color[1] = color[1].saturating_sub(15);
                        color[2] = color[2].saturating_sub(15);
                    }
                };
                let index = (self.height as usize - 1 - y as usize)
                    * self.four_width
                    + ray.draw_x_offset;
                tex_y += tex_y_step;
                if color[3] == 0 {
                    continue;
                }
                let rgba = &mut data[index..index + 4];
                rgba.copy_from_slice(&blend(rgba, color))
                //assert!(tex_y <= 16.0);
            }
        }
    }
}

#[inline(always)]
fn blend(background: &[u8], foreground: [u8; 4]) -> [u8; 4] {
    let alpha = foreground[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    let blended_r =
        (foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha) as u8;
    let blended_g =
        (foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha) as u8;
    let blended_b =
        (foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha) as u8;
    let blended_a = (255.0 * alpha + background[3] as f32 * inv_alpha) as u8;

    [blended_r, blended_g, blended_b, blended_a]
}
