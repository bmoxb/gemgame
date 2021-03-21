mod tiles;

use std::collections::HashMap;

use macroquad::prelude as quad;
use shared::{
    maps::{entities::Entity, Map, TileCoords},
    Id
};

use crate::{maps::ClientMap, AssetManager, TextureKey};

const POSITION_CORRECTED_MOVEMENT_TIME: f32 = 0.025;

/// Handles the drawing of a game map.
pub struct Renderer {
    /// The camera context in which the map will be rendered.
    camera: quad::Camera2D,
    /// The width and height (in camera space) that each tile will be draw as.
    tile_draw_size: f32,
    /// The width and height (in pixels) that each individual tile on the tiles texture is.
    tile_texture_rect_size: u16,
    my_entity_rendering: EntityRendering,
    remote_entity_rendering: HashMap<Id, EntityRendering>
}

impl Renderer {
    pub fn new(tile_draw_size: f32, tile_texture_rect_size: u16, my_entity_pos: TileCoords) -> Self {
        Renderer {
            camera: quad::Camera2D::default(),
            tile_draw_size,
            tile_texture_rect_size,
            my_entity_rendering: EntityRendering::with_static_position(my_entity_pos, tile_draw_size),
            remote_entity_rendering: HashMap::new()
        }
    }

    /// Draws the tiles & entities than are within the bounds of the camera's viewport.
    pub fn draw(&mut self, map: &ClientMap, my_entity_contained: &Entity, assets: &AssetManager, delta: f32) {
        // Adjust camera zoom so that textures don't become distorted when the screen is resized:

        self.camera.zoom = {
            if quad::screen_width() > quad::screen_height() {
                quad::vec2(1.0, quad::screen_width() / quad::screen_height())
            }
            else {
                quad::vec2(quad::screen_height() / quad::screen_width(), 1.0)
            }
        };

        // TODO: Camera follow player movement.

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

        for (entity_id, entity_rendering) in &mut self.remote_entity_rendering {
            if let Some(entity) = map.entity_by_id(*entity_id) {
                entity_rendering.update_and_draw(
                    delta,
                    entity,
                    assets.texture(TextureKey::Entities),
                    self.tile_draw_size
                );
            }
        }

        self.my_entity_rendering.update_and_draw(
            delta,
            my_entity_contained,
            assets.texture(TextureKey::Entities),
            self.tile_draw_size
        );
    }

    /// Begin the animated movement of this client's player entity to the specified position. This method is to be
    /// called by the [`crate::maps::entities::MyEntity::move_towards_checked`] method.
    pub fn my_entity_moved(&mut self, from_coords: TileCoords, to_coords: TileCoords, movement_time: f32) {
        self.my_entity_rendering = EntityRendering::new(from_coords, to_coords, movement_time, self.tile_draw_size);
    }

    /// Begin a shorter animation of this client's entity to the specified position. This method is to be called by the
    /// [`crate::maps::entities::MyEntity::received_movement_reconciliation'] method.
    pub fn my_entity_position_corrected(&mut self, incorrect_coords: TileCoords, correct_coords: TileCoords) {
        self.my_entity_rendering = EntityRendering::new(
            incorrect_coords,
            correct_coords,
            POSITION_CORRECTED_MOVEMENT_TIME,
            self.tile_draw_size
        );
    }

    /// Begin the animated movement of the specified remote entity to the given position. This method is to be called by
    /// the [`ClientMap::set_remote_entity_position`].
    pub fn remote_entity_moved(
        &mut self, entity_id: Id, from_coords: TileCoords, to_coords: TileCoords, movement_time: f32
    ) {
        self.remote_entity_rendering
            .insert(entity_id, EntityRendering::new(from_coords, to_coords, movement_time, self.tile_draw_size));
    }

    pub fn add_remove_entity(&mut self, entity_id: Id, coords: TileCoords) {
        self.remote_entity_rendering
            .insert(entity_id, EntityRendering::with_static_position(coords, self.tile_draw_size));
    }

    pub fn remove_remote_entity(&mut self, entity_id: Id) {
        self.remote_entity_rendering.remove(&entity_id);
    }
}

struct EntityRendering {
    current_pos: quad::Vec2,
    destination_pos: quad::Vec2,
    movement: quad::Vec2,
    current_time: f32,
    movement_time: f32
}

impl EntityRendering {
    fn new(from_coords: TileCoords, to_coords: TileCoords, movement_time: f32, tile_draw_size: f32) -> Self {
        let start_pos = tile_coords_to_vec2(from_coords, tile_draw_size);
        let destination_pos = tile_coords_to_vec2(to_coords, tile_draw_size);

        EntityRendering {
            current_pos: start_pos,
            destination_pos,
            movement: (destination_pos - start_pos) / movement_time,
            current_time: 0.0,
            movement_time
        }
    }

    fn with_static_position(coords: TileCoords, tile_draw_size: f32) -> Self {
        EntityRendering::new(coords, coords, 0.0, tile_draw_size)
    }

    fn update_and_draw(
        &mut self, delta: f32, _entity: &Entity, _entities_texture: quad::Texture2D, tile_draw_size: f32
    ) {
        // Update draw position:

        self.current_time += delta;
        self.current_pos += self.movement * delta;

        if self.current_time >= self.movement_time {
            self.current_pos = self.destination_pos;
        }

        // TODO: Draw entity.
        quad::draw_rectangle(self.current_pos.x, self.current_pos.y, tile_draw_size, tile_draw_size, quad::RED);
    }
}

fn tile_coords_to_vec2(coords: TileCoords, tile_draw_size: f32) -> quad::Vec2 {
    quad::vec2(coords.x as f32 * tile_draw_size, coords.y as f32 * tile_draw_size)
}
