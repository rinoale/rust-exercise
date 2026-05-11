mod interactions;
mod orders;
mod resources;
mod scripted_playback;
mod state;
mod station_actions;
mod stations;

use bevy::prelude::*;

use crate::GameSet;

pub use interactions::DirtySpill;
pub use orders::Order;
pub use resources::{missing_requirements_label, ResourceChange, TavernResource};
pub use scripted_playback::{ScriptedPlaybackConfig, ScriptedPlaybackPlugin};
pub use station_actions::{
    apply_station_action, load_station_action_catalog, StationActionCatalog, StationCooldown,
    STATION_ACTIONS_PATH,
};
pub use state::TavernState;
pub use stations::{Station, StationKind};

use interactions::handle_interactions;
use station_actions::tick_station_cooldowns;
use state::{cleanup_notice, tick_tavern_state};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        let station_actions =
            load_station_action_catalog(STATION_ACTIONS_PATH).unwrap_or_else(|error| {
                eprintln!("Falling back to built-in station actions: {error}");
                StationActionCatalog::default()
            });

        app.init_resource::<TavernState>()
            .insert_resource(station_actions)
            .add_systems(
                Update,
                (
                    tick_tavern_state,
                    tick_station_cooldowns,
                    handle_interactions,
                    cleanup_notice,
                )
                    .chain()
                    .in_set(GameSet::Gameplay),
            );
    }
}
