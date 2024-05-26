use crate::world::textures::TextureDataRef;

use super::{ColumnDrawer, Side};

// TODO write tests for each draw call function to check for overflows
pub(super) struct WallDrawData<'a> {
    pub texture_data: TextureDataRef<'a>,
    pub bottom_wall_level: f32,
    pub top_wall_level: f32,
}

impl<'a> ColumnDrawer<'a> {
    pub(super) fn draw_wall(
        &self,
        wall_data: WallDrawData,
        column: &mut [u8],
    ) -> (usize, usize) {
        let bottom_draw_bound = self.bottom_draw_bound;
        let top_draw_bound = self.top_draw_bound;
        let cam = self.camera;
        let ray = self.ray;
        let ambient = self.current_room.data.ambient_light_intensity();

        if wall_data.texture_data.is_empty() {
            return (bottom_draw_bound, top_draw_bound);
        }

        let normal = match ray.hit_wall_side {
            Side::Vertical => {
                if ray.dir.x > 0.0 {
                    // side facing west hit
                    super::NORMAL_X_NEGATIVE
                } else {
                    // side facing east hit
                    super::NORMAL_X_POSITIVE
                }
            }
            Side::Horizontal => {
                if ray.dir.z > 0.0 {
                    // side facing south hit
                    super::NORMAL_Z_NEGATIVE
                } else {
                    // side facing north hit
                    super::NORMAL_Z_POSITIVE
                }
            }
        };

        let (texture, tex_width, tex_height) = (
            wall_data.texture_data.data,
            wall_data.texture_data.width as usize,
            wall_data.texture_data.height as usize,
        );

        // Calculate wall pixel height for the parts above and below the middle
        let half_wall_pixel_height = cam.f_half_height / ray.wall_dist;
        let pixels_to_bottom = half_wall_pixel_height
            * (ray.origin.y - wall_data.bottom_wall_level)
            - cam.y_shearing;
        let pixels_to_top = half_wall_pixel_height
            * (wall_data.top_wall_level - ray.origin.y)
            + cam.y_shearing;
        let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;
        // From which pixel to begin drawing and on which to end
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
        let tex_y_step = (wall_data.top_wall_level - wall_data.bottom_wall_level)
            * tex_height as f32
            / full_wall_pixel_height
            * 0.5;
        let mut tex_y =
            (draw_from as f32 + pixels_to_bottom - cam.f_half_height) * tex_y_step;

        // Precomputed variables for performance increase
        let four_tex_width = tex_width * 4;
        let four_tex_x = tex_x * 4;

        let diffuse = ((-ray.dir).dot(normal)).max(0.0);
        // Multiply by the canvas aspect ratio so the light has a shape of a circle.
        let flashlight_x = ray.plane_x * cam.view_aspect;
        // Smooth out the flashlight intensity using the distance
        let flashlight_intensity = (1.0
            - (ray.wall_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
            * super::FLASHLIGHT_INTENSITY
            * diffuse;

        let spotlight = ((super::SPOTLIGHT_DISTANCE - ray.wall_dist)
            / super::SPOTLIGHT_SMOOTHED_DISTANCE)
            .clamp(0.0, 1.0)
            * super::SPOTLIGHT_STRENGTH;

        column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, pixel)| {
                let tex_y_pos = tex_y.round() as usize % tex_height;
                let i = (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
                let color = &texture[i..i + 3];

                let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
                let flashlight_radius =
                    (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt();
                let t = 1.0
                    - ((flashlight_radius - super::FLASHLIGHT_INNER_RADIUS)
                        / (super::FLASHLIGHT_OUTER_RADIUS
                            - super::FLASHLIGHT_INNER_RADIUS))
                        .clamp(0.0, 1.0);
                let flashlight = t * t * (3.0 - t * 2.0) * flashlight_intensity;
                // Modify pixel
                let t = ambient + (flashlight * self.use_flashlight).max(spotlight);
                pixel.iter_mut().zip(color).for_each(|(dest, &src)| {
                    *dest = (src as f32 * t) as u8;
                });

                // TODO maybe make it so `tex_y_step` is being subtracted.
                tex_y += tex_y_step;
            });
        (draw_from, draw_to)
    }
}
