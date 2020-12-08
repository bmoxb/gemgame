use raylib::prelude::*;

const CAMERA_MOVEMENT_SPEED: f32 = 250.0;

/// Handles the drawing of the game world.
pub struct Renderer {
    camera: Camera2D,
    /// The width and height (in pixels) of the sprite/texture of each tile.
    individual_tile_size: i32
}

impl Renderer {
    pub fn new(handle: &RaylibHandle, individual_tile_size: i32) -> Self {
        Renderer {
            camera: Camera2D {
                target: Vector2::new(0.0, 0.0),
                offset: Vector2::new((handle.get_screen_width() / 2) as f32,
                                     (handle.get_screen_height() / 2) as f32),
                rotation: 0.0,
                zoom: 1.0
            },
            individual_tile_size
        }
    }

    pub fn centre_camera_on(&mut self, x: i32, y: i32) {
        self.camera.target.x = x as f32;
        self.camera.target.y = y as f32;
    }

    pub fn arrow_key_camera_movement(&mut self, handle: &mut RaylibHandle, delta: f32) {
        let change = (CAMERA_MOVEMENT_SPEED * delta).round();

        if handle.is_key_down(KeyboardKey::KEY_LEFT) { self.camera.target.x -= change; }
        if handle.is_key_down(KeyboardKey::KEY_RIGHT) { self.camera.target.x += change; }
        if handle.is_key_down(KeyboardKey::KEY_UP) { self.camera.target.y -= change; }
        if handle.is_key_down(KeyboardKey::KEY_DOWN) { self.camera.target.y += change; }
    }

    /// Draws the tiles & entities surrounding the player than are within view
    /// (both in terms of in-game visibility ([`maps::Tile::seen`] property) as
    /// well as whether or not a tile is actually within the camera's viewport).
    pub fn draw(&self, draw: &mut RaylibDrawHandle, tiles_texture: &Texture2D, world: &mut super::World) {
        let mut draw2d = draw.begin_mode2D(self.camera);

        // Tiles:

        // TODO: Decide on the appropriate range of tiles to draw based on camera position.
        for grid_x in -30..30 {
            for grid_y in -30..30 {
                let tile = world.current_map.tile_at(grid_x, grid_y);

                let rec = tile.texture_rec(self.individual_tile_size);
                let pos = Vector2::new((grid_x * self.individual_tile_size) as f32,
                                       (grid_y * self.individual_tile_size) as f32);

                draw2d.draw_texture_rec(tiles_texture, rec, pos, Color::WHITE);

                #[cfg(debug_assertions)]
                draw2d.draw_rectangle_lines(grid_x * self.individual_tile_size, grid_y * self.individual_tile_size,
                                            self.individual_tile_size, self.individual_tile_size,
                                            Color::PINK);
            }
        }

        // Entities:

        // TODO: ...
    }
}