use macroquad::prelude as quad;
use shared::maps::{entities::Entity, TileCoords};

use super::tile_coords_to_vec2;

const ANIMATION_FRAMES_PER_MOVEMENT: isize = 4;

pub struct Renderer {
    pub current_pos: quad::Vec2,
    destination_pos: quad::Vec2,
    movement: quad::Vec2,
    current_time: f32,
    movement_time: f32
}

impl Renderer {
    pub fn new(from_coords: TileCoords, to_coords: TileCoords, movement_time: f32, tile_draw_size: f32) -> Self {
        let start_pos = tile_coords_to_vec2(from_coords, tile_draw_size);
        let destination_pos = tile_coords_to_vec2(to_coords, tile_draw_size);

        Renderer {
            current_pos: start_pos,
            destination_pos,
            movement: (destination_pos - start_pos) / movement_time,
            current_time: 0.0,
            movement_time
        }
    }

    pub fn with_static_position(coords: TileCoords, tile_draw_size: f32) -> Self {
        Renderer::new(coords, coords, 0.0, tile_draw_size)
    }

    /// Update draw position and animations.
    pub fn update(&mut self, delta: f32) {
        self.current_time += delta;
        self.current_pos += self.movement * delta;

        if self.current_time >= self.movement_time {
            self.current_pos = self.destination_pos;
        }
    }

    /// Draw the lower portion of the entity (the body).
    pub fn draw_lower(&self, _entity: &Entity, _entities_texture: quad::Texture2D, tile_draw_size: f32) {
        // TODO
        quad::draw_rectangle(self.current_pos.x, self.current_pos.y, tile_draw_size, tile_draw_size, quad::RED);
    }

    /// Draw the upper portion of the entity (head, face, hands, etc.)
    pub fn draw_upper(&self, _entity: &Entity, _entities_texture: quad::Texture2D, tile_draw_size: f32) {
        // TODO
    }
}
