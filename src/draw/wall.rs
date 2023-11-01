use crate::textures::TextureDataRef;

use super::{blend, RayHit, Raycaster, Side};

// TODO write tests for each draw call function to check for overflows
impl Raycaster {
    // Draws full and transparent walls.
    pub fn draw_bottom_wall(
        &self,
        hit: RayHit,
        texture_data: TextureDataRef,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        bottom_y_bound: f32,
        top_y_bound: f32,
        column: &mut [u8],
    ) -> usize {
        if texture_data.is_empty() {
            return top_draw_bound
        }
        let texture = match hit.side {
            Side::Vertical => texture_data.light_shade,
            Side::Horizontal => texture_data.medium_shade,
        };
        let (tex_width, tex_height) =
            (texture_data.width as usize, texture_data.height as usize);

        // Calculate wall pixel height for the parts above and below the middle
        let half_wall_pixel_height =
            self.f_half_height / hit.wall_dist * self.plane_dist;
        let pixels_to_bottom = half_wall_pixel_height
            * (-bottom_y_bound + self.pos.y)
            - self.y_shearing;
        let pixels_to_top = half_wall_pixel_height * (top_y_bound - self.pos.y)
            + self.y_shearing;
        let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

        // From which pixel to begin drawing and on which to end
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        if draw_from == draw_to {
            return draw_to
        }

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
            / full_wall_pixel_height
            / (2.0 / (top_y_bound - bottom_y_bound));
        let mut tex_y = (draw_from as f32 + pixels_to_bottom
            - self.f_half_height)
            * tex_y_step;
        let draw_fn = match texture_data.transparency {
            true => draw_transparent_wall_pixel,
            false => draw_full_wall_pixel,
        };

        // Precomputed variables for performance increase
        let four_tex_width = tex_width * 4;
        let four_tex_x = tex_x * 4;
        column
            .chunks_exact_mut(4)
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|dest| {
                //if dest[3] != 255 {
                let tex_y_pos = tex_y.round() as usize % tex_height;
                let i =
                    (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
                let src = &texture[i..i + 4];

                // Draw the pixel:
                draw_fn(dest, src);
                //}
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

        draw_to
    }

    pub fn draw_top_wall(
        &self,
        hit: RayHit,
        texture_data: TextureDataRef,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        bottom_y_bound: f32,
        top_y_bound: f32,
        column: &mut [u8],
    ) -> usize {
        if texture_data.is_empty() {
            return bottom_draw_bound
        }
        let texture = match hit.side {
            Side::Vertical => texture_data.light_shade,
            Side::Horizontal => texture_data.medium_shade,
        };
        let (tex_width, tex_height) =
            (texture_data.width as usize, texture_data.height as usize);

        // Calculate wall pixel height for the parts above and below the middle
        let half_wall_pixel_height =
            self.f_half_height / hit.wall_dist * self.plane_dist;
        let pixels_to_bottom = half_wall_pixel_height
            * (-bottom_y_bound + self.pos.y)
            - self.y_shearing;
        let pixels_to_top = half_wall_pixel_height * (top_y_bound - self.pos.y)
            + self.y_shearing;
        let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

        // From which pixel to begin drawing and on which to end
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        if draw_from == draw_to {
            return draw_from
        }

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
            / full_wall_pixel_height
            / (2.0 / (top_y_bound - bottom_y_bound));
        let mut tex_y = (draw_from as f32 + pixels_to_bottom
            - self.f_half_height)
            * tex_y_step;
        let draw_fn = match texture_data.transparency {
            true => draw_transparent_wall_pixel,
            false => draw_full_wall_pixel,
        };

        // Precomputed variables for performance increase
        let four_tex_width = tex_width * 4;
        let four_tex_x = tex_x * 4;
        column
            .chunks_exact_mut(4)
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|dest| {
                //if dest[3] != 255 {
                let tex_y_pos = tex_y.round() as usize % tex_height;
                let i =
                    (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
                let src = &texture[i..i + 4];

                // Draw the pixel:
                draw_fn(dest, src);
                //}
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

        draw_from
    }
}

#[inline]
fn draw_full_wall_pixel(dest: &mut [u8], color: &[u8]) {
    if dest[3] == 0 {
        dest.copy_from_slice(color);
    } else {
        dest.copy_from_slice(&blend(color, dest));
    }
}

#[inline]
fn draw_transparent_wall_pixel(dest: &mut [u8], color: &[u8]) {
    let a = color[3];
    if a == 0 {
        return;
    }
    if a == 255 {
        draw_full_wall_pixel(dest, color);
    } else {
        dest.copy_from_slice(&blend(color, dest));
    }
}
