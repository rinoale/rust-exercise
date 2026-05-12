use bevy::prelude::*;

use crate::actors::{ActorAnimation, ActorVisualMotion, Customer, CustomerState, Player};
use crate::config::INTERACT_RADIUS;
use crate::gameplay::{DirtySpill, Station, StationActionCatalog, StationCooldown, TavernState};
use crate::universe::{DebugSettings, GridPosition, distance_grid};

#[derive(Component)]
pub(super) struct HudText;

#[derive(Component)]
pub(super) struct PromptText;

pub(super) fn setup_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(330.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.032, 0.028, 0.82)),
        ))
        .with_child((
            Text::new(""),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.91, 0.86, 0.74)),
            HudText,
        ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                bottom: Val::Px(18.0),
                width: Val::Px(390.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.035, 0.032, 0.028, 0.78)),
        ))
        .with_child((
            Text::new(""),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::srgb(0.98, 0.88, 0.58)),
            PromptText,
        ));
}

pub(super) fn update_ui(
    debug_settings: Res<DebugSettings>,
    tavern_state: Res<TavernState>,
    player_query: Query<&GridPosition, With<Player>>,
    player_animation: Query<(&ActorVisualMotion, &ActorAnimation), With<Player>>,
    customers: Query<(&Customer, &GridPosition)>,
    stations: Query<(&Station, &StationCooldown, &GridPosition)>,
    spills: Query<(&DirtySpill, &GridPosition)>,
    station_actions: Res<StationActionCatalog>,
    mut hud_text: Query<&mut Text, With<HudText>>,
    mut prompt_text: Query<&mut Text, (With<PromptText>, Without<HudText>)>,
) {
    if let Ok(mut text) = hud_text.get_single_mut() {
        let total_guests = customers.iter().count();
        let waiting_guests = customers
            .iter()
            .filter(|(customer, _)| customer.state == CustomerState::Waiting)
            .count();
        let debug_line = debug_status_line(&debug_settings, &player_animation);
        let notice_line = if tavern_state.notice.is_empty() {
            String::new()
        } else {
            format!("\n{}", tavern_state.notice)
        };

        **text = format!(
            "Last Call Tavern\n{:02}:00  Threat {:>3.0}%  Rep {}  Morale {}\nGuests  Waiting {} / In room {}  Served {}  Missed {}\nSupplies  Food {}  Water {}  Scraps {}\nServices  Warmth {}  Rumors {}  Goods {}{}{}",
            tavern_state.day_time.floor() as i32,
            tavern_state.threat,
            tavern_state.reputation,
            tavern_state.morale,
            waiting_guests,
            total_guests,
            tavern_state.served,
            tavern_state.missed,
            tavern_state.food,
            tavern_state.water,
            tavern_state.scraps,
            tavern_state.warmth,
            tavern_state.rumors,
            tavern_state.trade_goods,
            debug_line,
            notice_line
        );
    }

    if let Ok(mut text) = prompt_text.get_single_mut() {
        **text = interaction_prompt(
            &player_query,
            &customers,
            &stations,
            &spills,
            &station_actions,
        );
    }
}

fn interaction_prompt(
    player_query: &Query<&GridPosition, With<Player>>,
    customers: &Query<(&Customer, &GridPosition)>,
    stations: &Query<(&Station, &StationCooldown, &GridPosition)>,
    spills: &Query<(&DirtySpill, &GridPosition)>,
    station_actions: &StationActionCatalog,
) -> String {
    let Ok(player_position) = player_query.get_single() else {
        return String::new();
    };
    let player_grid = player_position.value;

    for (customer, customer_position) in customers {
        if customer.state == CustomerState::Waiting
            && distance_grid(player_grid, customer_position.value) <= INTERACT_RADIUS
        {
            return format!("E: Serve {}", customer.order.label());
        }
    }

    for (_, spill_position) in spills {
        if distance_grid(player_grid, spill_position.value) <= INTERACT_RADIUS {
            return "E: Clean spill".to_string();
        }
    }

    for (station, cooldown, station_position) in stations {
        if distance_grid(player_grid, station_position.value) <= INTERACT_RADIUS {
            if let Some(remaining) = cooldown.remaining_seconds() {
                return format!("{} cooling down: {remaining:.0}s", station.kind.label());
            }

            if let Some(action) = station_actions.action_for(station.kind) {
                return format!("E: {} - {}", station.kind.label(), action.label);
            }

            return format!("E: Use {}", station.kind.label());
        }
    }

    "Move through the room and tend what needs attention.".to_string()
}

fn debug_status_line(
    debug_settings: &DebugSettings,
    player_animation: &Query<(&ActorVisualMotion, &ActorAnimation), With<Player>>,
) -> String {
    if !debug_settings.show_grid
        && !debug_settings.show_collision
        && !debug_settings.show_depth_anchors
        && !debug_settings.show_actor_animation
    {
        return String::new();
    }

    let animation = if debug_settings.show_actor_animation {
        player_animation
            .get_single()
            .map(|(motion, animation)| {
                format!(
                    "\nAnim  moving:{} facing:{:?} {}",
                    on_off(motion.moving),
                    motion.facing,
                    animation.debug_summary()
                )
            })
            .unwrap_or_else(|_| "\nAnim  unavailable".to_string())
    } else {
        String::new()
    };

    format!(
        "\nDebug  grid:{} collision:{} depth:{} anim:{}{}",
        on_off(debug_settings.show_grid),
        on_off(debug_settings.show_collision),
        on_off(debug_settings.show_depth_anchors),
        on_off(debug_settings.show_actor_animation),
        animation
    )
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
