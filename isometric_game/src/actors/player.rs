use bevy::prelude::*;

use crate::actors::{ActorAnimation, ActorVisualMotion};
use crate::config::PLAYER_SPEED;
use crate::rendering::{character_sprite, character_y_offset};
use crate::universe::{
    CollisionShape, GridPosition, IsoRenderable, UniverseLayer, WorldMap, iso_translation_at,
    move_with_collision_at,
};

#[derive(Component)]
pub struct Player;

pub(super) fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start = Vec2::new(18.0, 10.0);
    let layer = UniverseLayer::Character;
    let y_offset = character_y_offset();

    commands.spawn((
        character_sprite(asset_server.load("characters/bartender_turnaround/idle_se.png")),
        GridPosition::new(start),
        IsoRenderable::new(y_offset, layer),
        Transform::from_translation(iso_translation_at(start, 0, y_offset, layer, 0.0)),
        ActorVisualMotion::new(start),
        ActorAnimation::new("bartender"),
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
    grid_position.value = move_with_collision_at(
        grid_position.value,
        step,
        grid_position.z,
        &world_map,
        CollisionShape::ACTOR,
    )
    .end;
}
