use crate::gameplay::{ResourceChange, TavernResource};

const MEAL_CHANGES: [ResourceChange; 4] = [
    ResourceChange::new(TavernResource::Food, -1),
    ResourceChange::new(TavernResource::Scraps, 2),
    ResourceChange::new(TavernResource::Reputation, 1),
    ResourceChange::new(TavernResource::Morale, 1),
];
const DRINK_CHANGES: [ResourceChange; 3] = [
    ResourceChange::new(TavernResource::Water, -1),
    ResourceChange::new(TavernResource::Scraps, 1),
    ResourceChange::new(TavernResource::Reputation, 1),
];
const BOTH_CHANGES: [ResourceChange; 5] = [
    ResourceChange::new(TavernResource::Food, -1),
    ResourceChange::new(TavernResource::Water, -1),
    ResourceChange::new(TavernResource::Scraps, 3),
    ResourceChange::new(TavernResource::Reputation, 1),
    ResourceChange::new(TavernResource::Morale, 1),
];
const SHELTER_CHANGES: [ResourceChange; 5] = [
    ResourceChange::new(TavernResource::Warmth, -1),
    ResourceChange::new(TavernResource::Water, -1),
    ResourceChange::new(TavernResource::Scraps, 2),
    ResourceChange::new(TavernResource::Reputation, 1),
    ResourceChange::new(TavernResource::Morale, 1),
];
const RUMOR_CHANGES: [ResourceChange; 4] = [
    ResourceChange::new(TavernResource::Rumors, -1),
    ResourceChange::new(TavernResource::Scraps, 1),
    ResourceChange::new(TavernResource::Reputation, 2),
    ResourceChange::new(TavernResource::Threat, -1),
];
const TRADE_CHANGES: [ResourceChange; 3] = [
    ResourceChange::new(TavernResource::TradeGoods, -1),
    ResourceChange::new(TavernResource::Scraps, 4),
    ResourceChange::new(TavernResource::Reputation, 1),
];

#[derive(Clone, Copy)]
pub enum Order {
    Meal,
    Drink,
    Both,
    Shelter,
    Rumor,
    Trade,
}

impl Order {
    pub fn label(self) -> &'static str {
        match self {
            Order::Meal => "meal",
            Order::Drink => "drink",
            Order::Both => "meal and drink",
            Order::Shelter => "shelter",
            Order::Rumor => "road rumor",
            Order::Trade => "trade parcel",
        }
    }

    pub fn station_hint(self) -> &'static str {
        match self {
            Order::Meal | Order::Both => "pantry or stove",
            Order::Drink => "water still",
            Order::Shelter => "stove",
            Order::Rumor => "radio",
            Order::Trade => "pantry",
        }
    }

    pub fn changes(self) -> &'static [ResourceChange] {
        match self {
            Order::Meal => &MEAL_CHANGES,
            Order::Drink => &DRINK_CHANGES,
            Order::Both => &BOTH_CHANGES,
            Order::Shelter => &SHELTER_CHANGES,
            Order::Rumor => &RUMOR_CHANGES,
            Order::Trade => &TRADE_CHANGES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn orders_cover_multiple_station_needs() {
        let hints: HashSet<&'static str> = [
            Order::Meal,
            Order::Drink,
            Order::Both,
            Order::Shelter,
            Order::Rumor,
            Order::Trade,
        ]
        .into_iter()
        .map(Order::station_hint)
        .collect();

        assert!(hints.len() >= 3);
    }
}
