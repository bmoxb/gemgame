use raylib::prelude::*;

use super::State;

use crate::{
    TextureKey, asset_management::AssetManager,
    world::{ World, rendering::Renderer }
};

pub struct Game {
    game_world: World,
    world_renderer: Renderer
    // game world, entities, etc.
}

impl Game {
    pub fn new(world_title: String, handle: &RaylibHandle) -> Self {
        Game {
            game_world: World::load(world_title).unwrap(),
            world_renderer: Renderer::new(handle, 32)
        }
    }
}

impl State for Game {
    fn title(&self) -> &'static str { "Game" }

    fn begin(&mut self, assets: &mut AssetManager<TextureKey>, handle: &mut RaylibHandle, thread: &RaylibThread) {
        assets.require_texture(TextureKey::Tiles, handle, thread)
    }

    fn update(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread, delta: f32) -> Option<Box<dyn State>> {
        //let player = self.game_world.get_player_entity();
        //self.world_renderer.centre_camera_on(...);

        self.world_renderer.arrow_key_camera_movement(handle, delta);

        None
    }

    fn draw(&mut self, draw: &mut RaylibDrawHandle, assets: &AssetManager<TextureKey>) {
        self.world_renderer.draw(draw, assets.texture(&TextureKey::Tiles), &mut self.game_world);
    }
}