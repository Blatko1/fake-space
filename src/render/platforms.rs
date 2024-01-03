// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! trying to implement sprite entities
// is difficult due to existence of transparent walls
// and their fully transparent parts
// TODO problem! adding unsafe could improve performance
use super::{column::DrawParams, RayCaster};

impl RayCaster {
    pub(super) fn draw_bottom_platform(
        &self,
        draw_params: DrawParams,
        column: &mut [u8],
    ) -> usize {
        let texture_manager = draw_params.texture_manager;
        let bottom_draw_bound = draw_params.bottom_draw_bound;
        let top_draw_bound = draw_params.top_draw_bound;
        let ray = draw_params.ray;
        let tile = draw_params.tile;
        let closer_wall_dist = draw_params.closer_wall_dist;
        let further_wall_dist = draw_params.further_wall_dist;
        let tile_x = draw_params.tile_x;
        let tile_z = draw_params.tile_z;

        let y_level = tile.level2;

        let bottom_platform_texture = texture_manager.get(tile.bottom_platform_tex);
        if bottom_platform_texture.is_empty() {
            return bottom_draw_bound;
        }
        let (texture, tex_width, tex_height) = (
            bottom_platform_texture.data,
            bottom_platform_texture.width as usize,
            bottom_platform_texture.height as usize,
        );

        // Draw from (always drawing from bottom to top):
        let half_wall_pixel_height =
            self.f_half_height / closer_wall_dist * self.plane_dist;
        let pixels_to_top =
            half_wall_pixel_height * (y_level - ray.origin.y) + self.y_shearing;
        let draw_from = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            self.f_half_height / further_wall_dist * self.plane_dist;

        let pixels_to_top =
            half_wall_pixel_height * (y_level - ray.origin.y) + self.y_shearing;
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(draw_from, top_draw_bound);

        let ray_dir = ray.caster_dir - self.plane_h;
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        column
            .chunks_exact_mut(4)
            .rev()
            .enumerate()
            .skip(self.height as usize - draw_to)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = ((ray.origin.y - y_level) / 2.0) * self.f_height
                    / (y as f32 - self.f_height / 2.0 + self.y_shearing)
                    * self.plane_dist;
                let step = tile_step_factor * row_dist;
                let pos = ray.origin + ray_dir * row_dist + step * ray.x as f32;
                let tex_x = ((tex_width as f32 * (pos.x - tile_x as f32)) as usize)
                    .min(tex_width - 1);
                let tex_y = ((tex_height as f32 * (pos.z - tile_z as f32)) as usize)
                    .min(tex_height - 1);
                let i = tex_width * 4 * tex_y + tex_x * 4;
                let color = &texture[i..i + 4];
                rgba.copy_from_slice(color);
            });
        /*if let Some(first) = column.chunks_exact_mut(4).nth(draw_to) {
            first.copy_from_slice(&[255, 255, 255, 255]);
        };
        if let Some(first) = column.chunks_exact_mut(4).nth(draw_from) {
            first.copy_from_slice(&[255, 0, 0, 255]);
        };*/

        draw_to
    }

    pub(super) fn draw_top_platform(
        &self,
        draw_params: DrawParams,
        column: &mut [u8],
    ) -> usize {
        let texture_manager = draw_params.texture_manager;
        let bottom_draw_bound = draw_params.bottom_draw_bound;
        let top_draw_bound = draw_params.top_draw_bound;
        let ray = draw_params.ray;
        let tile = draw_params.tile;
        let closer_wall_dist = draw_params.closer_wall_dist;
        let further_wall_dist = draw_params.further_wall_dist;
        let tile_x = draw_params.tile_x;
        let tile_z = draw_params.tile_z;

        let y_level = tile.level3;

        let top_platform_texture = texture_manager.get(tile.top_platform_tex);
        if top_platform_texture.is_empty() {
            return top_draw_bound;
        }
        let (texture, tex_width, tex_height) = (
            top_platform_texture.data,
            top_platform_texture.width as usize,
            top_platform_texture.height as usize,
        );

        // Draw from:
        let half_wall_pixel_height =
            self.f_half_height / further_wall_dist * self.plane_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (-y_level + self.origin.y) - self.y_shearing;
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            self.f_half_height / closer_wall_dist * self.plane_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (-y_level + ray.origin.y) - self.y_shearing;
        let draw_to = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(draw_from, top_draw_bound);

        let ray_dir = ray.caster_dir - self.plane_h;
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        column
            .chunks_exact_mut(4)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = ((-ray.origin.y + y_level) / 2.0) * self.f_height
                    / (y as f32 - self.f_height / 2.0 - self.y_shearing)
                    * self.plane_dist;
                let step = tile_step_factor * row_dist;
                let pos = ray.origin + ray_dir * row_dist + step * ray.x as f32;
                let tex_x = ((tex_width as f32 * (pos.x - tile_x as f32)) as usize)
                    .min(tex_width - 1);
                let tex_y = ((tex_height as f32 * (pos.z - tile_z as f32)) as usize)
                    .min(tex_height - 1);
                let i = tex_width * 4 * tex_y + tex_x * 4;
                let color = &texture[i..i + 4];
                rgba.copy_from_slice(color);
            });
        /*if let Some(first) = column.chunks_exact_mut(4).nth(draw_to) {
            first.copy_from_slice(&[255, 255, 255, 255]);
        };
        if let Some(first) = column.chunks_exact_mut(4).nth(draw_from) {
            first.copy_from_slice(&[255, 0, 0, 255]);
        };*/

        draw_from
    }
}
