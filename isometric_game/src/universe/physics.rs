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

#[cfg(test)]
pub fn move_with_collision(
    current: Vec2,
    requested_delta: Vec2,
    world_map: &WorldMap,
    shape: CollisionShape,
) -> KinematicMove {
    move_with_collision_at(current, requested_delta, 0, world_map, shape)
}

pub fn move_with_collision_at(
    current: Vec2,
    requested_delta: Vec2,
    z: i32,
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
    if is_walkable_body_path_at(start, full, z, world_map, shape) {
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
    let x_walkable = requested_delta.x.abs() > MIN_DELTA
        && is_walkable_body_path_at(start, x_only, z, world_map, shape);
    let y_walkable = requested_delta.y.abs() > MIN_DELTA
        && is_walkable_body_path_at(start, y_only, z, world_map, shape);

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

pub fn step_toward_with_collision_at(
    current: Vec2,
    target: Vec2,
    speed: f32,
    delta_seconds: f32,
    z: i32,
    world_map: &WorldMap,
    shape: CollisionShape,
) -> KinematicMove {
    let delta = target - current;
    let distance = delta.length();
    if distance <= MIN_DELTA {
        return move_with_collision_at(current, Vec2::ZERO, z, world_map, shape);
    }

    let max_step = speed.max(0.0) * delta_seconds.max(0.0);
    let requested_delta = if distance <= max_step {
        delta
    } else {
        delta / distance * max_step
    };

    move_with_collision_at(current, requested_delta, z, world_map, shape)
}

pub fn is_walkable_body_at(
    position: Vec2,
    z: i32,
    world_map: &WorldMap,
    shape: CollisionShape,
) -> bool {
    collision_samples(position, shape)
        .into_iter()
        .all(|sample| world_map.is_walkable_world_at(sample, z))
}

fn is_walkable_body_path_at(
    from: Vec2,
    to: Vec2,
    z: i32,
    world_map: &WorldMap,
    shape: CollisionShape,
) -> bool {
    if !is_walkable_body_at(to, z, world_map, shape) {
        return false;
    }

    collision_samples(from, shape)
        .into_iter()
        .zip(collision_samples(to, shape))
        .all(|(from_sample, to_sample)| {
            sample_path_is_walkable_at(from_sample, to_sample, z, world_map)
        })
}

fn sample_path_is_walkable_at(from: Vec2, to: Vec2, z: i32, world_map: &WorldMap) -> bool {
    if !world_map.is_walkable_world_at(to, z) {
        return false;
    }

    let from_cell = rounded_cell(from);
    let to_cell = rounded_cell(to);
    let delta = to_cell - from_cell;
    if delta == IVec2::ZERO {
        return true;
    }

    let x_step = delta.x.signum();
    let y_step = delta.y.signum();

    if delta.x.abs() > 1 || delta.y.abs() > 1 {
        return false;
    }

    if x_step != 0 {
        let next = from_cell + IVec2::new(x_step, 0);
        if world_map.is_edge_blocked_between_at(from_cell, next, z) {
            return false;
        }
    }

    if y_step != 0 {
        let next = from_cell + IVec2::new(0, y_step);
        if world_map.is_edge_blocked_between_at(from_cell, next, z) {
            return false;
        }
    }

    if x_step != 0 && y_step != 0 {
        let x_first = from_cell + IVec2::new(x_step, 0);
        let y_first = from_cell + IVec2::new(0, y_step);
        if world_map.is_edge_blocked_between_at(x_first, to_cell, z)
            || world_map.is_edge_blocked_between_at(y_first, to_cell, z)
        {
            return false;
        }
    }

    true
}

fn rounded_cell(position: Vec2) -> IVec2 {
    IVec2::new(position.x.round() as i32, position.y.round() as i32)
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

    #[test]
    fn movement_collision_uses_actor_level() {
        let mut world_map = WorldMap::with_levels(4, 4, 2);
        world_map.set_blocked_at(2, 1, 1, true);

        let lower = move_with_collision_at(
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            0,
            &world_map,
            CollisionShape { radius: 0.0 },
        );
        let upper = move_with_collision_at(
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            1,
            &world_map,
            CollisionShape { radius: 0.0 },
        );

        assert_eq!(lower.end, Vec2::new(2.0, 1.0));
        assert_eq!(upper.end, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn edge_wall_blocks_crossing_without_blocking_cells() {
        let mut world_map = WorldMap::new(4, 4);
        world_map.set_edge_wall_at(1, 1, 0, crate::universe::EdgeDirection::S, true);

        let movement = move_with_collision(
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            &world_map,
            CollisionShape { radius: 0.0 },
        );

        assert_eq!(movement.end, Vec2::new(1.0, 1.0));
        assert!(movement.hit_x);
    }

    #[test]
    fn edge_wall_blocks_diagonal_corner_crossing() {
        let mut world_map = WorldMap::new(4, 4);
        world_map.set_edge_wall_at(1, 1, 0, crate::universe::EdgeDirection::W, true);

        let movement = move_with_collision(
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 1.0),
            &world_map,
            CollisionShape { radius: 0.0 },
        );

        assert_ne!(movement.end, Vec2::new(2.0, 2.0));
    }
}
