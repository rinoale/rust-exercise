mod meshes;
mod sprites;

use bevy::prelude::*;

pub use meshes::{create_diamond_mesh, create_rect_mesh};
pub use sprites::{character_sprite, tile_sprite};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, _app: &mut App) {}
}
