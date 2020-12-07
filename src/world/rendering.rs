use raylib::prelude::*;

/// Handles the drawing of the game world.
pub struct Renderer {
    camera: Camera2D,
    /// The loaded texture containing the sprites for each type of tile.
    //tiles_texture: Texture2D,
    /// The width and height (in pixels) of the sprite/texture of each tile.
    individual_tile_size: i32
}

impl Renderer {
    pub fn new(/*tiles_texture: Texture2D, */individual_tile_size: i32) -> Self {
        Renderer {
            camera: Camera2D {
                target: Vector2::new(0.0, 0.0), // TODO
                offset: Vector2::new(0.0, 0.0),
                rotation: 0.0,
                zoom: 1.0
            },
            //tiles_texture,
            individual_tile_size
        }
    }

    pub fn centre_camera_on(&mut self, x: i32, y: i32) {
        self.camera.target.x = x as f32;
        self.camera.target.y = y as f32;
    }

    /// Draws the tiles & entities surrounding the player than are within view
    /// (both in terms of in-game visibility ([`maps::Tile::seen`] property) as
    /// well as whether or not a tile is actually within the camera's viewport).
    pub fn draw(&self, draw: &mut RaylibDrawHandle, world: &mut super::World) {
        let mut draw2d = draw.begin_mode2D(self.camera);

        // Tiles:

        // TODO: Decide on the appropriate range of tiles to draw based on camera position.
        for grid_x in 0..15 {
            for grid_y in 0..15 {
                let tile = world.current_map.tile_at(grid_x, grid_y);

                let texture_rec = Rectangle::EMPTY;

                /*draw2d.draw_texture_rec(&self.tiles_texture, texture_rec, position,
                                      Color::WHITE);*/

                #[cfg(debug_assertions)]
                draw2d.draw_rectangle_lines(grid_x * self.individual_tile_size, grid_y * self.individual_tile_size,
                                          self.individual_tile_size, self.individual_tile_size,
                                          Color::PINK);
            }
        }

        // Entities:
    }
}