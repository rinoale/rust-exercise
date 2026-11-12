use crate::voxel::VoxelVertex;
use glam::{IVec3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Cube,
    TallBox,
    LongBox,
    Gentle,
    Diagonal,
    Steep,
}

impl Shape {
    pub const ALL: [Self; 6] = [
        Self::Cube,
        Self::TallBox,
        Self::LongBox,
        Self::Gentle,
        Self::Diagonal,
        Self::Steep,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Cube => "CUBE",
            Self::TallBox => "TALL BOX",
            Self::LongBox => "LONG BOX",
            Self::Gentle => "22.62 RAMP",
            Self::Diagonal => "45.00 RAMP",
            Self::Steep => "67.38 RAMP",
        }
    }
    pub fn dimensions(self) -> IVec3 {
        match self {
            Self::Cube | Self::Diagonal => IVec3::new(5, 5, 5),
            Self::TallBox | Self::Steep => IVec3::new(5, 12, 5),
            Self::LongBox | Self::Gentle => IVec3::new(12, 5, 5),
        }
    }
    pub fn is_ramp(self) -> bool {
        matches!(self, Self::Gentle | Self::Diagonal | Self::Steep)
    }
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Cube => [0.26, 0.48, 0.23],
            Self::TallBox => [0.36, 0.43, 0.49],
            Self::LongBox => [0.60, 0.43, 0.24],
            Self::Gentle => [0.12, 0.64, 0.59],
            Self::Diagonal => [0.94, 0.58, 0.19],
            Self::Steep => [0.60, 0.34, 0.68],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Piece {
    /// Minimum corner on the integer design-unit lattice.
    pub origin: IVec3,
    pub shape: Shape,
    pub rotation: u8,
    pub color: [f32; 3],
}

