mod interactions;
mod orders;
mod resources;
mod scripted_playback;
mod state;
mod station_actions;
mod stations;

use bevy::prelude::*;

use crate::{GameSet, app_in_play_mode};

pub use interactions::DirtySpill;
pub(crate) use interactions::vertical_link_destination;
pub use orders::Order;
pub use resources::{ResourceChange, TavernResource, missing_requirements_label};
pub use scripted_playback::{ScriptedPlaybackConfig, ScriptedPlaybackPlugin};
pub use state::TavernState;
pub use station_actions::{
    STATION_ACTIONS_PATH, StationActionCatalog, StationCooldown, apply_station_action,
    load_station_action_catalog,
};
pub use stations::{Station, StationKind};

use interactions::handle_interactions;
use state::{cleanup_notice, tick_tavern_state};
use station_actions::tick_station_cooldowns;

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
                    .run_if(app_in_play_mode)
                    .in_set(GameSet::Gameplay),
            );
    }
}
