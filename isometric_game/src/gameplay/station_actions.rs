use std::collections::HashMap;
use std::fs;
use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

use crate::gameplay::{ResourceChange, StationKind, TavernState};

pub const STATION_ACTIONS_PATH: &str = "data/gameplay/station_actions.ron";

#[derive(Resource)]
pub struct StationActionCatalog {
    actions: HashMap<StationKind, StationAction>,
}

impl StationActionCatalog {
    pub fn action_for(&self, kind: StationKind) -> Option<&StationAction> {
        self.actions.get(&kind)
    }

    fn from_actions(actions: Vec<StationAction>) -> Result<Self, String> {
        let mut indexed = HashMap::new();

        for action in actions {
            if indexed.insert(action.station, action).is_some() {
                return Err("duplicate station action entry".to_string());
            }
        }

        Ok(Self { actions: indexed })
    }
}

impl Default for StationActionCatalog {
    fn default() -> Self {
        Self::from_actions(vec![
            StationAction {
                station: StationKind::Pantry,
                label: "Restock pantry".to_string(),
                cooldown_seconds: 6.0,
                changes: vec![
                    ResourceChange::new(crate::gameplay::TavernResource::Scraps, -2),
                    ResourceChange::new(crate::gameplay::TavernResource::Food, 3),
                    ResourceChange::new(crate::gameplay::TavernResource::TradeGoods, 1),
                ],
                success_notice: "Pantry stocked from sealed tins.".to_string(),
            },
            StationAction {
                station: StationKind::WaterStill,
                label: "Filter water".to_string(),
                cooldown_seconds: 5.0,
                changes: vec![
                    ResourceChange::new(crate::gameplay::TavernResource::Scraps, -1),
                    ResourceChange::new(crate::gameplay::TavernResource::Water, 2),
                ],
                success_notice: "The still coughs up clean water.".to_string(),
            },
            StationAction {
                station: StationKind::Stove,
                label: "Warm the room".to_string(),
                cooldown_seconds: 8.0,
                changes: vec![
                    ResourceChange::new(crate::gameplay::TavernResource::Food, -1),
                    ResourceChange::new(crate::gameplay::TavernResource::Water, -1),
                    ResourceChange::new(crate::gameplay::TavernResource::Warmth, 2),
                    ResourceChange::new(crate::gameplay::TavernResource::Morale, 1),
                ],
                success_notice: "A hot pot steadies the room.".to_string(),
            },
            StationAction {
                station: StationKind::Radio,
                label: "Scan for rumors".to_string(),
                cooldown_seconds: 10.0,
                changes: vec![
                    ResourceChange::new(crate::gameplay::TavernResource::Scraps, -1),
                    ResourceChange::new(crate::gameplay::TavernResource::Rumors, 2),
                    ResourceChange::new(crate::gameplay::TavernResource::Threat, -6),
                ],
                success_notice: "Static clears into useful road talk.".to_string(),
            },
        ])
        .expect("built-in station actions should be valid")
    }
}

#[derive(Clone, Deserialize)]
pub struct StationAction {
    pub station: StationKind,
    pub label: String,
    #[serde(default)]
    pub cooldown_seconds: f32,
    #[serde(default)]
    pub changes: Vec<ResourceChange>,
    pub success_notice: String,
}

#[derive(Component, Default)]
pub struct StationCooldown {
    timer: Option<Timer>,
}

impl StationCooldown {
    pub fn start(&mut self, seconds: f32) {
        if seconds > 0.0 {
            self.timer = Some(Timer::from_seconds(seconds, TimerMode::Once));
        }
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        let Some(timer) = &mut self.timer else {
            return;
        };

        timer.tick(delta);
        if timer.finished() {
            self.timer = None;
        }
    }

    pub fn remaining_seconds(&self) -> Option<f32> {
        self.timer.as_ref().map(|timer| timer.remaining_secs())
    }

    pub fn is_ready(&self) -> bool {
        self.timer.is_none()
    }
}

pub fn load_station_action_catalog(path: &str) -> Result<StationActionCatalog, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let actions: Vec<StationAction> = ron::de::from_str(&source)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    StationActionCatalog::from_actions(actions)
}

