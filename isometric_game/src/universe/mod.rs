mod coordinates;
mod debug;
mod direction;
mod layers;
mod map;
mod physics;

use bevy::prelude::*;

use crate::GameSet;

pub use coordinates::{distance_grid, grid_to_screen, iso_translation};
pub use debug::DebugSettings;
pub use direction::IsoDirection;
pub use layers::{IsoRenderable, UniverseLayer};
pub use map::WorldMap;
pub use physics::{CollisionShape, move_with_collision, step_toward_with_collision};

use debug::{draw_debug_universe, toggle_debug_settings};

pub struct UniversePlugin;

impl Plugin for UniversePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugSettings>().add_systems(
            Update,
            (
                toggle_debug_settings.in_set(GameSet::Input),
                apply_iso_transforms.in_set(GameSet::RenderSync),
                draw_debug_universe.in_set(GameSet::Debug),
            ),
        );
    }
}

fn apply_iso_transforms(mut query: Query<(&GridPosition, &IsoRenderable, &mut Transform)>) {
    for (grid_position, renderable, mut transform) in &mut query {
        transform.translation = iso_translation(
            grid_position.value,
            renderable.y_offset,
            renderable.layer,
            renderable.depth_offset,
        );
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct GridPosition {
    pub value: Vec2,
}

impl GridPosition {
    pub fn new(value: Vec2) -> Self {
        Self { value }
    }
}
