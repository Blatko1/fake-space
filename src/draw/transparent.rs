use super::{blend, RayHit, Raycaster, Side};

impl Raycaster {
    pub fn draw_transparent(
        &self,
        hit: RayHit,
        data: &mut [u8],
    ) {
        let mut color = [0, 0, 0, 0];

        let tex = hit.texture.unwrap();
        let (texture, tex_width, tex_height, bottom_height, top_height) = (
            tex.texture,
            tex.width,
            tex.height,
            tex.bottom_height,
            tex.top_height,
        );

        // TODO better names
        let full_line_pixel_height =
            (self.height as f32 / (hit.wall_dist) / self.aspect) as i32;
        let top_height =
            ((full_line_pixel_height / 2) as f32 * top_height) as i32;
        let bottom_height =
            ((full_line_pixel_height / 2) as f32 * bottom_height) as i32;
        let line_height = top_height.saturating_add(bottom_height);

        let begin = (self.int_half_height - bottom_height).max(0) as u32;
        let end = ((self.int_half_height + top_height).max(0) as u32)
            .min(self.height - 1);

        let tex_height_minus_one = tex_height as f32 - 1.0;
        let tex_x = match hit.side {
            Side::Vertical if hit.dir.x > 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
            }
            Side::Horizontal if hit.dir.z < 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as u32 - 1
            }
            _ => (hit.wall_x * tex_width as f32) as u32,
        };
        let four_tex_x = tex_x * 4;
        let tex_y_step = tex_height as f32 / line_height as f32;
        let mut tex_y = (begin as f32 + bottom_height as f32
            - self.float_half_height)
            * tex_y_step;

        for y in begin..end {
            let index = (self.height as usize - 1 - y as usize)
                * self.four_width
                + hit.screen_x as usize * 4;
            let rgba = &mut data[index..index + 4];
            let alpha = rgba[3];
            if alpha == 255 {
                tex_y += tex_y_step;
                continue;
            }

            let tex_y_pos = tex_y.min(tex_height_minus_one).round() as u32;
            let i = ((tex_height - tex_y_pos - 1) * tex_width * 4 + four_tex_x)
                as usize;
            color.copy_from_slice(&texture[i..i + 4]);
            tex_y += tex_y_step;
            let a = color[3];
            if a == 0 {
                continue;
            }
            if let Side::Horizontal = hit.side {
                color[0] = color[0].saturating_sub(15);
                color[1] = color[1].saturating_sub(15);
                color[2] = color[2].saturating_sub(15);
            }
            if a == 255 {
                if alpha == 0 {
                    rgba.copy_from_slice(&color);
                } else {
                    rgba.copy_from_slice(&blend(&color, rgba));
                }
            } else {
                rgba.copy_from_slice(&blend(&color, rgba));
            }
        }
    }
}
