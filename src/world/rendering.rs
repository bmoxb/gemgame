use raylib::prelude::*;

use super::{ World, maps::Tile, entities::Entity };

use crate::asset_management::Palette;

const CAMERA_MOVEMENT_SPEED: f32 = 250.0;

/// Handles the drawing of the game world.
pub struct Renderer {
    /// The camera context in which the game world will be rendered.
    camera: Camera2D,
    /// The width and height (in pixels) of the sprite/texture of each tile.
    individual_tile_size: i32
}

impl Renderer {
    pub fn new(individual_tile_size: i32) -> Self {
        Renderer {
            camera: Camera2D {
                target: Vector2::new(0.0, 0.0),
                offset: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                zoom: 5.0
            },
            individual_tile_size
        }
    }

    pub fn update_camera_centre(&mut self, handle: &RaylibHandle) {
        self.camera.offset.x = (handle.get_screen_width() / 2) as f32;
        self.camera.offset.y = (handle.get_screen_height() / 2) as f32;
    }

    pub fn centre_camera_on(&mut self, x: i32, y: i32) {
        self.camera.target.x = x as f32;
        self.camera.target.y = y as f32;
    }

    pub fn arrow_key_camera_movement(&mut self, handle: &mut RaylibHandle, delta: f32) {
        let change = CAMERA_MOVEMENT_SPEED * delta;

        if handle.is_key_down(KeyboardKey::KEY_LEFT) { self.camera.target.x -= change; }
        if handle.is_key_down(KeyboardKey::KEY_RIGHT) { self.camera.target.x += change; }
        if handle.is_key_down(KeyboardKey::KEY_UP) { self.camera.target.y -= change; }
        if handle.is_key_down(KeyboardKey::KEY_DOWN) { self.camera.target.y += change; }
    }

    /// Draws the tiles & entities surrounding the player than are within view
    /// (both in terms of in-game visibility ([`maps::Tile::seen`] property) as
    /// well as whether or not a tile is actually within the camera's viewport).
    pub fn draw(&self, draw: &mut RaylibDrawHandle, tiles_texture: &Texture2D, entities_texture: &Texture2D, colours: &Palette, world: &mut World) {
        let half_width = draw.get_screen_width() as f32 / 2.0 / self.camera.zoom;
        let half_height = draw.get_screen_height() as f32 / 2.0 / self.camera.zoom;

        let mut draw2d = draw.begin_mode2D(self.camera);

        // Tiles:

        for grid_x in (self.camera.target.x - half_width).floor() as i32 / self.individual_tile_size - 1..
                      (self.camera.target.x + half_width).ceil() as i32 / self.individual_tile_size + 1 {

            for grid_y in (self.camera.target.y - half_height).floor() as i32 / self.individual_tile_size - 1..
                          (self.camera.target.y + half_height).ceil() as i32 / self.individual_tile_size + 1 {
                let x = grid_x * self.individual_tile_size;
                let y = grid_y * self.individual_tile_size;

                let tile = world.current_map.tile_at(grid_x, grid_y);

                self.draw_tile(tile, tiles_texture, colours, &mut draw2d, x, y);
                #[cfg(debug_assertions)]
                self.draw_tile_debug(tile, colours, &mut draw2d, x, y);
            }
        }

        // Entities:

        for entity in world.current_map.iterate_non_player_entities() {
            self.draw_entity(&entity, entities_texture, colours, &mut draw2d);
        }
    }

    fn draw_tile(&self, tile: &Tile, texture: &Texture2D, colours: &Palette, draw: &mut RaylibMode2D<RaylibDrawHandle>, x: i32, y: i32) {
        let rectangle = tile.texture_rec(self.individual_tile_size);
        let colour = tile.texture_col(colours);
        let pos = Vector2::new(x as f32, y as f32);

        draw.draw_texture_rec(texture, rectangle, pos, colour);
    }

    fn draw_tile_debug(&self, tile: &Tile, colours: &Palette, draw: &mut RaylibMode2D<RaylibDrawHandle>, x: i32, y: i32) {
        draw.draw_line(x - 1, y, x + 1, y, colours.debug);
        draw.draw_line(x, y - 1, x, y + 1, colours.debug);

        if tile.blocking() {
            let centre_x = x + (self.individual_tile_size / 2);
            let centre_y = y + (self.individual_tile_size / 2);

            draw.draw_line(centre_x + 1, centre_y + 1, centre_x - 1, centre_y - 1, colours.debug);
            draw.draw_line(centre_x + 1, centre_y - 1, centre_x - 1, centre_y + 1, colours.debug);
        }
    }

    fn draw_entity(&self, entity: &Entity, texture: &Texture2D, colours: &Palette, draw: &mut RaylibMode2D<RaylibDrawHandle>) {}
}