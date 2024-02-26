use crate::render::{DrawParams, Side};
use crate::world::textures::TextureDataRef;
use glam::Vec3;
use std::ptr;

// TODO solve maybe? entering new rooms rotates camera which rotates the skybox
pub fn draw_skybox(mut draw_params: DrawParams, column: &mut [u8]) {
    let cam = draw_params.camera;
    let ray = &mut draw_params.ray;
    ray.wall_dist = 1.0;

    let wall_texture;

    let half_wall_pixel_height;
    // Skybox wall to the north
    if ray.dir.z >= 0.0 && ray.dir.x.abs() <= ray.dir.z {
        let t = 0.5 / ray.dir.z;
        ray.wall_offset = 0.5 + t * ray.dir.x;
        ray.hit_wall_side = Side::Horizontal;
        //half_wall_pixel_height =
        //    cam.f_half_height / (ray.delta_dist_z * 0.5) * cam.plane_dist;
        half_wall_pixel_height = cam.f_height / ray.delta_dist_z * cam.plane_dist;

        wall_texture = draw_params.texture_manager.get(draw_params.skybox.north);
    }
    // Skybox wall to the east
    else if ray.dir.x >= 0.0 && ray.dir.z.abs() <= ray.dir.x {
        let t = 0.5 / ray.dir.x;
        // ray.wall_offset = 1.0 - (0.5 + t * ray.dir.z);
        ray.wall_offset = 0.5 - t * ray.dir.z;
        ray.hit_wall_side = Side::Vertical;
        half_wall_pixel_height = cam.f_height / ray.delta_dist_x * cam.plane_dist;

        wall_texture = draw_params.texture_manager.get(draw_params.skybox.east);
    }
    // Skybox wall to the west
    else if ray.dir.x < 0.0 && ray.dir.z.abs() <= -ray.dir.x {
        let t = 0.5 / ray.dir.x;
        ray.wall_offset = 0.5 - t * ray.dir.z;
        ray.hit_wall_side = Side::Vertical;
        half_wall_pixel_height = cam.f_height / ray.delta_dist_x * cam.plane_dist;

        wall_texture = draw_params.texture_manager.get(draw_params.skybox.west);
    }
    // Skybox wall to the south
    else {
        let t = 0.5 / ray.dir.z;
        ray.wall_offset = 0.5 + t * ray.dir.x;
        ray.hit_wall_side = Side::Horizontal;
        half_wall_pixel_height = cam.f_height / ray.delta_dist_z * cam.plane_dist;

        wall_texture = draw_params.texture_manager.get(draw_params.skybox.south);
    };

    draw_skybox_top(draw_params, half_wall_pixel_height, column);
    draw_skybox_bottom(draw_params, half_wall_pixel_height, column);
    draw_skybox_wall(draw_params, half_wall_pixel_height, wall_texture, column);
}

fn draw_skybox_top(
    draw_params: DrawParams,
    half_wall_pixel_height: f32,
    column: &mut [u8],
) {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;

    let top_texture = draw_params.texture_manager.get(draw_params.skybox.top);

    let (texture, tex_width, tex_height) = (
        top_texture.data,
        top_texture.width as usize,
        top_texture.height as usize,
    );

    // TODO wrong names
    // Draw from:
    let pixels_to_bottom = half_wall_pixel_height * 0.5 - cam.y_shearing;
    let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    // Draw to:
    let draw_to = top_draw_bound;

    // Variables used for reducing the amount of calculations and for optimization
    let ray_dir = ray.camera_dir - ray.horizontal_plane;
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    let pos_factor = ray_dir + tile_step_factor * ray.column_index as f32;
    let row_dist_factor = cam.f_half_height * cam.plane_dist;
    let half_h_plus_shearing = cam.f_half_height + cam.y_shearing;
    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, rgba)| {
            let row_dist = row_dist_factor / (y as f32 - half_h_plus_shearing);
            let pos = Vec3::new(0.5, 0.5, 0.5) + row_dist * pos_factor;
            let tex_x = ((tex_width as f32 * pos.x) as usize).min(tex_width - 1);
            let tex_y = ((tex_height as f32 * pos.z) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x);
            let color = &texture[i..i + 4];
            //rgba.copy_from_slice(color);
            unsafe {
                ptr::copy_nonoverlapping(color.as_ptr(), rgba.as_mut_ptr(), rgba.len())
            }
        });
}

