use bevy::prelude::*;

#[derive(Resource)]
pub struct WorldMap {
    pub width: i32,
    pub height: i32,
    blocked: Vec<bool>,
}

impl WorldMap {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            blocked: vec![false; (width * height) as usize],
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some((y * self.width + x) as usize)
        } else {
            None
        }
    }

    pub fn set_blocked(&mut self, x: i32, y: i32, blocked: bool) {
        if let Some(index) = self.index(x, y) {
            self.blocked[index] = blocked;
        }
    }

    pub fn block_nearest(&mut self, grid: Vec2) {
        self.set_blocked(grid.x.round() as i32, grid.y.round() as i32, true);
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        self.index(x, y)
            .map(|index| self.blocked[index])
            .unwrap_or(true)
    }

    pub fn is_walkable_world(&self, grid: Vec2) -> bool {
        let x = grid.x.round() as i32;
        let y = grid.y.round() as i32;
        !self.is_blocked(x, y)
    }
}
