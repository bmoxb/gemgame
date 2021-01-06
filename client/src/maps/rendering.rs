use core::maps::{Tile, TileCoords};

use macroquad::prelude as quad;

/// Handles the drawing of a game map.
pub struct Renderer {
    /// The camera context in which the map will be rendered.
    camera: quad::Camera2D,
    /// The width and height (in camera space) that each tile will be draw as.
    tile_draw_size: f32
}

impl Renderer {
    pub fn new(tile_draw_size: f32) -> Self { Renderer { camera: quad::Camera2D::default(), tile_draw_size } }

    /// Draws the tiles & entities than are within the bounds of the camera's viewport.
    pub fn draw(
        &mut self,
        map: &mut super::ClientMap /* , tiles_texture: &quad::Texture2D, entities_texture: &quad::Texture2D */
    ) {
        quad::set_camera(self.camera);

        // Tiles:

        for tile_x in ((self.camera.target.x - 1.0) / self.tile_draw_size).floor() as i32
            ..((self.camera.target.x + 1.0) / self.tile_draw_size).ceil() as i32
        {
            for tile_y in ((self.camera.target.y - 1.0) / self.tile_draw_size).floor() as i32
                ..((self.camera.target.y + 1.0) / self.tile_draw_size).ceil() as i32
            {
                if let Some(tile) = map.tile_at(TileCoords { x: tile_x, y: tile_y }) {
                    let draw_x = tile_x as f32 * self.tile_draw_size;
                    let draw_y = tile_y as f32 * self.tile_draw_size;

                    self.draw_tile(tile, /* tiles_texture, */ draw_x, draw_y);
                    #[cfg(debug_assertions)]
                    self.draw_tile_debug_info(tile, draw_x, draw_y);
                }
            }
        }

        // Entities:

        // ...

        quad::set_default_camera();
    }

    fn draw_tile(&self, tile: &Tile, /* texture: &quad::Texture2D, */ x: f32, y: f32) {
        //quad::draw_texture_rec(...)
        quad::draw_rectangle(x, y, self.tile_draw_size, self.tile_draw_size, quad::RED);
    }

    #[cfg(debug_assertions)]
    fn draw_tile_debug_info(&self, tile: &Tile, x: f32, y: f32) {
        quad::draw_rectangle_lines(
            x,
            y,
            self.tile_draw_size,
            self.tile_draw_size,
            self.tile_draw_size * 0.025,
            quad::GRAY
        );
    }
}
