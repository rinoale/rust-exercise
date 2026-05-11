use bevy::prelude::*;

use crate::actors::{Customer, CustomerState, Player};
use crate::config::INTERACT_RADIUS;
use crate::gameplay::{DirtySpill, Station, TavernState};
use crate::universe::{distance_grid, DebugSettings, GridPosition};

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
    customers: Query<(&Customer, &GridPosition)>,
    stations: Query<(&Station, &GridPosition)>,
    spills: Query<(&DirtySpill, &GridPosition)>,
    mut hud_text: Query<&mut Text, With<HudText>>,
    mut prompt_text: Query<&mut Text, (With<PromptText>, Without<HudText>)>,
) {
    if let Ok(mut text) = hud_text.get_single_mut() {
        **text = format!(
            "Last Call Tavern\nTime {:02}:00  Threat {:>3.0}%\nFood {}  Water {}  Scraps {}\nReputation {}  Morale {}\nServed {}  Missed {}\nDebug grid:{} collision:{} depth:{}\n{}",
            tavern_state.day_time.floor() as i32,
            tavern_state.threat,
            tavern_state.food,
            tavern_state.water,
            tavern_state.scraps,
            tavern_state.reputation,
            tavern_state.morale,
            tavern_state.served,
            tavern_state.missed,
            on_off(debug_settings.show_grid),
            on_off(debug_settings.show_collision),
            on_off(debug_settings.show_depth_anchors),
            tavern_state.notice
        );
    }

    if let Ok(mut text) = prompt_text.get_single_mut() {
        **text = interaction_prompt(&player_query, &customers, &stations, &spills);
    }
}

fn interaction_prompt(
    player_query: &Query<&GridPosition, With<Player>>,
    customers: &Query<(&Customer, &GridPosition)>,
    stations: &Query<(&Station, &GridPosition)>,
    spills: &Query<(&DirtySpill, &GridPosition)>,
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

    for (station, station_position) in stations {
        if distance_grid(player_grid, station_position.value) <= INTERACT_RADIUS {
            return format!("E: Use {}", station.kind.label());
        }
    }

    "Move through the room and tend what needs attention.".to_string()
}

fn on_off(value: bool) -> &'static str {
    if value {
        "on"
    } else {
        "off"
    }
}
