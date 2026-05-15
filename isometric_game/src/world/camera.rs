use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::actors::Player;
use crate::config::CAMERA_LERP;
use crate::universe::grid_to_screen_at;
use crate::world::SceneEditorState;

#[derive(Component)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

pub fn camera_follow_player(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let target = Vec3::new(
        player_transform.translation.x,
        player_transform.translation.y + 20.0,
        camera_transform.translation.z,
    );
    camera_transform.translation = camera_transform
        .translation
        .lerp(target, CAMERA_LERP * time.delta_secs());
}

pub fn camera_zoom(
    mut scroll_events: EventReader<MouseWheel>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let Ok(mut projection) = camera_query.get_single_mut() else {
        return;
    };

    for event in scroll_events.read() {
        let scroll = match event.unit {
            MouseScrollUnit::Line => event.y * 0.1,
            MouseScrollUnit::Pixel => event.y * 0.001,
        };
        projection.scale = (projection.scale - scroll).clamp(0.45, 2.4);
    }
}

pub fn camera_follow_editor_cursor(
    editor: Res<SceneEditorState>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    time: Res<Time>,
) {
    if !editor.enabled {
        return;
    }

    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let target = grid_to_screen_at(editor.cursor.as_vec2(), editor.cursor_z);
    let target = Vec3::new(target.x, target.y, camera_transform.translation.z);
    camera_transform.translation = camera_transform
        .translation
        .lerp(target, CAMERA_LERP * time.delta_secs());
}
