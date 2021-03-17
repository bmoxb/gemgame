use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};

use super::TileCoords;
use crate::Id;

/// Type alias for a hash map of entity IDs to entities.
pub type Entities = HashMap<Id, Entity>;

const STANDARD_HUMAN_MOVEMENT_TIME: f32 = 0.1;
const RUNNING_HUMAN_MOVEMENT_TIME: f32 = STANDARD_HUMAN_MOVEMENT_TIME * 0.75;
const BOMB_MOVEMENT_TIME: f32 = 0.025;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entity {
    /// The position of the entity within its current map.
    pub pos: TileCoords,
    /// Direction that this entity is facing/travelling towards.
    pub direction: Direction,
    /// The 'variety' of this entity (e.g. human, monster, etc.)
    pub variety: Variety
}

impl Entity {
    /// The amount of time in seconds taken for the entity to move to an adjacent tile.
    pub fn movement_time(&self) -> f32 {
        match self.variety {
            Variety::Human { has_running_shoes, .. } => {
                if has_running_shoes {
                    RUNNING_HUMAN_MOVEMENT_TIME
                }
                else {
                    STANDARD_HUMAN_MOVEMENT_TIME
                }
            }
            Variety::Bomb { .. } => BOMB_MOVEMENT_TIME
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
        write!(f, "entity at {} facing/going {} of variety {}", self.pos, self.direction, self.variety)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Variety {
    Human {
        /// The name of this human entity.
        name: String,
        /// Emotional expression of this human entity (angry, shocked, etc.) Defaults to a neutral expression.
        facial_expression: FacialExpression,
        /// Style of this human entity's hair. Defaults to a quiff.
        hair_style: HairStyle,
        /// Whether or not this human entity has increased movement speed.
        has_running_shoes: bool
    },
    Bomb {
        /// Where this bomb entity is travelling to.
        target_pos: TileCoords,
        /// ID of the entity that threw this bomb.
        thrown_by: Id
    }
}

impl Variety {
    pub fn new_human() -> Self {
        Variety::Human {
            name: "abc".to_string(), // TODO: Generate human names.
            facial_expression: FacialExpression::default(),
            hair_style: HairStyle::default(),
            has_running_shoes: false
        }
    }
}

impl fmt::Display for Variety {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Variety::Human { name, facial_expression, hair_style, has_running_shoes } => {
                write!(
                    f,
                    "human called '{}' with hair style {} with {} facial expression {} running shoes",
                    name,
                    hair_style,
                    facial_expression,
                    if *has_running_shoes { "with" } else { "without" }
                )
            }
            Variety::Bomb { target_pos, thrown_by } => {
                write!(f, "bomb targeting {} thrown by entity wth ID {}", target_pos, thrown_by)
            }
        }
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

impl Default for Direction {
    fn default() -> Self {
        Direction::Down
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

impl Default for HairStyle {
    fn default() -> Self {
        HairStyle::Quiff
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

impl Default for FacialExpression {
    fn default() -> Self {
        FacialExpression::Neutral
    }
}
