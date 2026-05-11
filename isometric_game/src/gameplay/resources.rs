use serde::Deserialize;

use crate::gameplay::TavernState;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq)]
pub enum TavernResource {
    Food,
    Water,
    Scraps,
    Reputation,
    Morale,
    Threat,
    Rumors,
    Warmth,
    TradeGoods,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ResourceChange {
    pub resource: TavernResource,
    pub amount: i32,
}

impl ResourceChange {
    pub const fn new(resource: TavernResource, amount: i32) -> Self {
        Self { resource, amount }
    }
}

pub fn missing_requirements_label(state: &TavernState, changes: &[ResourceChange]) -> String {
    let missing: Vec<&'static str> = changes
        .iter()
        .filter(|change| {
            change.amount < 0
                && change.resource.requires_stock_for_negative()
                && state.resource_value(change.resource) < -change.amount
        })
        .map(|change| change.resource.label())
        .collect();

    if missing.is_empty() {
        "required supplies".to_string()
    } else {
        missing.join(", ")
    }
}

impl TavernResource {
    pub fn label(self) -> &'static str {
        match self {
            TavernResource::Food => "food",
            TavernResource::Water => "water",
            TavernResource::Scraps => "scraps",
            TavernResource::Reputation => "reputation",
            TavernResource::Morale => "morale",
            TavernResource::Threat => "threat",
            TavernResource::Rumors => "rumors",
            TavernResource::Warmth => "warmth",
            TavernResource::TradeGoods => "trade goods",
        }
    }

    pub fn requires_stock_for_negative(self) -> bool {
        matches!(
            self,
            TavernResource::Food
                | TavernResource::Water
                | TavernResource::Scraps
                | TavernResource::Rumors
                | TavernResource::Warmth
                | TavernResource::TradeGoods
        )
    }
}
