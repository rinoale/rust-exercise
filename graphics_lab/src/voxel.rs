//! Engine-independent voxel world, ray picking, inventory, and player physics.
use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Mat4, Vec3};

pub const SIZE: IVec3 = IVec3::new(40, 24, 40);
pub const REACH: f32 = 6.0;
const RADIUS: f32 = 0.3;
const HEIGHT: f32 = 1.8;
const EPSILON: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Block {
    Grass,
    Dirt,
    Stone,
    Wood,
}
impl Block {
    pub const ALL: [Self; 4] = [Self::Grass, Self::Dirt, Self::Stone, Self::Wood];
    pub fn name(self) -> &'static str {
        match self {
            Self::Grass => "Grass",
            Self::Dirt => "Dirt",
            Self::Stone => "Stone",
            Self::Wood => "Wood",
        }
    }
    pub fn color(self) -> [f32; 3] {
        match self {
            Self::Grass => [0.24, 0.52, 0.12],
            Self::Dirt => [0.40, 0.23, 0.11],
            Self::Stone => [0.43, 0.47, 0.51],
            Self::Wood => [0.52, 0.31, 0.13],
        }
    }
}

pub struct World {
    blocks: Vec<Option<Block>>,
    pub revision: u64,
}
impl World {
    pub fn empty() -> Self {
        Self {
            blocks: vec![None; (SIZE.x * SIZE.y * SIZE.z) as usize],
            revision: 0,
        }
    }
    pub fn contains(p: IVec3) -> bool {
        p.cmpge(IVec3::ZERO).all() && p.cmplt(SIZE).all()
    }
    fn index(p: IVec3) -> usize {
        (p.x + SIZE.x * (p.z + SIZE.z * p.y)) as usize
    }
    pub fn get(&self, p: IVec3) -> Option<Block> {
        if Self::contains(p) {
            self.blocks[Self::index(p)]
        } else {
            None
        }
    }
    pub fn set(&mut self, p: IVec3, block: Option<Block>) -> bool {
        if !Self::contains(p) || self.get(p) == block {
            return false;
        }
        self.blocks[Self::index(p)] = block;
        self.revision += 1;
        true
    }
    pub fn terrain() -> Self {
        let mut world = Self::empty();
        for x in 0..SIZE.x {
            for z in 0..SIZE.z {
                let hill = ((x as f32 * 0.23).sin() * (z as f32 * 0.19).cos() * 2.5 + 2.5) as i32;
                let top = if (15..=24).contains(&x) && (15..=26).contains(&z) {
                    4
                } else {
                    3 + hill
                };
                for y in 0..=top {
                    world.set(
                        IVec3::new(x, y, z),
                        Some(if y == top {
                            Block::Grass
                        } else if y >= top - 2 {
                            Block::Dirt
                        } else {
                            Block::Stone
                        }),
                    );
                }
            }
        }
        // Small stacks near spawn make digging and building immediately discoverable.
        for (x, z, height) in [(18, 17, 2), (23, 18, 3), (24, 22, 1)] {
            for y in 5..5 + height {
                world.set(IVec3::new(x, y, z), Some(Block::Wood));
            }
        }
        world
    }

    /// Amanatides-Woo traversal: exact voxel faces, including negative directions.
    pub fn raycast(&self, origin: Vec3, direction: Vec3, reach: f32) -> Option<Hit> {
        let direction = direction.try_normalize()?;
        if !origin.is_finite() || !reach.is_finite() || reach < 0.0 {
            return None;
        }
        let mut cell = origin.floor().as_ivec3();
        let step = direction.signum().as_ivec3();
        let mut next = Vec3::splat(f32::INFINITY);
        let mut delta = Vec3::splat(f32::INFINITY);
        for axis in 0..3 {
            if direction[axis] != 0.0 {
                let boundary = cell[axis] as f32 + if step[axis] > 0 { 1.0 } else { 0.0 };
                next[axis] = (boundary - origin[axis]) / direction[axis];
                delta[axis] = direction[axis].abs().recip();
            }
        }
        let mut normal = IVec3::ZERO;
        let mut distance = 0.0;
        loop {
            if let Some(block) = self.get(cell) {
                return Some(Hit {
                    cell,
                    normal,
                    block,
                    distance,
                });
            }
            let axis = if next.x <= next.y && next.x <= next.z {
                0
            } else if next.y <= next.z {
                1
            } else {
                2
            };
            distance = next[axis];
            if distance > reach {
                return None;
            }
            cell[axis] += step[axis];
            next[axis] += delta[axis];
            normal = IVec3::ZERO;
            normal[axis] = -step[axis];
        }
    }

