// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! trying to implement sprite entities
// is difficult due to existence of transparent walls
// and their fully transparent parts
// TODO problem! adding unsafe could improve performance

use super::{camera::Camera, DrawParams};

pub(super) fn draw_bottom_platform(
    cam: &Camera,
    draw_params: DrawParams,
    column: &mut [u8],
) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    let closer_wall_dist = draw_params.closer_wall_dist;
    let further_wall_dist = draw_params.further_wall_dist;
    let tile_x = draw_params.tile_x;
    let tile_z = draw_params.tile_z;

    let ground_texture = draw_params.texture_manager.get(tile.ground_tex);
    if ground_texture.is_empty() {
        return bottom_draw_bound;
    }
    let (texture, tex_width, tex_height) = (
        ground_texture.data,
        ground_texture.width as usize,
        ground_texture.height as usize,
    );

    // Draw from (always drawing from bottom to top):
    let half_wall_pixel_height = cam.f_half_height / closer_wall_dist * cam.plane_dist;
    let pixels_to_top =
        half_wall_pixel_height * (tile.ground_level - ray.origin.y) + cam.y_shearing;
    let draw_from = ((cam.f_half_height + pixels_to_top) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    // Draw to:
    let half_wall_pixel_height = cam.f_half_height / further_wall_dist * cam.plane_dist;

    let pixels_to_top =
        half_wall_pixel_height * (tile.ground_level - ray.origin.y) + cam.y_shearing;
    let draw_to =
        ((cam.f_half_height + pixels_to_top) as usize).clamp(draw_from, top_draw_bound);

    let ray_dir = ray.camera_dir - ray.horizontal_plane;
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    column
        .chunks_exact_mut(4)
        .rev()
        .enumerate()
        .skip(cam.view_height as usize - draw_to)
        .take(draw_to - draw_from)
        .for_each(|(y, rgba)| {
            let row_dist = ((ray.origin.y - tile.ground_level) / 2.0) * cam.f_height
                / (y as f32 - cam.f_height / 2.0 + cam.y_shearing)
                * cam.plane_dist;
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
    cam: &Camera,
    draw_params: DrawParams,
    column: &mut [u8],
) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    let closer_wall_dist = draw_params.closer_wall_dist;
    let further_wall_dist = draw_params.further_wall_dist;
    let tile_x = draw_params.tile_x;
    let tile_z = draw_params.tile_z;

    let top_platform_texture = draw_params.texture_manager.get(tile.ceiling_tex);
    if top_platform_texture.is_empty() {
        return top_draw_bound;
    }
    let (texture, tex_width, tex_height) = (
        top_platform_texture.data,
        top_platform_texture.width as usize,
        top_platform_texture.height as usize,
    );

    // Draw from:
    let half_wall_pixel_height = cam.f_half_height / further_wall_dist * cam.plane_dist;
    let pixels_to_bottom =
        half_wall_pixel_height * (-tile.ceiling_level + ray.origin.y) - cam.y_shearing;
    let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    // Draw to:
    let half_wall_pixel_height = cam.f_half_height / closer_wall_dist * cam.plane_dist;
    let pixels_to_bottom =
        half_wall_pixel_height * (-tile.ceiling_level + ray.origin.y) - cam.y_shearing;
    let draw_to = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(draw_from, top_draw_bound);

    let ray_dir = ray.camera_dir - ray.horizontal_plane;
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, rgba)| {
            let row_dist = ((-ray.origin.y + tile.ceiling_level) / 2.0) * cam.f_height
                / (y as f32 - cam.f_height / 2.0 - cam.y_shearing)
                * cam.plane_dist;
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
