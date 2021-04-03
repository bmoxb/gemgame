use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

/// Represents the types of gems which may be collected by players.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gem {
    Emerald,
    Ruby,
    Diamond
}

/// Represents the potential yield of gems produced from the smashing of a rock tile.
pub struct GemYield {
    pub gem: Gem,
    pub minimum_quantity: u32,
    pub maximum_quantity: u32
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GemCollection {
    collection: HashMap<Gem, u32>
}

impl GemCollection {
    pub fn get_quantity(&self, gem: Gem) -> u32 {
        *self.collection.get(&gem).unwrap_or(&0)
    }

    pub fn increase_quantity(&mut self, gem: Gem, increase: u32) {
        *self.collection.entry(gem).or_default() += increase;
    }
}

impl fmt::Display for GemCollection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?} x {}, {:?} x {}, {:?} x {})",
            Gem::Emerald,
            self.get_quantity(Gem::Emerald),
            Gem::Ruby,
            self.get_quantity(Gem::Ruby),
            Gem::Diamond,
            self.get_quantity(Gem::Diamond)
        )
    }
}
