//! Module containing all code relating to game 'states' (e.g. the main menu
//! state, the settings state, the gameplay state, etc.)

mod game;

use raylib::prelude::*;

pub trait State {
    fn title(&self) -> &'static str;
    fn update<'a>(&mut self, handle: &mut RaylibHandle, delta: f32) -> Option<Box<dyn State>>;
    fn draw(&self, draw: &mut RaylibDrawHandle);
}

pub struct MainMenu {}

impl MainMenu {
    pub fn new() -> Self {
        MainMenu {}
    }
}

impl State for MainMenu {
    fn title(&self) -> &'static str { "Main Menu" }

    fn update(&mut self, handle: &mut RaylibHandle, delta: f32) -> Option<Box<dyn State>> {
        match handle.get_key_pressed() {
            Some(KeyboardKey::KEY_SPACE) => {
                let world_title = "My World".to_string();
                Some(Box::new(game::Game::new(world_title)))
            }

            _ => None
        }
    }

    fn draw(&self, draw: &mut RaylibDrawHandle) {
        let text = "Press Space";
        let width = measure_text(text, 50);

        draw.draw_text("Press Space",
                       (crate::WINDOW_WIDTH - width) / 2,
                       crate::WINDOW_HEIGHT / 2,
                       50, Color::WHITE);
    }
}