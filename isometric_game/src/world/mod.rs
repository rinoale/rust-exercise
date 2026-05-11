mod camera;
mod layout;
mod setup;

use bevy::prelude::*;

use crate::GameSet;

pub use layout::TavernMap;

use camera::{camera_follow_player, camera_zoom, setup_camera};
use layout::demo_world_map;
use setup::setup_world;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(demo_world_map())
            .add_systems(Startup, (setup_camera, setup_world).chain())
            .add_systems(
                Update,
                (camera_follow_player, camera_zoom).chain().in_set(GameSet::Camera),
            );
    }
}
