mod tiles;

use macroquad::prelude as quad;
use shared::maps::{Map, TileCoords};

/// Handles the drawing of a game map.
pub struct Renderer {
    /// The camera context in which the map will be rendered.
    camera: quad::Camera2D,
    /// The width and height (in camera space) that each tile will be draw as.
    tile_draw_size: f32,
    /// The width and height (in pixels) that each individual tile on the tiles texture is.
    tile_texture_rect_size: u16
}

impl Renderer {
    pub fn new(tile_draw_size: f32, tile_texture_rect_size: u16) -> Self {
        Renderer { camera: quad::Camera2D::default(), tile_draw_size, tile_texture_rect_size }
    }

    /// Draws the tiles & entities than are within the bounds of the camera's viewport.
    pub fn draw(
        &mut self, map: &mut super::ClientMap, tiles_texture: quad::Texture2D, _entities_texture: quad::Texture2D
    ) {
        // Adjust camera zoom so that textures don't become distorted when the screen is resized:
        self.camera.zoom = {
            if quad::screen_width() > quad::screen_height() {
                quad::vec2(1.0, quad::screen_width() / quad::screen_height())
            }
            else {
                quad::vec2(quad::screen_height() / quad::screen_width(), 1.0)
            }
        };

        // Begin drawing in camera space:
        quad::set_camera(self.camera);

        // Tiles:

        for tile_x in ((self.camera.target.x - 1.0) / self.tile_draw_size).floor() as i32
            ..((self.camera.target.x + 1.0) / self.tile_draw_size).ceil() as i32
        {
            for tile_y in ((self.camera.target.y - 1.0) / self.tile_draw_size).floor() as i32
                ..((self.camera.target.y + 1.0) / self.tile_draw_size).ceil() as i32
            {
                let draw_x = tile_x as f32 * self.tile_draw_size;
                let draw_y = tile_y as f32 * self.tile_draw_size;

                // If the tile at the specified coordinates is in a chunk that is already loaded then it will be drawn.
                // Otherwise, a grey placeholder rectangle will be drawn in its place until the required chunk is
                // received from the server.

                if let Some(tile) = map.loaded_tile_at(TileCoords { x: tile_x, y: tile_y }) {
                    tiles::draw(tile, draw_x, draw_y, self.tile_draw_size, self.tile_texture_rect_size, tiles_texture);
                }
                else {
                    tiles::draw_pending_tile(draw_x, draw_y, self.tile_draw_size);
                }
            }
        }

        // Entities:

        // ...

        // Return to drawing in screen space:
        quad::set_default_camera();
    }
}
