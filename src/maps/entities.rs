use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use super::TileCoords;
use crate::Id;

/// Type alias for a hash map of entity IDs to entities.
pub type Entities = HashMap<Id, Entity>;

const STANDARD_MOVEMENT_TIME: f32 = 0.11;
const RUNNING_MOVEMENT_TIME: f32 = STANDARD_MOVEMENT_TIME * 0.75;

/// An 'entity' in the context of the GemGame codebase refers specifically to the player characters that exist within
/// the game world.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    /// The position of the entity within its current map.
    pub pos: TileCoords,
    /// Direction that this entity is facing/travelling towards.
    pub direction: Direction,
    /// Emotional expression of this human entity (angry, shocked, etc.) Defaults to a neutral expression.
    pub facial_expression: FacialExpression,
    /// Style of this human entity's hair. Defaults to a quiff.
    pub hair_style: HairStyle,
    pub clothing_colour: ClothingColour,
    pub skin_colour: SkinColour,
    pub hair_colour: HairColour,
    /// Whether or not this human entity has increased movement speed.
    pub has_running_shoes: bool
}

impl Entity {
    /// The amount of time in seconds taken for the entity to move to an adjacent tile.
    pub fn movement_time(&self) -> f32 {
        if self.has_running_shoes {
            RUNNING_MOVEMENT_TIME
        }
        else {
            STANDARD_MOVEMENT_TIME
        }
    }

    /// Modify entity position without performing any sort of checks.
    pub fn move_towards_unchecked(&mut self, direction: Direction) {
        let new_pos = direction.apply(self.pos);
        self.pos = new_pos;
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "entity at {} facing {} with facial expression {} with hair style {} {} running shoes",
            self.pos,
            self.direction,
            self.facial_expression,
            self.hair_style,
            if self.has_running_shoes { "with" } else { "without" }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    pub fn apply(&self, mut coords: TileCoords) -> TileCoords {
        match self {
            Direction::Down => coords.y -= 1,
            Direction::Up => coords.y += 1,
            Direction::Left => coords.x -= 1,
            Direction::Right => coords.x += 1
        }
        coords
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Direction::Up => write!(f, "🡑 up"),
            Direction::Down => write!(f, "🡓 down"),
            Direction::Left => write!(f, "🡐 left"),
            Direction::Right => write!(f, "🡒 right")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum HairStyle {
    Quiff,
    Mohawk,
    Fringe
}

impl fmt::Display for HairStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HairStyle::Quiff => write!(f, "short quiff"),
            HairStyle::Mohawk => write!(f, "edgy mohawk"),
            HairStyle::Fringe => write!(f, "simple fringe")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum FacialExpression {
    /// Neutral 😐 facial expression.
    Neutral,
    /// Angry 😠 facial expression (both eyebrows slanted inward).
    Angry,
    /// Shocked/surprised 😲 facial expression (both eyebrows slanted outward, mouth opened wide).
    Shocked,
    /// Skeptical/suspicious 🤨 facial expression (single eyebrow slanted outward).
    Skeptical
}

impl fmt::Display for FacialExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FacialExpression::Neutral => write!(f, "😐 neutral"),
            FacialExpression::Angry => write!(f, "😠 angry"),
            FacialExpression::Shocked => write!(f, "😲 shocked/surprised"),
            FacialExpression::Skeptical => write!(f, "🤨 skeptical/suspicious")
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ClothingColour {
    Grey,
    White,
    Red,
    Blue,
    Green
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SkinColour {
    Black,
    Brown,
    Pale,
    White
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum HairColour {
    Black,
    Brown,
    Blonde,
    White,
    Blue,
    Red
}
