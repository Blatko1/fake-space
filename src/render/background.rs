use std::f32::consts::PI;
use glam::{Vec2, Vec3};
use crate::render::{DrawParams, Side};
use crate::world::textures::Texture;

const QUARTER_PI: f32 = PI / 4.0;
const THREE_QUARTER_PI: f32 = PI * 3.0 / 4.0;

pub fn draw_background(draw_params: DrawParams, column: &mut[u8]) {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let mut ray = draw_params.ray;
    ray.wall_dist = 1.0;

    let (tex_width, tex_height, texture);

    // TODO sky box is temporarily hard-coded
    let half_wall_pixel_height;
    // Skybox wall to the north
    if ray.dir.z >= 0.0 && ray.dir.x.abs() <= ray.dir.z {
        let t = 0.5 / ray.dir.z;
        ray.wall_offset = 0.5 + t * ray.dir.x;
        ray.wall_side_hit = Side::Horizontal;
        half_wall_pixel_height = cam.f_half_height / (ray.delta_dist_z * 0.5) * cam.plane_dist;

        let tex = draw_params.texture_manager.get(Texture::ID(8));
        (tex_width, tex_height) = (
            tex.width as usize,
            tex.height as usize,
        );
        texture = tex.data;
    }
    // Skybox wall to the east
    else if ray.dir.x >= 0.0 && ray.dir.z.abs() <= ray.dir.x {
        let t = 0.5 / ray.dir.x;
        ray.wall_offset = 1.0 - (0.5 + t * ray.dir.z);
        ray.wall_side_hit = Side::Vertical;
        half_wall_pixel_height = cam.f_half_height / (ray.delta_dist_x * 0.5) * cam.plane_dist;

        let tex = draw_params.texture_manager.get(Texture::ID(9));
        (tex_width, tex_height) = (
            tex.width as usize,
            tex.height as usize,
        );
        texture = tex.data;
    }
    // Skybox wall to the west
    else if ray.dir.x < 0.0 && ray.dir.z.abs() <= -ray.dir.x {
        let t = 0.5 / ray.dir.x;
        ray.wall_offset = 1.0 - (0.5 + t * ray.dir.z);
        ray.wall_side_hit = Side::Vertical;
        half_wall_pixel_height = cam.f_half_height / (ray.delta_dist_x * 0.5) * cam.plane_dist;

        let tex = draw_params.texture_manager.get(Texture::ID(11));
        (tex_width, tex_height) = (
            tex.width as usize,
            tex.height as usize,
        );
        texture = tex.data;
    }
    // Skybox wall to the south
    else {
        let t = 0.5 / ray.dir.z;
        ray.wall_offset = 0.5 + t * ray.dir.x;
        ray.wall_side_hit = Side::Horizontal;
        half_wall_pixel_height = cam.f_half_height / (ray.delta_dist_z * 0.5) * cam.plane_dist;

        let tex = draw_params.texture_manager.get(Texture::ID(10));
        (tex_width, tex_height) = (
            tex.width as usize,
            tex.height as usize,
        );
        texture = tex.data;
    };


    let pixels_to_bottom =
        half_wall_pixel_height - cam.y_shearing;
    let pixels_to_top =
        half_wall_pixel_height + cam.y_shearing;
    let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

    // From which pixel to begin drawing and on which to end
    let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);
    let draw_to = ((cam.f_half_height + pixels_to_top) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    let tex_x = (ray.wall_offset * tex_width as f32) as usize;
    let tex_y_step = tex_height as f32
        / full_wall_pixel_height;
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
            dest.copy_from_slice(src);
            //}
            // TODO maybe make it so `tex_y_step` is being subtracted.
            tex_y += tex_y_step;
        });
}