//! Module containing all code relating to game 'states' (e.g. the main menu state, the settings state, the gameplay
//! state, etc.)

pub mod game;
pub mod pregame;

use crate::TextureKey;

pub trait State {
    fn required_textures(&self) -> &[TextureKey] { &[] }
    fn title(&self) -> &'static str;
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>>;
}
