use super::{RayHit, Raycaster, Side};

use crate::{
    map::WallTile,
    textures::{BLUE_BRICK_TEXTURE, LIGHT_PLANK_TEXTURE},
};
// TODO write tests for each draw call function to check for overflows
impl Raycaster {
    pub fn draw_wall(
        &self,
        hit: RayHit,
        wall_tile: WallTile,
        data: &mut [u8],
    ) {
        let mut color = [0, 0, 0, 0];

        let (texture, tex_width, tex_height, bottom_height, top_height) =
            match wall_tile {
                WallTile::BlueBrick => BLUE_BRICK_TEXTURE,
                WallTile::LightPlank => LIGHT_PLANK_TEXTURE,
            };

        // TODO better names
        let full_line_pixel_height =
            (self.height as f32 / hit.wall_dist / self.aspect) as i32;
        let top_height =
            ((full_line_pixel_height / 2) as f32 * top_height) as i32;
        let bottom_height =
            ((full_line_pixel_height / 2) as f32 * bottom_height) as i32;
        let line_height = top_height.saturating_add(bottom_height);

        let begin = (self.int_half_height - bottom_height).max(0) as u32;
        let end = ((self.int_half_height + top_height).max(0) as u32)
            .min(self.height - 1);

        let tex_height_minus_one = tex_height as f32 - 1.0;
        let tex_x = match hit.side {
            Side::Vertical if hit.dir.x > 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
            }

            Side::Horizontal if hit.dir.z < 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
            }
            _ => (hit.wall_x * tex_width as f32) as u32,
        };
        let four_tex_x = tex_x * 4;
        let tex_y_step = tex_height as f32 / line_height as f32;
        let mut tex_y = (begin as f32 + bottom_height as f32
            - self.float_half_height)
            * tex_y_step;
        // TODO fix texture mapping.
        for y in begin..end {
            let index = (self.height as usize - 1 - y as usize)
                * self.four_width
                + hit.screen_x as usize * 4;
            let rgba = &mut data[index..index + 4];
            let alpha = rgba[3];
            if alpha == 255 {
                tex_y += tex_y_step;
                continue;
            }
            //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
            let tex_y_pos = tex_y.min(tex_height_minus_one).round() as u32;
            let i = ((tex_height - tex_y_pos - 1) * tex_width * 4 + four_tex_x)
                as usize;
            color.copy_from_slice(&texture[i..i + 4]);
            match hit.side {
                Side::Vertical => (),
                Side::Horizontal => {
                    color[0] = color[0].saturating_sub(15);
                    color[1] = color[1].saturating_sub(15);
                    color[2] = color[2].saturating_sub(15);
                }
            };
            if alpha == 0 {
                rgba.copy_from_slice(&color);
            } else {
                rgba.copy_from_slice(&blend(&color, rgba));
            }
            tex_y += tex_y_step;
        }
    }
}

// TODO maybe have only one blend function
#[inline(always)]
fn blend(background: &[u8], foreground: &[u8]) -> [u8; 4] {
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
