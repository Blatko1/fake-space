use crate::world::{RoomID, TilePosition};

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
    pub ground_level: f32,
    pub link: Option<(RoomID, PortalID)>,
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

pub enum PortalRotationDifference {
    None,
    AnticlockwiseDeg90,
    ClockwiseDeg90,
    Deg180,
}
