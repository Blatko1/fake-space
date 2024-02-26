use super::{blend, DrawParams, Side};

// TODO write tests for each draw call function to check for overflows
// Draws full and transparent walls.
pub(super) fn draw_bottom_wall(draw_params: DrawParams, column: &mut [u8]) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    let ambient = draw_params.ambient_light;

    let bottom_wall_texture = draw_params.texture_manager.get(tile.bottom_pillar_tex);
    if bottom_wall_texture.is_empty() {
        return top_draw_bound;
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
    let texture = bottom_wall_texture.light_shade;
    let (tex_width, tex_height) = (
        bottom_wall_texture.width as usize,
        bottom_wall_texture.height as usize,
    );

    // Calculate wall pixel height for the parts above and below the middle
    let half_wall_pixel_height = cam.f_half_height / ray.wall_dist * cam.plane_dist;
    let pixels_to_bottom =
        half_wall_pixel_height * (ray.origin.y - tile.bottom_level) - cam.y_shearing;
    let pixels_to_top =
        half_wall_pixel_height * (tile.ground_level - ray.origin.y) + cam.y_shearing;
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
    //let tex_y_step = tex_height as f32
    //    / full_wall_pixel_height
    //    / (2.0 / (tile.ground_level - tile.bottom_level));
    let tex_y_step = (tile.ground_level - tile.bottom_level) * tex_height as f32
        / full_wall_pixel_height
        * 0.5;
    let mut tex_y =
        (draw_from as f32 + pixels_to_bottom - cam.f_half_height) * tex_y_step;
    let draw_fn = match bottom_wall_texture.transparency {
        true => draw_transparent_wall_pixel,
        false => draw_full_wall_pixel,
    };

    // Precomputed variables for performance increase
    let four_tex_width = tex_width * 4;
    let four_tex_x = tex_x * 4;

    //let flashlight_x = 1.0 - ((ray.column_index as f32 - cam.view_width as f32 / 2.0) / (cam.view_width as f32 / 2.0)).abs();

    // TODO idk why this gives negative results
    let diffuse = (-ray.dir.dot(normal)).max(0.0);
    let flashlight_x =
        (2.0 * ray.column_index as f32 * cam.width_recip - 1.0) * cam.aspect;
    // Smooth out the flashlight intensity using the distance
    let flashlight_intensity = (1.0
        - (ray.wall_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
        * super::FLASHLIGHT_INTENSITY
        * diffuse;

    // Smoothstep distance to get the spotlight
    let t = 1.0 - (ray.wall_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
    let spotlight = t * t * (3.0 - t * 2.0);

    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, pixel)| {
            //if dest[3] != 255 {
            let tex_y_pos = tex_y.round() as usize % tex_height;
            let i = (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
            let color = &texture[i..i + 4];

            // Draw the pixel:
            //draw_fn(dest, src);

            let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
            for (dest, src) in pixel[0..3].iter_mut().zip(color[0..3].iter()) {
                let flashlight_radius = (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt();
                let t = 1.0 - ((flashlight_radius - super::FLASHLIGHT_INNER_RADIUS) / (super::FLASHLIGHT_OUTER_RADIUS - super::FLASHLIGHT_INNER_RADIUS)).clamp(0.0, 1.0);
                let flashlight = t * t * (3.0 - t * 2.0) * flashlight_intensity;
                *dest = (*src as f32 * (flashlight + ambient)) as u8;

            }
            pixel[3] = color[3];
            //}
            // TODO maybe make it so `tex_y_step` is being subtracted.
            tex_y += tex_y_step;
        });
    draw_to
}

pub(super) fn draw_top_wall(draw_params: DrawParams, column: &mut [u8]) -> usize {
    let bottom_draw_bound = draw_params.bottom_draw_bound;
    let top_draw_bound = draw_params.top_draw_bound;
    let cam = draw_params.camera;
    let ray = draw_params.ray;
    let tile = draw_params.tile;
    let ambient = draw_params.ambient_light;

    let top_wall_texture = draw_params.texture_manager.get(tile.top_pillar_tex);
    if top_wall_texture.is_empty() {
        return bottom_draw_bound;
    }

    //let texture = match ray.wall_side_hit {
    //    Side::Vertical => top_wall_texture.light_shade,
    //    Side::Horizontal => top_wall_texture.medium_shade,
    //};
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
    let texture = top_wall_texture.light_shade;
    let (tex_width, tex_height) = (
        top_wall_texture.width as usize,
        top_wall_texture.height as usize,
    );
    let draw_fn = match top_wall_texture.transparency {
        true => draw_transparent_wall_pixel,
        false => draw_full_wall_pixel,
    };

    // Calculate wall pixel height for the parts above and below the middle
    let half_wall_pixel_height = cam.f_half_height / ray.wall_dist * cam.plane_dist;
    let pixels_to_bottom =
        half_wall_pixel_height * (-tile.ceiling_level + ray.origin.y) - cam.y_shearing;
    let pixels_to_top =
        half_wall_pixel_height * (tile.top_level - ray.origin.y) + cam.y_shearing;
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
    let tex_y_step = (tile.top_level - tile.ceiling_level) * tex_height as f32
        / full_wall_pixel_height
        * 0.5;
    let mut tex_y =
        (draw_from as f32 + pixels_to_bottom - cam.f_half_height) * tex_y_step;

    // Precomputed variables for performance increase
    let four_tex_width = tex_width * 4;
    let four_tex_x = tex_x * 4;

    // TODO idk why this gives negative results
    let diffuse = (-ray.dir.dot(normal)).max(0.0);
    let flashlight_x =
        (2.0 * ray.column_index as f32 * cam.width_recip - 1.0) * cam.aspect;
    // Smooth out the flashlight intensity using the distance
    let flashlight_intensity = (1.0
        - (ray.wall_dist / super::FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
        * super::FLASHLIGHT_INTENSITY
        * diffuse;

    // Smoothstep distance to get the spotlight
    let t = 1.0 - (ray.wall_dist / super::SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
    let spotlight = t * t * (3.0 - t * 2.0);

    column
        .chunks_exact_mut(4)
        .enumerate()
        .skip(draw_from)
        .take(draw_to - draw_from)
        .for_each(|(y, pixel)| {
            //if dest[3] != 255 {
            let tex_y_pos = tex_y.round() as usize % tex_height;
            let i = (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
            let color = &texture[i..i + 4];

            // Draw the pixel:
            //draw_fn(dest, src);

            let flashlight_y = 2.0 * y as f32 * cam.height_recip - 1.0;
            for (dest, src) in pixel[0..3].iter_mut().zip(color[0..3].iter()) {
                let flashlight_radius = (flashlight_x * flashlight_x + flashlight_y * flashlight_y).sqrt();
                let t = 1.0 - ((flashlight_radius - super::FLASHLIGHT_INNER_RADIUS) / (super::FLASHLIGHT_OUTER_RADIUS - super::FLASHLIGHT_INNER_RADIUS)).clamp(0.0, 1.0);
                let flashlight = t * t * (3.0 - t * 2.0) * flashlight_intensity;
                *dest = (*src as f32 * (flashlight + ambient)) as u8;
            }
            pixel[3] = color[3];
            //}
            // TODO maybe make it so `tex_y_step` is being subtracted.
            tex_y += tex_y_step;
        });

    draw_from
}

#[inline]
fn draw_full_wall_pixel(dest: &mut [u8], color: &[u8]) {
    if dest[3] == 0 {
        dest.copy_from_slice(color);
    } else {
        dest.copy_from_slice(&blend(color, dest));
    }
}

#[inline]
fn draw_transparent_wall_pixel(dest: &mut [u8], color: &[u8]) {
    let a = color[3];
    if a == 0 {
        return;
    }
    if a == 255 {
        draw_full_wall_pixel(dest, color);
    } else {
        dest.copy_from_slice(&blend(color, dest));
    }
}
