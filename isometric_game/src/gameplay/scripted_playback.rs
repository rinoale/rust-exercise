use bevy::app::AppExit;
use bevy::prelude::*;

use crate::actors::{Customer, CustomerState, Player};
use crate::gameplay::interactions::{InteractionOutcome, interact_at};
use crate::gameplay::{
    DirtySpill, Station, StationActionCatalog, StationCooldown, StationKind, TavernState,
};
use crate::universe::{DebugSettings, GridPosition};
use crate::world::NavPoint;
use crate::{GameSet, app_in_play_mode};

const SCRIPT_SPEED: f32 = 8.0;
const MAX_RUNTIME_SECONDS: f32 = 45.0;
const STATION_ROUTE: [StationKind; 4] = [
    StationKind::Pantry,
    StationKind::WaterStill,
    StationKind::Stove,
    StationKind::Radio,
];

#[derive(Resource, Clone, Copy)]
pub struct ScriptedPlaybackConfig {
    pub script: ScriptedPlaybackKind,
}

impl ScriptedPlaybackConfig {
    pub fn from_script_name(name: &str) -> Result<Self, String> {
        Ok(Self {
            script: ScriptedPlaybackKind::from_name(name)?,
        })
    }
}

#[derive(Clone, Copy)]
pub enum ScriptedPlaybackKind {
    SmokeWalkAndServe,
}

impl ScriptedPlaybackKind {
    fn from_name(name: &str) -> Result<Self, String> {
        match name {
            "smoke_walk_and_serve" => Ok(Self::SmokeWalkAndServe),
            other => Err(format!(
                "unknown script '{other}', expected smoke_walk_and_serve"
            )),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SmokeWalkAndServe => "smoke_walk_and_serve",
        }
    }
}

pub struct ScriptedPlaybackPlugin;

impl Plugin for ScriptedPlaybackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptedPlaybackState>()
            .add_systems(Startup, setup_scripted_playback)
            .add_systems(
                Update,
                run_scripted_playback
                    .run_if(app_in_play_mode)
                    .in_set(GameSet::Gameplay),
            );
    }
}

#[derive(Resource)]
struct ScriptedPlaybackState {
    phase: PlaybackPhase,
    runtime_seconds: f32,
    route_index: usize,
    started: bool,
}

