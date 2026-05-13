use bevy::prelude::*;
use rand::Rng;

use crate::actors::{ActorAnimation, ActorVisualMotion};
use crate::config::{CUSTOMER_LIMIT, CUSTOMER_SPEED};
use crate::gameplay::{Order, TavernState};
use crate::rendering::{character_sprite, character_y_offset};
use crate::universe::{
    CollisionShape, GridPosition, IsoRenderable, UniverseLayer, WorldMap, distance_grid,
    iso_translation_at, step_toward_with_collision_at,
};
use crate::world::{NavPoint, TavernMap, VerticalLink};

#[derive(Component)]
pub struct Customer {
    pub target: NavPoint,
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
    let (character_id, character_asset) = match rng.gen_range(0..4) {
        0 => ("traveler", "characters/traveler.png"),
        1 => ("courier", "characters/courier.png"),
        2 => ("wounded", "characters/wounded.png"),
        _ => ("traveler", "characters/traveler.png"),
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
    let spawn = tavern
        .entrance
        .offset(Vec2::new(rng.gen_range(1.0..2.3), rng.gen_range(-0.7..0.8)));

    commands.spawn((
        character_sprite(asset_server.load(character_asset)),
        GridPosition::at(spawn.value, spawn.z),
        IsoRenderable::new(y_offset, layer),
        Transform::from_translation(iso_translation_at(
            spawn.value,
            spawn.z,
            y_offset,
            layer,
            0.0,
        )),
        ActorVisualMotion::new(spawn.value),
        ActorAnimation::new(character_id),
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
    world_map: Res<WorldMap>,
    tavern_query: Query<&TavernMap>,
    mut customers: Query<(Entity, &mut Customer, &mut GridPosition)>,
) {
    let Ok(tavern) = tavern_query.get_single() else {
        return;
    };

    for (entity, mut customer, mut grid_position) in &mut customers {
        let final_target = match customer.state {
            CustomerState::Entering | CustomerState::Waiting => customer.target,
            CustomerState::Leaving => tavern.entrance.offset(Vec2::new(1.8, 0.0)),
        };
        if let Some(exit) =
            vertical_exit_toward_destination(&grid_position, final_target, &tavern.vertical_links)
        {
            grid_position.value = exit.value;
            grid_position.z = exit.z;
            continue;
        }

        let target = next_customer_waypoint(&grid_position, final_target, &tavern.vertical_links);

        let delta = target.value - grid_position.value;
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
            grid_position.value = step_toward_with_collision_at(
                grid_position.value,
                target.value,
                CUSTOMER_SPEED,
                time.delta_secs(),
                grid_position.z,
                &world_map,
                CollisionShape::ACTOR,
            )
            .end;
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

fn pick_free_seat(tavern: &TavernMap, customers: &Query<&Customer>) -> Option<NavPoint> {
    let mut rng = rand::thread_rng();
    let free: Vec<NavPoint> = tavern
        .seats
        .iter()
        .copied()
        .filter(|seat| {
            customers.iter().all(|customer| {
                customer.target.z != seat.z
                    || distance_grid(customer.target.value, seat.value) > 0.4
            })
        })
        .collect();

    if free.is_empty() {
        None
    } else {
        Some(free[rng.gen_range(0..free.len())])
    }
}

fn next_customer_waypoint(
    current: &GridPosition,
    destination: NavPoint,
    links: &[VerticalLink],
) -> NavPoint {
    if current.z == destination.z {
        return destination;
    }

    links
        .iter()
        .filter_map(|link| {
            let entry = link.entry_for_level(current.z)?;
            let exit = link.exit_from_entry(entry)?;
            (exit.z == destination.z).then_some((
                entry,
                entry.value.distance(current.value) + exit.value.distance(destination.value),
            ))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(entry, _)| entry)
        .unwrap_or_else(|| NavPoint::at(destination.value, current.z))
}

fn vertical_exit_toward_destination(
    current: &GridPosition,
    destination: NavPoint,
    links: &[VerticalLink],
) -> Option<NavPoint> {
    if current.z == destination.z {
        return None;
    }

    links
        .iter()
        .filter_map(|link| link.destination_from_near(current.value, current.z, 0.08))
        .find(|exit| exit.z == destination.z)
}
