use bevy::prelude::*;

use crate::config::{MAP_HEIGHT, MAP_WIDTH, PLAYER_SPEED};
use crate::rendering::{character_sprite, character_y_offset};
use crate::universe::{iso_translation, GridPosition, IsoRenderable, UniverseLayer, WorldMap};

#[derive(Component)]
pub struct Player;

pub(super) fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start = Vec2::new(18.0, 10.0);
    let layer = UniverseLayer::Character;
    let y_offset = character_y_offset();

    commands.spawn((
        character_sprite(asset_server.load("characters/bartender.png")),
        GridPosition::new(start),
        IsoRenderable::new(y_offset, layer),
        Transform::from_translation(iso_translation(start, y_offset, layer, 0.0)),
        Player,
    ));
}

pub(super) fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    world_map: Res<WorldMap>,
    mut player_query: Query<&mut GridPosition, With<Player>>,
) {
    let Ok(mut grid_position) = player_query.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        direction += Vec2::new(-1.0, -1.0);
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        direction += Vec2::new(1.0, 1.0);
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        direction += Vec2::new(-1.0, 1.0);
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        direction += Vec2::new(1.0, -1.0);
    }

    if direction.length_squared() == 0.0 {
        return;
    }

    let step = direction.normalize() * PLAYER_SPEED * time.delta_secs();
    let current = grid_position.value;

    if let Some(next) = walkable_candidate(current, step, &world_map) {
        grid_position.value = next;
    }
}

fn walkable_candidate(current: Vec2, step: Vec2, world_map: &WorldMap) -> Option<Vec2> {
    let full = clamp_to_map(current + step);
    if world_map.is_walkable_world(full) {
        return Some(full);
    }

    let x_only = clamp_to_map(current + Vec2::new(step.x, 0.0));
    if world_map.is_walkable_world(x_only) {
        return Some(x_only);
    }

    let y_only = clamp_to_map(current + Vec2::new(0.0, step.y));
    if world_map.is_walkable_world(y_only) {
        return Some(y_only);
    }

    None
}

fn clamp_to_map(grid: Vec2) -> Vec2 {
    Vec2::new(
        grid.x.clamp(0.0, (MAP_WIDTH - 1) as f32),
        grid.y.clamp(0.0, (MAP_HEIGHT - 1) as f32),
    )
}
