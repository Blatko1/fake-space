use glam::Vec3;

use crate::object::ModelManager;

use super::{RayCast, RayHit, Raycaster, Side};

impl Raycaster {
    pub fn draw_object(
        &self,
        ray: &RayCast,
        obj_hit: &RayHit,
        models: &ModelManager,
        data: &mut [u8],
    ) {
        let object_hit = obj_hit.object.unwrap();
        let object = object_hit.obj.get_object(models);
        let dimension = object.dimension() as f32;
        let dimension_i = dimension as i32;
        let ray_origin = self.pos * dimension;
        let obj_x_pos = (object_hit.obj_map_pos_x as f32) * dimension;
        let obj_z_pos = (object_hit.obj_map_pos_z as f32) * dimension;
        // North is in front (positive Z)
        let (top_left_point, top_side, voxel_side) = match obj_hit.side {
            Side::Vertical => {
                // is east side hit
                if ray.dir.x > 0.0 {
                    let top_left =
                        Vec3::new(obj_x_pos, dimension, obj_z_pos + dimension);
                    (top_left, Vec3::new(0.0, 0.0, -dimension), VoxelSide::Left)
                }
                // is west side hit
                else {
                    let top_left =
                        Vec3::new(obj_x_pos + dimension, dimension, obj_z_pos);
                    (top_left, Vec3::new(0.0, 0.0, dimension), VoxelSide::Right)
                }
            }
            Side::Horizontal => {
                // is north side hit
                if ray.dir.z > 0.0 {
                    let top_left = Vec3::new(obj_x_pos, dimension, obj_z_pos);
                    (top_left, Vec3::new(dimension, 0.0, 0.0), VoxelSide::Front)
                }
                // is south side hit
                else {
                    let top_left = Vec3::new(
                        obj_x_pos + dimension,
                        dimension,
                        obj_z_pos + dimension,
                    );
                    (top_left, Vec3::new(-dimension, 0.0, 0.0), VoxelSide::Back)
                }
            }
        };
        let left_side = Vec3::new(0.0, -dimension, 0.0);
        // Calculate the normal vector (N) of the rectangle's surface.
        let rectangle_normal = top_side.cross(left_side);
        for y in 0..self.height {
            let mut color: [u8; 4] = [123, 234, 100, 255];
            // Y-coordinate on the vertical camera plane (range [-1.0, 1.0])
            let plane_y = 2.0 * (y as f32 * self.height_recip) - 1.0;
            // Ray direction for current pixel column
            let ray_dir = ray.dir + self.plane_v * plane_y;
            // Length of ray from one x/y/z side to next x/y/z side on the tile_map
            let delta_dist = Vec3::new(
                ray.delta_dist_x,
                1.0 / ray_dir.y.abs(),
                ray.delta_dist_z,
            );

            // Coordinates of the 3D model matrix the ray first interacts with
            let intersect = match rectangle_vector_intersection(
                top_left_point,
                top_side,
                left_side,
                rectangle_normal,
                ray_dir.normalize(),
                ray_origin,
                voxel_side,
            ) {
                Some(i) => i,
                None => continue,
            };
            let mut side_dist_x = delta_dist.x
                * if ray_dir.x < 0.0 {
                    intersect.x.fract()
                } else {
                    1.0 - intersect.x.fract()
                };
            let mut side_dist_y = delta_dist.y
                * if ray_dir.y < 0.0 {
                    intersect.y.fract()
                } else {
                    1.0 - intersect.y.fract()
                };
            let mut side_dist_z = delta_dist.z
                * if ray_dir.z < 0.0 {
                    intersect.z.fract()
                } else {
                    1.0 - intersect.z.fract()
                };
            let mut grid_x = (intersect.x - obj_x_pos) as i32;
            let mut grid_z = (intersect.z - obj_z_pos) as i32;
            let mut grid_y = intersect.y as i32;
            let (step_x, step_y, step_z) = (
                ray_dir.x.signum() as i32,
                ray_dir.y.signum() as i32,
                ray_dir.z.signum() as i32,
            );

            let mut side = voxel_side;
            match side {
                VoxelSide::Top | VoxelSide::Right | VoxelSide::Back => {
                    if side_dist_x < side_dist_y {
                        if side_dist_x < side_dist_z {
                            grid_x += step_x;
                            if grid_x < 0 {
                                continue;
                            }
                            side_dist_x += delta_dist.x;
                        } else {
                            grid_z += step_z;
                            if grid_z < 0 {
                                continue;
                            }
                            side_dist_z += delta_dist.z;
                        }
                    } else if side_dist_y < side_dist_z {
                        grid_y += step_y;
                        if grid_y < 0 {
                            continue;
                        }
                        side_dist_y += delta_dist.y;
                    } else {
                        grid_z += step_z;
                        if grid_z < 0 {
                            continue;
                        }
                        side_dist_z += delta_dist.z;
                    }
                }
                _ => (),
            }
            loop {
                let voxel = object.get_voxel(
                    grid_x as usize,
                    grid_y as usize,
                    grid_z as usize,
                );
                if voxel.is_none() {
                    break;
                }
                if let Some(1) = voxel {
                    match side {
                        VoxelSide::Top => (),
                        VoxelSide::Bottom => {
                            color[0] = color[0].saturating_sub(75);
                            color[1] = color[1].saturating_sub(75);
                            color[2] = color[2].saturating_sub(75);
                        }
                        VoxelSide::Left => {
                            color[0] = color[0].saturating_sub(25);
                            color[1] = color[1].saturating_sub(25);
                            color[2] = color[2].saturating_sub(25);
                        }
                        VoxelSide::Right => {
                            color[0] = color[0].saturating_sub(55);
                            color[1] = color[1].saturating_sub(55);
                            color[2] = color[2].saturating_sub(55);
                        }
                        VoxelSide::Front => {
                            color[0] = color[0].saturating_sub(15);
                            color[1] = color[1].saturating_sub(15);
                            color[2] = color[2].saturating_sub(15);
                        }
                        VoxelSide::Back => {
                            color[0] = color[0].saturating_sub(65);
                            color[1] = color[1].saturating_sub(65);
                            color[2] = color[2].saturating_sub(65);
                        }
                    }
                    let index = (self.height as usize - 1 - y as usize)
                        * self.four_width
                        + ray.screen_x as usize * 4;
                    data[index..index + 4].copy_from_slice(&color);
                    break;
                }
                if side_dist_x < side_dist_y {
                    if side_dist_x < side_dist_z {
                        grid_x += step_x;
                        if grid_x < 0 || grid_x >= dimension_i {
                            break;
                        }
                        side = if step_x.is_positive() {
                            VoxelSide::Left
                        } else {
                            VoxelSide::Right
                        };
                        side_dist_x += delta_dist.x;
                    } else {
                        grid_z += step_z;
                        if grid_z < 0 || grid_z >= dimension_i {
                            break;
                        }
                        side = if step_z.is_positive() {
                            VoxelSide::Front
                        } else {
                            VoxelSide::Back
                        };
                        side_dist_z += delta_dist.z;
                    }
                } else if side_dist_y < side_dist_z {
                    grid_y += step_y;
                    if grid_y < 0 || grid_y >= dimension_i {
                        break;
                    }
                    side = if step_y.is_positive() {
                        VoxelSide::Bottom
                    } else {
                        VoxelSide::Top
                    };
                    side_dist_y += delta_dist.y;
                } else {
                    grid_z += step_z;
                    if grid_z < 0 || grid_z >= dimension_i {
                        break;
                    }
                    side = if step_z.is_positive() {
                        VoxelSide::Front
                    } else {
                        VoxelSide::Back
                    };
                    side_dist_z += delta_dist.z;
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VoxelSide {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

#[inline]
fn rectangle_vector_intersection(
    corner: Vec3,
    top_side: Vec3,
    left_side: Vec3,
    rectangle_normal: Vec3,
    ray_dir: Vec3,
    ray_origin: Vec3,
    side: VoxelSide,
) -> Option<Vec3> {
    // Calculate the intersection parameter 'a'.
    let a = rectangle_normal.dot(corner - ray_origin)
        / ray_dir.dot(rectangle_normal);

    // Calculate the intersection point P on the ray.
    let mut intersection_point = ray_origin + a * ray_dir;

    match side {
        VoxelSide::Top | VoxelSide::Bottom => intersection_point.y = corner.y,
        VoxelSide::Left | VoxelSide::Right => intersection_point.x = corner.x,
        VoxelSide::Front | VoxelSide::Back => intersection_point.z = corner.z,
    }

    let top_side_len = top_side.length();
    let left_side_len = left_side.length();
    // Calculate the vectors P0P, Q1, and Q2.
    let p0p = intersection_point - corner;
    let q1: f32 = p0p.dot(top_side) / top_side_len;
    let q2: f32 = p0p.dot(left_side) / left_side_len;

    // Check if the intersection point is inside the rectangle.
    if 0.0 <= q1 && q1 <= top_side_len && 0.0 <= q2 && q2 <= left_side_len {
        Some(intersection_point)
    } else {
        None
    }
}

/*#[test]
fn rect_vec_intersection_test() {
    let corner = Vec3::new(2.0, 3.0, 1.0);
    let top_side = Vec3::new(2.0, 0.0, 0.0);
    let left_side = Vec3::new(0.0, -2.0, 0.0);
    let rectangle_normal = top_side.cross(left_side);
    let ray_origin = Vec3::new(3.0, 1.0, 2.0);
    let ray_dir = Vec3::new(0.0, 0.0, -1.0);

    assert_eq!(
        rectangle_vector_intersection(
            corner, top_side, left_side, rectangle_normal, ray_dir, ray_origin
        ),
        Some(Vec3::new(3.0, 1.0, 1.0))
    );

    let corner = Vec3::new(-1.0, 2.0, 6.0);
    let top_side = Vec3::new(2.0, 0.0, 0.0);
    let left_side = Vec3::new(0.0, -2.0, 0.0);
    let rectangle_normal = top_side.cross(left_side);
    let ray_origin = Vec3::new(0.0, 0.0, 0.0);
    let ray_dir = Vec3::new(0.1, 0.0, 1.0).normalize();
    assert_eq!(
        rectangle_vector_intersection(
            corner, top_side, left_side, rectangle_normal, ray_dir, ray_origin
        ),
        Some(Vec3::new(0.6, 0.0, 6.0))
    );

    let corner = Vec3::new(0.0, 1.0, 0.0);
    let top_side = Vec3::new(1.0, 0.0, 0.0);
    let left_side = Vec3::new(0.0, -1.0, 0.0);
    let rectangle_normal = top_side.cross(left_side);
    let ray_dir = Vec3::new(0.0, 0.0, 1.0);
    let ray_origin = Vec3::new(0.5, 0.5, -1.0);

    assert_eq!(
        rectangle_vector_intersection(
            corner, top_side, left_side, rectangle_normal, ray_dir, ray_origin
        ),
        Some(Vec3::new(0.5, 0.5, 0.0))
    );

    let corner = Vec3::new(1.0, 1.0, 0.0);
    let top_side = Vec3::new(-1.0, 0.0, 0.0);
    let left_side = Vec3::new(0.0, -1.0, 0.0);
    let rectangle_normal = top_side.cross(left_side);
    let ray_dir = Vec3::new(0.1, 0.0, -1.0).normalize();
    let ray_origin = Vec3::new(0.5, 0.5, 1.0);

    assert_eq!(
        rectangle_vector_intersection(
            corner, top_side, left_side, rectangle_normal, ray_dir, ray_origin
        ),
        Some(Vec3::new(0.6, 0.5, 0.0))
    );
}
*/
