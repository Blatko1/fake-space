/*use glam::Vec3;

use crate::{textures::{TextureDataRef, TextureManager}, map::Map};

use super::{Side, RayHit, RayCaster};

type DrawnFromY = usize;
type DrawnToY = usize;

pub(super) struct SingleFrameRenderer<'a> {
    ray_caster: &'a RayCaster,
    map: &'a Map,
    texture_manager: &'a TextureManager,
    dest: &'a mut [u8]
}

impl<'a> SingleFrameRenderer<'a> {
    fn new(ray_caster: &RayCaster, map: &Map, texture_manager: &TextureManager, dest: &mut [u8] ) -> Self {
        Self {
            ray_caster,
            map,
            texture_manager,
            dest,
        }
    }
    fn render(&self) {
        let height = self.ray_caster.height as f32;
        let width = self.ray_caster.width as f32;
        let half_height = height / 2.0;
        let width_recip = width.recip();

        let dir = self.ray_caster.dir;
        let plane_h = self.ray_caster.plane_h;
        let y_shearing = self.ray_caster.y_shearing;
        let pos = self.ray_caster.pos;

        // For each pixel column on the screen
        self.dest.chunks_exact_mut(height as usize * 4)
            .enumerate()
            .for_each(|(x, column)| {
                // X-coordinate on the horizontal camera plane (range [-1.0, 1.0])
                let plane_x = 2.0 * (x as f32 * width_recip) - 1.0;
                // Ray direction for current pixel column
                let ray_dir = dir + plane_h * plane_x;
                // Length of ray from one x/z side to next x/z side on the tile_map
                let delta_dist_x = 1.0 / ray_dir.x.abs();
                let delta_dist_z = 1.0 / ray_dir.z.abs();
                // Distance to nearest x side
                let mut side_dist_x = delta_dist_x
                    * if ray_dir.x < 0.0 {
                        pos.x.fract()
                    } else {
                        1.0 - pos.x.fract()
                    };
                // Distance to nearest z side
                let mut side_dist_z = delta_dist_z
                    * if ray_dir.z < 0.0 {
                        pos.z.fract()
                    } else {
                        1.0 - pos.z.fract()
                    };
                // Coordinates of the map tile the raycaster is in
                let mut map_x = pos.x as i32;
                let mut map_z = pos.z as i32;
                let (step_x, step_z) =
                    (ray_dir.x.signum() as i32, ray_dir.z.signum() as i32);

                // DDA loop
                let mut previous_perp_wall_dist = 0.0;
                let mut bottom_draw_bound = 0usize;
                let mut top_draw_bound = height as usize;
                loop {
                    let current_map_x = map_x;
                    let current_map_z = map_z;
                    // Distance to the first hit wall's x/z side if the wall isn't empty
                    let side = if side_dist_x < side_dist_z {
                        map_x += step_x;
                        side_dist_x += delta_dist_x;
                        Side::Vertical
                    } else {
                        map_z += step_z;
                        side_dist_z += delta_dist_z;
                        Side::Horizontal
                    };
                    // Calculate perpetual wall distance from the camera and wall_x.
                    // wall_x represents which part of wall was hit from the left border (0.0)
                    // to the right border (0.99999) and everything in between in range <0.0, 1.0>
                    let (perp_wall_dist, wall_x) = match side {
                        Side::Vertical => {
                            let dist = side_dist_x - delta_dist_x;
                            let wall_x = pos.z + dist * ray_dir.z;
                            (dist.max(0.0), wall_x - wall_x.floor())
                        }
                        Side::Horizontal => {
                            let dist = side_dist_z - delta_dist_z;
                            let wall_x = pos.x + dist * ray_dir.x;
                            (dist.max(0.0), wall_x - wall_x.floor())
                        }
                    };
                    /* ---------------------------------------------------
                    ---- DRAWING BOTTOM AND TOP PLATFORM OF CURRENT TILE ----
                       --------------------------------------------------- */
                    let current_tile =
                        match self.map.get_tile(current_map_x, current_map_z) {
                            Some(t) => t,
                            None => {
                                // draw non moving background
                                break;
                            }
                        };
                    let bottom_platform_texture = self.texture_manager.get(current_tile.bottom_platform_tex);
                    if !bottom_platform_texture.is_empty() {
                        let (texture, tex_width, tex_height) = (
                            bottom_platform_texture.data,
                            bottom_platform_texture.width as usize,
                            bottom_platform_texture.height as usize,
                        );
                        let y_level = current_tile.level2;
                        // Draw from:
                        let half_wall_pixel_height = half_height
                            / previous_perp_wall_dist
                            * self.plane_dist;
                        let pixels_to_top = half_wall_pixel_height
                            * (y_level - pos.y)
                            + y_shearing;
                        let draw_from = ((half_height + pixels_to_top)
                            as usize)
                            .clamp(bottom_draw_bound, top_draw_bound);
                        // Draw to:
                        let half_wall_pixel_height = half_height
                            / perp_wall_dist
                            * self.plane_dist;

                        let pixels_to_top = half_wall_pixel_height
                            * (y_level - pos.y)
                            + y_shearing;
                        let draw_to = ((half_height + pixels_to_top)
                            as usize)
                            .clamp(draw_from, top_draw_bound);

                        let ray_dir = dir - plane_h;
                        let tile_step_factor =
                            plane_h * 2.0 * width_recip;
                        column
                            .chunks_exact_mut(4)
                            .rev()
                            .enumerate()
                            .skip(self.height as usize - draw_to)
                            .take(draw_to - draw_from)
                            .for_each(|(y, rgba)| {
                                let row_dist = ((self.pos.y - y_level) / 2.0)
                                    * self.f_height
                                    / (y as f32 - self.f_height / 2.0
                                        + self.y_shearing)
                                    * self.plane_dist;
                                let step = tile_step_factor * row_dist;
                                let pos = self.pos
                                    + ray_dir * row_dist
                                    + step * x as f32;
                                let tex_x = ((tex_width as f32
                                    * (pos.x - current_map_x as f32))
                                    as usize)
                                    .min(tex_width - 1);
                                let tex_y = ((tex_height as f32
                                    * (pos.z - current_map_z as f32))
                                    as usize)
                                    .min(tex_height - 1);
                                let i = tex_width * 4 * tex_y + tex_x * 4;
                                let color = &texture[i..i + 4];
                                rgba.copy_from_slice(color);
                            });

                            bottom_draw_bound = draw_to;
                        /*if let Some(first) = column.chunks_exact_mut(4).nth(draw_to) {
                            first.copy_from_slice(&[255, 255, 255, 255]);
                        };
                        if let Some(first) = column.chunks_exact_mut(4).nth(draw_from) {
                            first.copy_from_slice(&[255, 0, 0, 255]);
                        };*/
                    }
                    // Draw top part of cube
                    /*let drawn_to = self.draw_bottom_platform(
                        previous_perp_wall_dist,
                        perp_wall_dist,
                        bottom_draw_bound,
                        top_draw_bound,
                        current_tile.level2,
                        textures.get(current_tile.bottom_platform),
                        x as u32,
                        current_map_x as f32,
                        current_map_z as f32,
                        column,
                    );
                    bottom_draw_bound = drawn_to;
                    // Draw top part of cube
                    let drawn_from = self.draw_top_platform(
                        previous_perp_wall_dist,
                        perp_wall_dist,
                        bottom_draw_bound,
                        top_draw_bound,
                        current_tile.level3,
                        textures.get(current_tile.top_platform),
                        x as u32,
                        current_map_x as f32,
                        current_map_z as f32,
                        column,
                    );
                    top_draw_bound = drawn_from;
                    let next_tile = match tile_map.get_tile(map_x, map_z) {
                        Some(t) => t,
                        None => {
                            // draw non moving background
                            break;
                        }
                    };
                    let hit = RayHit {
                        screen_x: x as u32,
                        dir: ray_dir,
                        wall_dist: perp_wall_dist,
                        side,
                        wall_x,
                        bottom_draw_bound,
                        top_draw_bound,
                        delta_dist_x,
                        delta_dist_z,
                    };
                    let drawn_to = self.draw_bottom_wall(
                        hit,
                        textures.get(next_tile.pillar1_tex),
                        bottom_draw_bound,
                        top_draw_bound,
                        next_tile.level1,
                        next_tile.level2,
                        column,
                    );
                    bottom_draw_bound = drawn_to.max(bottom_draw_bound);
                    let drawn_from = self.draw_top_wall(
                        hit,
                        textures.get(next_tile.pillar2_tex),
                        bottom_draw_bound,
                        top_draw_bound,
                        next_tile.level3,
                        next_tile.level4,
                        column,
                    );
                    top_draw_bound = drawn_from.min(top_draw_bound);*/

                    previous_perp_wall_dist = perp_wall_dist;
                }
            });
    }

    pub fn draw_bottom_platform(
        &self,
        draw_from_wall_dist: f32,
        draw_to_wall_dist: f32,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        y_level: f32,
        texture_data: TextureDataRef<'_>,
        draw_x: u32,
        position_x: f32,
        position_z: f32,
        column: &mut [u8],
    ) -> usize {
        if texture_data.is_empty() {
            return bottom_draw_bound;
        }
        let (texture, tex_width, tex_height) = (
            texture_data.data,
            texture_data.width as usize,
            texture_data.height as usize,
        );

        // Draw from (alway drawing from bottom to top):
        let half_wall_pixel_height =
            self.f_half_height / draw_from_wall_dist * self.plane_dist;
        let pixels_to_top =
            half_wall_pixel_height * (y_level - self.pos.y) + self.y_shearing;
        let draw_from = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            self.f_half_height / draw_to_wall_dist * self.plane_dist;

        let pixels_to_top =
            half_wall_pixel_height * (y_level - self.pos.y) + self.y_shearing;
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(draw_from, top_draw_bound);

        let ray_dir = self.dir - self.plane_h;
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        column
            .chunks_exact_mut(4)
            .rev()
            .enumerate()
            .skip(self.height as usize - draw_to)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = ((self.pos.y - y_level) / 2.0) * self.f_height
                    / (y as f32 - self.f_height / 2.0 + self.y_shearing)
                    * self.plane_dist;
                let step = tile_step_factor * row_dist;
                let pos = self.pos + ray_dir * row_dist + step * draw_x as f32;
                let tex_x = ((tex_width as f32 * (pos.x - position_x))
                    as usize)
                    .min(tex_width - 1);
                let tex_y = ((tex_height as f32 * (pos.z - position_z))
                    as usize)
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

    pub fn draw_top_platform(
        &self,
        draw_from_wall_dist: f32,
        draw_to_wall_dist: f32,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        y_level: f32,
        texture_data: TextureDataRef<'_>,
        draw_x: u32,
        position_x: f32,
        position_z: f32,
        column: &mut [u8],
    ) -> usize {
        if texture_data.is_empty() {
            return top_draw_bound;
        }

        let (texture, tex_width, tex_height) = (
            texture_data.data,
            texture_data.width as usize,
            texture_data.height as usize,
        );

        // Draw from:
        let half_wall_pixel_height =
            self.f_half_height / draw_to_wall_dist * self.plane_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (-y_level + self.pos.y) - self.y_shearing;
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        // Draw to:
        let half_wall_pixel_height =
            self.f_half_height / draw_from_wall_dist * self.plane_dist;
        let pixels_to_bottom =
            half_wall_pixel_height * (-y_level + self.pos.y) - self.y_shearing;
        let draw_to = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(draw_from, top_draw_bound);

        let ray_dir = self.dir - self.plane_h;
        let tile_step_factor = self.plane_h * 2.0 * self.width_recip;
        column
            .chunks_exact_mut(4)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = ((-self.pos.y + y_level) / 2.0) * self.f_height
                    / (y as f32 - self.f_height / 2.0 - self.y_shearing)
                    * self.plane_dist;
                let step = tile_step_factor * row_dist;
                let pos = self.pos + ray_dir * row_dist + step * draw_x as f32;
                let tex_x = ((tex_width as f32 * (pos.x - position_x))
                    as usize)
                    .min(tex_width - 1);
                let tex_y = ((tex_height as f32 * (pos.z - position_z))
                    as usize)
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

    pub fn draw_bottom_wall(
        &self,
        hit: RayHit,
        texture_data: TextureDataRef,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        bottom_y_bound: f32,
        top_y_bound: f32,
        column: &mut [u8],
    ) -> usize {
        if texture_data.is_empty() {
            return top_draw_bound;
        }
        let texture = match hit.side {
            Side::Vertical => texture_data.light_shade,
            Side::Horizontal => texture_data.medium_shade,
        };
        let (tex_width, tex_height) =
            (texture_data.width as usize, texture_data.height as usize);

        // Calculate wall pixel height for the parts above and below the middle
        let half_wall_pixel_height =
            self.f_half_height / hit.wall_dist * self.plane_dist;
        let pixels_to_bottom = half_wall_pixel_height
            * (-bottom_y_bound + self.pos.y)
            - self.y_shearing;
        let pixels_to_top = half_wall_pixel_height * (top_y_bound - self.pos.y)
            + self.y_shearing;
        let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

        // From which pixel to begin drawing and on which to end
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        if draw_from == draw_to {
            return draw_to;
        }

        let tex_x = match hit.side {
            Side::Vertical if hit.dir.x > 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as usize - 1
            }
            Side::Horizontal if hit.dir.z < 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as usize - 1
            }
            _ => (hit.wall_x * tex_width as f32) as usize,
        };
        let tex_y_step = tex_height as f32
            / full_wall_pixel_height
            / (2.0 / (top_y_bound - bottom_y_bound));
        let mut tex_y = (draw_from as f32 + pixels_to_bottom
            - self.f_half_height)
            * tex_y_step;
        let draw_fn = match texture_data.transparency {
            true => draw_transparent_wall_pixel,
            false => draw_full_wall_pixel,
        };

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
                let i =
                    (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
                let src = &texture[i..i + 4];

                // Draw the pixel:
                draw_fn(dest, src);
                //}
                // TODO maybe make it so `tex_y_step` is being subtracted.
                tex_y += tex_y_step;
            });

        /*if let Some(first) = column
        .chunks_exact_mut(4)
        .nth(draw_from)
        {
            first.copy_from_slice(&[255, 100, 255, 255]);
        };
        if let Some(first) = column
            .chunks_exact_mut(4)
            .nth(draw_to)
        {
            first.copy_from_slice(&[255, 100, 0, 255]);
        };*/

        draw_to
    }

    pub fn draw_top_wall(
        &self,
        hit: RayHit,
        texture_data: TextureDataRef,
        bottom_draw_bound: usize,
        top_draw_bound: usize,
        bottom_y_bound: f32,
        top_y_bound: f32,
        column: &mut [u8],
    ) -> usize {
        if texture_data.is_empty() {
            return bottom_draw_bound;
        }
        let texture = match hit.side {
            Side::Vertical => texture_data.light_shade,
            Side::Horizontal => texture_data.medium_shade,
        };
        let (tex_width, tex_height) =
            (texture_data.width as usize, texture_data.height as usize);

        // Calculate wall pixel height for the parts above and below the middle
        let half_wall_pixel_height =
            self.f_half_height / hit.wall_dist * self.plane_dist;
        let pixels_to_bottom = half_wall_pixel_height
            * (-bottom_y_bound + self.pos.y)
            - self.y_shearing;
        let pixels_to_top = half_wall_pixel_height * (top_y_bound - self.pos.y)
            + self.y_shearing;
        let full_wall_pixel_height = pixels_to_top + pixels_to_bottom;

        // From which pixel to begin drawing and on which to end
        let draw_from = ((self.f_half_height - pixels_to_bottom) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);
        let draw_to = ((self.f_half_height + pixels_to_top) as usize)
            .clamp(bottom_draw_bound, top_draw_bound);

        if draw_from == draw_to {
            return draw_from;
        }

        let tex_x = match hit.side {
            Side::Vertical if hit.dir.x > 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as usize - 1
            }
            Side::Horizontal if hit.dir.z < 0.0 => {
                tex_width - (hit.wall_x * tex_width as f32) as usize - 1
            }
            _ => (hit.wall_x * tex_width as f32) as usize,
        };
        let tex_y_step = tex_height as f32
            / full_wall_pixel_height
            / (2.0 / (top_y_bound - bottom_y_bound));
        let mut tex_y = (draw_from as f32 + pixels_to_bottom
            - self.f_half_height)
            * tex_y_step;
        let draw_fn = match texture_data.transparency {
            true => draw_transparent_wall_pixel,
            false => draw_full_wall_pixel,
        };

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
                let i =
                    (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
                let src = &texture[i..i + 4];

                // Draw the pixel:
                draw_fn(dest, src);
                //}
                // TODO maybe make it so `tex_y_step` is being subtracted.
                tex_y += tex_y_step;
            });

        /*if let Some(first) = column
        .chunks_exact_mut(4)
        .nth(draw_from)
        {
            first.copy_from_slice(&[255, 100, 255, 255]);
        };
        if let Some(first) = column
            .chunks_exact_mut(4)
            .nth(draw_to)
        {
            first.copy_from_slice(&[255, 100, 0, 255]);
        };*/

        draw_from
    }
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

// TODO convert to unsafe for speed
#[inline(always)]
fn blend(background: &[u8], foreground: &[u8]) -> [u8; 4] {
    let alpha = foreground[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    [
        ((foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha)
            as u8),
        ((foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha)
            as u8),
        ((foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha)
            as u8),
        (255.0 * alpha + background[3] as f32 * inv_alpha) as u8,
    ]
}*/
