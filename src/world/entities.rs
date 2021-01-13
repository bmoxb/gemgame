use std::fmt;

use serde::{Deserialize, Serialize};

use super::maps::TileCoords;
use crate::Id;

#[derive(Serialize, Deserialize)]
pub struct Entity {
    id: Id,
    pos: TileCoords,
    direction: Direction
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "entity {} at {} facing {}", self.id, self.pos, self.direction)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Up => write!(f, "^"),
            Direction::Down => write!(f, "v"),
            Direction::Left => write!(f, "<"),
            Direction::Right => write!(f, ">")
        }
    }
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
        assert_eq!(Id::new(0x0123456789ABCDEF).to_string(), "0123-4567-89AB-CDEF");
    }
}
