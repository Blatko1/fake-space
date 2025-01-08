use std::f32::consts::PI;

use glam::Vec3;

use crate::raycaster::PointXZ;

use super::room::RoomID;

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
    pub position: PointXZ<u64>,
    pub center: PointXZ<f32>,
    pub ground_level: f32,
    pub link: Option<(RoomID, PortalID)>,
}

impl Portal {
    /// Returns new position and a difference in angle
    pub fn teleport(&self, mut origin: Vec3, dest: Portal) -> (Vec3, f32) {
        let mut yaw_angle_difference = 0.0;
        let offset_x = self.center.x - origin.x;
        let offset_z = self.center.z - origin.z;
        origin.y += dest.ground_level - self.ground_level;
        match self.direction {
            PortalDirection::North => match dest.direction {
                PortalDirection::North => {
                    yaw_angle_difference = PI;
                    origin.x = dest.center.x + offset_x;
                    origin.z = dest.center.z + 1.0 + offset_z;
                }
                PortalDirection::South => {
                    origin.x = dest.center.x - offset_x;
                    origin.z = dest.center.z - 1.0 - offset_z;
                }
                PortalDirection::East => {
                    yaw_angle_difference = PI * 0.5;
                    origin.x = dest.center.x + 1.0 + offset_z;
                    origin.z = dest.center.z - offset_x;
                }
                PortalDirection::West => {
                    yaw_angle_difference = -PI * 0.5;
                    origin.x = dest.center.x - 1.0 - offset_z;
                    origin.z = dest.center.z + offset_x;
                }
            },
            PortalDirection::South => match dest.direction {
                PortalDirection::North => {
                    origin.x = dest.center.x - offset_x;
                    origin.z = dest.center.z + 1.0 - offset_z;
                }
                PortalDirection::South => {
                    yaw_angle_difference = PI;
                    origin.x = dest.center.x + offset_x;
                    origin.z = dest.center.z - 1.0 + offset_z;
                }
                PortalDirection::East => {
                    yaw_angle_difference = -PI * 0.5;
                    origin.x = dest.center.x + 1.0 - offset_z;
                    origin.z = dest.center.z + offset_x;
                }
                PortalDirection::West => {
                    yaw_angle_difference = PI * 0.5;
                    origin.x = dest.center.x - 1.0 + offset_z;
                    origin.z = dest.center.z - offset_x;
                }
            },
            PortalDirection::East => match dest.direction {
                PortalDirection::North => {
                    yaw_angle_difference = -PI * 0.5;
                    origin.x = dest.center.x - offset_z;
                    origin.z = dest.center.z + 1.0 + offset_x;
                }
                PortalDirection::South => {
                    yaw_angle_difference = PI * 0.5;
                    origin.x = dest.center.x + offset_z;
                    origin.z = dest.center.z - 1.0 - offset_x;
                }
                PortalDirection::East => {
                    yaw_angle_difference = PI;
                    origin.x = dest.center.x + 1.0 + offset_x;
                    origin.z = dest.center.z + offset_z;
                }
                PortalDirection::West => {
                    origin.x = dest.center.x - 1.0 - offset_x;
                    origin.z = dest.center.z - offset_z;
                }
            },
            PortalDirection::West => match dest.direction {
                PortalDirection::North => {
                    yaw_angle_difference = PI * 0.5;
                    origin.x = dest.center.x + offset_z;
                    origin.z = dest.center.z + 1.0 - offset_x;
                }
                PortalDirection::South => {
                    yaw_angle_difference = -PI * 0.5;
                    origin.x = dest.center.x - offset_z;
                    origin.z = dest.center.z - 1.0 + offset_x;
                }
                PortalDirection::East => {
                    origin.x = dest.center.x + 1.0 - offset_x;
                    origin.z = dest.center.z - offset_z;
                }
                PortalDirection::West => {
                    yaw_angle_difference = PI;
                    origin.x = dest.center.x - 1.0 + offset_x;
                    origin.z = dest.center.z + offset_z;
                }
            },
        };
        (origin, yaw_angle_difference)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortalDirection {
    East = 0,
    North = 90,
    West = 180,
    South = 270
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

#[test]
fn teleportation_test() {
    let mut src = Portal {
        id: PortalID(0),
        direction: PortalDirection::North,
        position: PointXZ::new(2, 3),
        center: PointXZ::new(2.5, 3.5),
        ground_level: 2.0,
        link: None,
    };
    let mut dest = Portal {
        id: PortalID(0),
        direction: PortalDirection::North,
        position: PointXZ::new(2, 3),
        center: PointXZ::new(2.5, 3.5),
        ground_level: 3.0,
        link: None,
    };
    let origin = Vec3::new(2.2, 0.0, 3.8);
    dest.direction = PortalDirection::North;
    assert_eq!(
        (Vec3::new(2.8, 1.0, 4.2), PI),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::South;
    assert_eq!(
        (Vec3::new(2.2, 1.0, 2.8), 0.0),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::East;
    assert_eq!(
        (Vec3::new(3.2, 1.0, 3.2), PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::West;
    assert_eq!(
        (Vec3::new(1.8, 1.0, 3.8), -PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    src.direction = PortalDirection::South;
    let origin = Vec3::new(2.4, 0.0, 3.2);
    dest.direction = PortalDirection::North;
    assert_eq!(
        (Vec3::new(2.4, 1.0, 4.2), 0.0),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::South;
    assert_eq!(
        (Vec3::new(2.6, 1.0, 2.8), PI),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::East;
    assert_eq!(
        (Vec3::new(3.2, 1.0, 3.6), -PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::West;
    assert_eq!(
        (Vec3::new(1.8, 1.0, 3.4), PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    src.direction = PortalDirection::East;
    let origin = Vec3::new(2.8, 0.0, 3.2);
    dest.direction = PortalDirection::North;
    assert_eq!(
        (Vec3::new(2.2, 1.0, 4.2), -PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::South;
    assert_eq!(
        (Vec3::new(2.8, 1.0, 2.8), PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::East;
    assert_eq!(
        (Vec3::new(3.2, 1.0, 3.8), PI),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::West;
    assert_eq!(
        (Vec3::new(1.8, 1.0, 3.2), 0.0),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    src.direction = PortalDirection::West;
    let origin = Vec3::new(2.2, 0.0, 3.2);
    dest.direction = PortalDirection::North;
    assert_eq!(
        (Vec3::new(2.8, 1.0, 4.2), PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::South;
    assert_eq!(
        (Vec3::new(2.2, 1.0, 2.8), -PI * 0.5),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::East;
    assert_eq!(
        (Vec3::new(3.2, 1.0, 3.2), 0.0),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );

    dest.direction = PortalDirection::West;
    assert_eq!(
        (Vec3::new(1.8, 1.0, 3.8), PI),
        src.teleport(origin, dest),
        "from: {:?}, to: {:?}",
        src.direction,
        dest.direction
    );
}
