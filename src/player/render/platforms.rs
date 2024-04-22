// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! adding unsafe could improve performance

use glam::Vec3;

use crate::world::textures::TextureDataRef;

use super::ColumnDrawer;

pub(super) struct PlatformDrawData<'a> {
    pub texture_data: TextureDataRef<'a>,
    pub height_level: f32,
    pub normal: Vec3,
    pub draw_from_dist: f32,
    pub draw_to_dist: f32,
}

impl<'a> ColumnDrawer<'a> {
    pub(super) fn draw_platform(
        &self,
        platform_data: PlatformDrawData,
        column: &mut [u8],
    ) -> (usize, usize) {
        let bottom_draw_bound = self.bottom_draw_bound;
        let top_draw_bound = self.top_draw_bound;
        let cam = self.camera;
        let ray = self.ray;
        let ambient = self.current_room.data.ambient_light_intensity();
        let normal = platform_data.normal;

        if platform_data.texture_data.is_empty() {
            return (bottom_draw_bound, top_draw_bound);
        }
        let (texture, tex_width, tex_height) = (
            platform_data.texture_data.data,
            platform_data.texture_data.width as usize,
            platform_data.texture_data.height as usize,
        );

        // Draw from (always drawing from bottom to top):
        let half_wall_pixel_height =
            cam.f_half_height / platform_data.draw_from_dist;
        let pixels_to_top = half_wall_pixel_height
            * (platform_data.height_level - ray.origin.y)
            + cam.y_shearing;
        let draw_from = ((cam.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            cam.f_half_height / platform_data.draw_to_dist;
        let pixels_to_top = half_wall_pixel_height
            * (platform_data.height_level - ray.origin.y)
            + cam.y_shearing;
        let draw_to = ((cam.f_half_height + pixels_to_top) as usize)
            .clamp(draw_from, top_draw_bound);

        // Variables used for reducing the amount of calculations and for optimization
        let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
        let pos_factor = ray.camera_dir - ray.horizontal_plane
            + tile_step_factor * ray.column_index as f32;
        let row_dist_factor = cam.f_half_height * cam.plane_dist;

        // Multiply by the canvas aspect ratio so the light has a shape of a circle.
        let flashlight_x = ray.plane_x * cam.view_aspect;

        column
            .chunks_exact_mut(4)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, pixel)| {
                let row_dist = (platform_data.height_level - ray.origin.y)
                    * row_dist_factor
                    / (y as f32 - cam.y_shearing - cam.f_half_height);
                let mut ray_dir = row_dist * pos_factor;
                let pos = ray.origin + ray_dir;

                let tex_x =
                    ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
                let tex_y =
                    ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
                let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
                let color = &texture[i..i + 4];

                // Calculate the diffuse lightning by finding the direction of the ray with pitch
                ray_dir.y += ray.origin.y - platform_data.height_level;
                let diffuse = ray_dir.normalize().dot(normal);
                // Smooth out the flashlight intensity using the distance
                let flashlight_intensity = (1.0
                    - (row_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
                    * super::FLASHLIGHT_INTENSITY
                    * diffuse;
                let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;

                // Smoothstep distance to get the spotlight
                let s = 1.0 - (row_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
                let spotlight = s * s * (3.0 - s * 2.0) * super::SPOTLIGHT_STRENGTH;

                let flashlight_radius =
                    (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt();
                let t = 1.0
                    - ((flashlight_radius - super::FLASHLIGHT_INNER_RADIUS)
                        / (super::FLASHLIGHT_OUTER_RADIUS
                            - super::FLASHLIGHT_INNER_RADIUS))
                        .clamp(0.0, 1.0);
                let flashlight = t * t * (3.0 - t * 2.0) * flashlight_intensity;
                // Modify pixel
                pixel[0..3].iter_mut().zip(color[0..3].iter()).for_each(
                    |(dest, &src)| {
                        *dest = (src as f32 * (flashlight + ambient + spotlight)) as u8;
                    },
                );
                //pixel[3] = color[3];
            });

        (draw_from, draw_to)
    }
}
