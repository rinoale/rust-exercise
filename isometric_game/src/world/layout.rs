use bevy::prelude::*;

use crate::config::{
    MAP_HEIGHT, MAP_WIDTH, TAVERN_MAX_X, TAVERN_MAX_Y, TAVERN_MIN_X, TAVERN_MIN_Y,
};
use crate::universe::WorldMap;

#[derive(Component)]
pub struct TavernMap {
    pub seats: Vec<Vec2>,
    pub entrance: Vec2,
}

pub fn demo_world_map() -> WorldMap {
    let mut map = WorldMap::new(MAP_WIDTH, MAP_HEIGHT);

    for x in TAVERN_MIN_X..=TAVERN_MAX_X {
        map.set_blocked(x, TAVERN_MIN_Y, true);
    }
    for y in TAVERN_MIN_Y + 1..=TAVERN_MAX_Y {
        map.set_blocked(TAVERN_MIN_X, y, true);
    }

    for x in 8..=15 {
        map.block_nearest(Vec2::new(x as f32, 6.6));
    }

    for grid in [
        Vec2::new(8.1, 6.0),
        Vec2::new(6.4, 12.2),
        Vec2::new(9.2, 12.5),
        Vec2::new(18.2, 6.3),
        Vec2::new(11.0, 9.4),
        Vec2::new(13.2, 9.4),
        Vec2::new(15.4, 9.4),
        Vec2::new(12.2, 11.6),
        Vec2::new(14.5, 11.6),
        Vec2::new(16.7, 11.6),
        Vec2::new(21.0, 5.0),
        Vec2::new(22.0, 6.0),
        Vec2::new(3.0, 15.0),
        Vec2::new(4.0, 16.0),
    ] {
        map.block_nearest(grid);
    }

    map
}

pub fn is_tavern_floor(x: i32, y: i32) -> bool {
    (TAVERN_MIN_X..=TAVERN_MAX_X).contains(&x) && (TAVERN_MIN_Y..=TAVERN_MAX_Y).contains(&y)
}

pub fn is_tavern_porch(x: i32, y: i32) -> bool {
    (TAVERN_MIN_X - 1..=TAVERN_MAX_X + 1).contains(&x)
        && (TAVERN_MAX_Y..=TAVERN_MAX_Y + 2).contains(&y)
}
