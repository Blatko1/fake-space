use super::{
    ray::Ray, Side, FLASHLIGHT_DISTANCE, FLASHLIGHT_INTENSITY, NORMAL_X_NEGATIVE,
    NORMAL_X_POSITIVE, NORMAL_Y_NEGATIVE, NORMAL_Y_POSITIVE, NORMAL_Z_NEGATIVE,
    NORMAL_Z_POSITIVE,
};
use crate::player::camera::Camera;
use crate::world::model::ModelDataRef;
use dot_vox::Color;
use glam::Vec3;
use rayon::{iter::{IndexedParallelIterator, ParallelIterator}, slice::ParallelSliceMut};

pub fn draw_objects(objects: Vec<ObjectDrawData>, camera: &Camera, column: &mut [u8]) {
    column.par_chunks_exact_mut(4).enumerate()
        .for_each(|(screen_y, pixel)| {
            // Filter objects which are not visible
            let visible_objects = objects.iter().rev().filter(|object| {
                object.bottom_draw_bound <= screen_y && object.top_draw_bound > screen_y
            });
            visible_objects.for_each(|obj| obj.draw_pixel(screen_y as f32, camera, pixel))
        });
}

pub struct ObjectDrawData<'a> {
    pub pos: Vec3,
    pub model_data: ModelDataRef<'a>,
    pub ray: Ray,
    pub ambient_light_intensity: f32,
    pub bottom_draw_bound: usize,
    pub top_draw_bound: usize,
}

impl<'a> ObjectDrawData<'a> {
    fn draw_pixel(&self, screen_y: f32, camera: &Camera, pixel: &mut [u8]) {
        let ray = self.ray;
        let dimension = self.model_data.dimension as f32;
        let obj_pos = self.pos;

        // TODO why do y positions have to be divided by 2
        let mut ray_origin = ray.origin * dimension;
        ray_origin.y *= 0.5;

        let (top_left_point, top_side, voxel_side) = match ray.hit_wall_side {
            Side::Vertical => {
                // is east side hit
                if ray.dir.x > 0.0 {
                    let top_left = Vec3::new(
                        obj_pos.x,
                        obj_pos.y + dimension,
                        obj_pos.z + dimension,
                    );
                    (top_left, Vec3::new(0.0, 0.0, -dimension), VoxelSide::Left)
                }
                // is west side hit
                else {
                    let top_left = Vec3::new(
                        obj_pos.x + dimension,
                        obj_pos.y + dimension,
                        obj_pos.z,
                    );
                    (top_left, Vec3::new(0.0, 0.0, dimension), VoxelSide::Right)
                }
            }
            Side::Horizontal => {
                // is north side hit
                if ray.dir.z > 0.0 {
                    let top_left = Vec3::new(obj_pos.x, obj_pos.y + dimension, obj_pos.z);
                    (top_left, Vec3::new(dimension, 0.0, 0.0), VoxelSide::Front)
                }
                // is south side hit
                else {
                    let top_left = Vec3::new(
                        obj_pos.x + dimension,
                        obj_pos.y + dimension,
                        obj_pos.z + dimension,
                    );
                    (top_left, Vec3::new(-dimension, 0.0, 0.0), VoxelSide::Back)
                }
            }
        };
        let left_side = Vec3::new(0.0, -dimension, 0.0);
        // Calculate the normal vector (N) of the rectangle's surface.
        let rectangle_normal = top_side.cross(left_side);

        // Y-coordinate on the vertical camera plane (range [-1.0, 1.0])
        let plane_y = (screen_y - camera.y_shearing) * camera.height_recip * 2.0 - 1.0;
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
                    (intersection.y - obj_pos.y).fract()
                } else {
                    1.0 - (intersection.y - obj_pos.y).fract()
                };
            let mut side_dist_z = delta_dist.z
                * if ray_dir.z < 0.0 {
                    intersection.z.fract()
                } else {
                    1.0 - intersection.z.fract()
                };

            let mut grid_x = (intersection.x - obj_pos.x).max(0.0) as i32;
            let mut grid_z = (intersection.z - obj_pos.z).max(0.0) as i32;
            let mut grid_y = (intersection.y - obj_pos.y).max(0.0) as i32;
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
                                return;
                            }
                            side_dist_x += delta_dist.x;
                        } else {
                            grid_z += step_z;
                            if grid_z < 0 {
                                return;
                            }
                            side_dist_z += delta_dist.z;
                        }
                    } else if side_dist_y < side_dist_z {
                        grid_y += step_y;
                        if grid_y < 0 {
                            return;
                        }
                        side_dist_y += delta_dist.y;
                    } else {
                        grid_z += step_z;
                        if grid_z < 0 {
                            return;
                        }
                        side_dist_z += delta_dist.z;
                    }
                }
                _ => (),
            }
            loop {
                let voxel = self.model_data.get_voxel(
                    grid_x as u32,
                    grid_y as u32,
                    grid_z as u32,
                );
                if let Some(Color { r, g, b, a: 255 }) = voxel {
                    // Spotlight doesn't work since distance represents
                    // distance to whole voxel, not point on voxel
                    let x = grid_x as f32 + obj_pos.x - ray_origin.x;
                    let y = (grid_y as f32 + obj_pos.y - ray_origin.y) * 2.0;
                    let z = grid_z as f32 + obj_pos.z - ray_origin.z;
                    let distance =
                        ((x * x + y * y + z * z) / (dimension * dimension)).sqrt();

                    let normal = side.normal();
                    let diffuse = (-ray_dir.dot(normal)).max(0.0);
                    // Multiply by the canvas aspect ratio so the light has a shape of a circle.
                    let flashlight_x = ray.plane_x * camera.view_aspect;
                    // Smooth out the flashlight intensity using the distance
                    let flashlight_intensity = (1.0
                        - (distance / FLASHLIGHT_DISTANCE).clamp(0.0, 1.0))
                        * FLASHLIGHT_INTENSITY
                        * diffuse;

                    let flashlight_y = 2.0 * screen_y * camera.height_recip - 1.0;
                    let flashlight_radius = (flashlight_x * flashlight_x
                        + flashlight_y * flashlight_y)
                        .sqrt();
                    let t = 1.0
                        - ((flashlight_radius - super::FLASHLIGHT_INNER_RADIUS)
                            / (super::FLASHLIGHT_OUTER_RADIUS
                                - super::FLASHLIGHT_INNER_RADIUS))
                            .clamp(0.0, 1.0);
                    let flashlight = t * t * (3.0 - t * 2.0) * flashlight_intensity;
                    let light = flashlight + self.ambient_light_intensity;
                    // Red channel
                    pixel[0] = (r as f32 * light) as u8;
                    // Green channel
                    pixel[1] = (g as f32 * light) as u8;
                    // Blue channel
                    pixel[2] = (b as f32 * light) as u8;
                    //pixel[3] = 255;
                    break;
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

impl VoxelSide {
    #[inline]
    pub fn normal(self) -> Vec3 {
        match self {
            VoxelSide::Top => NORMAL_Y_POSITIVE,
            VoxelSide::Bottom => NORMAL_Y_NEGATIVE,
            VoxelSide::Left => NORMAL_X_NEGATIVE,
            VoxelSide::Right => NORMAL_X_POSITIVE,
            VoxelSide::Front => NORMAL_Z_NEGATIVE,
            VoxelSide::Back => NORMAL_Z_POSITIVE,
        }
    }
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

    // Check if the intersection point is inside the rectangle:
    // - only check if the intersection point is too low
    if q2 <= left_side_len {
        Some(intersection_point)
    } else {
        None
    }
}
