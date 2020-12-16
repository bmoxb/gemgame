//! Module for all entities in the game world. Also manages the controlling of
//! entities (i.e. AI) as well as the abilities certain entities can have (e.g.
//! the ability to attack other entities, to carry items, etc.)

use std::rc::Rc;

use raylib::prelude::*;

use super::{ Coord, maps::Map };

pub struct Entity {
    /// Controller for this entity.
    controller: Rc<dyn Controller>,
    /// Horizontal position in tiles on current map.
    x: Coord,
    /// Vertical position in tiles on current map.
    y: Coord,
    /// The width of this entity in tiles (usually `1`).
    width: u8,
    /// The height of this entity in tiles (usually `1`).
    height: u8
    // optional abilities, etc.
}

impl Entity {
    pub fn new(controller: Rc<dyn Controller>, x: Coord, y: Coord) -> Self {
        Entity { controller, x, y, width: 1, height: 1 }
    }

    /// Have an entity perform its actions for this turn by calling upon its
    /// controller (more specifically, by calling the [`Controller::my_turn`]
    /// method). Will return `true` when the entity has completed its turn.
    pub fn your_turn(&mut self, map: &mut Map, handle: &RaylibHandle) -> bool {
        let control = self.controller.clone(); // Clone the reference, not the
                                               // controller itself.
        control.my_turn(self, map, handle)
    }
}

pub trait Controller {
    fn my_turn(&self, entity: &mut Entity, map: &mut Map, handle: &RaylibHandle) -> bool;
}

pub struct PlayerController;

impl Controller for PlayerController {
    fn my_turn(&self, entity: &mut Entity, map: &mut Map, handle: &RaylibHandle) -> bool {
        if handle.is_key_pressed(KeyboardKey::KEY_W) { entity.y -= 1; true }
        else { false }
    }
}