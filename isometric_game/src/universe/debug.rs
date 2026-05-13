use bevy::prelude::*;

use crate::config::{TILE_HEIGHT, TILE_WIDTH};
use crate::universe::{GridPosition, IsoRenderable, UniverseLayer, WorldMap, grid_to_screen_at};

#[derive(Resource)]
pub struct DebugSettings {
    pub show_grid: bool,
    pub show_collision: bool,
    pub show_depth_anchors: bool,
    pub show_actor_animation: bool,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            show_grid: false,
            show_collision: false,
            show_depth_anchors: false,
            show_actor_animation: false,
        }
    }
}

pub(super) fn toggle_debug_settings(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug: ResMut<DebugSettings>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        debug.show_grid = !debug.show_grid;
    }
    if keyboard.just_pressed(KeyCode::F2) {
        debug.show_collision = !debug.show_collision;
    }
    if keyboard.just_pressed(KeyCode::F3) {
        debug.show_depth_anchors = !debug.show_depth_anchors;
    }
    if keyboard.just_pressed(KeyCode::F4) {
        debug.show_actor_animation = !debug.show_actor_animation;
    }
}

pub(super) fn draw_debug_universe(
    mut gizmos: Gizmos,
    debug: Res<DebugSettings>,
    world_map: Res<WorldMap>,
    renderables: Query<(&GridPosition, &IsoRenderable)>,
) {
    if debug.show_grid || debug.show_collision {
        for z in 0..world_map.levels {
            for y in 0..world_map.height {
                for x in 0..world_map.width {
                    let blocked = world_map.is_blocked_at(x, y, z);
                    let level_alpha = (0.34 - z as f32 * 0.07).max(0.12);
                    let color = if debug.show_collision {
                        if blocked {
                            Color::srgba(0.95, 0.16, 0.12, 0.9)
                        } else {
                            Color::srgba(0.25, 0.85, 0.42, level_alpha)
                        }
                    } else {
                        Color::srgba(0.85, 0.85, 0.78, level_alpha)
                    };
                    draw_debug_cell(&mut gizmos, Vec2::new(x as f32, y as f32), z, color);

                    if debug.show_collision && blocked {
                        let center = grid_to_screen_at(Vec2::new(x as f32, y as f32), z);
                        gizmos.line_2d(
                            center + Vec2::new(-10.0, -5.0),
                            center + Vec2::new(10.0, 5.0),
                            Color::srgba(1.0, 0.05, 0.03, 0.85),
                        );
                        gizmos.line_2d(
                            center + Vec2::new(-10.0, 5.0),
                            center + Vec2::new(10.0, -5.0),
                            Color::srgba(1.0, 0.05, 0.03, 0.85),
                        );
                    }
                }
            }
        }
    }

    if debug.show_depth_anchors {
        for (grid_position, renderable) in &renderables {
            let anchor = grid_to_screen_at(grid_position.value, grid_position.z);
            let color = match renderable.layer {
                UniverseLayer::Ground | UniverseLayer::Floor => Color::srgba(0.8, 0.8, 0.8, 0.35),
                UniverseLayer::Decal => Color::srgba(0.35, 0.75, 1.0, 0.75),
                UniverseLayer::Prop => Color::srgba(1.0, 0.8, 0.25, 0.85),
                UniverseLayer::Wall => Color::srgba(0.9, 0.35, 1.0, 0.85),
                UniverseLayer::Character => Color::srgba(0.2, 1.0, 0.25, 0.95),
                UniverseLayer::Roof => Color::srgba(1.0, 0.2, 0.2, 0.85),
                UniverseLayer::Overlay => Color::srgba(1.0, 1.0, 1.0, 0.85),
            };
            gizmos.circle_2d(anchor, 4.0, color);
            gizmos.line_2d(
                anchor,
                anchor + Vec2::new(0.0, renderable.y_offset),
                Color::srgba(1.0, 1.0, 1.0, 0.35),
            );
        }
    }
}

fn draw_debug_cell(gizmos: &mut Gizmos, grid: Vec2, z: i32, color: Color) {
    let center = grid_to_screen_at(grid, z);
    let top = center + Vec2::new(0.0, TILE_HEIGHT * 0.5);
    let right = center + Vec2::new(TILE_WIDTH * 0.5, 0.0);
    let bottom = center + Vec2::new(0.0, -TILE_HEIGHT * 0.5);
    let left = center + Vec2::new(-TILE_WIDTH * 0.5, 0.0);

    gizmos.line_2d(top, right, color);
    gizmos.line_2d(right, bottom, color);
    gizmos.line_2d(bottom, left, color);
    gizmos.line_2d(left, top, color);
}