    fn collides(&self, feet: Vec3) -> bool {
        let min = feet + Vec3::new(-RADIUS + EPSILON, EPSILON, -RADIUS + EPSILON);
        let max = feet + Vec3::new(RADIUS - EPSILON, HEIGHT - EPSILON, RADIUS - EPSILON);
        // Keep the finite prototype world bounded horizontally and above.
        if min.x < 0.0
            || min.z < 0.0
            || max.x >= SIZE.x as f32
            || max.z >= SIZE.z as f32
            || max.y >= SIZE.y as f32
        {
            return true;
        }
        let lo = min.floor().as_ivec3();
        let hi = max.floor().as_ivec3();
        for y in lo.y..=hi.y {
            for z in lo.z..=hi.z {
                for x in lo.x..=hi.x {
                    if self.get(IVec3::new(x, y, z)).is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Emit only exposed faces; internal block faces never reach the GPU.
    pub fn mesh(&self) -> Vec<VoxelVertex> {
        let cube = crate::scene::cube();
        let mut mesh = Vec::new();
        for y in 0..SIZE.y {
            for z in 0..SIZE.z {
                for x in 0..SIZE.x {
                    let cell = IVec3::new(x, y, z);
                    let Some(block) = self.get(cell) else {
                        continue;
                    };
                    for face in 0..6 {
                        let normal = Vec3::from_slice(&cube.vertices[face * 4].normal);
                        if self.get(cell + normal.as_ivec3()).is_some() {
                            continue;
                        }
                        let color = if block == Block::Grass && normal.y < 0.5 {
                            Block::Dirt.color()
                        } else {
                            block.color()
                        };
                        for index in [0, 1, 2, 0, 2, 3] {
                            let vertex = cube.vertices[face * 4 + index];
                            let position = Vec3::from_slice(&vertex.position) * 0.5
                                + Vec3::splat(0.5)
                                + cell.as_vec3();
                            mesh.push(VoxelVertex {
                                position: position.extend(1.0).to_array(),
                                normal: normal.extend(0.0).to_array(),
                                color: [color[0], color[1], color[2], 1.0],
                                uv: [vertex.uv[0] * 0.5, vertex.uv[1] * 0.5],
                                padding: [0.0; 2],
                            });
                        }
                    }
                }
            }
        }
        mesh
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub cell: IVec3,
    pub normal: IVec3,
    pub block: Block,
    pub distance: f32,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VoxelVertex {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub color: [f32; 4],
    pub uv: [f32; 2],
    pub padding: [f32; 2],
}

#[derive(Default, Clone, Copy)]
pub struct Input {
    pub forward: f32,
    pub right: f32,
}
pub struct Player {
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub velocity_y: f32,
    pub grounded: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            feet: Vec3::new(20.5, 5.0, 23.5),
            yaw: -0.35,
            pitch: -0.14,
            velocity_y: 0.0,
            grounded: false,
        }
    }
}
impl Player {
    pub fn eye(&self) -> Vec3 {
        self.feet + Vec3::Y * 1.62
    }
    pub fn direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        )
    }
    pub fn look(&mut self, dx: f64, dy: f64) {
        self.yaw = (self.yaw + dx as f32 * 0.0025) % std::f32::consts::TAU;
        self.pitch = (self.pitch - dy as f32 * 0.0025).clamp(-1.54, 1.54);
    }
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(70_f32.to_radians(), aspect, 0.05, 100.0)
            * Mat4::look_to_rh(self.eye(), self.direction(), Vec3::Y)
    }
    pub fn overlaps(&self, cell: IVec3) -> bool {
        let lo = self.feet + Vec3::new(-RADIUS, 0.0, -RADIUS);
        let hi = self.feet + Vec3::new(RADIUS, HEIGHT, RADIUS);
        lo.cmplt(cell.as_vec3() + Vec3::ONE).all() && hi.cmpgt(cell.as_vec3()).all()
    }
    pub fn jump(&mut self) {
        if self.grounded {
            self.velocity_y = 7.5;
            self.grounded = false;
        }
    }
    pub fn tick(&mut self, world: &World, input: Input, dt: f32) {
        // Small substeps prevent tunneling even during slow frames.
        let dt = dt.clamp(0.0, 0.1);
        let steps = (dt / (1.0 / 120.0)).ceil().max(1.0) as u32;
        let dt = dt / steps as f32;
        let forward = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        let movement = (forward * input.forward + right * input.right).clamp_length_max(1.0) * 4.5;
        for _ in 0..steps {
            self.velocity_y -= 22.0 * dt;
            self.grounded = false;
            for axis in [0, 2, 1] {
                let amount = if axis == 1 {
                    self.velocity_y
                } else {
                    movement[axis]
                } * dt;
                let mut candidate = self.feet;
                candidate[axis] += amount;
                if world.collides(candidate) {
                    // Find the contact point without rounding the player's other coordinates.
                    let mut low = 0.0;
                    let mut high = 1.0;
                    for _ in 0..14 {
                        let mid = (low + high) * 0.5;
                        candidate = self.feet;
                        candidate[axis] += amount * mid;
                        if world.collides(candidate) {
                            high = mid;
                        } else {
                            low = mid;
                        }
                    }
                    self.feet[axis] += amount * low;
                    if axis == 1 {
                        self.grounded = amount < 0.0;
                        self.velocity_y = 0.0;
                    }
                } else {
                    self.feet = candidate;
                }
            }
            if self.feet.y < -12.0 {
                self.respawn(world);
            }
        }
    }
    pub fn respawn(&mut self, world: &World) {
        *self = Self::default();
        if !world.collides(self.feet)
            && world
                .get((self.feet - Vec3::Y * 0.01).floor().as_ivec3())
                .is_some()
        {
            return;
        }
        // Find safe standing room even if the spawn column was edited.
        for z in (0..SIZE.z).rev() {
            for x in 0..SIZE.x {
                for y in (1..SIZE.y - 1).rev() {
                    let feet = Vec3::new(x as f32 + 0.5, y as f32, z as f32 + 0.5);
                    if world.get(IVec3::new(x, y - 1, z)).is_some() && !world.collides(feet) {
                        self.feet = feet;
                        return;
                    }
                }
            }
        }
    }
}

pub struct Game {
    pub world: World,
    pub player: Player,
    pub inventory: [u32; 4],
    pub selected: usize,
}
impl Default for Game {
    fn default() -> Self {
        Self {
            world: World::terrain(),
            player: Player::default(),
            inventory: [16, 32, 32, 16],
            selected: 1,
        }
    }
}
impl Game {
    pub fn target(&self) -> Option<Hit> {
        self.world
            .raycast(self.player.eye(), self.player.direction(), REACH)
    }
    pub fn dig(&mut self) -> bool {
        let Some(hit) = self.target() else {
            return false;
        };
        self.world.set(hit.cell, None);
        self.inventory[hit.block as usize] += 1;
        true
    }
    pub fn place(&mut self) -> bool {
        let Some(hit) = self.target() else {
            return false;
        };
        let cell = hit.cell + hit.normal;
        if hit.normal == IVec3::ZERO
            || self.inventory[self.selected] == 0
            || self.player.overlaps(cell)
            || self.world.get(cell).is_some()
        {
            return false;
        }
        if !self.world.set(cell, Some(Block::ALL[self.selected])) {
            return false;
        }
        self.inventory[self.selected] -= 1;
        true
    }
}
