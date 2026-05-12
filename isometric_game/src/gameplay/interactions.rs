use bevy::prelude::*;
use rand::Rng;

use crate::actors::{Customer, CustomerState, Player};
use crate::config::INTERACT_RADIUS;
use crate::gameplay::{
    Station, StationActionCatalog, StationCooldown, TavernState, apply_station_action,
    missing_requirements_label,
};
use crate::rendering::create_diamond_mesh;
use crate::universe::{GridPosition, IsoRenderable, UniverseLayer, distance_grid, iso_translation};

#[derive(Component)]
pub struct DirtySpill;

#[derive(Clone, Debug, PartialEq)]
pub enum InteractionOutcome {
    ServedCustomer { order: String },
    CleanedSpill,
    UsedStation { station: String },
    NotReady,
    NothingInReach,
}

pub(super) fn handle_interactions(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&GridPosition, With<Player>>,
    mut customers: Query<(&mut Customer, &GridPosition), Without<Player>>,
    mut stations: Query<(&Station, &GridPosition, &mut StationCooldown), Without<Player>>,
    spills: Query<(Entity, &GridPosition), (With<DirtySpill>, Without<Player>)>,
    station_actions: Res<StationActionCatalog>,
    mut tavern_state: ResMut<TavernState>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_position) = player_query.get_single() else {
        return;
    };

    interact_at(
        player_position.value,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut customers,
        &mut stations,
        &spills,
        &station_actions,
        &mut tavern_state,
    );
}

pub(super) fn interact_at(
    player_grid: Vec2,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    customers: &mut Query<(&mut Customer, &GridPosition), Without<Player>>,
    stations: &mut Query<(&Station, &GridPosition, &mut StationCooldown), Without<Player>>,
    spills: &Query<(Entity, &GridPosition), (With<DirtySpill>, Without<Player>)>,
    station_actions: &StationActionCatalog,
    tavern_state: &mut TavernState,
) -> InteractionOutcome {
    for (mut customer, customer_position) in customers.iter_mut() {
        if customer.state == CustomerState::Waiting
            && distance_grid(player_grid, customer_position.value) <= INTERACT_RADIUS
        {
            let order = customer.order.label().to_string();
            serve_customer(&mut customer, tavern_state, commands, meshes, materials);
            return InteractionOutcome::ServedCustomer { order };
        }
    }

    for (entity, spill_position) in spills.iter() {
        if distance_grid(player_grid, spill_position.value) <= INTERACT_RADIUS {
            commands.entity(entity).despawn_recursive();
            tavern_state.morale = (tavern_state.morale + 1).min(10);
            tavern_state.set_notice("The floor is clean enough for one more night.");
            return InteractionOutcome::CleanedSpill;
        }
    }

    for (station, station_position, mut cooldown) in stations.iter_mut() {
        if distance_grid(player_grid, station_position.value) <= INTERACT_RADIUS {
            let Some(action) = station_actions.action_for(station.kind) else {
                tavern_state.set_notice(format!("{} is not ready for use.", station.kind.label()));
                return InteractionOutcome::NotReady;
            };
            apply_station_action(tavern_state, station.kind.label(), action, &mut cooldown);
            return InteractionOutcome::UsedStation {
                station: station.kind.label().to_string(),
            };
        }
    }

    tavern_state.set_notice("Nothing useful is within reach.");
    InteractionOutcome::NothingInReach
}

fn serve_customer(
    customer: &mut Customer,
    tavern_state: &mut TavernState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let order = customer.order;
    let changes = order.changes();
    if !tavern_state.can_apply(changes) {
        let missing = missing_requirements_label(tavern_state, changes);
        tavern_state.set_notice(format!(
            "Need {} for a {}. Check the {}.",
            missing,
            order.label(),
            order.station_hint()
        ));
        return;
    }

    tavern_state.apply_changes(changes);
    tavern_state.served += 1;
    customer.state = CustomerState::Leaving;
    tavern_state.set_notice(format!("Served a {}.", order.label()));

    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.28) {
        let spill_grid =
            customer.target + Vec2::new(rng.gen_range(-0.45..0.45), rng.gen_range(-0.35..0.35));
        spawn_spill(commands, meshes, materials, spill_grid);
    }
}

fn spawn_spill(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid: Vec2,
) {
    let layer = UniverseLayer::Decal;
    commands.spawn((
        Mesh2d(meshes.add(create_diamond_mesh(28.0, 8.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
            0.16, 0.08, 0.04, 0.68,
        )))),
        GridPosition::new(grid),
        IsoRenderable::with_offset(2.0, layer, 8.0),
        Transform::from_translation(iso_translation(grid, 2.0, layer, 8.0)),
        DirtySpill,
    ));
}
