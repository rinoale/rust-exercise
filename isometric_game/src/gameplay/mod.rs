mod interactions;
mod orders;
mod state;
mod stations;

use bevy::prelude::*;

use crate::GameSet;

pub use interactions::DirtySpill;
pub use orders::Order;
pub use state::TavernState;
pub use stations::{Station, StationKind};

use interactions::handle_interactions;
use state::{cleanup_notice, tick_tavern_state};

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TavernState>().add_systems(
            Update,
            (tick_tavern_state, handle_interactions, cleanup_notice)
                .chain()
                .in_set(GameSet::Gameplay),
        );
    }
}
