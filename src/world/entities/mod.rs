//! Module for all entities in the game world. Also manages the controlling of
//! entities (i.e. AI) as well as the abilities certain entities can have (e.g.
//! the ability to attack other entities, to carry items, etc.)

use std::rc::Rc;
use super::maps::Map;

pub struct Entity {
    /// Controller for this entity.
    controller: Rc<dyn Controller>,
    /// Horizontal position in tiles on current map.
    x: usize,
    /// Vertical position in tiles on current map.
    y: usize
    // optional abilities, etc.
}

impl Entity {
    /// Have an entity perform its actions for this turn by calling upon its
    /// controller (more specifically, by calling the [`Controller::my_turn`]
    /// method).
    fn your_turn(&mut self, map: &Map) {
        let control = self.controller.clone(); // Clone the reference, not the
                                               // controller itself.
        control.my_turn(self, map)
    }
}

trait Controller {
    fn my_turn(&self, entity: &mut Entity, map: &Map);
}

struct PlayerController {}
//impl Controller for PlayerController {}