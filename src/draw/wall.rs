use crate::{
    draw::top_bottom,
    textures::{TextureDataRef, TextureManager},
};

use super::{blend, RayHit, Raycaster, Side};

// TODO write tests for each draw call function to check for overflows
impl Raycaster {
    // Draws full and transparent walls.
    pub fn draw_wall(
        &self,
        hit: RayHit,
        texture_data: TextureDataRef,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        bottom_y_bound: f32,
        top_y_bound: f32,
        column: &mut [u8],
    ) -> (usize, usize) {
        if texture_data.is_empty() {
            return (bottom_draw_bound, top_draw_bound);
        }
        let (tex_width, tex_height) =
            (texture_data.width as usize, texture_data.height as usize);
        let texture = match hit.side {
            Side::Vertical => texture_data.light_shade,
            Side::Horizontal => texture_data.medium_shade,
        };

        // TODO better names
        // Calculate wall pixel height for the parts above and below the middle
        let wall_pixel_height =
            self.height as f32 / hit.wall_dist * self.plane_dist;
        let half_wall_height = (wall_pixel_height / 2.0) as f32;
        let top_height =
            half_wall_height * (top_y_bound - self.pos.y) + self.y_shearing;
        let bottom_height =
            half_wall_height * (-bottom_y_bound + self.pos.y) - self.y_shearing;
        let wall_full_height = top_height + bottom_height;

        // From which pixel to begin drawing and on which to end
        let draw_from = ((self.f_half_height - bottom_height) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        let draw_to = ((self.f_half_height + top_height) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        //let wall_full_height = (draw_to - draw_from) as f32;
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
        let tex_y_step = tex_height as f32
            / wall_full_height
            / (2.0 / (top_y_bound - bottom_y_bound));
        let mut tex_y = (draw_from as f32 + bottom_height - self.f_half_height)
            * tex_y_step;
        //  println!("tex_y {}", draw_from as f32 + bottom_height - self.float_half_height);
        let draw_fn = match texture_data.transparency {
            true => draw_transparent_wall_pixel,
            false => draw_full_wall_pixel,
        };

        // Precomputed variables for performance increase
        let width = self.width as usize;
        let four_tex_width = tex_width * 4;
        let four_tex_x = tex_x * 4;
        column
            .chunks_exact_mut(4)
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|rgba| {
                if rgba[3] != 255 {
                    let tex_y_pos = tex_y.round() as usize % tex_height;
                    let i = (tex_height - tex_y_pos - 1) * four_tex_width
                        + four_tex_x;
                    let color = &texture[i..i + 4];

                    // Draw the pixel:
                    draw_fn(rgba, color);
                }
                // TODO maybe make it so `tex_y_step` is being subtracted.
                tex_y += tex_y_step;
            });

        /*if let Some(first) = column
        .chunks_exact_mut(4)
        .nth(draw_from)
        {
            first.copy_from_slice(&[255, 100, 255, 255]);
        };
        if let Some(first) = column
            .chunks_exact_mut(4)
            .nth(draw_to)
        {
            first.copy_from_slice(&[255, 100, 0, 255]);
        };*/

        (draw_from, draw_to)
    }
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
