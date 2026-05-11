use bevy::prelude::*;

#[derive(Resource)]
pub struct TavernState {
    pub food: i32,
    pub water: i32,
    pub scraps: i32,
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
