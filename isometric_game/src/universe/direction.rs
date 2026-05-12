use bevy::prelude::*;
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize)]
pub enum IsoDirection {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl IsoDirection {
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::NorthEast,
        Self::East,
        Self::SouthEast,
        Self::South,
        Self::SouthWest,
        Self::West,
        Self::NorthWest,
    ];

    pub fn grid_delta(self) -> IVec2 {
        match self {
            Self::North => IVec2::new(-1, 0),
            Self::NorthEast => IVec2::new(-1, -1),
            Self::East => IVec2::new(0, -1),
            Self::SouthEast => IVec2::new(1, -1),
            Self::South => IVec2::new(1, 0),
            Self::SouthWest => IVec2::new(1, 1),
            Self::West => IVec2::new(0, 1),
            Self::NorthWest => IVec2::new(-1, 1),
        }
    }

    pub fn grid_vec(self) -> Vec2 {
        self.grid_delta().as_vec2()
    }

    pub fn from_grid_delta(delta: Vec2) -> Option<Self> {
        if delta.length_squared() <= 0.0001 {
            return None;
        }

        Self::ALL.into_iter().max_by(|left, right| {
            let left_dot = normalized_dot(delta, left.grid_vec());
            let right_dot = normalized_dot(delta, right.grid_vec());
            left_dot.total_cmp(&right_dot)
        })
    }
}

fn normalized_dot(a: Vec2, b: Vec2) -> f32 {
    a.normalize_or_zero().dot(b.normalize_or_zero())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_iso_direction_grid_axes_are_diagonal_on_screen() {
        assert_eq!(IsoDirection::North.grid_delta(), IVec2::new(-1, 0));
        assert_eq!(IsoDirection::East.grid_delta(), IVec2::new(0, -1));
        assert_eq!(IsoDirection::South.grid_delta(), IVec2::new(1, 0));
        assert_eq!(IsoDirection::West.grid_delta(), IVec2::new(0, 1));
    }

    #[test]
    fn keyboard_style_screen_motion_resolves_to_iso_diagonals() {
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(-1.0, -1.0)),
            Some(IsoDirection::NorthEast)
        );
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(1.0, -1.0)),
            Some(IsoDirection::SouthEast)
        );
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(1.0, 1.0)),
            Some(IsoDirection::SouthWest)
        );
        assert_eq!(
            IsoDirection::from_grid_delta(Vec2::new(-1.0, 1.0)),
            Some(IsoDirection::NorthWest)
        );
    }
}
