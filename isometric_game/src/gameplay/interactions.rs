use bevy::prelude::*;
use rand::Rng;

use crate::actors::{Customer, CustomerState, Player};
use crate::config::INTERACT_RADIUS;
use crate::gameplay::{Station, StationKind, TavernState};
use crate::rendering::create_diamond_mesh;
use crate::universe::{distance_grid, iso_translation, GridPosition, IsoRenderable, UniverseLayer};

#[derive(Component)]
pub struct DirtySpill;

pub(super) fn handle_interactions(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_query: Query<&GridPosition, With<Player>>,
    mut customers: Query<(&mut Customer, &GridPosition)>,
    stations: Query<(&Station, &GridPosition)>,
    spills: Query<(Entity, &GridPosition), With<DirtySpill>>,
    mut tavern_state: ResMut<TavernState>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Ok(player_position) = player_query.get_single() else {
        return;
    };
    let player_grid = player_position.value;

    for (mut customer, customer_position) in &mut customers {
        if customer.state == CustomerState::Waiting
            && distance_grid(player_grid, customer_position.value) <= INTERACT_RADIUS
        {
            serve_customer(
                &mut customer,
                &mut tavern_state,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
            return;
        }
    }

    for (entity, spill_position) in &spills {
        if distance_grid(player_grid, spill_position.value) <= INTERACT_RADIUS {
            commands.entity(entity).despawn_recursive();
            tavern_state.morale = (tavern_state.morale + 1).min(10);
            tavern_state.set_notice("The floor is clean enough for one more night.");
            return;
        }
    }

    for (station, station_position) in &stations {
        if distance_grid(player_grid, station_position.value) <= INTERACT_RADIUS {
            interact_with_station(station.kind, &mut tavern_state);
            return;
        }
    }

    tavern_state.set_notice("Nothing useful is within reach.");
}

fn serve_customer(
    customer: &mut Customer,
    tavern_state: &mut TavernState,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let order = customer.order;
    if tavern_state.food < order.food_cost() || tavern_state.water < order.water_cost() {
        tavern_state.set_notice(format!("Need supplies for a {}.", order.label()));
        return;
    }

    tavern_state.food -= order.food_cost();
    tavern_state.water -= order.water_cost();
    tavern_state.scraps += order.scrap_value();
    tavern_state.reputation += 1;
    tavern_state.morale = (tavern_state.morale + 1).min(10);
    tavern_state.served += 1;
    customer.state = CustomerState::Leaving;
    tavern_state.set_notice(format!(
        "Served a {}. +{} scraps.",
        order.label(),
        order.scrap_value()
    ));

    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.28) {
        let spill_grid =
            customer.target + Vec2::new(rng.gen_range(-0.45..0.45), rng.gen_range(-0.35..0.35));
        spawn_spill(commands, meshes, materials, spill_grid);
    }
}

fn interact_with_station(kind: StationKind, tavern_state: &mut TavernState) {
    match kind {
        StationKind::Pantry => {
            if tavern_state.scraps >= 2 {
                tavern_state.scraps -= 2;
                tavern_state.food += 3;
                tavern_state.set_notice("Pantry stocked from sealed tins.");
            } else {
                tavern_state.set_notice("The pantry needs 2 scraps to restock.");
            }
        }
        StationKind::WaterStill => {
            if tavern_state.scraps >= 1 {
                tavern_state.scraps -= 1;
                tavern_state.water += 2;
                tavern_state.set_notice("The still coughs up clean water.");
            } else {
                tavern_state.set_notice("The still needs scrap filters.");
            }
        }
        StationKind::Stove => {
            if tavern_state.food >= 1 && tavern_state.water >= 1 {
                tavern_state.food -= 1;
                tavern_state.water -= 1;
                tavern_state.morale = (tavern_state.morale + 2).min(10);
                tavern_state.set_notice("A hot pot steadies the room.");
            } else {
                tavern_state.set_notice("The stove needs food and water.");
            }
        }
        StationKind::Radio => {
            tavern_state.threat = (tavern_state.threat - 6.0).max(0.0);
            tavern_state.set_notice("Static clears long enough to dodge trouble.");
        }
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
