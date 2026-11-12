use super::{Piece, Shape};
use crate::voxel::VoxelVertex;
use glam::{IVec3, Vec3};

pub struct World {
    pub pieces: Vec<Piece>,
    pub revision: u64,
}
pub struct Hit {
    pub index: usize,
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
}

impl World {
    pub fn empty() -> Self {
        Self {
            pieces: Vec::new(),
            revision: 0,
        }
    }
    pub fn add(&mut self, piece: Piece) {
        self.pieces.push(piece);
        self.revision += 1;
    }
    pub fn mesh(&self) -> Vec<VoxelVertex> {
        self.pieces
            .iter()
            .enumerate()
            .flat_map(|(id, p)| p.mesh(id))
            .collect()
    }
    pub fn raycast(&self, origin: Vec3, direction: Vec3, reach: f32) -> Option<Hit> {
        let direction = direction.try_normalize()?;
        if !origin.is_finite() || !reach.is_finite() || reach < 0. {
            return None;
        }
        let mut nearest = reach;
        let mut hit = None;
        for (index, piece) in self.pieces.iter().enumerate() {
            let (lo, hi) = piece.bounds();
            if origin.clamp(lo, hi).distance(origin) > nearest {
                continue;
            }
            if let Some((distance, normal)) = piece.raycast(origin, direction, nearest) {
                nearest = distance;
                hit = Some(Hit {
                    index,
                    point: origin + direction * distance,
                    normal,
                    distance,
                });
            }
        }
        hit
    }
    pub fn collides(&self, feet: Vec3) -> bool {
        self.pieces
            .iter()
            .any(|p| p.intersects_body(feet, 0.25, 1.7))
    }
    pub fn support(&self, feet: Vec3, max_height: f32) -> Option<(f32, bool)> {
        self.pieces
            .iter()
            .filter_map(|p| {
                let h = p.support(feet, 0.25)?;
                (h <= max_height).then_some((h, p.shape.is_ramp()))
            })
            .max_by(|a, b| a.0.total_cmp(&b.0))
    }
    pub fn landscape() -> Self {
        let mut world = Self::empty();
        // All visible solids are members of the six-piece family.
        for x in -18..18 {
            for z in -18..18 {
                let mut p = Piece::new(IVec3::new(x * 5, -5, z * 5), Shape::Cube, 0);
                let shade = ((x * 13 + z * 7).rem_euclid(7) as f32) * 0.008;
                p.color = [0.23 + shade, 0.40 + shade, 0.18 + shade];
                world.add(p);
            }
        }
        // Three broad, colored test ramps, side by side, all rising away from spawn.
        for (x, shape, color) in [
            (-40, Shape::Gentle, [0.12, 0.64, 0.59]),
            (-10, Shape::Diagonal, [0.94, 0.58, 0.19]),
            (20, Shape::Steep, [0.60, 0.34, 0.68]),
        ] {
            for lane in 0..4 {
                let mut p = Piece::new(IVec3::new(x + lane * 5, 0, 10), shape, 3);
                p.color = color;
                world.add(p);
                let support = if shape == Shape::Steep {
                    Shape::TallBox
                } else {
                    Shape::Cube
                };
                let mut block = Piece::new(IVec3::new(x + lane * 5, 0, 5), support, 0);
                block.color = color;
                world.add(block);
            }
        }
        // Terraced hill: repeated shallow wedges join cubes at exact 5-unit heights.
        for level in 0..4 {
            for lane in 0..7 {
                let x = -65 + level * 12;
                for lower in 0..level {
                    world.add(Piece::new(
                        IVec3::new(x, lower * 5, -35 + lane * 5),
                        Shape::LongBox,
                        0,
                    ));
                }
                let mut ramp =
                    Piece::new(IVec3::new(x, level * 5, -35 + lane * 5), Shape::Gentle, 0);
                ramp.color = [0.32 + level as f32 * 0.035, 0.55, 0.25];
                world.add(ramp);
            }
        }
        // A 5:12 roof: the two long halves meet at an exact ridge.
        for x in 0..6 {
            for z in 0..5 {
                if x == 0 || x == 5 || z == 0 || (z == 4 && x != 2 && x != 3) {
                    let mut p =
                        Piece::new(IVec3::new(15 + x * 5, 0, -40 + z * 5), Shape::TallBox, 0);
                    p.color = [0.67, 0.56, 0.39];
                    world.add(p);
                }
            }
        }
        for lane in 0..6 {
            for (z, rotation) in [(-40, 1), (-28, 3)] {
                let mut p = Piece::new(IVec3::new(15 + lane * 5, 12, z), Shape::Gentle, rotation);
                p.color = [0.62, 0.25, 0.14];
                world.add(p);
            }
        }
        // Equal-height monument: twelve 5-high cubes beside five 12-high boxes.
        for n in 0..12 {
            let mut p = Piece::new(IVec3::new(55, n * 5, -35), Shape::Cube, 0);
            p.color = if n % 2 == 0 {
                [0.91, 0.66, 0.24]
            } else {
                [0.68, 0.43, 0.13]
            };
            world.add(p);
        }
        for n in 0..5 {
            let mut p = Piece::new(IVec3::new(65, n * 12, -35), Shape::TallBox, 0);
            p.color = if n % 2 == 0 {
                [0.20, 0.65, 0.65]
            } else {
                [0.13, 0.44, 0.49]
            };
            world.add(p);
        }
        // Stylized trees made only from cubes and tall boxes.
        for (x, z) in [
            (-65, 40),
            (-55, -60),
            (-10, -60),
            (45, 45),
            (65, 5),
            (65, -60),
        ] {
            let mut trunk = Piece::new(IVec3::new(x, 0, z), Shape::TallBox, 0);
            trunk.color = [0.37, 0.23, 0.12];
            world.add(trunk);
            for dx in -1_i32..=1 {
                for dz in -1_i32..=1 {
                    for y in 0..if dx.abs() + dz.abs() < 2 { 2 } else { 1 } {
                        let mut p = Piece::new(
                            IVec3::new(x + dx * 5, 12 + y * 5, z + dz * 5),
                            Shape::Cube,
                            0,
                        );
                        p.color = [0.16, 0.34 + y as f32 * 0.04, 0.16];
                        world.add(p);
                    }
                }
            }
        }
        world
    }
}
