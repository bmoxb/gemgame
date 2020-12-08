//! Module containing all code relating to game 'states' (e.g. the main menu
//! state, the settings state, the gameplay state, etc.)

mod game;

use crate::{ TextureKey, asset_management::AssetManager };

use raylib::prelude::*;

pub trait State {
    fn title(&self) -> &'static str;

    fn begin(&mut self, assets: &mut AssetManager<TextureKey>, handle: &mut RaylibHandle, thread: &RaylibThread) {}
    fn update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, delta: f32) -> Option<Box<dyn State>>;
    fn draw(&mut self, draw: &mut RaylibDrawHandle, assets: &AssetManager<TextureKey>);
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
                Some(Box::new(game::Game::new(world_title, handle)))
            }

            _ => None
        }
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle, assets: &AssetManager<TextureKey>) {
        let text = "Press Space";
        let width = measure_text(text, 50);

        draw.draw_text("Press Space",
                       (draw.get_screen_width() - width) / 2,
                       draw.get_screen_height() / 2,
                       50, Color::WHITE);
    }
}