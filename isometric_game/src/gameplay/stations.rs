use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component)]
pub struct Station {
    pub kind: StationKind,
}

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq)]
pub enum StationKind {
    Pantry,
    WaterStill,
    Stove,
    Radio,
}

impl StationKind {
    pub fn label(self) -> &'static str {
        match self {
            StationKind::Pantry => "pantry",
            StationKind::WaterStill => "water still",
            StationKind::Stove => "stove",
            StationKind::Radio => "radio",
        }
    }
}
