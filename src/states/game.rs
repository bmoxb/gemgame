use raylib::prelude::*;

use super::State;

use crate::{
    AssetManager, TextureKey,
    world::{ World, rendering::Renderer }
};

pub struct Game {
    game_world: World,
    world_renderer: Renderer
    // game world, entities, etc.
}

impl Game {
    pub fn new(world_title: String) -> Self {
        Game {
            game_world: World::load(world_title).unwrap(),
            world_renderer: Renderer::new(16)
        }
    }
}

impl State for Game {
    fn title(&self) -> &'static str { "Game" }

    fn begin(&mut self, assets: &mut AssetManager, handle: &mut RaylibHandle, thread: &RaylibThread) {
        assets.require_texture(TextureKey::Tiles, handle, thread);

        self.game_world.save(); // TODO: temp
    }

    fn update(&mut self, handle: &mut RaylibHandle, delta: f32) -> Option<Box<dyn State>> {
        self.world_renderer.update_camera_centre(handle);

        //let player = self.game_world.get_player_entity();
        //self.world_renderer.centre_camera_on(...);

        self.world_renderer.arrow_key_camera_movement(handle, delta);

        None
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle, assets: &AssetManager) {
        //self.world_renderer.draw(draw, assets.texture(&TextureKey::Tiles), &mut self.game_world);
    }
}