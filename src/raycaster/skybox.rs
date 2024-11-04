use glam::Vec3;
use std::ptr;

use super::{camera::Camera, ray::Ray};
use super::Side;
use crate::textures::{SkyboxTexturesRef, TextureDataRef};

/// Draws over the whole screen (simultaneously clears the whole screen).
pub(super) struct SkyboxSegment<'a> {
    camera: &'a Camera,
    ray: Ray,
    textures: SkyboxTexturesRef<'a>,
    bottom_draw_bound: usize,
    top_draw_bound: usize,
}

impl<'a> SkyboxSegment<'a> {
    pub fn new(
        camera: &'a Camera,
        ray: Ray,
        textures: SkyboxTexturesRef<'a>,
    ) -> SkyboxSegment<'a> {
        SkyboxSegment {
            camera,
            ray,
            textures,
            bottom_draw_bound: 0,
            top_draw_bound: camera.view_height as usize,
        }
    }

    // TODO solve maybe? entering new rooms rotates camera which rotates the skybox
    pub fn draw_skybox(mut self, column: &mut [u8]) {
        let cam = self.camera;
        let ray = &mut self.ray;
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
            half_wall_pixel_height = cam.f_height / ray.delta_dist_z;

            wall_texture = self.textures.north;
        }
        // Skybox wall to the east
        else if ray.dir.x >= 0.0 && ray.dir.z.abs() <= ray.dir.x {
            let t = 0.5 / ray.dir.x;
            // ray.wall_offset = 1.0 - (0.5 + t * ray.dir.z);
            ray.wall_offset = 0.5 - t * ray.dir.z;
            ray.hit_wall_side = Side::Vertical;
            half_wall_pixel_height = cam.f_height / ray.delta_dist_x;

            wall_texture = self.textures.east;
        }
        // Skybox wall to the west
        else if ray.dir.x < 0.0 && ray.dir.z.abs() <= -ray.dir.x {
            let t = 0.5 / ray.dir.x;
            ray.wall_offset = 0.5 - t * ray.dir.z;
            ray.hit_wall_side = Side::Vertical;
            half_wall_pixel_height = cam.f_height / ray.delta_dist_x;

            wall_texture = self.textures.west;
        }
        // Skybox wall to the south
        else {
            let t = 0.5 / ray.dir.z;
            ray.wall_offset = 0.5 + t * ray.dir.x;
            ray.hit_wall_side = Side::Horizontal;
            half_wall_pixel_height = cam.f_height / ray.delta_dist_z;

            wall_texture = self.textures.south;
        };

        self.draw_skybox_top(half_wall_pixel_height, column);
        self.draw_skybox_bottom(half_wall_pixel_height, column);
        self.draw_skybox_wall(half_wall_pixel_height, wall_texture, column);
    }

    fn draw_skybox_top(&self, half_wall_pixel_height: f32, column: &mut [u8]) {
        let cam = self.camera;
        let ray = self.ray;

        let top_texture = self.textures.top;
        if top_texture.is_empty() {
            column.fill(0);
            return;
        }

        let (texture, tex_width, tex_height) = (
            top_texture.data,
            top_texture.width as usize,
            top_texture.height as usize,
        );

        // TODO wrong names
        // Draw from:
        let pixels_to_bottom = half_wall_pixel_height * 0.5 - cam.y_shearing;
        let draw_from = ((cam.f_half_height - pixels_to_bottom) as usize)
            .clamp(self.bottom_draw_bound, self.top_draw_bound);

        // Draw to:
        let draw_to = self.top_draw_bound;

        // Variables used for reducing the amount of calculations and for optimization
        let ray_dir = self.camera.forward_dir - self.camera.horizontal_plane;
        let tile_step_factor = self.camera.horizontal_plane * 2.0 * cam.width_recip;
        let pos_factor = ray_dir + tile_step_factor * ray.column_index as f32;
        let half_h_plus_shearing = cam.f_half_height + cam.y_shearing;
        column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist = cam.f_half_height / (y as f32 - half_h_plus_shearing);
                let pos = Vec3::new(0.5, 0.5, 0.5) + row_dist * pos_factor;
                let tex_x = ((tex_width as f32 * pos.x) as usize).min(tex_width - 1);
                let tex_y = ((tex_height as f32 * pos.z) as usize).min(tex_height - 1);
                let i = 4 * (tex_width * tex_y + tex_x);
                let color = &texture[i..i + 3];

                unsafe {
                    ptr::copy_nonoverlapping(
                        color.as_ptr(),
                        rgba.as_mut_ptr(),
                        rgba.len(),
                    )
                }
            });
    }

    fn draw_skybox_bottom(&self, half_wall_pixel_height: f32, column: &mut [u8]) {
        let cam = self.camera;
        let ray = self.ray;

        let bottom_texture = self.textures.bottom;
        if bottom_texture.is_empty() {
            column.fill(0);
            return;
        }

        let (texture, tex_width, tex_height) = (
            bottom_texture.data,
            bottom_texture.width as usize,
            bottom_texture.height as usize,
        );

        // TODO wrong names
        // Draw from:
        let draw_from = self.bottom_draw_bound;

        // Draw to:
        let pixels_to_bottom = half_wall_pixel_height * 0.5 + cam.y_shearing;
        let draw_to = ((cam.f_half_height + pixels_to_bottom) as usize)
            .clamp(draw_from, self.top_draw_bound);

        // Variables used for reducing the amount of calculations and for optimization
        let tile_step_factor = self.camera.horizontal_plane * 2.0 * cam.width_recip;
        let pos_factor = self.camera.forward_dir - self.camera.horizontal_plane
            + tile_step_factor * ray.column_index as f32;
        column
            .chunks_exact_mut(3)
            .enumerate()
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|(y, rgba)| {
                let row_dist =
                    cam.f_half_height / (y as f32 - cam.f_half_height - cam.y_shearing);
                let pos = Vec3::new(0.5, 0.5, 0.5) + row_dist * pos_factor;
                let tex_x =
                    ((tex_width as f32 * (1.0 - pos.x)) as usize).min(tex_width - 1);
                let tex_y = ((tex_height as f32 * pos.z) as usize).min(tex_height - 1);
                let i = 4 * (tex_width * tex_y + tex_x);
                let color = &texture[i..i + 3];

                unsafe {
                    ptr::copy_nonoverlapping(
                        color.as_ptr(),
                        rgba.as_mut_ptr(),
                        rgba.len(),
                    )
                }
            });
    }

    fn draw_skybox_wall(
        &self,
        half_wall_pixel_height: f32,
        texture: TextureDataRef,
        column: &mut [u8],
    ) {
        let cam = self.camera;
        let ray = self.ray;

        if texture.is_empty() {
            column.fill(0);
            return;
        }
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
            .clamp(self.bottom_draw_bound, self.top_draw_bound);
        let draw_to = ((cam.f_half_height + pixels_to_top) as usize)
            .clamp(draw_from, self.top_draw_bound);

        let tex_x = (ray.wall_offset * tex_width as f32) as usize;
        let tex_y_step = tex_height as f32 / full_wall_pixel_height;
        let mut tex_y =
            (draw_from as f32 + pixels_to_bottom - cam.f_half_height) * tex_y_step;
        // Precomputed variables for performance increase
        let four_tex_width = tex_width * 4;
        let four_tex_x = tex_x * 4;
        column
            .chunks_exact_mut(3)
            .skip(draw_from)
            .take(draw_to - draw_from)
            .for_each(|dest| {
                let tex_y_pos = tex_y.round() as usize % tex_height;
                let i = (tex_height - tex_y_pos - 1) * four_tex_width + four_tex_x;
                let src = &texture[i..i + 3];

                // Draw the pixel:
                unsafe {
                    ptr::copy_nonoverlapping(src.as_ptr(), dest.as_mut_ptr(), dest.len())
                }
                // TODO maybe make it so `tex_y_step` is being subtracted.
                tex_y += tex_y_step;
            });
    }
}
