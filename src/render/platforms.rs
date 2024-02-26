// TODO problem! some textures below the walls
// are bleeding out when further away
// TODO problem! adding unsafe could improve performance

use super::DrawParams;
use glam::Vec3;

pub(super) fn draw_bottom_platform(draw_params: DrawParams, column: &mut [u8]) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let mut ray = draw_params.ray;
    let tile = draw_params.tile;
    let ambient = draw_params.ambient_light;
    let normal = super::NORMAL_Y_POSITIVE;
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
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    let pos_factor = ray.camera_dir - ray.horizontal_plane
        + tile_step_factor * ray.column_index as f32;
    let row_dist_factor = cam.f_half_height * cam.plane_dist;
    //let shearing_plus_half_height = cam.y_shearing + cam.f_half_height;

    // Calculating lightning
    let flashlight_x =
        (2.0 * ray.column_index as f32 * cam.width_recip - 1.0) * cam.aspect;
    // Smoothstep distance to get the spotlight
    let t = 1.0 - (ray.wall_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
    let spotlight = t * t * (3.0 - t * 2.0);

    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, pixel)| {
            let row_dist = (tile.ground_level - ray.origin.y) * row_dist_factor
                / (y as f32 - cam.y_shearing - cam.f_half_height);
            let mut ray_dir = row_dist * pos_factor;
            let pos = ray.origin + ray_dir;

            //let tex_x = ((tex_width as f32 * (pos.x - tile_x as f32)) as usize)
            //    .min(tex_width - 1);
            //let tex_y = ((tex_height as f32 * (pos.z - tile_z as f32)) as usize)
            //    .min(tex_height - 1);
            let tex_x = ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
            let tex_y =
                ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
            let color = &texture[i..i + 4];

            // Calculate the diffuse lightning by finding the direction of the ray with pitch
            ray_dir.y += ray.origin.y - tile.ground_level;
            let diffuse = ray_dir.normalize().dot(normal);
            // Smooth out the flashlight intensity using the distance
            let flashlight_intensity = (1.0
                - (row_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
                * super::FLASHLIGHT_INTENSITY
                * diffuse;
            let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
            //if ray.column_index as u32 == cam.view_width / 2 && y as u32 == cam.view_height / 2 {
            //    println!("P: dif: {}, intesn: {}, dir: {}", diffuse, flashlight_intensity, ray_dir);
            //}
            for (dest, src) in pixel[0..3].iter_mut().zip(color[0..3].iter()) {
                let flashlight_radius = (super::FLASHLIGHT_RADIUS
                    - (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt())
                .clamp(0.0, 1.0);
                let flashlight = (flashlight_radius * flashlight_intensity).max(0.0);
                *dest = (*src as f32 * (flashlight + ambient)) as u8;
            }
            pixel[3] = color[3];
        });

    draw_to
}

pub(super) fn draw_top_platform(draw_params: DrawParams, column: &mut [u8]) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    let ambient = draw_params.ambient_light;
    let normal = super::NORMAL_Y_NEGATIVE;
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

    let flashlight_x =
        (2.0 * ray.column_index as f32 * cam.width_recip - 1.0) * cam.aspect;
    // Smoothstep distance to get the spotlight
    let t = 1.0 - (ray.wall_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
    let spotlight = t * t * (3.0 - t * 2.0);

    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, pixel)| {
            let row_dist = (tile.ceiling_level - ray.origin.y) * row_dist_factor
                / (y as f32 - shearing_plus_half_height);
            let pos = ray.origin + row_dist * pos_factor;
            let mut dir = row_dist * pos_factor;
            dir.y += pos.y - tile.ceiling_level;
            let diffuse = dir.normalize().dot(normal);
            //let tex_x = ((tex_width as f32 * (pos.x - tile_x as f32)) as usize)
            //    .min(tex_width - 1);
            //let tex_y = ((tex_height as f32 * (pos.z - tile_z as f32)) as usize)
            //    .min(tex_height - 1);
            let tex_x = ((tex_width as f32 * pos.x.fract()) as usize).min(tex_width - 1);
            let tex_y =
                ((tex_height as f32 * pos.z.fract()) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x); //tex_width * 4 * tex_y + tex_x * 4
            let color = &texture[i..i + 4];
            pixel.copy_from_slice(color);

            // Smooth out the flashlight intensity using the distance
            let flashlight_intensity = (1.0
                - (row_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
                * super::FLASHLIGHT_INTENSITY
                * diffuse;
            let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
            for (dest, src) in pixel[0..3].iter_mut().zip(color[0..3].iter()) {
                let flashlight_radius = (super::FLASHLIGHT_RADIUS
                    - (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt())
                .clamp(0.0, 1.0);
                let flashlight = (flashlight_radius * flashlight_intensity).max(0.0);
                *dest = (*src as f32 * (flashlight + ambient)) as u8;
            }
            pixel[3] = color[3];
        });
    draw_from
}
