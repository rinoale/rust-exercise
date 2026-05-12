mod blueprint;
mod camera;
mod cutaway;
mod import;
mod layout;
mod spawn;

use bevy::prelude::*;

use crate::GameSet;

pub use blueprint::TavernMap;

use camera::{camera_follow_player, camera_zoom, setup_camera};
use cutaway::update_cutaways;
use import::{DEMO_TAVERN_PATH, load_world_blueprint};
use layout::demo_tavern_blueprint;
use spawn::spawn_world;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let blueprint = load_world_blueprint(DEMO_TAVERN_PATH).unwrap_or_else(|error| {
            eprintln!("Falling back to built-in demo tavern blueprint: {error}");
            demo_tavern_blueprint()
        });
        let world_map = blueprint.build_world_map();

        app.insert_resource(blueprint)
            .insert_resource(world_map)
            .add_systems(Startup, (setup_camera, spawn_world).chain())
            .add_systems(Update, update_cutaways.in_set(GameSet::RenderSync))
            .add_systems(
                Update,
                (camera_follow_player, camera_zoom)
                    .chain()
                    .in_set(GameSet::Camera),
            );
    }
}
