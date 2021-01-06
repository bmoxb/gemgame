use macroquad::prelude as quad;
use shared::maps::{Tile, TileCoords};

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
        &mut self, map: &mut super::ClientMap, tiles_texture: &quad::Texture2D, entities_texture: &quad::Texture2D
    ) {
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
                // received from the server. Note that the `super::ClientMap::tile_at` method will automatically queue
                // the required chunk to be fetched as necessary.

                if let Some(tile) = map.tile_at(TileCoords { x: tile_x, y: tile_y }) {
                    //quad::draw_texture_rec(...)
                    quad::draw_rectangle(draw_x, draw_y, self.tile_draw_size, self.tile_draw_size, quad::RED);
                }
                else {
                    let offset = self.tile_draw_size * 0.2;
                    let reduced_size = self.tile_draw_size - offset * 2.0;

                    quad::draw_rectangle(draw_x + offset, draw_y + offset, reduced_size, reduced_size, quad::GRAY);
                }
            }
        }

        // Entities:

        // ...

        // Return to drawing in screen space:
        quad::set_default_camera();
    }
}
