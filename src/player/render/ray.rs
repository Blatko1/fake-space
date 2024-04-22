use crate::world::portal::{Portal, PortalRotationDifference};
use glam::Vec3;

use super::{PointXZ, Side};

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

    /// Origin of the Camera from which the ray was cast.
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
}

impl Ray {
    pub fn dda_step(&mut self) {
        if self.side_dist_x < self.side_dist_z {
            self.wall_dist = self.side_dist_x.max(0.0);
            self.next_tile.x += self.step_x;
            self.side_dist_x += self.delta_dist_x;
            self.hit_wall_side = Side::Vertical;
            let wall_offset = self.origin.z + self.wall_dist * self.dir.z;
            self.wall_offset = wall_offset - wall_offset.floor();
        } else {
            self.wall_dist = self.side_dist_z.max(0.0);
            self.next_tile.z += self.step_z;
            self.side_dist_z += self.delta_dist_z;
            self.hit_wall_side = Side::Horizontal;
            let wall_offset = self.origin.x + self.wall_dist * self.dir.x;
            self.wall_offset = wall_offset - wall_offset.floor();
        }
    }

    pub fn portal_teleport(&mut self, src: Portal, dest: Portal) {
        self.origin.x += (dest.position.x as i64 - self.next_tile.x) as f32;
        self.origin.z += (dest.position.z as i64 - self.next_tile.z) as f32;
        self.origin.y -= src.ground_level - dest.ground_level;
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
                // Rotate 90 degrees clockwise and reposition the origin
                let origin_x = dest.center.x - (dest.center.z - self.origin.z);
                let origin_z = dest.center.z + (dest.center.x - self.origin.x);

                self.origin.x = origin_x;
                self.origin.z = origin_z;
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
                // Rotate 90 degrees anticlockwise and reposition the origin
                let origin_x = dest.center.x + (dest.center.z - self.origin.z);
                let origin_z = dest.center.z - (dest.center.x - self.origin.x);

                self.origin.x = origin_x;
                self.origin.z = origin_z;
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
