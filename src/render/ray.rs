use glam::Vec3;

use super::camera::Camera;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub x: usize,
    pub origin: Vec3,
    pub dir: Vec3,
    pub camera_dir: Vec3,
    pub horizontal_plane: Vec3,
    pub delta_dist_x: f32,
    pub delta_dist_z: f32,
    pub side_dist_x: f32,
    pub side_dist_z: f32,
    pub next_tile_x: i64,
    pub next_tile_z: i64,
    pub step_x: i64,
    pub step_z: i64,
}

impl Ray {
    pub fn cast_with_camera(x: usize, camera: &Camera) -> Self {
        Self::cast(x, camera.origin, camera.dir, camera.view_width, camera.horizontal_plane)
    }

    pub fn cast(x: usize, origin: Vec3, camera_dir: Vec3, view_width: u32, horizontal_camera_plane: Vec3) -> Self {
        // X-coordinate on the horizontal camera plane (range [-1.0, 1.0])
        let plane_x = 2.0 * (x as f32 * (view_width as f32).recip()) - 1.0;
        // Ray direction for current pixel column
        let ray_dir = camera_dir + horizontal_camera_plane * plane_x;
        // Length of ray from one x/z side to next x/z side on the tile_map
        let delta_dist_x = 1.0 / ray_dir.x.abs();
        let delta_dist_z = 1.0 / ray_dir.z.abs();
        // Distance to nearest x side
        let side_dist_x = delta_dist_x
            * if ray_dir.x < 0.0 {
                origin.x.fract()
            } else {
                1.0 - origin.x.fract()
            };
        // Distance to nearest z side
        let side_dist_z = delta_dist_z
            * if ray_dir.z < 0.0 {
                origin.z.fract()
            } else {
                1.0 - origin.z.fract()
            };

        Self {
            x,
            origin,
            dir: ray_dir,
            horizontal_plane: horizontal_camera_plane,
            camera_dir,
            delta_dist_x,
            delta_dist_z,
            side_dist_x,
            side_dist_z,
            // Coordinates of the map tile the camera is in
            next_tile_x: origin.x as i64,
            next_tile_z: origin.z as i64,
            step_x: ray_dir.x.signum() as i64,
            step_z: ray_dir.z.signum() as i64,
        }
    }
}