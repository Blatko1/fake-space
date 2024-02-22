// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! adding unsafe could improve performance

use super::DrawParams;

pub(super) fn draw_bottom_platform(draw_params: DrawParams, column: &mut [u8]) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    //let tile_x = draw_params.tile_x;
    //let tile_z = draw_params.tile_z;

    let ground_texture = draw_params.texture_manager.get(tile.ground_tex);
    if ground_texture.is_empty() {
        return top_draw_bound;
    }
    let (texture, tex_width, tex_height) = (
        ground_texture.data,
        ground_texture.width as usize,
        ground_texture.height as usize,
    );

    // Draw from (always drawing from bottom to top):
    let half_wall_pixel_height =
        cam.f_half_height / ray.previous_wall_dist * cam.plane_dist;
    let pixels_to_top =
        half_wall_pixel_height * (tile.ground_level - ray.origin.y) + cam.y_shearing;
    let draw_from = ((cam.f_half_height + pixels_to_top) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    // Draw to:
    let half_wall_pixel_height = cam.f_half_height / ray.wall_dist * cam.plane_dist;

    let pixels_to_top =
        half_wall_pixel_height * (tile.ground_level - ray.origin.y) + cam.y_shearing;
    let draw_to =
        ((cam.f_half_height + pixels_to_top) as usize).clamp(draw_from, top_draw_bound);

    // Variables used for reducing the amount of calculations and for optimization
    let ray_dir = ray.camera_dir - ray.horizontal_plane;
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    let pos_factor = ray_dir + tile_step_factor * ray.column_index as f32;
    let row_dist_factor = cam.f_half_height * cam.plane_dist;
    let shearing_plus_half_height = cam.y_shearing + cam.f_half_height;

    let flashlight_x = (2.0 * ray.column_index as f32 * cam.width_recip - 1.0) * cam.aspect;
    let flashlight_intensity_factor = (1.0 - (ray.wall_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0)) * super::FLASHLIGHT_INTENSITY;
    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, rgba)| {
            let row_dist = -(ray.origin.y - tile.ground_level) * row_dist_factor
                / (y as f32 - shearing_plus_half_height);
            let pos = ray.origin + row_dist * pos_factor;
            //let tex_x = ((tex_width as f32 * (pos.x - tile_x as f32)) as usize)
            //    .min(tex_width - 1);
            //let tex_y = ((tex_height as f32 * (pos.z - tile_z as f32)) as usize)
            //    .min(tex_height - 1);
            let tex_x = ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
            let tex_y =
                ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
            let color = &texture[i..i + 4];
            rgba.copy_from_slice(color);

            let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
            for color in &mut rgba[0..3] {
                let t = 1.0 - (row_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
                let spotlight = t * t * (3.0 - t * 2.0);
                let flashlight_intensity = (super::FLASHLIGHT_RADIUS - (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt()) * flashlight_intensity_factor;
                let intensity = flashlight_intensity.max(0.0) + spotlight + draw_params.ambient_light;
                *color = (*color as f32 * intensity) as u8;
            }
        });

    draw_to
}

pub(super) fn draw_top_platform(draw_params: DrawParams, column: &mut [u8]) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    //let tile_x = draw_params.tile_x;
    //let tile_z = draw_params.tile_z;

    let top_platform_texture = draw_params.texture_manager.get(tile.ceiling_tex);
    if top_platform_texture.is_empty() {
        return bottom_draw_bound;
    }
    let (texture, tex_width, tex_height) = (
        top_platform_texture.data,
        top_platform_texture.width as usize,
        top_platform_texture.height as usize,
    );

    // TODO WRONG NAMES: pixels_to_bottom, etc.
    // Draw from:
    let half_wall_pixel_height = cam.f_half_height / ray.wall_dist * cam.plane_dist;
    let pixels_to_bottom =
        half_wall_pixel_height * (-tile.ceiling_level + ray.origin.y) - cam.y_shearing;
    let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    // Draw to:
    let half_wall_pixel_height =
        cam.f_half_height / ray.previous_wall_dist * cam.plane_dist;
    let pixels_to_bottom =
        half_wall_pixel_height * (-tile.ceiling_level + ray.origin.y) - cam.y_shearing;
    let draw_to = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(draw_from, top_draw_bound);

    let ray_dir = ray.camera_dir - ray.horizontal_plane;
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    let pos_factor = ray_dir + tile_step_factor * ray.column_index as f32;
    let row_dist_factor = cam.f_half_height * cam.plane_dist;
    let shearing_plus_half_height = cam.y_shearing + cam.f_half_height;

    let flashlight_x = (2.0 * ray.column_index as f32 * cam.width_recip - 1.0) * cam.aspect;
    let flashlight_intensity_factor = (1.0 - (ray.wall_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0)) * super::FLASHLIGHT_INTENSITY;
    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, rgba)| {
            let row_dist = (-ray.origin.y + tile.ceiling_level) * row_dist_factor
                / (y as f32 - shearing_plus_half_height);
            let pos = ray.origin + row_dist * pos_factor;
            //let tex_x = ((tex_width as f32 * (pos.x - tile_x as f32)) as usize)
            //    .min(tex_width - 1);
            //let tex_y = ((tex_height as f32 * (pos.z - tile_z as f32)) as usize)
            //    .min(tex_height - 1);
            let tex_x = ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
            let tex_y =
                ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
            let color = &texture[i..i + 4];
            rgba.copy_from_slice(color);

            let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
            for color in &mut rgba[0..3] {
                let t = 1.0 - (row_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
                let spotlight = t * t * (3.0 - t * 2.0);
                let flashlight_intensity = (super::FLASHLIGHT_RADIUS - (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt()) * flashlight_intensity_factor;
                let intensity = flashlight_intensity.max(0.0) + spotlight + draw_params.ambient_light;
                *color = (*color as f32 * intensity) as u8;
            }
        });
    draw_from
}
