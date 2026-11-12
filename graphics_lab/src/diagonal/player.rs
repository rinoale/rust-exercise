use super::World;
use crate::voxel::Input;
use glam::{Mat4, Vec3};

pub struct Player {
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub velocity_y: f32,
    pub grounded: bool,
    pub flying: bool,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            feet: Vec3::new(0., 0.001, 11.),
            yaw: -0.10,
            pitch: -0.04,
            velocity_y: 0.,
            grounded: true,
            flying: false,
        }
    }
}
impl Player {
    pub fn eye(&self) -> Vec3 {
        self.feet + Vec3::Y * 1.55
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
        Mat4::perspective_rh(70_f32.to_radians(), aspect, 0.05, 180.)
            * Mat4::look_to_rh(self.eye(), self.direction(), Vec3::Y)
    }
    pub fn jump(&mut self) {
        if self.grounded {
            self.velocity_y = 7.;
            self.grounded = false;
        }
    }
    pub fn respawn(&mut self, world: &World) {
        *self = Self::default();
        // A built-over spawn must not trap the player inside geometry.
        if let Some((height, _)) = world.support(self.feet, 100.) {
            self.feet.y = height + 0.001;
        }
    }
    pub fn overview(&mut self) {
        self.feet = Vec3::new(-21., 16., 26.);
        let direction = (Vec3::new(-1., 2., -1.) - self.eye()).normalize();
        self.yaw = direction.x.atan2(-direction.z);
        self.pitch = direction.y.asin();
        self.flying = true;
        self.velocity_y = 0.;
        self.grounded = false;
    }
    pub fn tick(&mut self, world: &World, input: Input, vertical: f32, fast: bool, dt: f32) {
        let dt = dt.clamp(0., 0.1);
        let forward = Vec3::new(self.yaw.sin(), 0., -self.yaw.cos());
        let right = Vec3::new(self.yaw.cos(), 0., self.yaw.sin());
        let move_xz = (forward * input.forward + right * input.right).clamp_length_max(1.);
        if self.flying {
            self.feet += (move_xz + Vec3::Y * vertical).clamp_length_max(1.)
                * dt
                * if fast { 18. } else { 8. };
            self.feet = self
                .feet
                .clamp(Vec3::new(-40., -10., -40.), Vec3::new(40., 40., 40.));
            return;
        }
        let steps = (dt * 120.).ceil().max(1.) as u32;
        let dt = dt / steps as f32;
        let movement = move_xz * if fast { 6. } else { 3.5 };
        for _ in 0..steps {
            let grounded = self.grounded;
            for axis in [0, 2] {
                let mut next = self.feet;
                next[axis] += movement[axis] * dt;
                if grounded && let Some((height, ramp)) = world.support(next, self.feet.y + 0.17) {
                    let rise = height - self.feet.y;
                    if (ramp || rise <= 0.01) && rise.abs() < 0.17 {
                        next.y = height + 0.001;
                    }
                }
                if !world.collides(next) {
                    self.feet = next;
                }
            }
            self.velocity_y -= 20. * dt;
            let amount = self.velocity_y * dt;
            let mut next = self.feet + Vec3::Y * amount;
            self.grounded = false;
            if world.collides(next) {
                let (mut low, mut high) = (0., 1.);
                for _ in 0..16 {
                    let mid = (low + high) * 0.5;
                    if world.collides(self.feet + Vec3::Y * amount * mid) {
                        high = mid;
                    } else {
                        low = mid;
                    }
                }
                next = self.feet + Vec3::Y * amount * low;
                self.grounded = amount < 0.;
                self.velocity_y = 0.;
            }
            self.feet = next;
            if self.feet.y < -8. {
                self.respawn(world);
            }
        }
    }
}
