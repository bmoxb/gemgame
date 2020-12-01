use raylib::prelude::*;

use super::maps;

/// Handles the drawing of the game world.
struct Renderer {
    camera: Camera2D
    // zoom, tile sizes, other drawing-specific options/state
}

impl Renderer {
    fn new() -> Self {
        Renderer {
            camera: Camera2D {
                target: Vector2::new(0.0, 0.0), // TODO
                offset: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                zoom: 1.0
            }
        }
    }

    /// Draws the tiles surrounding the player than are within view (both in
    /// terms of in-game visibility ([`maps::Tile::seen`] property) as well as
    /// whether or not a tile is actually within the camera's viewport).
    fn draw(&self, draw: &mut RaylibDrawHandle, world: &mut super::World) {
        let mut draw2d = draw.begin_mode2D(self.camera);
    }
}