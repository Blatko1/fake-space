use glam::Vec3;
use crate::render::PointXZ;
use crate::world::portal::{Portal, PortalRotationDifference};

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
    pub next_tile: PointXZ<i64>,
    pub step_x: i64,
    pub step_z: i64,
}

impl Ray {
    pub fn cast_with_camera(x: usize, cam: &Camera) -> Self {
        let origin = cam.origin;
        let camera_dir = cam.dir;
        let horizontal_plane = cam.horizontal_plane;

        // X-coordinate on the horizontal camera plane (range [-1.0, 1.0])
        let plane_x = 2.0 * (x as f32 * cam.width_recip) - 1.0;
        // Ray direction for current pixel column
        let ray_dir = camera_dir + cam.horizontal_plane * plane_x;
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
            horizontal_plane,
            camera_dir,
            delta_dist_x,
            delta_dist_z,
            side_dist_x,
            side_dist_z,
            // Coordinates of the tile from which the ray starts
            next_tile: PointXZ {x: origin.x as i64, z: origin.z as i64},
            step_x: ray_dir.x.signum() as i64,
            step_z: ray_dir.z.signum() as i64,
        }
    }

    pub fn portal_teleport(&mut self, src: Portal, dest: Portal) {
        self.origin.x += (dest.position.x as i64 - self.next_tile.x) as f32;
        self.origin.z += (dest.position.z as i64 - self.next_tile.z) as f32;
        self.origin.y -= src.ground_level - dest.ground_level;
        self.next_tile = PointXZ {x: dest.position.x as i64, z: dest.position.z as i64};

        match src
            .direction
            .rotation_difference(dest.direction)
        {
            PortalRotationDifference::None => (),
            PortalRotationDifference::ClockwiseDeg90 => {
                // Rotate 90 degrees clockwise and reposition the origin
                let origin_x = dest.center.x - (dest.center.z - self.origin.z);
                let origin_z = dest.center.z + (dest.center.x - self.origin.x);
                self.origin.x = origin_x;
                self.origin.z = origin_z;
                self.dir = Vec3::new(self.dir.z, 0.0, -self.dir.x);
                self.camera_dir =
                    Vec3::new(self.camera_dir.z, 0.0, -self.camera_dir.x);
                self.horizontal_plane = Vec3::new(
                    self.horizontal_plane.z,
                    0.0,
                    -self.horizontal_plane.x,
                );

                std::mem::swap(&mut self.delta_dist_x, &mut self.delta_dist_z);
                std::mem::swap(&mut self.side_dist_x, &mut self.side_dist_z);
                self.step_x = self.dir.x.signum() as i64;
                self.step_z = self.dir.z.signum() as i64;
            }
            PortalRotationDifference::AnticlockwiseDeg90 => {
                // Rotate 90 degrees anticlockwise and reposition the origin
                let origin_x = dest.center.x + (dest.center.z - self.origin.z);
                let origin_z = dest.center.z - (dest.center.x - self.origin.x);
                self.origin.x = origin_x;
                self.origin.z = origin_z;
                self.horizontal_plane = Vec3::new(
                    -self.horizontal_plane.z,
                    0.0,
                    self.horizontal_plane.x,
                );
                self.camera_dir =
                    Vec3::new(-self.camera_dir.z, 0.0, self.camera_dir.x);
                self.dir = Vec3::new(-self.dir.z, 0.0, self.dir.x);

                std::mem::swap(&mut self.delta_dist_x, &mut self.delta_dist_z);
                std::mem::swap(&mut self.side_dist_x, &mut self.side_dist_z);
                self.step_x = self.dir.x.signum() as i64;
                self.step_z = self.dir.z.signum() as i64;
            }
            PortalRotationDifference::Deg180 => {
                // Rotate 180 degrees and reposition the origin
                self.origin.x = dest.center.x + (dest.center.x - self.origin.x);
                self.origin.z = dest.center.z + (dest.center.z - self.origin.z);
                self.dir = -self.dir;
                self.camera_dir = -self.camera_dir;
                self.horizontal_plane = -self.horizontal_plane;
                self.step_x = -self.step_x;
                self.step_z = -self.step_z;
            }
        }
    }
}
