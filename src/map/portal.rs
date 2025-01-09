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
    pub fn teleport_to(&self, origin: Vec3, dest: Portal) -> Vec3 {
        let offset_x = self.center.x - origin.x;
        let offset_z = self.center.z - origin.z;
        let mut new_origin = Vec3::new(dest.center.x, origin.y + dest.ground_level - self.ground_level, dest.center.z);

        match self.direction {
            PortalDirection::North => match dest.direction {
                PortalDirection::North => {
                    new_origin.x += offset_x;
                    new_origin.z += 1.0 + offset_z;
                }
                PortalDirection::South => {
                    new_origin.x += - offset_x;
                    new_origin.z += - 1.0 - offset_z;
                }
                PortalDirection::East => {
                    new_origin.x += 1.0 + offset_z;
                    new_origin.z += - offset_x;
                }
                PortalDirection::West => {
                    new_origin.x += - 1.0 - offset_z;
                    new_origin.z += offset_x;
                }
            },
            PortalDirection::South => match dest.direction {
                PortalDirection::North => {
                    new_origin.x += - offset_x;
                    new_origin.z += 1.0 - offset_z;
                }
                PortalDirection::South => {
                    new_origin.x += offset_x;
                    new_origin.z += - 1.0 + offset_z;
                }
                PortalDirection::East => {
                    new_origin.x += 1.0 - offset_z;
                    new_origin.z += offset_x;
                }
                PortalDirection::West => {
                    new_origin.x += - 1.0 + offset_z;
                    new_origin.z += - offset_x;
                }
            },
            PortalDirection::East => match dest.direction {
                PortalDirection::North => {
                    new_origin.x += - offset_z;
                    new_origin.z += 1.0 + offset_x;
                }
                PortalDirection::South => {
                    new_origin.x += offset_z;
                    new_origin.z += - 1.0 - offset_x;
                }
                PortalDirection::East => {
                    new_origin.x += 1.0 + offset_x;
                    new_origin.z += offset_z;
                }
                PortalDirection::West => {
                    new_origin.x += - 1.0 - offset_x;
                    new_origin.z += - offset_z;
                }
            },
            PortalDirection::West => match dest.direction {
                PortalDirection::North => {
                    new_origin.x += offset_z;
                    new_origin.z += 1.0 - offset_x;
                }
                PortalDirection::South => {
                    new_origin.x += - offset_z;
                    new_origin.z += - 1.0 + offset_x;
                }
                PortalDirection::East => {
                    new_origin.x += 1.0 - offset_x;
                    new_origin.z += - offset_z;
                }
                PortalDirection::West => {
                    new_origin.x += - 1.0 + offset_x;
                    new_origin.z += offset_z;
                }
            },
        };
        new_origin
    }

    pub fn direction_difference(&self, other: &Self) -> PortalDirectionDifference {
        self.direction.difference(other.direction)
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
    pub fn difference(self, other: Self) -> PortalDirectionDifference {
        match other as i32 - self as i32 {
            0 => PortalDirectionDifference::None,
            -90 | 270 => PortalDirectionDifference::AnticlockwiseDeg90,
            90 | -270 => PortalDirectionDifference::ClockwiseDeg90,
            180 | -180 => PortalDirectionDifference::Deg180,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PortalDirectionDifference {
    None,
    AnticlockwiseDeg90,
    ClockwiseDeg90,
    Deg180,
}

/*#[test]
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
*/