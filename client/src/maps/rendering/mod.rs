mod entities;
mod tiles;

use std::collections::{HashMap, HashSet};

use macroquad::prelude as quad;
use shared::{
    maps::{entities::Entity, Map, OffsetCoords, TileCoords},
    Id
};

use self::tiles::animations::Animation;
use crate::{maps::ClientMap, AssetManager, TextureKey};

const ENTITY_POSITION_CORRECTED_MOVEMENT_TIME: f32 = 0.025;

/// Handles the drawing of a game map.
#[derive(Default)]
pub struct MapRenderer {
    /// The camera context in which the map will be rendered.
    camera: quad::Camera2D,
    /// The width and height (in camera space) that each tile will be draw as.
    tile_draw_size: f32,
    /// The width and height (in pixels) that each individual tile on the tiles texture is.
    single_tile_texture_size: u16,
    /// The entity renderer for this client's player entity.
    my_entity_renderer: entities::Renderer,
    /// Entity renderers for remote player entities (mapped to by entity IDs).
    remote_entity_renderers: HashMap<Id, entities::Renderer>,
    tile_change_animations: HashMap<TileCoords, tiles::animations::Once>
}

impl MapRenderer {
    pub fn new(tile_draw_size: f32, single_tile_texture_size: u16, my_entity_pos: TileCoords) -> Self {
        MapRenderer {
            tile_draw_size,
            single_tile_texture_size,
            my_entity_renderer: entities::Renderer::new(my_entity_pos, tile_draw_size),
            ..Default::default()
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

        // Update this client's entity and centre camera around it:

        self.my_entity_renderer.update(delta);
        self.camera.target =
            self.my_entity_renderer.current_pos + quad::vec2(self.tile_draw_size / 2.0, self.tile_draw_size / 2.0);

        // Begin drawing in camera space:
        quad::set_camera(self.camera);

        // Establish the area of the map that is actually on-screen:

        let on_screen_tiles_left_boundary = ((self.camera.target.x - 1.0) / self.tile_draw_size).floor() as i32;
        let on_screen_tiles_right_boundary = ((self.camera.target.x + 1.0) / self.tile_draw_size).ceil() as i32;
        let on_screen_tiles_bottom_boundary = ((self.camera.target.y - 1.0) / self.tile_draw_size).floor() as i32;
        let on_screen_tiles_top_boundary = ((self.camera.target.y + 1.0) / self.tile_draw_size).ceil() as i32;

        // Draw tiles:

        let mut tile_coords;
        let mut draw_pos;

        for tile_x in on_screen_tiles_left_boundary..on_screen_tiles_right_boundary {
            for tile_y in on_screen_tiles_bottom_boundary..on_screen_tiles_top_boundary {
                tile_coords = TileCoords { x: tile_x, y: tile_y };
                draw_pos = tile_coords_to_vec2(tile_coords, self.tile_draw_size);

                // If the tile at the specified coordinates is in a chunk that is already loaded then it will be drawn.
                // Otherwise, a grey placeholder rectangle will be drawn in its place until the required chunk is
                // received from the server.

                if let Some(tile) = map.loaded_tile_at(tile_coords) {
                    let chunk_corner = tile_coords.as_chunk_offset_coords() == OffsetCoords { x: 0, y: 0 };
                    tiles::draw_with_stateless_animation(
                        tile,
                        draw_pos,
                        self.tile_draw_size,
                        self.single_tile_texture_size,
                        assets.texture(TextureKey::Tiles),
                        chunk_corner
                    );
                }
                else {
                    tiles::draw_pending(draw_pos, self.tile_draw_size);
                }
            }
        }

        // Draw (and remove completed) tile transition animations:

        let mut concluded_animations = HashSet::new();

        for (coords, animation) in &self.tile_change_animations {
            let draw_pos = tile_coords_to_vec2(*coords, self.tile_draw_size);

            // TODO: Draw only if on-screen, like entities below.
            animation.draw(
                draw_pos,
                self.tile_draw_size,
                self.single_tile_texture_size,
                assets.texture(TextureKey::Tiles)
            );

            if animation.has_concluded() {
                concluded_animations.insert(*coords);
            }
        }

        self.tile_change_animations.retain(|key, _| !concluded_animations.contains(key));

        // Update remote entities:

        for renderer in self.remote_entity_renderers.values_mut() {
            renderer.update(delta);
        }

        // Draw entities:

        let remote_entities_to_draw: Vec<(&Entity, &entities::Renderer)> = self
            .remote_entity_renderers
            .iter()
            .filter_map(|(id, renderer)| {
                if let Some(entity) = map.entity_by_id(*id) {
                    // Is the entity actually on screen?
                    if on_screen_tiles_left_boundary <= entity.pos.x
                        && entity.pos.x <= on_screen_tiles_right_boundary
                        && on_screen_tiles_bottom_boundary <= entity.pos.y
                        && entity.pos.y <= on_screen_tiles_top_boundary
                    {
                        return Some((entity, renderer));
                    }
                }
                None
            })
            .collect();

        // Draw lower portion of each on-screen entity:

        let my_entity_iter = std::iter::once((my_entity_contained, &self.my_entity_renderer));
        let all_entities_iter = remote_entities_to_draw.into_iter().chain(my_entity_iter);

        for (entity, renderer) in all_entities_iter.clone() {
            renderer.draw_lower(
                entity,
                assets.texture(TextureKey::Entities),
                self.tile_draw_size,
                self.single_tile_texture_size
            );
        }

        // Draw upper portion of each on-screen entity:

        for (entity, renderer) in all_entities_iter {
            renderer.draw_upper(
                entity,
                assets.texture(TextureKey::Entities),
                self.tile_draw_size,
                self.single_tile_texture_size
            );
        }
    }

    /// Begin the animated movement of this client's player entity to the specified position. This method is to be
    /// called by the [`crate::maps::entities::MyEntity::move_towards_checked`] method.
    pub fn my_entity_moved(&mut self, to_coords: TileCoords, movement_time: f32, frame_changes: usize) {
        self.my_entity_renderer.do_movement(to_coords, movement_time, frame_changes, self.tile_draw_size);
    }

    /// Begin a shorter animation of this client's entity to the specified position. This method is to be called by the
    /// [`crate::maps::entities::MyEntity::received_movement_reconciliation'] method.
    pub fn my_entity_position_corrected(&mut self, correct_coords: TileCoords) {
        self.my_entity_renderer.do_movement(
            correct_coords,
            ENTITY_POSITION_CORRECTED_MOVEMENT_TIME,
            1,
            self.tile_draw_size
        );
    }

    /// Begin the animated movement of the specified remote entity to the given position. This method is to be called by
    /// the [`ClientMap::set_remote_entity_position`] method.
    pub fn remote_entity_moved(
        &mut self, entity_id: Id, to_coords: TileCoords, movement_time: f32, frame_changes: usize
    ) {
        self.remote_entity_renderers.entry(entity_id).or_default().do_movement(
            to_coords,
            movement_time,
            frame_changes,
            self.tile_draw_size
        );
    }

    pub fn add_remote_entity(&mut self, entity_id: Id, coords: TileCoords) {
        self.remote_entity_renderers.insert(entity_id, entities::Renderer::new(coords, self.tile_draw_size));
    }

    pub fn remove_remote_entity(&mut self, entity_id: Id) {
        self.remote_entity_renderers.remove(&entity_id);
    }

    /// Has a smashing animation play at the specified coordinates. This method is to be called when a rock tile is
    // turned into a smashed rock by the [`ClientMap::some_entity_moved_to`] method.
    pub fn rock_tile_smashed(&mut self, coords: TileCoords) {
        self.tile_change_animations.insert(coords, tiles::new_rock_smash_animation());
    }
}

fn tile_coords_to_vec2(coords: TileCoords, tile_draw_size: f32) -> quad::Vec2 {
    quad::vec2(coords.x as f32 * tile_draw_size, coords.y as f32 * tile_draw_size)
}
