use crate::textures::TextureManager;

use super::{blend, RayHit, Raycaster, Side};

// TODO write tests for each draw call function to check for overflows
impl Raycaster {
    // Draws full and transparent walls.
    /*pub fn draw_wall(&self, hit: RayHit, textures: &TextureManager, data: &mut [u8]) {
        // Find out what texture to use when drawing
        let tex = textures.get(id);
        let (tex_width, tex_height, bottom_height, top_height) = (
            tex.width as usize,
            tex.height as usize,
            tex.bottom_height,
            tex.top_height,
        );
        let texture = match hit.side {
            Side::Vertical => tex.texture,
            Side::Horizontal => tex.texture_darkened,
        };

        // TODO better names
        // Calculate wall pixel height for the parts above and below the middle
        let wall_pixel_height =
            (self.height as f32 / hit.wall_dist * self.plane_dist) as i32;
        let half_wall_height = (wall_pixel_height / 2) as f32;
        let top_height = half_wall_height
            * ((1.0 - self.pos.y) * 2.0 + (top_height - 1.0))
            + self.y_shearing;
        let bottom_height = half_wall_height
            * (self.pos.y * 2.0 + (bottom_height - 1.0))
            - self.y_shearing;
        let wall_full_height = top_height + bottom_height;

        // From which pixel to begin drawing and on which to end
        let begin = (self.float_half_height - bottom_height)
            .max(0.0)
            .min(self.height as f32 - 1.0) as usize;
        let end = (self.float_half_height + top_height)
            .max(0.0)
            .min(self.height as f32 - 1.0) as usize;

        // Texture mapping variables
        let tex_x = match hit.side {
            Side::Vertical if hit.dir.x > 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as usize - 1
            }

            Side::Horizontal if hit.dir.z < 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as usize - 1
            }
            _ => (hit.wall_x * tex_width as f32) as usize,
        };
        let tex_y_step = tex_height as f32 / wall_full_height;
        let mut tex_y = (begin as f32 + bottom_height - self.float_half_height)
            * tex_y_step;
        let draw_fn: fn(target: &mut [u8], color: &[u8]) =
            match tex.has_transparency {
                true => draw_transparent_wall_pixel,
                false => draw_full_wall_pixel,
            };

        // Precomputed variables for performance increase
        let width = self.width as usize;
        let four_tex_width = tex_width * 4;
        let four_tex_x = tex_x * 4;
        data.chunks_exact_mut(4)
            .rev()
            .skip(width - 1 - hit.screen_x as usize)
            .step_by(width)
            .skip(begin)
            .take(end - begin)
            .for_each(|rgba| {
                if rgba[3] != 255 {
                    let tex_y_pos = (tex_y as usize).min(tex_height - 1);
                    let i = (tex_height - tex_y_pos - 1) * four_tex_width
                        + four_tex_x;
                    let color = &texture[i..i + 4];

                    // Draw the pixel:
                    draw_fn(rgba, color);
                }
                // TODO maybe make it so `tex_y_step` is being subtracted.
                tex_y += tex_y_step;
            });
    }*/
}

#[inline]
fn draw_full_wall_pixel(target: &mut [u8], color: &[u8]) {
    if target[3] == 0 {
        target.copy_from_slice(color);
    } else {
        target.copy_from_slice(&blend(color, target));
    }
}

#[inline]
fn draw_transparent_wall_pixel(target: &mut [u8], color: &[u8]) {
    let a = color[3];
    if a == 0 {
        return;
    }
    if a == 255 {
        draw_full_wall_pixel(target, color);
    } else {
        target.copy_from_slice(&blend(color, target));
    }
}
