use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::gems::Gem;

pub trait Item {
    fn get_price(&self) -> (Gem, u32);
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoolItem {
    RunningShoes
}

impl Item for BoolItem {
    fn get_price(&self) -> (Gem, u32) {
        match self {
            BoolItem::RunningShoes => (Gem::Emerald, 20)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuantitativeItem {
    Bomb
}

impl Item for QuantitativeItem {
    fn get_price(&self) -> (Gem, u32) {
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

    //pub fn take(&mut self, itm: BoolItem) { ... }

    pub fn take_quantity(&mut self, itm: QuantitativeItem, quantity: u32) {
        *self.quantitive_items.entry(itm).or_insert(0) -= quantity;
    }
}
