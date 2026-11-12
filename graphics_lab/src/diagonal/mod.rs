//! A world of integer-sized boxes and triangular prisms. Five design units
//! equal one rendering unit; the 5:12 proportions are preserved everywhere.
mod geometry;
mod player;
mod world;

pub use geometry::{Piece, Shape};
pub use player::Player;
pub use world::{Hit, World};

use glam::{IVec3, Vec3};

pub struct Game {
    pub world: World,
    pub player: Player,
    pub selected: usize,
    pub rotation: u8,
    pub message: String,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            world: World::landscape(),
            player: Player::default(),
            selected: 3,
            rotation: 0,
            message: "Walk the three ramps. Build your own with unlimited pieces.".into(),
        }
    }
}

impl Game {
    pub fn target(&self) -> Option<Hit> {
        self.world
            .raycast(self.player.eye(), self.player.direction(), 12.0)
    }

    /// Snap to the one-design-unit lattice, then attach the new bounding box
    /// flush to an axis-aligned target face. A sloping face uses the target top.
    pub fn candidate(&self) -> Option<Piece> {
        let hit = self.target()?;
        let target = &self.world.pieces[hit.index];
        let mut piece = Piece::new(IVec3::ZERO, Shape::ALL[self.selected], self.rotation);
        let size = piece.size();
        let mut p = ((hit.point - size * 0.5) * 5.0).round() / 5.0;
        let (lo, hi) = target.bounds();
        if hit.normal.y.abs() > 0.01 {
            p.y = if hit.normal.y > 0.0 {
                hi.y
            } else {
                lo.y - size.y
            };
        } else if hit.normal.x.abs() > 0.5 {
            p.x = if hit.normal.x > 0.0 {
                hi.x
            } else {
                lo.x - size.x
            };
            p.y = lo.y;
        } else {
            p.z = if hit.normal.z > 0.0 {
                hi.z
            } else {
                lo.z - size.z
            };
            p.y = lo.y;
        }
        piece.origin = (p * 5.0).round().as_ivec3();
        Some(piece)
    }

    pub fn can_place(&self, piece: &Piece) -> bool {
        let (lo, hi) = piece.bounds();
        lo.y >= -1.0 && hi.y <= 30.0 && lo.x >= -30.0 && hi.x <= 30.0
            && lo.z >= -30.0 && hi.z <= 30.0
            && !piece.intersects_body(self.player.feet, 0.25, 1.7)
            // Reserve the entire box footprint: simple, predictable building.
            && !self.world.pieces.iter().any(|other| {
                let (a, b) = other.bounds();
                lo.cmplt(b - Vec3::splat(0.001)).all()
                    && hi.cmpgt(a + Vec3::splat(0.001)).all()
            })
    }

    pub fn place(&mut self) -> bool {
        let Some(piece) = self.candidate() else {
            return false;
        };
        if !self.can_place(&piece) {
            self.message = "Space occupied or too close. Aim at another face.".into();
            return false;
        }
        self.message = format!("Placed {}", piece.shape.name());
        self.world.add(piece);
        true
    }

    pub fn dig(&mut self) -> bool {
        let Some(hit) = self.target() else {
            return false;
        };
        if self.world.pieces[hit.index].origin.y < 0 {
            self.message = "The foundation stays. Build on its top face.".into();
            return false;
        }
        self.message = format!("Removed {}", self.world.pieces[hit.index].shape.name());
        self.world.pieces.remove(hit.index);
        self.world.revision += 1;
        true
    }
}
