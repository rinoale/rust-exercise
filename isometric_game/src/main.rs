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
use world::{UNTITLED_SCENE_PATH, WorldPlugin, WorldRuntimeConfig};

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
    let launch_config = match LaunchConfig::from_args(std::env::args().skip(1)) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            print_usage();
            std::process::exit(2);
        }
    };

    let mut app = App::new();
    app.insert_resource(launch_config.world)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Last Call Tavern".into(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
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

    if let Some(scripted_playback) = launch_config.scripted_playback {
        app.insert_resource(scripted_playback)
            .add_plugins(ScriptedPlaybackPlugin);
    }

    app.run();
}

struct LaunchConfig {
    world: WorldRuntimeConfig,
    scripted_playback: Option<ScriptedPlaybackConfig>,
}

impl LaunchConfig {
    fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let mut world = WorldRuntimeConfig::default();
        let mut scripted_playback = None;
        let mut explicit_world = false;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--world" => {
                    world.world_path = args
                        .next()
                        .ok_or_else(|| "--world requires a path".to_string())?;
                    explicit_world = true;
                }
                "--editor" => {
                    world.editor_enabled = true;
                    world.scene_selector_enabled = true;
                }
                "--script" => {
                    let name = args
                        .next()
                        .ok_or_else(|| "--script requires a script name".to_string())?;
                    scripted_playback = Some(ScriptedPlaybackConfig::from_script_name(&name)?);
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument '{other}'")),
            }
        }

        if world.editor_enabled && !explicit_world {
            world.start_empty_scene = true;
            world.world_path = UNTITLED_SCENE_PATH.to_string();
        }

        Ok(Self {
            world,
            scripted_playback,
        })
    }
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --bin isometric_game -- [--world data/worlds/demo_tavern.ron] [--editor] [--script smoke_walk_and_serve]"
    );
}
