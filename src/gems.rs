/// Represents the types of gems which may be collected by players.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gem {
    Emerald,
    Ruby,
    Diamond
}

/// Represents the potential yield of gems produced from the smashing of a rock tile.
pub struct GemYield {
    pub gem: Gem,
    pub minimum_quantity: usize,
    pub maximum_quantity: usize
}
