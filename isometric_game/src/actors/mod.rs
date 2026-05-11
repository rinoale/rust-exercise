mod customer;
mod player;

use bevy::prelude::*;

use crate::GameSet;

pub use customer::{Customer, CustomerState};
pub use player::Player;

use customer::{customer_patience, move_customers, spawn_customers, CustomerSpawner};
use player::{player_movement, setup_player};

pub struct ActorsPlugin;

impl Plugin for ActorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomerSpawner>()
            .add_systems(Startup, setup_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    spawn_customers,
                    move_customers,
                    customer_patience,
                )
                    .chain()
                    .in_set(GameSet::Actors),
            );
    }
}
