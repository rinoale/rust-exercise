mod actors;
mod config;
mod gameplay;
mod rendering;
mod ui;
mod universe;
mod world;

use actors::ActorsPlugin;
use bevy::prelude::*;
use gameplay::GameplayPlugin;
use rendering::RenderingPlugin;
use ui::UiPlugin;
use universe::UniversePlugin;
use world::WorldPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    Input,
    Actors,
    Gameplay,
    RenderSync,
    Camera,
    Ui,
    Debug,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Last Call Tavern".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.045, 0.045, 0.04)))
        .configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Actors,
                GameSet::Gameplay,
                GameSet::RenderSync,
                GameSet::Camera,
                GameSet::Ui,
                GameSet::Debug,
            )
                .chain(),
        )
        .add_plugins((
            UniversePlugin,
            RenderingPlugin,
            WorldPlugin,
            ActorsPlugin,
            GameplayPlugin,
            UiPlugin,
        ))
        .run();
}
