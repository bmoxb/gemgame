use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::gems::Gem;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoolItem {
    EnergyDrink,
    RunningShoes
}

impl BoolItem {
    pub fn get_price(&self) -> (Gem, u32) {
        match self {
            BoolItem::EnergyDrink => (Gem::Emerald, 10),
            BoolItem::RunningShoes => (Gem::Emerald, 25)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuantitativeItem {
    Bomb
}

impl QuantitativeItem {
    pub fn get_price(&self) -> (Gem, u32) {
        match self {
            QuantitativeItem::Bomb => (Gem::Ruby, 5)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Inventory {
    bool_items: HashMap<BoolItem, bool>,
    quantitive_items: HashMap<QuantitativeItem, u32>
}

impl Inventory {
    pub fn has(&self, itm: BoolItem) -> bool {
        *self.bool_items.get(&itm).unwrap_or(&false)
    }

    pub fn give(&mut self, itm: BoolItem) {
        *self.bool_items.entry(itm).or_insert(true) = true;
    }

    pub fn has_how_many(&self, itm: QuantitativeItem) -> u32 {
        *self.quantitive_items.get(&itm).unwrap_or(&0)
    }

    pub fn give_quantity(&mut self, itm: QuantitativeItem, quantity: u32) {
        *self.quantitive_items.entry(itm).or_insert(0) += quantity;
    }
}
