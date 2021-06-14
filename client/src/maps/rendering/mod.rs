mod animations;
mod bombs;
mod entities;
mod tiles;

use std::collections::HashMap;

use animations::Animation;
use macroquad::prelude as quad;
use shared::{
    maps::{entities::Entity, ChunkCoords, Map, OffsetCoords, TileCoords},
    Id
};

use crate::{maps::ClientMap, AssetManager, TextureKey};

/// The width and height (in camera space) that each tile will be draw as.
const TILE_DRAW_SIZE: f32 = 0.1;

/// The width and height (in pixels) that each individual tile on the tiles texture is.
const SINGLE_TILE_TEXTURE_SIZE: u16 = 16;

/// The time taken for the movement to complete when an entity's position is corrected.
const ENTITY_POSITION_CORRECTED_MOVEMENT_TIME: f32 = 0.025;

/// Handles the drawing of a game map.
#[derive(Default)]
pub struct MapRenderer {
    /// The camera context in which the map will be rendered.
    camera: quad::Camera2D,
    /// The entity renderer for this client's player entity.
    my_entity_renderer: entities::Renderer,
    /// Entity renderers for remote player entities (mapped to by entity IDs).
    remote_entity_renderers: HashMap<Id, entities::Renderer>,
    /// Stores animations for transitions between tile types.
    tile_change_animations: HashMap<TileCoords, animations::Once>,
    /// Stores pairs of bomb explosion animations and lists of positions where animations should play.
    exploding_bomb_animations: Vec<(animations::Once, Vec<TileCoords>)>
}

impl MapRenderer {
    pub fn new(my_entity_pos: TileCoords) -> Self {
        MapRenderer { my_entity_renderer: entities::Renderer::new(my_entity_pos), ..Default::default() }
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
            self.my_entity_renderer.current_pos + quad::vec2(TILE_DRAW_SIZE / 2.0, TILE_DRAW_SIZE / 2.0);

        // Begin drawing in camera space:
        quad::set_camera(self.camera);

        // Establish the area of the map that is actually on-screen:

        let on_screen_tiles_left_boundary = ((self.camera.target.x - 1.0) / TILE_DRAW_SIZE).floor() as i32;
        let on_screen_tiles_right_boundary = ((self.camera.target.x + 1.0) / TILE_DRAW_SIZE).ceil() as i32;
        let on_screen_tiles_bottom_boundary = ((self.camera.target.y - 1.0) / TILE_DRAW_SIZE).floor() as i32;
        let on_screen_tiles_top_boundary = ((self.camera.target.y + 1.0) / TILE_DRAW_SIZE).ceil() as i32;

        // Draw tiles:

        let mut tile_coords;
        let mut draw_pos;

        for tile_x in on_screen_tiles_left_boundary..on_screen_tiles_right_boundary {
            for tile_y in on_screen_tiles_bottom_boundary..on_screen_tiles_top_boundary {
                tile_coords = TileCoords { x: tile_x, y: tile_y };
                draw_pos = tile_coords_to_vec2(tile_coords, TILE_DRAW_SIZE);

                // If the tile at the specified coordinates is in a chunk that is already loaded then it will be drawn.
                // Otherwise, a grey placeholder rectangle will be drawn in its place until the required chunk is
                // received from the server.

                if let Some(tile) = map.loaded_tile_at(tile_coords) {
                    let chunk_corner = tile_coords.as_chunk_offset_coords() == OffsetCoords { x: 0, y: 0 };

                    tiles::draw_with_stateless_animation(
                        tile,
                        draw_pos,
                        TILE_DRAW_SIZE,
                        assets.texture(TextureKey::Tiles),
                        chunk_corner
                    );
                }
                else {
                    tiles::draw_pending(draw_pos, TILE_DRAW_SIZE);
                }
            }
        }

        // Identify the coordinates of chunks that are on-screen:

        let mut on_screen_chunk_coords = Vec::new();

        let bottom_left_on_screen_chunk_coords =
            TileCoords { x: on_screen_tiles_left_boundary, y: on_screen_tiles_bottom_boundary }.as_chunk_coords();
        let top_right_on_screen_chunk_coords =
            TileCoords { x: on_screen_tiles_right_boundary, y: on_screen_tiles_top_boundary }.as_chunk_coords();

        for x in bottom_left_on_screen_chunk_coords.x..top_right_on_screen_chunk_coords.x + 1 {
            for y in bottom_left_on_screen_chunk_coords.y..top_right_on_screen_chunk_coords.y + 1 {
                on_screen_chunk_coords.push(ChunkCoords { x, y });
            }
        }

        // Draw undetonated bombs:

        for chunk in on_screen_chunk_coords.into_iter().filter_map(|coords| map.loaded_chunk_at(coords)) {
            // Iterate all bomb positions within the chunk irrespective of who placed them:
            for bomb_coords in chunk.get_undetonated_bomb_positions() {
                let draw_pos = tile_coords_to_vec2(*bomb_coords, TILE_DRAW_SIZE);
                bombs::draw_undetonated_bomb(draw_pos, TILE_DRAW_SIZE, assets.texture(TextureKey::Bombs));
            }
        }

        // Draw (and remove completed) tile transition animations:

        self.tile_change_animations.retain(|coords, animation| {
            // TODO: Draw only if on-screen, like entities below.
            let draw_pos = tile_coords_to_vec2(*coords, TILE_DRAW_SIZE);
            animation.draw(draw_pos, SINGLE_TILE_TEXTURE_SIZE, TILE_DRAW_SIZE, assets.texture(TextureKey::Tiles));

            !animation.has_concluded()
        });

        // Update remote entities:

        for renderer in self.remote_entity_renderers.values_mut() {
            renderer.update(delta);
        }

        // Draw entities:

        let remote_entities_to_draw = self.remote_entity_renderers.iter().filter_map(|(id, renderer)| {
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
        });

        // Draw lower portion of each on-screen entity:

        let my_entity_iter = std::iter::once((my_entity_contained, &self.my_entity_renderer));
        let all_entities_iter = remote_entities_to_draw.chain(my_entity_iter);

        for (entity, renderer) in all_entities_iter.clone() {
            renderer.draw_lower(entity, assets.texture(TextureKey::Entities), TILE_DRAW_SIZE);
        }

        // Draw upper portion of each on-screen entity:

        for (entity, renderer) in all_entities_iter {
            renderer.draw_upper(entity, assets.texture(TextureKey::Entities), TILE_DRAW_SIZE);
        }

        // Draw exploding bombs:

        self.exploding_bomb_animations.retain(|(animation, positions)| {
            for pos in positions {
                let mut coords = tile_coords_to_vec2(*pos, TILE_DRAW_SIZE);

                // Offset the drawing position of the exploding bomb based on how much larger an exploding bomb is
                // versus a regular tile:
                let offset = TILE_DRAW_SIZE * (bombs::DETONATING_BOMB_FRAME_SIZE_MULTIPLIER / 2) as f32;
                coords.x -= offset;
                coords.y -= offset;

                animation.draw(
                    coords,
                    SINGLE_TILE_TEXTURE_SIZE * bombs::DETONATING_BOMB_FRAME_SIZE_MULTIPLIER,
                    TILE_DRAW_SIZE * bombs::DETONATING_BOMB_FRAME_SIZE_MULTIPLIER as f32,
                    assets.texture(TextureKey::Bombs)
                );
            }

            !animation.has_concluded()
        });
    }

