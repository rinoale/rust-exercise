use bevy::prelude::*;

use crate::universe::WorldMap;

const MIN_DELTA: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionShape {
    pub radius: f32,
}

impl CollisionShape {
    pub const ACTOR: Self = Self { radius: 0.28 };
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KinematicMove {
    pub start: Vec2,
    pub requested_delta: Vec2,
    pub accepted_delta: Vec2,
    pub end: Vec2,
    pub hit_x: bool,
    pub hit_y: bool,
}

pub fn move_with_collision(
    current: Vec2,
    requested_delta: Vec2,
    world_map: &WorldMap,
    shape: CollisionShape,
) -> KinematicMove {
    let start = clamp_to_map(current, world_map, shape);
    if requested_delta.length_squared() <= MIN_DELTA * MIN_DELTA {
        return KinematicMove {
            start,
            requested_delta,
            accepted_delta: start - current,
            end: start,
            hit_x: false,
            hit_y: false,
        };
    }

    let full = clamp_to_map(start + requested_delta, world_map, shape);
    if is_walkable_body(full, world_map, shape) {
        return KinematicMove {
            start,
            requested_delta,
            accepted_delta: full - current,
            end: full,
            hit_x: false,
            hit_y: false,
        };
    }

    let x_only = clamp_to_map(start + Vec2::new(requested_delta.x, 0.0), world_map, shape);
    let y_only = clamp_to_map(start + Vec2::new(0.0, requested_delta.y), world_map, shape);
    let x_walkable =
        requested_delta.x.abs() > MIN_DELTA && is_walkable_body(x_only, world_map, shape);
    let y_walkable =
        requested_delta.y.abs() > MIN_DELTA && is_walkable_body(y_only, world_map, shape);

    let end = match (x_walkable, y_walkable) {
        (true, true) => {
            if requested_delta.x.abs() >= requested_delta.y.abs() {
                x_only
            } else {
                y_only
            }
        }
        (true, false) => x_only,
        (false, true) => y_only,
        (false, false) => start,
    };

    let accepted_delta = end - start;

    KinematicMove {
        start,
        requested_delta,
        accepted_delta: end - current,
        end,
        hit_x: requested_delta.x.abs() > MIN_DELTA
            && (accepted_delta.x - requested_delta.x).abs() > MIN_DELTA,
        hit_y: requested_delta.y.abs() > MIN_DELTA
            && (accepted_delta.y - requested_delta.y).abs() > MIN_DELTA,
    }
}

pub fn step_toward_with_collision(
    current: Vec2,
    target: Vec2,
    speed: f32,
    delta_seconds: f32,
    world_map: &WorldMap,
    shape: CollisionShape,
) -> KinematicMove {
    let delta = target - current;
    let distance = delta.length();
    if distance <= MIN_DELTA {
        return move_with_collision(current, Vec2::ZERO, world_map, shape);
    }

    let max_step = speed.max(0.0) * delta_seconds.max(0.0);
    let requested_delta = if distance <= max_step {
        delta
    } else {
        delta / distance * max_step
    };

    move_with_collision(current, requested_delta, world_map, shape)
}

pub fn is_walkable_body(position: Vec2, world_map: &WorldMap, shape: CollisionShape) -> bool {
    collision_samples(position, shape)
        .into_iter()
        .all(|sample| world_map.is_walkable_world(sample))
}

fn collision_samples(position: Vec2, shape: CollisionShape) -> Vec<Vec2> {
    if shape.radius <= MIN_DELTA {
        return vec![position];
    }

    let radius = shape.radius;
    let diagonal = radius * 0.70710677;
    vec![
        position,
        position + Vec2::new(radius, 0.0),
        position + Vec2::new(-radius, 0.0),
        position + Vec2::new(0.0, radius),
        position + Vec2::new(0.0, -radius),
        position + Vec2::new(diagonal, diagonal),
        position + Vec2::new(-diagonal, diagonal),
        position + Vec2::new(diagonal, -diagonal),
        position + Vec2::new(-diagonal, -diagonal),
    ]
}

fn clamp_to_map(position: Vec2, world_map: &WorldMap, shape: CollisionShape) -> Vec2 {
    Vec2::new(
        clamp_axis(position.x, world_map.width, shape.radius),
        clamp_axis(position.y, world_map.height, shape.radius),
    )
}

fn clamp_axis(value: f32, cell_count: i32, radius: f32) -> f32 {
    if cell_count <= 1 {
        return 0.0;
    }

    let min = radius.min((cell_count - 1) as f32);
    let max = ((cell_count - 1) as f32 - radius).max(min);
    value.clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_map_accepts_requested_motion() {
        let world_map = WorldMap::new(4, 4);
        let movement = move_with_collision(
            Vec2::new(1.0, 1.0),
            Vec2::new(0.5, 0.0),
            &world_map,
            CollisionShape { radius: 0.0 },
        );

        assert_eq!(movement.end, Vec2::new(1.5, 1.0));
        assert!(!movement.hit_x);
        assert!(!movement.hit_y);
    }

    #[test]
    fn blocked_diagonal_motion_slides_on_open_axis() {
        let mut world_map = WorldMap::new(4, 4);
        world_map.set_blocked(2, 2, true);

        let movement = move_with_collision(
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 1.0),
            &world_map,
            CollisionShape { radius: 0.0 },
        );

        assert_eq!(movement.end, Vec2::new(2.0, 1.0));
        assert!(movement.hit_y);
    }

    #[test]
    fn actor_radius_blocks_before_center_enters_obstacle_cell() {
        let mut world_map = WorldMap::new(4, 4);
        world_map.set_blocked(2, 1, true);

        let movement = move_with_collision(
            Vec2::new(1.2, 1.0),
            Vec2::new(0.6, 0.0),
            &world_map,
            CollisionShape { radius: 0.3 },
        );

        assert_eq!(movement.end, Vec2::new(1.2, 1.0));
        assert!(movement.hit_x);
    }

    #[test]
    fn movement_is_clamped_to_map_bounds() {
        let world_map = WorldMap::new(4, 4);

        let movement = move_with_collision(
            Vec2::new(0.2, 0.2),
            Vec2::new(-2.0, -2.0),
            &world_map,
            CollisionShape { radius: 0.25 },
        );

        assert_eq!(movement.end, Vec2::new(0.25, 0.25));
    }
}
