use crate::textures::{TextureDataRef, TextureID};

use super::{ray::Ray, FrameRenderer, Side};

impl<'a> FrameRenderer<'a> {
    // TODO test if it's better to pass argument in struct or not.
    // TODO for every write in the for loop make a special function made just for that
    // which takes color pixel as
    // TODO pass as argument column segment between bounds, so 2 arguments less
    pub fn render_wall(&self, params: WallRenderParams) -> (usize, usize) {
        let bottom_draw_bound = params.bottom_draw_bound;
        let top_draw_bound = params.top_draw_bound;
        let ray = params.ray;
        let bottom_level = params.bottom_level;
        let top_level = params.top_level;
        
        let half_wall_pixel_height = self.half_view_height / ray.wall_dist;
                if params.texture.is_empty() {
                    return (bottom_draw_bound, top_draw_bound);
                }
                let (texture, tex_width, tex_height) = (
                    params.texture.data,
                    params.texture.width as usize,
                    params.texture.height as usize,
                );

                // Calculate wall pixel height for the parts above and below the middle
                let pixels_to_bottom = half_wall_pixel_height
                    * (ray.origin.y - bottom_level)
                    - self.y_shearing;
                let pixels_to_top =
                    half_wall_pixel_height * (top_level - ray.origin.y) + self.y_shearing;
                let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

                let draw_from = ((self.half_view_height - pixels_to_bottom) as usize)
                    .clamp(bottom_draw_bound, top_draw_bound);
                let draw_to = ((self.half_view_height + pixels_to_top) as usize)
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
                let tex_y_step = (top_level - bottom_level) * tex_height as f32
                    / full_wall_pixel_height
                    * 0.5;
                let mut tex_y = (draw_from as f32 + pixels_to_bottom
                    - self.half_view_height)
                    * tex_y_step;

                let segment = params.column
                    .chunks_exact_mut(3)
                    .enumerate()
                    .skip(draw_from)
                    .take(draw_to - draw_from);

                for (_y, pixel) in segment {
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

pub struct WallRenderParams<'a> {
    pub ray: Ray, pub bottom_draw_bound: usize, pub top_draw_bound: usize,pub bottom_level: f32,
    pub top_level: f32,
    pub texture: TextureDataRef<'a>, pub column: &'a mut [u8]
}