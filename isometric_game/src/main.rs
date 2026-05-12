mod actors;
mod config;
mod gameplay;
mod rendering;
mod ui;
mod universe;
mod world;

use actors::ActorsPlugin;
use bevy::prelude::*;
use gameplay::{GameplayPlugin, ScriptedPlaybackConfig, ScriptedPlaybackPlugin};
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
    VisualAnimation,
    Camera,
    Ui,
    Debug,
}

fn main() {
    let scripted_playback = match ScriptedPlaybackConfig::from_args(std::env::args().skip(1)) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("Usage: cargo run -- --script smoke_walk_and_serve");
            std::process::exit(2);
        }
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
            GameSet::VisualAnimation,
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
    ));

    if let Some(scripted_playback) = scripted_playback {
        app.insert_resource(scripted_playback)
            .add_plugins(ScriptedPlaybackPlugin);
    }

    app.run();
}
