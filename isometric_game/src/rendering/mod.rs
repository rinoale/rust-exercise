mod anchors;
mod asset_registry;
mod meshes;
mod sprites;

use bevy::prelude::*;

pub use anchors::RenderAnchor;
pub use asset_registry::{ASSET_REGISTRY_PATH, AssetRegistry, load_asset_registry};
pub use meshes::{create_diamond_mesh, create_rect_mesh};
pub use sprites::{character_sprite, character_y_offset, tile_sprite};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, _app: &mut App) {}
}