pub(super) fn tick_station_cooldowns(
    time: Res<Time>,
    mut stations: Query<&mut StationCooldown>,
) {
    for mut cooldown in &mut stations {
        cooldown.tick(time.delta());
    }
}

pub fn apply_station_action(
    tavern_state: &mut TavernState,
    station_label: &str,
    action: &StationAction,
    cooldown: &mut StationCooldown,
) {
    if !cooldown.is_ready() {
        if let Some(remaining) = cooldown.remaining_seconds() {
            tavern_state.set_notice(format!("{station_label} needs {remaining:.0}s before reuse."));
        }
        return;
    }

    if !tavern_state.can_apply(&action.changes) {
        let missing = crate::gameplay::missing_requirements_label(tavern_state, &action.changes);
        tavern_state.set_notice(format!("{} needs {}.", action.label, missing));
        return;
    }

    tavern_state.apply_changes(&action.changes);
    cooldown.start(action.cooldown_seconds);
    tavern_state.set_notice(action.success_notice.clone());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::Order;
    use std::time::Duration;

    #[test]
    fn station_action_file_loads() {
        let catalog =
            load_station_action_catalog(STATION_ACTIONS_PATH).expect("station actions should load");

        assert!(catalog.action_for(StationKind::Pantry).is_some());
        assert!(catalog.action_for(StationKind::WaterStill).is_some());
        assert!(catalog.action_for(StationKind::Stove).is_some());
        assert!(catalog.action_for(StationKind::Radio).is_some());
    }

    #[test]
    fn radio_can_reduce_threat_below_action_amount() {
        let catalog = StationActionCatalog::default();
        let action = catalog
            .action_for(StationKind::Radio)
            .expect("radio action should exist");
        let mut state = TavernState {
            threat: 2.0,
            scraps: 1,
            ..default()
        };
        let mut cooldown = StationCooldown::default();

        apply_station_action(&mut state, "radio", action, &mut cooldown);

        assert_eq!(state.threat, 0.0);
        assert_eq!(state.rumors, 3);
        assert_eq!(state.scraps, 0);
    }

    #[test]
    fn five_minute_service_loop_changes_resources_meaningfully() {
        let catalog = StationActionCatalog::default();
        let mut state = TavernState::default();
        let mut pantry = StationCooldown::default();
        let mut still = StationCooldown::default();
        let mut stove = StationCooldown::default();
        let mut radio = StationCooldown::default();
        let orders = [
            Order::Meal,
            Order::Drink,
            Order::Shelter,
            Order::Rumor,
            Order::Trade,
            Order::Both,
        ];

        for step in 0..60 {
            use_station_if_possible(&catalog, &mut state, StationKind::Pantry, &mut pantry);
            use_station_if_possible(&catalog, &mut state, StationKind::WaterStill, &mut still);
            use_station_if_possible(&catalog, &mut state, StationKind::Stove, &mut stove);
            use_station_if_possible(&catalog, &mut state, StationKind::Radio, &mut radio);

            if step % 3 == 0 {
                let order = orders[(step / 3) % orders.len()];
                if state.can_apply(order.changes()) {
                    state.apply_changes(order.changes());
                    state.served += 1;
                } else {
                    state.missed += 1;
                }
            }

            for cooldown in [&mut pantry, &mut still, &mut stove, &mut radio] {
                cooldown.tick(Duration::from_secs(5));
            }
        }

        assert!(state.served >= 12);
        assert!(state.reputation > TavernState::default().reputation);
        assert_ne!(state.scraps, TavernState::default().scraps);
        assert!(state.missed <= 2);
    }

    fn use_station_if_possible(
        catalog: &StationActionCatalog,
        state: &mut TavernState,
        kind: StationKind,
        cooldown: &mut StationCooldown,
    ) {
        let action = catalog
            .action_for(kind)
            .expect("station action should exist in built-in catalog");

        if cooldown.is_ready() && state.can_apply(&action.changes) {
            apply_station_action(state, kind.label(), action, cooldown);
        }
    }
}