fn draw_skybox_bottom(
    draw_params: DrawParams,
    half_wall_pixel_height: f32,
    column: &mut [u8],
) {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;

    let bottom_texture = draw_params.texture_manager.get(draw_params.skybox.bottom);

    let (texture, tex_width, tex_height) = (
        bottom_texture.data,
        bottom_texture.width as usize,
        bottom_texture.height as usize,
    );

    // TODO wrong names
    // Draw from:
    let draw_from = bottom_draw_bound;

    // Draw to:
    let pixels_to_bottom = half_wall_pixel_height * 0.5 + cam.y_shearing;
    let draw_to = ((cam.f_half_height + pixels_to_bottom) as usize)
        .clamp(draw_from, top_draw_bound);

    // Variables used for reducing the amount of calculations and for optimization
    let ray_dir = ray.camera_dir - ray.horizontal_plane;
    let tile_step_factor = ray.horizontal_plane * 2.0 * cam.width_recip;
    let pos_factor = ray_dir + tile_step_factor * ray.column_index as f32;
    let row_dist_factor = cam.f_half_height * cam.plane_dist;
    let half_h_plus_shearing = cam.f_half_height + cam.y_shearing;
    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, rgba)| {
            let row_dist = row_dist_factor / (y as f32 - half_h_plus_shearing);
            let pos = Vec3::new(0.5, 0.5, 0.5) + row_dist * pos_factor;
            let tex_x = ((tex_width as f32 * pos.x) as usize).min(tex_width - 1);
            let tex_y = ((tex_height as f32 * pos.z) as usize).min(tex_height - 1);
            let i = 4 * (tex_width * tex_y + tex_x);
            let color = &texture[i..i + 4];
            //rgba.copy_from_slice(color);
            unsafe {
                ptr::copy_nonoverlapping(color.as_ptr(), rgba.as_mut_ptr(), rgba.len())
            }
        });
}

fn draw_skybox_wall(
    draw_params: DrawParams,
    half_wall_pixel_height: f32,
    texture: TextureDataRef,
    column: &mut [u8],
) {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;

    let (texture, tex_width, tex_height) = (
        texture.data,
        texture.width as usize,
        texture.height as usize,
    );

    let pixels_to_bottom = half_wall_pixel_height - cam.y_shearing;
    let pixels_to_top = half_wall_pixel_height + cam.y_shearing;
    let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

    // From which pixel to begin drawing and on which to end
    let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);
    let draw_to =
        ((cam.f_half_height + pixels_to_top) as usize).clamp(draw_from, top_draw_bound);

    let tex_x = (ray.wall_offset * tex_width as f32) as usize;
    let tex_y_step = tex_height as f32 / full_wall_pixel_height;
    let mut tex_y =
        (draw_from as f32 + pixels_to_bottom - cam.f_half_height) * tex_y_step;
    // Precomputed variables for performance increase
    let four_tex_width = tex_width * 4;
    let four_tex_x = tex_x * 4;
    column
        .chunks_exact_mut(4)
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|dest| {
            //if dest[3] != 255 {
            let tex_y_pos = tex_y.round() as usize % tex_height;
            let i = (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
            let src = &texture[i..i + 4];

            // Draw the pixel:
            //draw_fn(dest, src);
            //dest.copy_from_slice(src);
            unsafe {
                ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), dest.len())
            }
            //}
            // TODO maybe make it so `tex_y_step` is being subtracted.
            tex_y += tex_y_step;
        });
}
