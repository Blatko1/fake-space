use crate::map::portal::{Portal, Rotation};
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
    //pub plane_x: f32,

    /// Ray origin.
    pub origin: Vec3,
    /// Direction of the camera from which the ray was cast.
    pub camera_dir: Vec3,
    /// Horizontal plane of the Camera from which the ray was cast.
    pub horizontal_plane: Vec3,

    // Variables below change per each DDA step
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
    pub wall_side: WallSide,
    /// Offset which represent where exactly was the wall hit
    /// (at which x coordinate).
    pub wall_offset: f32,
}

impl Ray {
    pub fn new(camera: &Camera, origin: Vec3, column_index: usize) -> Ray {
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
                origin.x.fract()
            } else {
                1.0 - origin.x.fract()
            };
        // Distance to nearest z side
        let side_dist_z = delta_dist_z
            * if dir.z < 0.0 {
                origin.z.fract()
            } else {
                1.0 - origin.z.fract()
            };

        let wall_dist = 0.0;
        // West/East side
        let (side, wall_side, wall_offset) = if dir.x.abs() >= dir.z.abs() {
            let wall_offset = origin.z + wall_dist * dir.z;
            let wall_side = match dir.x >= 0.0 {
                true => WallSide::East,
                false => WallSide::West,
            };
            (Side::Vertical, wall_side, wall_offset - wall_offset.floor())
        }
        // North/South side
        else {
            let wall_offset = origin.x + wall_dist * dir.x;
            let wall_side = match dir.z >= 0.0 {
                true => WallSide::North,
                false => WallSide::South,
            };
            (
                Side::Horizontal,
                wall_side,
                wall_offset - wall_offset.floor(),
            )
        };

        Ray {
            column_index,
            dir,
            delta_dist_x,
            delta_dist_z,
            step_x: dir.x.signum() as i64,
            step_z: dir.z.signum() as i64,
            //plane_x,
            origin,
            camera_dir: camera.forward_dir,
            horizontal_plane: camera.horizontal_plane,

            // Variables that change per each DDA step
            side_dist_x,
            side_dist_z,
            next_tile: PointXZ::new(origin.x as i64, origin.z as i64),
            wall_dist,
            previous_wall_dist: wall_dist,
            hit_wall_side: side,
            wall_side,
            wall_offset,
        }
    }

    pub fn new_one_step(camera: &Camera, origin: Vec3, column_index: usize) -> Ray {
        let mut ray = Self::new(camera, origin, column_index);
        if ray.side_dist_x < ray.side_dist_z {
            ray.wall_dist = ray.side_dist_x.max(0.0);
            ray.next_tile.x += ray.step_x;
            ray.side_dist_x += ray.delta_dist_x;
            ray.hit_wall_side = Side::Vertical;
            let wall_offset = ray.origin.z + ray.wall_dist * ray.dir.z;
            ray.wall_offset = wall_offset - wall_offset.floor();
        } else {
            ray.wall_dist = ray.side_dist_z.max(0.0);
            ray.next_tile.z += ray.step_z;
            ray.side_dist_z += ray.delta_dist_z;
            ray.hit_wall_side = Side::Horizontal;
            let wall_offset = ray.origin.x + ray.wall_dist * ray.dir.x;
            ray.wall_offset = wall_offset - wall_offset.floor();
        }
        ray
    }

    pub fn rotate(&mut self, rotation: Rotation) {
        match rotation {
            Rotation::Deg180 => {}
            Rotation::ClockwiseDeg90 => {
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
                self.wall_side = match self.wall_side {
                    WallSide::North => WallSide::East,
                    WallSide::East => WallSide::South,
                    WallSide::South => WallSide::West,
                    WallSide::West => WallSide::North,
                };
            }
            Rotation::AnticlockwiseDeg90 => {
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
                self.wall_side = match self.wall_side {
                    WallSide::North => WallSide::West,
                    WallSide::East => WallSide::North,
                    WallSide::South => WallSide::East,
                    WallSide::West => WallSide::South,
                };
            }
            Rotation::Deg0 => {
                // No difference, so the ray should be rotated 180 degrees
                self.dir = -self.dir;
                self.camera_dir = -self.camera_dir;
                self.horizontal_plane = -self.horizontal_plane;
                self.step_x = -self.step_x;
                self.step_z = -self.step_z;
                self.wall_side = match self.wall_side {
                    WallSide::North => WallSide::South,
                    WallSide::East => WallSide::West,
                    WallSide::South => WallSide::North,
                    WallSide::West => WallSide::East,
                };
            }
        }
    }

    pub fn portal_teleport(&mut self, src: Portal, dest: Portal) {
        self.origin = src.teleport_to(self.origin, dest);

        self.next_tile = PointXZ::new(dest.position.x as i64, dest.position.z as i64);
        match self.hit_wall_side {
            Side::Vertical => {
                self.side_dist_x -= self.delta_dist_x;
            }
            Side::Horizontal => {
                self.side_dist_z -= self.delta_dist_z;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WallSide {
    North,
    East,
    South,
    West,
}