    /// Begin the animated movement of this client's player entity to the specified position. This method is to be
    /// called by the [`crate::maps::entities::MyEntity::move_towards_checked`] method.
    pub fn my_entity_moved(&mut self, to_coords: TileCoords, movement_time: f32, frame_changes: usize) {
        self.my_entity_renderer.do_movement(to_coords, movement_time, frame_changes, TILE_DRAW_SIZE);
    }

    /// Begin a shorter animation of this client's entity to the specified position. This method is to be called by the
    /// [`crate::maps::entities::MyEntity::received_movement_reconciliation'] method.
    pub fn my_entity_position_corrected(&mut self, correct_coords: TileCoords) {
        self.my_entity_renderer.do_movement(correct_coords, ENTITY_POSITION_CORRECTED_MOVEMENT_TIME, 1, TILE_DRAW_SIZE);
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
            TILE_DRAW_SIZE
        );
    }

    pub fn add_remote_entity(&mut self, entity_id: Id, coords: TileCoords) {
        self.remote_entity_renderers.insert(entity_id, entities::Renderer::new(coords));
    }

    pub fn remove_remote_entity(&mut self, entity_id: Id) {
        self.remote_entity_renderers.remove(&entity_id);
    }

    /// Has a smashing animation play at the specified coordinates. This method is to be called when a rock tile is
    // turned into a smashed rock by the [`ClientMap::some_entity_moved_to`] method.
    pub fn rock_tile_smashed(&mut self, coords: TileCoords) {
        self.tile_change_animations.insert(coords, tiles::new_rock_smash_animation());
    }

    pub fn bombs_detonated(&mut self, positions: Vec<TileCoords>) {
        self.exploding_bomb_animations.push((bombs::make_detonating_bomb_animation(), positions));
    }
}

fn tile_coords_to_vec2(coords: TileCoords, tile_draw_size: f32) -> quad::Vec2 {
    quad::vec2(coords.x as f32 * tile_draw_size, coords.y as f32 * tile_draw_size)
}
