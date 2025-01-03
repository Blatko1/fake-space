use crate::textures::TextureDataRef;

use super::{ColumnRenderer, Side};

// TODO write tests for each draw call function to check for overflows
pub(super) struct WallDrawData<'a> {
    pub texture_data: TextureDataRef<'a>,
    pub bottom_wall_level: f32,
    pub top_wall_level: f32,
}

impl<'a> ColumnRenderer<'a> {
    pub(super) fn draw_wall(
        &self,
        wall_data: WallDrawData,
        column: &mut [u8],
    ) -> (usize, usize) {
        let bottom_draw_bound = self.bottom_draw_bound;
        let top_draw_bound = self.top_draw_bound;
        let cam = self.camera;
        let ray = self.ray;
        let bottom_wall_level = wall_data.bottom_wall_level;
        let top_wall_level = wall_data.top_wall_level;
        let texture_data = wall_data.texture_data;

        if texture_data.is_empty() {
            return (bottom_draw_bound, top_draw_bound);
        }

        let (texture, tex_width, tex_height) = (
            texture_data.data,
            texture_data.width as usize,
            texture_data.height as usize,
        );

        // Calculate wall pixel height for the parts above and below the middle
        let half_wall_pixel_height = cam.f_half_height / ray.wall_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (ray.origin.y - bottom_wall_level) - cam.y_shearing;
        let pixels_to_top =
            half_wall_pixel_height * (top_wall_level - ray.origin.y) + cam.y_shearing;
        let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

        let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        let draw_to = ((cam.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        let tex_x = match ray.hit_wall_side {
            Side::Vertical if ray.dir.x > 0.0 => {
                tex_width - (ray.wall_offset * tex_width as f32) as usize - 1
            }
            Side::Horizontal if ray.dir.z < 0.0 => {
                tex_width - (ray.wall_offset * tex_width as f32) as usize - 1
            }
            _ => (ray.wall_offset * tex_width as f32) as usize,
        };
        let tex_y_step = (top_wall_level - bottom_wall_level) * tex_height as f32
            / full_wall_pixel_height
            * 0.5;
        let mut tex_y =
            (draw_from as f32 + pixels_to_bottom - cam.f_half_height) * tex_y_step;

        let segment = column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from);

        for (y, pixel) in segment {
            // avoids small artefacts let tex_y_pos = tex_y.round() as usize % tex_height;
            let tex_y_pos = tex_y as usize % tex_height;
            tex_y += tex_y_step;
            let i = 4 * ((tex_height - tex_y_pos - 1) * tex_width + tex_x);
            let color = &texture[i..i + 3];

            pixel.copy_from_slice(color);
        }
        (draw_from, draw_to)
    }
}
