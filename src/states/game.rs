use super::State;

use crate::world::World;

use raylib::prelude::*;

pub struct Game {
    world: World
    // game world, entities, etc.
}

impl Game {
    pub fn new(world_title: String) -> Self {
        Game { world: World::load(world_title).unwrap() }
    }
}

impl State for Game {
    fn title(&self) -> &'static str { "Game" }

    fn update(&mut self, handle: &mut RaylibHandle, delta: f32) -> Option<Box<dyn State>> { None }

    fn draw(&self, draw: &mut RaylibDrawHandle) {}
}