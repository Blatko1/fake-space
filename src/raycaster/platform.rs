use crate::textures::TextureDataRef;

use super::{ray::Ray, FrameRenderer};

impl<'a> FrameRenderer<'a> {
    pub fn render_platform(
        &self,
        params: PlatformRenderParams<'a>,
        column: &'a mut [u8],
    ) -> (usize, usize) {
        let bottom_draw_bound = params.bottom_draw_bound;
        let top_draw_bound = params.top_draw_bound;
        let ray = params.ray;
        let height = params.height;

        let (texture, tex_width, tex_height) = (
            params.texture.data,
            params.texture.width as usize,
            params.texture.height as usize,
        );

        let (draw_from_distance, draw_to_distance) = match params.platform_type {
            PlatformType::Floor => (ray.previous_wall_dist, ray.wall_dist),
            PlatformType::Ceiling => (ray.wall_dist, ray.previous_wall_dist),
        };

        // Draw from (always drawing from bottom to top):
        let half_wall_pixel_height = self.half_view_height / draw_from_distance;
        let pixels_to_top =
            half_wall_pixel_height * (height - ray.origin.y) + self.y_shearing;
        let draw_from = ((self.half_view_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height = self.half_view_height / draw_to_distance;
        let pixels_to_top =
            half_wall_pixel_height * (height - ray.origin.y) + self.y_shearing;
        let draw_to = ((self.half_view_height + pixels_to_top) as usize)
            .clamp(draw_from, top_draw_bound);

        // Variables used for reducing the amount of calculations and for optimization
        let tile_step_factor = ray.horizontal_plane * 2.0 * self.width_recip;
        let pos_factor = ray.camera_dir - ray.horizontal_plane
            + tile_step_factor * ray.column_index as f32;

        let blueprint = column
            .chunks_exact_mut(3)
            .skip(draw_from)
            .take(draw_to - draw_from);

        let denominator = (height - ray.origin.y) * self.half_view_height;

        // Through trial and error i found that it should be enumerated starting from 1.
        // Before, there was texture bleeding, but now no bleeding!
        let mut y_pixel_pos =
            1.0 + draw_from as f32 - self.y_shearing - self.half_view_height;

        for pixel in blueprint {
            let row_dist = denominator / y_pixel_pos;
            let pos = ray.origin + row_dist * pos_factor;

            // TODO try removing min and test for speed!!!
            let tex_x = ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
            let tex_y =
                ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
            let tex_y = match params.platform_type {
                PlatformType::Floor => tex_height - tex_y - 1,
                PlatformType::Ceiling => tex_y,
            };
            let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
            let color = &texture[i..i + 3];

            pixel.copy_from_slice(color);

            y_pixel_pos += 1.0;
        }
        (draw_from, draw_to)
    }
}

pub struct PlatformRenderParams<'a> {
    pub ray: Ray,
    pub bottom_draw_bound: usize,
    pub top_draw_bound: usize,
    pub height: f32,
    pub platform_type: PlatformType,
    pub texture: TextureDataRef<'a>,
}

pub enum PlatformType {
    Floor,
    Ceiling,
}
