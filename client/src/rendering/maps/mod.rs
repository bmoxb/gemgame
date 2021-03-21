mod tiles;

use std::collections::HashMap;

use macroquad::prelude as quad;
use shared::{
    maps::{Map, TileCoords},
    Id
};

use crate::{maps::ClientMap, AssetManager, TextureKey};

/// Handles the drawing of a game map.
pub struct Renderer {
    /// The camera context in which the map will be rendered.
    camera: quad::Camera2D,
    /// The width and height (in camera space) that each tile will be draw as.
    tile_draw_size: f32,
    /// The width and height (in pixels) that each individual tile on the tiles texture is.
    tile_texture_rect_size: u16,
    my_entity_movement: Option<EntityMovement>,
    remote_entity_movements: HashMap<Id, EntityMovement>
}

impl Renderer {
    pub fn new(tile_draw_size: f32, tile_texture_rect_size: u16) -> Self {
        Renderer {
            camera: quad::Camera2D::default(),
            tile_draw_size,
            tile_texture_rect_size,
            my_entity_movement: None,
            remote_entity_movements: HashMap::new()
        }
    }

    /// Draws the tiles & entities than are within the bounds of the camera's viewport.
    pub fn draw(&mut self, map: &ClientMap, assets: &AssetManager) {
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
                    tiles::draw(
                        tile,
                        draw_x,
                        draw_y,
                        self.tile_draw_size,
                        self.tile_texture_rect_size,
                        assets.texture(TextureKey::Tiles)
                    );
                }
                else {
                    tiles::draw_pending_tile(draw_x, draw_y, self.tile_draw_size);
                }
            }
        }

        // Entities:

        // TODO: Draw entities.
    }

    /// Begin the animated movement of this client's player entity to the specified position. This method is to be
    /// called by the [`crate::maps::entities::MyEntity::move_towards_checked`] method.
    pub fn my_entity_moved(&mut self, from_pos: TileCoords, to_pos: TileCoords, movement_time: f32) {}

    /// Begin a shorter animation of this client's entity to the specified position. This method is to be called by the
    /// [`crate::maps::entities::MyEntity::received_movement_reconciliation'] method.
    pub fn my_entity_position_corrected(&mut self, incorrect_pos: TileCoords, correct_pos: TileCoords) {}

    /// Begin the animated movement of the specified remote entity to the given position. This method is to be called by
    /// the [`ClientMap::set_remote_entity_position`].
    pub fn remote_entity_moved(&mut self, entity_id: Id, from_pos: TileCoords, to_pos: TileCoords, movement_time: f32) {
    }
}

struct EntityMovement {
    destination_pos: quad::Vec2,
    movement: quad::Vec2,
    current_pos: quad::Vec2,
    current_time: f32,
    movement_time: f32
}

impl EntityMovement {
    fn update(&mut self, delta: f32) {
        self.current_time += delta;
        self.current_pos += self.movement * delta;

        if self.current_time >= self.movement_time {
            self.current_pos = self.destination_pos;
        }
    }
}
