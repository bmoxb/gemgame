use std::fmt;

use serde::{Deserialize, Serialize};

use super::maps::TileCoords;
use crate::Id;

#[derive(Serialize, Deserialize, Clone)]
pub struct Entity {
    /// The ID used to refer to this entity over the network.
    pub id: Id,
    /// The name of this entity.
    pub name: String,
    /// The position of the entity within its current map.
    pub pos: TileCoords,
    /// Direction that this entity is facing.
    pub direction: Direction
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}' {} at {} facing {}", self.name, self.id, self.pos, self.direction)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Up => write!(f, "🡑"),
            Direction::Down => write!(f, "🡓"),
            Direction::Left => write!(f, "🡐"),
            Direction::Right => write!(f, "🡒")
        }
    }
}

impl Default for Direction {
    fn default() -> Self { Direction::Down }
}
