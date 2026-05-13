use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeDirection {
    N,
    E,
    S,
    W,
}

impl EdgeDirection {
    pub fn from_delta(delta: IVec2) -> Option<Self> {
        match (delta.x, delta.y) {
            (-1, 0) => Some(Self::N),
            (0, -1) => Some(Self::E),
            (1, 0) => Some(Self::S),
            (0, 1) => Some(Self::W),
            _ => None,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::N => Self::S,
            Self::E => Self::W,
            Self::S => Self::N,
            Self::W => Self::E,
        }
    }

    fn slot(self) -> usize {
        match self {
            Self::N => 0,
            Self::E => 1,
            Self::S => 2,
            Self::W => 3,
        }
    }
}

#[derive(Resource)]
pub struct WorldMap {
    pub width: i32,
    pub height: i32,
    pub levels: i32,
    blocked: Vec<bool>,
    edge_walls: Vec<bool>,
}

impl WorldMap {
    #[cfg(test)]
    pub fn new(width: i32, height: i32) -> Self {
        Self::with_levels(width, height, 1)
    }

    pub fn with_levels(width: i32, height: i32, levels: i32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let levels = levels.max(1);
        Self {
            width,
            height,
            levels,
            blocked: vec![false; (width * height * levels) as usize],
            edge_walls: vec![false; (width * height * levels * 4) as usize],
        }
    }

    fn index_at(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height && z >= 0 && z < self.levels {
            Some(((z * self.width * self.height) + (y * self.width + x)) as usize)
        } else {
            None
        }
    }

    fn edge_index_at(&self, x: i32, y: i32, z: i32, direction: EdgeDirection) -> Option<usize> {
        self.index_at(x, y, z)
            .map(|cell_index| cell_index * 4 + direction.slot())
    }

    #[cfg(test)]
    pub fn set_blocked(&mut self, x: i32, y: i32, blocked: bool) {
        self.set_blocked_at(x, y, 0, blocked);
    }

    pub fn set_blocked_at(&mut self, x: i32, y: i32, z: i32, blocked: bool) {
        if let Some(index) = self.index_at(x, y, z) {
            self.blocked[index] = blocked;
        }
    }

    pub fn block_nearest_at(&mut self, grid: Vec2, z: i32) {
        self.set_blocked_at(grid.x.round() as i32, grid.y.round() as i32, z, true);
    }

    #[cfg(test)]
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.is_blocked_at(x, y, 0)
    }

    pub fn is_blocked_at(&self, x: i32, y: i32, z: i32) -> bool {
        self.index_at(x, y, z)
            .map(|index| self.blocked[index])
            .unwrap_or(true)
    }

    pub fn is_walkable_world_at(&self, grid: Vec2, z: i32) -> bool {
        let x = grid.x.round() as i32;
        let y = grid.y.round() as i32;
        !self.is_blocked_at(x, y, z)
    }

    pub fn set_edge_wall_at(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        direction: EdgeDirection,
        blocked: bool,
    ) {
        if let Some(index) = self.edge_index_at(x, y, z, direction) {
            self.edge_walls[index] = blocked;
        }
    }

    pub fn is_edge_wall_at(&self, x: i32, y: i32, z: i32, direction: EdgeDirection) -> bool {
        self.edge_index_at(x, y, z, direction)
            .map(|index| self.edge_walls[index])
            .unwrap_or(false)
    }

    pub fn is_edge_blocked_between_at(&self, from: IVec2, to: IVec2, z: i32) -> bool {
        let delta = to - from;
        let Some(direction) = EdgeDirection::from_delta(delta) else {
            return true;
        };

        self.is_edge_wall_at(from.x, from.y, z, direction)
            || self.is_edge_wall_at(to.x, to.y, z, direction.opposite())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_is_stored_per_level() {
        let mut map = WorldMap::with_levels(4, 4, 2);

        map.set_blocked_at(2, 2, 1, true);

        assert!(!map.is_blocked_at(2, 2, 0));
        assert!(map.is_blocked_at(2, 2, 1));
        assert!(map.is_blocked_at(2, 2, 2));
    }

    #[test]
    fn edge_walls_block_crossing_between_adjacent_cells() {
        let mut map = WorldMap::with_levels(4, 4, 2);

        map.set_edge_wall_at(1, 1, 1, EdgeDirection::S, true);

        assert!(!map.is_edge_blocked_between_at(IVec2::new(1, 1), IVec2::new(2, 1), 0));
        assert!(map.is_edge_blocked_between_at(IVec2::new(1, 1), IVec2::new(2, 1), 1));
        assert!(map.is_edge_blocked_between_at(IVec2::new(2, 1), IVec2::new(1, 1), 1));
    }
}
