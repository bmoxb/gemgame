//! Module containing all code relating to game 'states' (e.g. the main menu
//! state, the settings state, the gameplay state, etc.)

mod game;

use raylib::prelude::*;

pub trait State {
    fn title(&self) -> &'static str;
    fn update<'a>(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, delta: f32) -> Option<Box<dyn State>>;
    fn draw(&mut self, draw: &mut RaylibDrawHandle);
}

pub struct MainMenu {}

impl MainMenu {
    pub fn new() -> Self {
        MainMenu {}
    }
}

impl State for MainMenu {
    fn title(&self) -> &'static str { "Main Menu" }

    fn update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, delta: f32) -> Option<Box<dyn State>> {
        match handle.get_key_pressed() {
            Some(KeyboardKey::KEY_SPACE) => {
                let world_title = "My World".to_string();
                let texture = handle.load_texture(thread, "assets/textures/tiles.png").unwrap(); // TODO: Proper asset manager!
                Some(Box::new(game::Game::new(world_title, handle, texture)))
            }

            _ => None
        }
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle) {
        let text = "Press Space";
        let width = measure_text(text, 50);

        draw.draw_text("Press Space",
                       (crate::WINDOW_WIDTH - width) / 2,
                       crate::WINDOW_HEIGHT / 2,
                       50, Color::WHITE);
    }
}