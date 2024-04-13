use std::f32::consts::PI;

use glam::Vec3;

use crate::{
    player::render::PointXZ,
    world::{RoomID, TilePosition},
};

#[derive(Debug, Clone, Copy)]
pub struct PortalID(pub usize);

#[derive(Debug, Copy, Clone)]
pub struct DummyPortal {
    pub id: PortalID,
    pub direction: PortalDirection,
}

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    pub id: PortalID,
    pub direction: PortalDirection,
    pub position: TilePosition,
    pub center: PointXZ<f32>,
    pub ground_level: f32,
    pub link: Option<(RoomID, PortalID)>,
}

impl Portal {
    pub fn teleport_to(&self, mut origin: Vec3, dest: Portal) -> (Vec3, f32) {
        let mut yaw_angle_difference = 0.0;
        let offset_x = self.center.x - origin.x;
        let offset_z = self.center.z - origin.z;
        origin.y += dest.ground_level - self.ground_level;
        match self.direction.rotation_difference(dest.direction) {
            PortalRotationDifference::None => {
                origin.x = dest.position.x as f32 + origin.x.fract();
                origin.z = dest.position.z as f32 + origin.z.fract();
            }
            PortalRotationDifference::ClockwiseDeg90 => {
                yaw_angle_difference = -PI * 0.5;
                origin.x = dest.center.x - offset_z;
                origin.z = dest.center.z + offset_x;
            }
            PortalRotationDifference::AnticlockwiseDeg90 => {
                yaw_angle_difference = PI * 0.5;
                origin.x = dest.center.x + offset_z;
                origin.z = dest.center.z - offset_x;
            }
            PortalRotationDifference::Deg180 => {
                yaw_angle_difference = PI;
                origin.x = dest.center.x + offset_x;
                origin.z = dest.center.z + offset_z;
            }
        }
        (origin, yaw_angle_difference)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortalDirection {
    North,
    South,
    East,
    West,
}

impl PortalDirection {
    pub fn rotation_difference(&self, other: Self) -> PortalRotationDifference {
        match self {
            PortalDirection::North => match other {
                PortalDirection::North => PortalRotationDifference::Deg180,
                PortalDirection::South => PortalRotationDifference::None,
                PortalDirection::East => PortalRotationDifference::AnticlockwiseDeg90,
                PortalDirection::West => PortalRotationDifference::ClockwiseDeg90,
            },
            PortalDirection::South => match other {
                PortalDirection::North => PortalRotationDifference::None,
                PortalDirection::South => PortalRotationDifference::Deg180,
                PortalDirection::East => PortalRotationDifference::ClockwiseDeg90,
                PortalDirection::West => PortalRotationDifference::AnticlockwiseDeg90,
            },
            PortalDirection::East => match other {
                PortalDirection::North => PortalRotationDifference::ClockwiseDeg90,
                PortalDirection::South => PortalRotationDifference::AnticlockwiseDeg90,
                PortalDirection::East => PortalRotationDifference::Deg180,
                PortalDirection::West => PortalRotationDifference::None,
            },
            PortalDirection::West => match other {
                PortalDirection::North => PortalRotationDifference::AnticlockwiseDeg90,
                PortalDirection::South => PortalRotationDifference::ClockwiseDeg90,
                PortalDirection::East => PortalRotationDifference::None,
                PortalDirection::West => PortalRotationDifference::Deg180,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortalRotationDifference {
    None,
    AnticlockwiseDeg90,
    ClockwiseDeg90,
    Deg180,
}
