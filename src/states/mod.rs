//! Module containing all code relating to game 'states' (e.g. the main menu
//! state, the settings state, the gameplay state, etc.)

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
        //println!("{:.5}", delta);

        match handle.get_key_pressed() {
            Some(_) => {
                if let Some(w) = crate::world::World::load("My World".to_string()) {
                    w.save();
                }
                None
            }
            _ => None
        }
    }

    fn draw(&self, draw: &mut RaylibDrawHandle) {
        draw.draw_circle(200, 200, 50.0, Color::PINK);
    }
}

struct Game {
    // game world, entities, etc.
}

impl Game {
    fn new() -> Self {
        Game {}
    }
}

impl State for Game {
    fn title(&self) -> &'static str { "Game" }

    fn update(&mut self, handle: &mut RaylibHandle, delta: f32) -> Option<Box<dyn State>> { None }

    fn draw(&self, draw: &mut RaylibDrawHandle) {}
}