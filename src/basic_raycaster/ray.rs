use crate::map::portal::{Portal, PortalRotationDifference};
use crate::raycaster::camera::Camera;
use glam::Vec3;

use super::{PointXZ, Side};

// TODO maybe rename to `MovingRay`
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Index of the column for which the Ray was cast.
    pub column_index: usize,
    /// Direction of the Ray.
    pub dir: Vec3,
    /// Distance the Ray needs to cover to reach a new
    /// vertical wall from the previous vertical wall.
    pub delta_dist_x: f32,
    /// Distance the Ray needs to cover to reach a new
    /// horizontal wall from the previous horizontal wall.
    pub delta_dist_z: f32,
    /// Specifies in which direction (left or right)
    /// the ray travels across the map.
    pub step_x: i64,
    /// Specifies in which direction (forwards or backwards)
    /// the ray travels across the map.
    pub step_z: i64,
    /// X-coordinate on the horizontal camera plane through which the Ray passes.
    pub plane_x: f32,

    /// Ray cast origin.
    pub origin: Vec3,
    /// Direction of the Camera from which the ray was cast.
    pub camera_dir: Vec3,
    /// Horizontal plane of the Camera from which the ray was cast.
    pub horizontal_plane: Vec3,

    // Variables that change per each DDA step
    /// Distance which the ray has already traveled over in order
    /// to reach a new vertical wall.
    pub side_dist_x: f32,
    /// Distance which the ray has already traveled over in order
    /// to reach a new horizontal wall.
    pub side_dist_z: f32,
    /// Coordinates of the tile the ray just hit.
    pub next_tile: PointXZ<i64>,
    /// Distance to the just hit wall on the hit tile.
    pub wall_dist: f32,
    /// Distance to the previous just hit wall.
    pub previous_wall_dist: f32,
    /// The side of which the wall was hit.
    pub hit_wall_side: Side,
    /// Offset which represent where exactly was the wall hit
    /// (at which x coordinate).
    pub wall_offset: f32,
    pub skybox_wall_offset: f32
}

impl Ray {
    pub fn camera_cast(camera: &Camera, column_index: usize) -> Ray {
        // X-coordinate on the horizontal camera plane (range [-1.0, 1.0])
        let plane_x = 2.0 * column_index as f32 * camera.width_recip - 1.0;
        // Ray direction for current pixel column
        let dir = camera.forward_dir + camera.horizontal_plane * plane_x;
        // Length of ray from one x/z side to next x/z side on the tile_map
        let delta_dist_z = 1.0 / dir.z.abs();
        let delta_dist_x = 1.0 / dir.x.abs();
        // Distance to nearest x side
        let side_dist_x = delta_dist_x
            * if dir.x < 0.0 {
                camera.origin.x.fract()
            } else {
                1.0 - camera.origin.x.fract()
            };
        // Distance to nearest z side
        let side_dist_z = delta_dist_z
            * if dir.z < 0.0 {
                camera.origin.z.fract()
            } else {
                1.0 - camera.origin.z.fract()
            };

        let wall_dist = 0.0;
        let (side, wall_offset) = if side_dist_x < side_dist_z {
            let wall_offset = camera.origin.z + wall_dist * dir.z;
            (Side::Vertical, wall_offset - wall_offset.floor())
        } else {
            let wall_offset = camera.origin.x + wall_dist * dir.x;
            (Side::Horizontal, wall_offset - wall_offset.floor())
        };

        let skybox_wall_offset = 
        if (dir.x >= 0.0 && dir.z.abs() <= dir.x) || 
            (dir.x < 0.0 && dir.z.abs() <= -dir.x) {
                let t = 0.5 / dir.x;
                0.5 - t * dir.z
            } else {
                let t = 0.5 / dir.z;
                0.5 + t * dir.x
            };

        Ray {
            column_index,
            dir,
            delta_dist_x,
            delta_dist_z,
            step_x: dir.x.signum() as i64,
            step_z: dir.z.signum() as i64,
            plane_x,

            origin: camera.origin,
            camera_dir: camera.forward_dir,
            horizontal_plane: camera.horizontal_plane,

            // Variables that change per each DDA step
            side_dist_x,
            side_dist_z,
            next_tile: PointXZ::new(camera.origin.x as i64, camera.origin.z as i64),
            wall_dist,
            previous_wall_dist: wall_dist,
            hit_wall_side: side,
            wall_offset,
            skybox_wall_offset
        }
    }

    pub fn portal_teleport(&mut self, src: Portal, dest: Portal) {
        let (new_origin, _) = src.teleport(self.origin, dest);
        self.origin = new_origin;
        self.next_tile = PointXZ::new(dest.position.x as i64, dest.position.z as i64);
        match self.hit_wall_side {
            Side::Vertical => {
                self.side_dist_x -= self.delta_dist_x;
            }
            Side::Horizontal => {
                self.side_dist_z -= self.delta_dist_z;
            }
        }

        match src.direction.rotation_difference(dest.direction) {
            PortalRotationDifference::None => {}
            PortalRotationDifference::ClockwiseDeg90 => {
                // Rotate 90 degrees clockwise
                self.dir = Vec3::new(self.dir.z, 0.0, -self.dir.x);
                self.camera_dir = Vec3::new(self.camera_dir.z, 0.0, -self.camera_dir.x);
                self.horizontal_plane =
                    Vec3::new(self.horizontal_plane.z, 0.0, -self.horizontal_plane.x);

                std::mem::swap(&mut self.delta_dist_x, &mut self.delta_dist_z);
                std::mem::swap(&mut self.side_dist_x, &mut self.side_dist_z);
                self.step_x = self.dir.x.signum() as i64;
                self.step_z = self.dir.z.signum() as i64;
                self.hit_wall_side = match self.hit_wall_side {
                    Side::Vertical => Side::Horizontal,
                    Side::Horizontal => Side::Vertical,
                };
            }
            PortalRotationDifference::AnticlockwiseDeg90 => {
                // Rotate 90 degrees anticlockwise
                self.dir = Vec3::new(-self.dir.z, 0.0, self.dir.x);
                self.camera_dir = Vec3::new(-self.camera_dir.z, 0.0, self.camera_dir.x);
                self.horizontal_plane =
                    Vec3::new(-self.horizontal_plane.z, 0.0, self.horizontal_plane.x);

                std::mem::swap(&mut self.delta_dist_x, &mut self.delta_dist_z);
                std::mem::swap(&mut self.side_dist_x, &mut self.side_dist_z);
                self.step_x = self.dir.x.signum() as i64;
                self.step_z = self.dir.z.signum() as i64;
                self.hit_wall_side = match self.hit_wall_side {
                    Side::Vertical => Side::Horizontal,
                    Side::Horizontal => Side::Vertical,
                };
            }
            PortalRotationDifference::Deg180 => {
                // Rotate 180 degrees
                self.dir = -self.dir;
                self.camera_dir = -self.camera_dir;
                self.horizontal_plane = -self.horizontal_plane;
                self.step_x = -self.step_x;
                self.step_z = -self.step_z;
            }
        }
    }
}
