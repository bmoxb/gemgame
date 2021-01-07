use std::fmt;

use serde::{Deserialize, Serialize};

use super::maps::TileCoords;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    id: Id,
    pos: TileCoords,
    direction: Direction //inventory: Box<dyn Inventory>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Id {
    value: u64
}

impl Id {
    pub fn new(value: u64) -> Self { Id { value } }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:4X}-{:4X}-{:4X}-{:4X}",
            self.value >> 48,
            (self.value >> 32) & 0xFFFF,
            (self.value >> 16) & 0xFFFF,
            self.value & 0xFFFF
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Default for Direction {
    fn default() -> Self { Direction::Down }
}

#[cfg(test)]
mod tests {
    use super::Id;

    #[test]
    fn test_id_display() {
        assert_eq!(Id::new(0).to_string(), "0000-0000-0000-0000");
    }
}