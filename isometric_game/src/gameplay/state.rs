use bevy::prelude::*;

use crate::gameplay::{ResourceChange, TavernResource};

#[derive(Resource)]
pub struct TavernState {
    pub food: i32,
    pub water: i32,
    pub scraps: i32,
    pub rumors: i32,
    pub warmth: i32,
    pub trade_goods: i32,
    pub reputation: i32,
    pub morale: i32,
    pub served: u32,
    pub missed: u32,
    pub threat: f32,
    pub day_time: f32,
    pub notice: String,
    pub notice_timer: Timer,
}

impl Default for TavernState {
    fn default() -> Self {
        Self {
            food: 8,
            water: 7,
            scraps: 4,
            rumors: 1,
            warmth: 2,
            trade_goods: 2,
            reputation: 1,
            morale: 7,
            served: 0,
            missed: 0,
            threat: 12.0,
            day_time: 18.0,
            notice: "The sign is lit. Keep the last warm room alive.".to_string(),
            notice_timer: Timer::from_seconds(6.0, TimerMode::Once),
        }
    }
}

impl TavernState {
    pub fn set_notice(&mut self, notice: impl Into<String>) {
        self.notice = notice.into();
        self.notice_timer = Timer::from_seconds(4.0, TimerMode::Once);
    }

    pub fn resource_value(&self, resource: TavernResource) -> i32 {
        match resource {
            TavernResource::Food => self.food,
            TavernResource::Water => self.water,
            TavernResource::Scraps => self.scraps,
            TavernResource::Reputation => self.reputation,
            TavernResource::Morale => self.morale,
            TavernResource::Threat => self.threat.round() as i32,
            TavernResource::Rumors => self.rumors,
            TavernResource::Warmth => self.warmth,
            TavernResource::TradeGoods => self.trade_goods,
        }
    }

    pub fn can_apply(&self, changes: &[ResourceChange]) -> bool {
        changes.iter().all(|change| {
            change.amount >= 0
                || !change.resource.requires_stock_for_negative()
                || self.resource_value(change.resource) >= -change.amount
        })
    }

    pub fn apply_changes(&mut self, changes: &[ResourceChange]) {
        for change in changes {
            self.apply_change(*change);
        }
    }

    fn apply_change(&mut self, change: ResourceChange) {
        match change.resource {
            TavernResource::Food => self.food = apply_non_negative(self.food, change.amount),
            TavernResource::Water => self.water = apply_non_negative(self.water, change.amount),
            TavernResource::Scraps => self.scraps = apply_non_negative(self.scraps, change.amount),
            TavernResource::Reputation => {
                self.reputation = apply_non_negative(self.reputation, change.amount)
            }
            TavernResource::Morale => self.morale = (self.morale + change.amount).clamp(0, 10),
            TavernResource::Threat => {
                self.threat = (self.threat + change.amount as f32).clamp(0.0, 100.0)
            }
            TavernResource::Rumors => self.rumors = apply_non_negative(self.rumors, change.amount),
            TavernResource::Warmth => self.warmth = apply_non_negative(self.warmth, change.amount),
            TavernResource::TradeGoods => {
                self.trade_goods = apply_non_negative(self.trade_goods, change.amount)
            }
        }
    }
}

fn apply_non_negative(value: i32, amount: i32) -> i32 {
    (value + amount).max(0)
}

pub(super) fn tick_tavern_state(time: Res<Time>, mut tavern_state: ResMut<TavernState>) {
    tavern_state.day_time = (tavern_state.day_time + time.delta_secs() * 0.025) % 24.0;
    tavern_state.threat = (tavern_state.threat + time.delta_secs() * 0.015).min(100.0);
}

pub(super) fn cleanup_notice(time: Res<Time>, mut tavern_state: ResMut<TavernState>) {
    if tavern_state.notice.is_empty() {
        return;
    }

    tavern_state.notice_timer.tick(time.delta());
    if tavern_state.notice_timer.finished() {
        tavern_state.notice.clear();
    }
}