impl Piece {
    pub fn new(origin: IVec3, shape: Shape, rotation: u8) -> Self {
        Self {
            origin,
            shape,
            rotation: rotation % 4,
            color: shape.color(),
        }
    }
    pub fn size(&self) -> Vec3 {
        let d = self.shape.dimensions().as_vec3() / 5.0;
        if self.rotation % 2 == 0 {
            d
        } else {
            Vec3::new(d.z, d.y, d.x)
        }
    }
    pub fn bounds(&self) -> (Vec3, Vec3) {
        let p = self.origin.as_vec3() / 5.0;
        (p, p + self.size())
    }
    fn transform(&self, v: Vec3) -> Vec3 {
        let d = self.shape.dimensions().as_vec3() / 5.0;
        let v = match self.rotation {
            0 => v,
            1 => Vec3::new(d.z - v.z, v.y, v.x),
            2 => Vec3::new(d.x - v.x, v.y, d.z - v.z),
            _ => Vec3::new(v.z, v.y, d.x - v.x),
        };
        v + self.origin.as_vec3() / 5.0
    }
    pub fn faces(&self) -> Vec<Vec<Vec3>> {
        let d = self.shape.dimensions().as_vec3() / 5.0;
        let v = |x, y, z| self.transform(Vec3::new(x, y, z));
        let a = v(0., 0., 0.);
        let b = v(d.x, 0., 0.);
        let c = v(d.x, 0., d.z);
        let e = v(0., 0., d.z);
        let f = v(0., d.y, 0.);
        let g = v(d.x, d.y, 0.);
        let h = v(d.x, d.y, d.z);
        let i = v(0., d.y, d.z);
        let mut faces = if self.shape.is_ramp() {
            vec![
                vec![a, e, c, b],
                vec![a, b, g],
                vec![e, h, c],
                vec![b, c, h, g],
                vec![a, g, h, e],
            ]
        } else {
            vec![
                vec![a, e, c, b],
                vec![f, g, h, i],
                vec![a, b, g, f],
                vec![e, i, h, c],
                vec![a, f, i, e],
                vec![b, c, h, g],
            ]
        };
        let center = if self.shape.is_ramp() {
            self.transform(Vec3::new(d.x * 2.0 / 3.0, d.y / 3.0, d.z / 2.0))
        } else {
            self.transform(d / 2.0)
        };
        for face in &mut faces {
            let normal = (face[1] - face[0]).cross(face[2] - face[0]);
            if normal.dot(face[0] - center) < 0.0 {
                face.reverse();
            }
        }
        faces
    }
    pub fn planes(&self) -> Vec<(Vec3, f32)> {
        self.faces()
            .iter()
            .map(|f| {
                let n = (f[1] - f[0]).cross(f[2] - f[0]).normalize();
                (n, n.dot(f[0]))
            })
            .collect()
    }
    pub fn mesh(&self, id: usize) -> Vec<VoxelVertex> {
        let mut mesh = Vec::new();
        for face in self.faces() {
            let n = (face[1] - face[0]).cross(face[2] - face[0]).normalize();
            let u = (face[1] - face[0]).normalize();
            let v = n.cross(u);
            for k in 1..face.len() - 1 {
                for p in [face[0], face[k], face[k + 1]] {
                    let delta = p - face[0];
                    let color = if n.y > 0.1 {
                        self.color
                    } else {
                        self.color.map(|c| c * 0.78)
                    };
                    mesh.push(VoxelVertex {
                        position: p.extend(1.).to_array(),
                        normal: n.extend(0.).to_array(),
                        color: [color[0], color[1], color[2], 1.],
                        uv: [delta.dot(u), delta.dot(v)],
                        padding: [id as f32 + 1., 0.],
                    });
                }
            }
        }
        mesh
    }
    pub fn raycast(&self, origin: Vec3, direction: Vec3, reach: f32) -> Option<(f32, Vec3)> {
        let mut enter = 0.0_f32;
        let mut exit = reach;
        let mut normal = Vec3::ZERO;
        for (n, d) in self.planes() {
            let denom = n.dot(direction);
            let distance = d - n.dot(origin);
            if denom.abs() < 1e-7 {
                if distance < 0.0 {
                    return None;
                }
            } else {
                let t = distance / denom;
                if denom < 0.0 {
                    if t >= enter {
                        enter = t;
                        normal = n;
                    }
                } else {
                    exit = exit.min(t);
                }
                if enter > exit {
                    return None;
                }
            }
        }
        (normal != Vec3::ZERO && enter <= reach).then_some((enter, normal))
    }
    /// Exact box/prism test for these upright, quarter-turn-rotated shapes.
    /// Their separating axes are XYZ and (for ramps) the sloping face normal.
    pub fn intersects_body(&self, feet: Vec3, radius: f32, height: f32) -> bool {
        let min = feet + Vec3::new(-radius, 0.0001, -radius);
        let max = feet + Vec3::new(radius, height - 0.0001, radius);
        let (lo, hi) = self.bounds();
        if !min.cmplt(hi - Vec3::splat(0.0001)).all() || !max.cmpgt(lo + Vec3::splat(0.0001)).all()
        {
            return false;
        }
        if !self.shape.is_ramp() {
            return true;
        }
        // Minimum projection of the body onto the outward slope normal.
        let (n, d) = self.planes().into_iter().find(|(n, _)| n.y > 0.01).unwrap();
        let nearest = Vec3::new(
            if n.x >= 0.0 { min.x } else { max.x },
            min.y,
            if n.z >= 0.0 { min.z } else { max.z },
        );
        n.dot(nearest) < d - 0.00001
    }
    pub fn support(&self, feet: Vec3, radius: f32) -> Option<f32> {
        let (lo, hi) = self.bounds();
        let x0 = (feet.x - radius).max(lo.x);
        let x1 = (feet.x + radius).min(hi.x);
        let z0 = (feet.z - radius).max(lo.z);
        let z1 = (feet.z + radius).min(hi.z);
        if x0 >= x1 || z0 >= z1 {
            return None;
        }
        if !self.shape.is_ramp() {
            return Some(hi.y);
        }
        let fraction = match self.rotation {
            0 => (x1 - lo.x) / (hi.x - lo.x),
            1 => (z1 - lo.z) / (hi.z - lo.z),
            2 => (hi.x - x0) / (hi.x - lo.x),
            _ => (hi.z - z0) / (hi.z - lo.z),
        };
        Some(lo.y + fraction * (hi.y - lo.y))
    }
}
