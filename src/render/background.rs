use std::f32::consts::PI;
use crate::render::{DrawParams, Side};

const QUARTER_PI: f32 = PI / 4.0;
const THREE_QUARTER_PI: f32 = PI * 3.0 / 4.0;

pub fn draw_background(draw_params: DrawParams, column: &mut[u8]) {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;

    let texture = draw_params.texture_manager.get(draw_params.background_tex);
    let (tex_width, tex_height) = (
        texture.width as usize,
        texture.height as usize,
    );
    let texture = texture.data;


    let half_wall_pixel_height = if ray.angle <= QUARTER_PI || ray.angle >= THREE_QUARTER_PI {
        cam.f_half_height / (ray.delta_dist_x * 0.5) * cam.plane_dist
    } else {
        cam.f_half_height / (ray.delta_dist_z * 0.5) * cam.plane_dist
    };
    let pixels_to_bottom =
        half_wall_pixel_height * (1.0) - cam.y_shearing;
    let pixels_to_top =
        half_wall_pixel_height * (1.0) + cam.y_shearing;
    let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;
    if ray.column_index == 0 {
        println!("ray: {}", ray.delta_dist_x);
    }

    // From which pixel to begin drawing and on which to end
    let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);
    let draw_to = ((cam.f_half_height + pixels_to_top) as usize)
        .clamp(bottom_draw_bound, top_draw_bound);

    let tex_x = match ray.wall_side_hit {
        Side::Vertical if ray.dir.x > 0.0 => {
            tex_width - (ray.wall_offset * tex_width as f32) as usize - 1
        }
        Side::Horizontal if ray.dir.z < 0.0 => {
            tex_width - (ray.wall_offset * tex_width as f32) as usize - 1
        }
        _ => (ray.wall_offset * tex_width as f32) as usize,
    };
    let tex_y_step = tex_height as f32
        / full_wall_pixel_height
        / (2.0 / (1.0));
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