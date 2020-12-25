use macroquad::prelude as quad;

use crate::networking::{ self, ConnectionTrait };
use super::State;

pub struct GameState {
    connection: networking::Connection
}

impl GameState {
    pub fn new(connection: networking::Connection) -> Self {
        GameState { connection }
    }
}

impl State for GameState {
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        quad::draw_text(self.title(), 0.0, 0.0, 32.0, quad::GREEN);
        // ...
        None
    }

    fn title(&self) -> &'static str { "Game" }
}