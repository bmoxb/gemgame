//! Module for all entities in the game world. Also manages the controlling of
//! entities (i.e. AI) as well as the abilities certain entities can have (e.g.
//! the ability to attack other entities, to carry items, etc.)

struct Entity {
    controller: Box<dyn Controller>,
    // optional abilities, etc.
}

trait Controller {}

struct PlayerController {}
impl Controller for PlayerController {}