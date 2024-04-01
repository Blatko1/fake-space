use crate::render::camera::Camera;
use crate::render::colors::COLOR_LUT;
use crate::render::ray::Ray;
use crate::render::{
    Side, FLASHLIGHT_DISTANCE, FLASHLIGHT_INTENSITY, NORMAL_X_NEGATIVE,
    NORMAL_X_POSITIVE, NORMAL_Y_NEGATIVE, NORMAL_Y_POSITIVE, NORMAL_Z_NEGATIVE,
    NORMAL_Z_POSITIVE, SPOTLIGHT_DISTANCE,
};
use crate::voxel::VoxelModelDataRef;
use glam::Vec3;

pub struct ObjectDrawData<'a> {
    pub pos_x: u64,
    pub pos_z: u64,
    pub pos_y: f32,
    pub model_data: VoxelModelDataRef<'a>,
    pub ray: Ray,
    pub ambient_light_intensity: f32,
    pub bottom_draw_bound: usize,
    pub top_draw_bound: usize,
}

pub fn draw_objects(objects: Vec<ObjectDrawData>, camera: &Camera, column: &mut [u8]) {
    if objects.is_empty() {
        return;
    }
    column
        .chunks_exact_mut(4)
        .enumerate()
        .for_each(|(y, pixel)| {
            let screen_y = y;
            let y = y as f32 - camera.y_shearing;
            // Filter models which are covered by walls or platforms
            for object in objects.iter().rev().filter(|object| {
                object.bottom_draw_bound <= screen_y && object.top_draw_bound > screen_y
            }) {
                let ray = object.ray;
                let ambient = object.ambient_light_intensity;
                let dimension = object.model_data.dimension as f32;
                let hit_side = ray.hit_wall_side;

                // TODO why do y positions have to be divided by 2
                let mut ray_origin = ray.origin * dimension;
                ray_origin.y *= 0.5;

                let obj_x_pos = object.pos_x as f32 * dimension;
                let obj_y_pos = object.pos_y * dimension * 0.5;
                let obj_z_pos = object.pos_z as f32 * dimension;
                let (top_left_point, top_side, voxel_side) = match hit_side {
                    Side::Vertical => {
                        // is east side hit
                        if ray.dir.x > 0.0 {
                            let top_left = Vec3::new(
                                obj_x_pos,
                                obj_y_pos + dimension,
                                obj_z_pos + dimension,
                            );
                            (top_left, Vec3::new(0.0, 0.0, -dimension), VoxelSide::Left)
                        }
                        // is west side hit
                        else {
                            let top_left = Vec3::new(
                                obj_x_pos + dimension,
                                obj_y_pos + dimension,
                                obj_z_pos,
                            );
                            (top_left, Vec3::new(0.0, 0.0, dimension), VoxelSide::Right)
                        }
                    }
                    Side::Horizontal => {
                        // is north side hit
                        if ray.dir.z > 0.0 {
                            let top_left =
                                Vec3::new(obj_x_pos, obj_y_pos + dimension, obj_z_pos);
                            (top_left, Vec3::new(dimension, 0.0, 0.0), VoxelSide::Front)
                        }
                        // is south side hit
                        else {
                            let top_left = Vec3::new(
                                obj_x_pos + dimension,
                                obj_y_pos + dimension,
                                obj_z_pos + dimension,
                            );
                            (top_left, Vec3::new(-dimension, 0.0, 0.0), VoxelSide::Back)
                        }
                    }
                };
                let left_side = Vec3::new(0.0, -dimension, 0.0);
                // Calculate the normal vector (N) of the rectangle's surface.
                let rectangle_normal = top_side.cross(left_side);

                // Y-coordinate on the vertical camera plane (range [-1.0, 1.0])
                let plane_y = y * camera.height_recip * 2.0 - 1.0;
                // Ray direction for current pixel column
                let ray_dir = ray.dir + camera.vertical_plane * plane_y;
                // Length of ray from one x/y/z side to next x/y/z side on the tile_map
                let delta_dist =
                    Vec3::new(ray.delta_dist_x, 1.0 / ray_dir.y.abs(), ray.delta_dist_z);

                // Somehow I don't need to normalize ray_dir since I am getting the same result
                // without normalization
                if let Some(intersection) = rectangle_vector_intersection(
                    top_left_point,
                    left_side,
                    rectangle_normal,
                    ray_dir,
                    ray_origin,
                    voxel_side,
                ) {
                    let mut side_dist_x = delta_dist.x
                        * if ray_dir.x < 0.0 {
                            intersection.x.fract()
                        } else {
                            1.0 - intersection.x.fract()
                        };
                    let mut side_dist_y = delta_dist.y
                        * if ray_dir.y < 0.0 {
                            (intersection.y - obj_y_pos).fract()
                        } else {
                            1.0 - (intersection.y - obj_y_pos).fract()
                        };
                    let mut side_dist_z = delta_dist.z
                        * if ray_dir.z < 0.0 {
                            intersection.z.fract()
                        } else {
                            1.0 - intersection.z.fract()
                        };

                    let mut grid_x = (intersection.x - obj_x_pos).max(0.0) as i32;
                    let mut grid_z = (intersection.z - obj_z_pos).max(0.0) as i32;
                    let mut grid_y = (intersection.y - obj_y_pos).max(0.0) as i32;
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
                        let voxel = object.model_data.get_voxel(
                            grid_x as usize,
                            grid_y as usize,
                            grid_z as usize,
                        );
                        match voxel {
                            Some(0) => (),
                            Some(&c) => {
                                let color = &COLOR_LUT[c as usize];
                                let x = grid_x as f32 + obj_x_pos - ray_origin.x;
                                let y = (grid_y as f32 + obj_y_pos - ray_origin.y) * 2.0;
                                let z = grid_z as f32 + obj_z_pos - ray_origin.z;
                                let distance = ((x * x + y * y + z * z)
                                    / (dimension * dimension))
                                    .sqrt();

                                let t =
                                    1.0 - (distance / SPOTLIGHT_DISTANCE).clamp(0.0, 1.0);
                                let spotlight =
                                    t * t * (3.0 - t * 2.0) * super::SPOTLIGHT_STRENGTH;
                                let normal = match side {
                                    VoxelSide::Top => NORMAL_Y_POSITIVE,
                                    VoxelSide::Bottom => NORMAL_Y_NEGATIVE,
                                    VoxelSide::Left => NORMAL_X_NEGATIVE,
                                    VoxelSide::Right => NORMAL_X_POSITIVE,
                                    VoxelSide::Front => NORMAL_Z_NEGATIVE,
                                    VoxelSide::Back => NORMAL_Z_POSITIVE,
                                };

                                let diffuse = (-ray_dir.dot(normal)).max(0.0);
                                let flashlight_x =
                                    (2.0 * ray.column_index as f32 * camera.width_recip
                                        - 1.0)
                                        * camera.aspect;
                                // Smooth out the flashlight intensity using the distance
                                let flashlight_intensity = (1.0
                                    - (distance / FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
                                    * FLASHLIGHT_INTENSITY
                                    * diffuse;

                                let flashlight_y =
                                    2.0 * screen_y as f32 * camera.height_recip - 1.0;
                                for (dest, src) in
                                    pixel[0..3].iter_mut().zip(color[0..3].iter())
                                {
                                    let flashlight_radius = (flashlight_x * flashlight_x
                                        + flashlight_y * flashlight_y)
                                        .sqrt();
                                    let t = 1.0
                                        - ((flashlight_radius
                                            - super::FLASHLIGHT_INNER_RADIUS)
                                            / (super::FLASHLIGHT_OUTER_RADIUS
                                                - super::FLASHLIGHT_INNER_RADIUS))
                                            .clamp(0.0, 1.0);
                                    let flashlight =
                                        t * t * (3.0 - t * 2.0) * flashlight_intensity;
                                    *dest = (*src as f32
                                        * (flashlight + ambient + spotlight))
                                        as u8;
                                }
                                pixel[3] = 255;
                                //darken_side(side, pixel);
                                break;
                            }

                            None => (),
                        }
                        if side_dist_x < side_dist_y {
                            if side_dist_x < side_dist_z {
                                grid_x += step_x;
                                if grid_x < 0 || grid_x >= dimension as i32 {
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
                                if grid_z < 0 || grid_z >= dimension as i32 {
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
                            if grid_y < 0 {
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
                            if grid_z < 0 || grid_z >= dimension as i32 {
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
        });
}

#[derive(Debug, Clone, Copy)]
pub enum VoxelSide {
    /// Facing +y
    Top,
    /// Facing -y
    Bottom,
    /// Facing -x
    Left,
    /// Facing +x
    Right,
    /// Facing -z
    Front,
    /// Facing +z
    Back,
}

/// The rectangle-vector intersection solution was taken directly
/// from [`StackOverflow`](https://stackoverflow.com/questions/8812073).
#[inline]
fn rectangle_vector_intersection(
    corner: Vec3,
    left_side: Vec3,
    rectangle_normal: Vec3,
    ray_dir: Vec3,
    ray_origin: Vec3,
    side: VoxelSide,
) -> Option<Vec3> {
    let a = rectangle_normal.dot(corner - ray_origin) / ray_dir.dot(rectangle_normal);

    let mut intersection_point = ray_origin + a * ray_dir;

    match side {
        VoxelSide::Top | VoxelSide::Bottom => intersection_point.y = corner.y,
        VoxelSide::Left | VoxelSide::Right => intersection_point.x = corner.x,
        VoxelSide::Front | VoxelSide::Back => intersection_point.z = corner.z,
    }

    let left_side_len = left_side.length();

    let p0p = intersection_point - corner;
    //let q1: f32 = p0p.dot(top_side) / top_side_len;
    let q2 = p0p.dot(left_side) / left_side_len;

    // Check if the intersection point is inside the rectangle.
    // Only check if the intersection point is too low.
    if q2 <= left_side_len {
        Some(intersection_point)
    } else {
        None
    }
}

/*#[inline]
fn darken_side(side: VoxelSide, color: &mut [u8]) {
    match side {
        VoxelSide::Top => (),
        VoxelSide::Bottom => {
            color[0] = color[0].saturating_sub(55);
            color[1] = color[1].saturating_sub(55);
            color[2] = color[2].saturating_sub(55);
        }
        VoxelSide::Left => {
            color[0] = color[0].saturating_sub(15);
            color[1] = color[1].saturating_sub(15);
            color[2] = color[2].saturating_sub(15);
        }
        VoxelSide::Right => {
            color[0] = color[0].saturating_sub(35);
            color[1] = color[1].saturating_sub(35);
            color[2] = color[2].saturating_sub(35);
        }
        VoxelSide::Front => {
            color[0] = color[0].saturating_sub(5);
            color[1] = color[1].saturating_sub(5);
            color[2] = color[2].saturating_sub(5);
        }
        VoxelSide::Back => {
            color[0] = color[0].saturating_sub(45);
            color[1] = color[1].saturating_sub(45);
            color[2] = color[2].saturating_sub(45);
        }
    }
}*/
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
