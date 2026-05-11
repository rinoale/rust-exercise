use bevy::prelude::*;
use rand::Rng;

use crate::config::{CUSTOMER_LIMIT, CUSTOMER_SPEED};
use crate::gameplay::{Order, TavernState};
use crate::rendering::{character_sprite, character_y_offset};
use crate::universe::{distance_grid, iso_translation, GridPosition, IsoRenderable, UniverseLayer};
use crate::world::TavernMap;

#[derive(Component)]
pub struct Customer {
    pub target: Vec2,
    pub state: CustomerState,
    pub patience: Timer,
    pub order: Order,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CustomerState {
    Entering,
    Waiting,
    Leaving,
}

#[derive(Resource)]
pub(super) struct CustomerSpawner {
    timer: Timer,
}

impl Default for CustomerSpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

pub(super) fn spawn_customers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut spawner: ResMut<CustomerSpawner>,
    tavern_query: Query<&TavernMap>,
    customers: Query<&Customer>,
) {
    spawner.timer.tick(time.delta());

    if !spawner.timer.just_finished() || customers.iter().count() >= CUSTOMER_LIMIT {
        return;
    }

    let Ok(tavern) = tavern_query.get_single() else {
        return;
    };

    let Some(target) = pick_free_seat(tavern, &customers) else {
        return;
    };

    let mut rng = rand::thread_rng();
    let spawn =
        tavern.entrance + Vec2::new(rng.gen_range(1.0..2.3), rng.gen_range(-0.7..0.8));
    let character = match rng.gen_range(0..4) {
        0 => "characters/traveler.png",
        1 => "characters/courier.png",
        2 => "characters/wounded.png",
        _ => "characters/traveler.png",
    };
    let order = match rng.gen_range(0..6) {
        0 => Order::Meal,
        1 => Order::Drink,
        2 => Order::Both,
        3 => Order::Shelter,
        4 => Order::Rumor,
        _ => Order::Trade,
    };
    let layer = UniverseLayer::Character;
    let y_offset = character_y_offset();

    commands.spawn((
        character_sprite(asset_server.load(character)),
        GridPosition::new(spawn),
        IsoRenderable::new(y_offset, layer),
        Transform::from_translation(iso_translation(spawn, y_offset, layer, 0.0)),
        Customer {
            target,
            state: CustomerState::Entering,
            patience: Timer::from_seconds(rng.gen_range(18.0..30.0), TimerMode::Once),
            order,
        },
    ));
}

pub(super) fn move_customers(
    mut commands: Commands,
    time: Res<Time>,
    tavern_query: Query<&TavernMap>,
    mut customers: Query<(Entity, &mut Customer, &mut GridPosition)>,
) {
    let Ok(tavern) = tavern_query.get_single() else {
        return;
    };

    for (entity, mut customer, mut grid_position) in &mut customers {
        let target = match customer.state {
            CustomerState::Entering | CustomerState::Waiting => customer.target,
            CustomerState::Leaving => tavern.entrance + Vec2::new(1.8, 0.0),
        };

        let delta = target - grid_position.value;
        if delta.length() < 0.08 {
            match customer.state {
                CustomerState::Entering => customer.state = CustomerState::Waiting,
                CustomerState::Leaving => {
                    commands.entity(entity).despawn_recursive();
                }
                CustomerState::Waiting => {}
            }
            continue;
        }

        if customer.state != CustomerState::Waiting {
            let step = delta.normalize_or_zero() * CUSTOMER_SPEED * time.delta_secs();
            grid_position.value += if step.length_squared() > delta.length_squared() {
                delta
            } else {
                step
            };
        }
    }
}

pub(super) fn customer_patience(
    time: Res<Time>,
    mut tavern_state: ResMut<TavernState>,
    mut customers: Query<&mut Customer>,
) {
    for mut customer in &mut customers {
        if customer.state != CustomerState::Waiting {
            continue;
        }

        customer.patience.tick(time.delta());
        if customer.patience.finished() {
            customer.state = CustomerState::Leaving;
            tavern_state.missed += 1;
            tavern_state.morale = (tavern_state.morale - 1).max(0);
            tavern_state.reputation = (tavern_state.reputation - 1).max(0);
            tavern_state.threat = (tavern_state.threat + 2.5).min(100.0);
            tavern_state.set_notice("A guest left hungry. Word spreads fast.");
        }
    }
}

fn pick_free_seat(tavern: &TavernMap, customers: &Query<&Customer>) -> Option<Vec2> {
    let mut rng = rand::thread_rng();
    let free: Vec<Vec2> = tavern
        .seats
        .iter()
        .copied()
        .filter(|seat| {
            customers
                .iter()
                .all(|customer| distance_grid(customer.target, *seat) > 0.4)
        })
        .collect();

    if free.is_empty() {
        None
    } else {
        Some(free[rng.gen_range(0..free.len())])
    }
}
