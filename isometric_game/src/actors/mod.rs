mod animation;
mod customer;
mod player;

use bevy::prelude::*;

use crate::{GameSet, app_in_play_mode, app_not_intro};

pub use customer::{Customer, CustomerState};
pub use player::Player;

pub use animation::{ActorAnimation, ActorVisualMotion};

use animation::{
    ActorAnimationCatalog, CHARACTER_ANIMATIONS_PATH, load_actor_animation_catalog,
    preload_actor_animation_assets, update_actor_animation, update_actor_visual_motion,
};
use customer::{CustomerSpawner, customer_patience, move_customers, spawn_customers};
use player::{player_movement, setup_player};

pub struct ActorsPlugin;

impl Plugin for ActorsPlugin {
    fn build(&self, app: &mut App) {
        let animation_catalog = load_actor_animation_catalog(CHARACTER_ANIMATIONS_PATH)
            .unwrap_or_else(|error| {
                eprintln!("{error}");
                ActorAnimationCatalog::default()
            });

        app.insert_resource(animation_catalog)
            .init_resource::<CustomerSpawner>()
            .add_systems(
                Startup,
                (preload_actor_animation_assets, setup_player).chain(),
            )
            .add_systems(
                Update,
                (
                    player_movement,
                    spawn_customers,
                    move_customers,
                    customer_patience,
                )
                    .chain()
                    .run_if(app_in_play_mode)
                    .in_set(GameSet::Actors),
            )
            .add_systems(
                Update,
                (update_actor_visual_motion, update_actor_animation)
                    .chain()
                    .run_if(app_not_intro)
                    .in_set(GameSet::VisualAnimation),
            );
    }
}
