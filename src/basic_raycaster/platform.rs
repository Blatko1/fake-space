use crate::textures::TextureDataRef;

use super::{ray::Ray, FrameRenderer};

impl<'a> FrameRenderer<'a> {
    pub fn render_platform(&self, params: PlatformRenderParams<'a>) -> (usize, usize) {
        let bottom_draw_bound = params.bottom_draw_bound;
        let top_draw_bound = params.top_draw_bound;
        let ray = params.ray;
        let height = params.height;

                // TODO idk if this 'if' is necessary
                if params.texture.is_empty() {
                    return (bottom_draw_bound, top_draw_bound);
                }
                
                let (texture, tex_width, tex_height) =
                (params.texture.data, params.texture.width as usize, params.texture.height as usize);


                // Draw from (always drawing from bottom to top):
                let half_wall_pixel_height = self.half_view_height / ray.previous_wall_dist;
                let pixels_to_top =
                    half_wall_pixel_height * (height - ray.origin.y) + self.y_shearing;
                let draw_from = ((self.half_view_height + pixels_to_top) as usize)
                    .clamp(bottom_draw_bound, top_draw_bound);

                // Draw to:
                let half_wall_pixel_height = self.half_view_height / ray.wall_dist;
                let pixels_to_top =
                    half_wall_pixel_height * (height - ray.origin.y) + self.y_shearing;
                let draw_to = ((self.half_view_height + pixels_to_top) as usize)
                    .clamp(draw_from, top_draw_bound);

                                // Variables used for reducing the amount of calculations and for optimization
            let tile_step_factor = ray.horizontal_plane * 2.0 * self.width_recip;
            let pos_factor = ray.camera_dir - ray.horizontal_plane
                + tile_step_factor * ray.column_index as f32;

                let segment =params. column
                    .chunks_exact_mut(3)
                    .enumerate()
                    .skip(draw_from)
                    .take(draw_to - draw_from);

                let denominator = (height - ray.origin.y) * self.half_view_height;
 
                for (y, pixel) in segment {
                    let row_dist = denominator
                        / (y as f32 - self.y_shearing - self.half_view_height);
                    let pos = ray.origin + row_dist * pos_factor;

                    let tex_x =
                        ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
                    let tex_y = ((tex_height as f32 * pos.z.fract()) as usize)
                        .min(tex_height - 1);
                    let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
                    let color = &texture[i..i + 3];

                    pixel.copy_from_slice(color);
                }
                (draw_from, draw_to)
    }
}

pub struct PlatformRenderParams<'a> {
    pub ray: Ray, pub bottom_draw_bound: usize, pub top_draw_bound: usize,pub height: f32,
    pub texture: TextureDataRef<'a>, pub column: &'a mut [u8]
}