// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! adding unsafe could improve performance

use glam::Vec3;

use crate::textures::TextureDataRef;

use super::ColumnRenderer;

pub(super) struct PlatformDrawData<'a> {
    pub texture_data: TextureDataRef<'a>,
    pub height_level: f32,
    pub draw_from_dist: f32,
    pub draw_to_dist: f32,
}

impl<'a> ColumnRenderer<'a> {
    pub(super) fn draw_platform(
        &self,
        platform_data: PlatformDrawData,
        column: &mut [u8],
    ) -> (usize, usize) {
        let bottom_draw_bound = self.bottom_draw_bound;
        let top_draw_bound = self.top_draw_bound;
        let cam = self.camera;
        let ray = self.ray;

        if platform_data.texture_data.is_empty() {
            return (bottom_draw_bound, top_draw_bound);
        }
        let (texture, tex_width, tex_height) = (
            platform_data.texture_data.data,
            platform_data.texture_data.width as usize,
            platform_data.texture_data.height as usize,
        );

        // Draw from (always drawing from bottom to top):
        let half_wall_pixel_height = cam.f_half_height / platform_data.draw_from_dist;
        let pixels_to_top = half_wall_pixel_height
            * (platform_data.height_level - ray.origin.y)
            + cam.y_shearing;
        let draw_from = ((cam.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height = cam.f_half_height / platform_data.draw_to_dist;
        let pixels_to_top = half_wall_pixel_height
            * (platform_data.height_level - ray.origin.y)
            + cam.y_shearing;
        let draw_to = ((cam.f_half_height + pixels_to_top) as usize)
            .clamp(draw_from, top_draw_bound);

        // Variables used for reducing the amount of calculations and for optimization
        let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
        let pos_factor = ray.camera_dir - ray.horizontal_plane
            + tile_step_factor * ray.column_index as f32;
        let denominator = (platform_data.height_level - ray.origin.y) * cam.f_half_height;

        let segment = column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from);

        for (y, pixel) in segment {
            let row_dist = denominator / (y as f32 - cam.y_shearing - cam.f_half_height);
            let pos = ray.origin + row_dist * pos_factor;

            let tex_x = ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
            let tex_y =
                ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
            let color = &texture[i..i + 3];

            pixel.copy_from_slice(color);
        }
        (draw_from, draw_to)
    }
}