impl Default for ScriptedPlaybackState {
    fn default() -> Self {
        Self {
            phase: PlaybackPhase::UseStation,
            runtime_seconds: 0.0,
            route_index: 0,
            started: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PlaybackPhase {
    UseStation,
    WaitForCustomer,
    ServeCustomer,
    Done,
}

fn setup_scripted_playback(
    config: Option<Res<ScriptedPlaybackConfig>>,
    mut debug: ResMut<DebugSettings>,
) {
    let Some(config) = config else {
        return;
    };

    debug.show_grid = true;
    debug.show_collision = true;
    debug.show_depth_anchors = true;
    println!(
        "[script:{}] starting; debug overlays enabled",
        config.script.label()
    );
}

fn run_scripted_playback(
    config: Option<Res<ScriptedPlaybackConfig>>,
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<ScriptedPlaybackState>,
    mut player_query: Query<&mut GridPosition, With<Player>>,
    mut customers: Query<(&mut Customer, &GridPosition), Without<Player>>,
    mut stations: Query<(&Station, &GridPosition, &mut StationCooldown), Without<Player>>,
    spills: Query<(Entity, &GridPosition), (With<DirtySpill>, Without<Player>)>,
    station_actions: Res<StationActionCatalog>,
    mut tavern_state: ResMut<TavernState>,
    mut exit: EventWriter<AppExit>,
) {
    let Some(config) = config else {
        return;
    };

    if !state.started {
        state.started = true;
        log_tavern_state(config.script, "initial", &tavern_state);
    }

    state.runtime_seconds += time.delta_secs();
    if state.runtime_seconds > MAX_RUNTIME_SECONDS {
        println!(
            "[script:{}] failed: timed out after {:.1}s",
            config.script.label(),
            state.runtime_seconds
        );
        exit.send(AppExit::from_code(1));
        return;
    }

    let Ok(mut player_position) = player_query.get_single_mut() else {
        return;
    };

    match state.phase {
        PlaybackPhase::UseStation => {
            let Some(station_step) = STATION_ROUTE.get(state.route_index).copied() else {
                state.phase = PlaybackPhase::WaitForCustomer;
                println!(
                    "[script:{}] waiting for a customer to reach a seat",
                    config.script.label()
                );
                return;
            };

            let Some(target) = station_approach_point(station_step, &mut stations) else {
                println!(
                    "[script:{}] failed: station '{}' was not found",
                    config.script.label(),
                    station_step.label()
                );
                exit.send(AppExit::from_code(1));
                return;
            };

            if move_player_toward(&mut player_position, target, time.delta_secs()) {
                let outcome = interact_at(
                    player_position.value,
                    player_position.z,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut customers,
                    &mut stations,
                    &spills,
                    &station_actions,
                    &mut tavern_state,
                );
                println!(
                    "[script:{}] station step {} -> {:?}",
                    config.script.label(),
                    station_step.label(),
                    outcome
                );
                state.route_index += 1;
            }
        }
        PlaybackPhase::WaitForCustomer => {
            if waiting_customer_position(&mut customers).is_some() {
                state.phase = PlaybackPhase::ServeCustomer;
                println!("[script:{}] customer ready", config.script.label());
            }
        }
        PlaybackPhase::ServeCustomer => {
            let Some(target) = waiting_customer_position(&mut customers) else {
                state.phase = PlaybackPhase::WaitForCustomer;
                return;
            };

            if move_player_toward(&mut player_position, target, time.delta_secs()) {
                let outcome = interact_at(
                    player_position.value,
                    player_position.z,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut customers,
                    &mut stations,
                    &spills,
                    &station_actions,
                    &mut tavern_state,
                );
                println!(
                    "[script:{}] customer step -> {:?}",
                    config.script.label(),
                    outcome
                );

                if matches!(outcome, InteractionOutcome::ServedCustomer { .. }) {
                    state.phase = PlaybackPhase::Done;
                }
            }
        }
        PlaybackPhase::Done => {
            log_tavern_state(config.script, "complete", &tavern_state);
            println!(
                "[script:{}] complete after {:.1}s",
                config.script.label(),
                state.runtime_seconds
            );
            exit.send(AppExit::Success);
        }
    }
}

fn station_approach_point(
    station_kind: StationKind,
    stations: &mut Query<(&Station, &GridPosition, &mut StationCooldown), Without<Player>>,
) -> Option<NavPoint> {
    stations.iter_mut().find_map(|(station, position, _)| {
        (station.kind == station_kind).then_some(NavPoint::at(
            position.value + approach_offset(station_kind),
            position.z,
        ))
    })
}

fn approach_offset(station_kind: StationKind) -> Vec2 {
    match station_kind {
        StationKind::Pantry => Vec2::new(0.9, 0.0),
        StationKind::WaterStill => Vec2::new(0.9, 0.0),
        StationKind::Stove => Vec2::new(0.0, 1.0),
        StationKind::Radio => Vec2::new(-0.9, 0.0),
    }
}

fn waiting_customer_position(
    customers: &mut Query<(&mut Customer, &GridPosition), Without<Player>>,
) -> Option<NavPoint> {
    customers.iter_mut().find_map(|(customer, position)| {
        (customer.state == CustomerState::Waiting)
            .then_some(NavPoint::at(position.value, position.z))
    })
}

fn move_player_toward(
    player_position: &mut GridPosition,
    target: NavPoint,
    delta_seconds: f32,
) -> bool {
    if player_position.z != target.z {
        return false;
    }

    let delta = target.value - player_position.value;
    if delta.length() <= 0.08 {
        player_position.value = target.value;
        return true;
    }

    let step = delta.normalize_or_zero() * SCRIPT_SPEED * delta_seconds;
    player_position.value += if step.length_squared() >= delta.length_squared() {
        delta
    } else {
        step
    };

    false
}

fn log_tavern_state(script: ScriptedPlaybackKind, label: &str, tavern_state: &TavernState) {
    println!(
        "[script:{}] {label}: served={} missed={} food={} water={} scraps={} warmth={} rumors={} goods={} morale={} rep={} threat={:.1}",
        script.label(),
        tavern_state.served,
        tavern_state.missed,
        tavern_state.food,
        tavern_state.water,
        tavern_state.scraps,
        tavern_state.warmth,
        tavern_state.rumors,
        tavern_state.trade_goods,
        tavern_state.morale,
        tavern_state.reputation,
        tavern_state.threat
    );
}
