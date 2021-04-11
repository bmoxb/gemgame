use std::collections::HashMap;

use shared::maps::{Chunk, OffsetCoords, Tile, CHUNK_HEIGHT, CHUNK_TILE_COUNT, CHUNK_WIDTH};

#[derive(Default)]
pub struct ChunkPlan {
    tile_categories: HashMap<(i32, i32), TileCategory>
}

impl ChunkPlan {
    pub fn set_category_at(&mut self, offset_x: i32, offset_y: i32, category: TileCategory) {
        self.tile_categories.insert((offset_x, offset_y), category);
    }

    pub fn remove_juttting_and_unconnected_tiles(&mut self) {
        // TODO
    }

    pub fn to_chunk(
        &self, dirt_transitions: &TransitionTiles, water_transitions: &TransitionTiles,
        mut place_non_transition_tile: impl FnMut(TileCategory) -> Tile
    ) -> Chunk {
        let mut chunk = Chunk::default();

        for offset_x in 0..CHUNK_WIDTH {
            for offset_y in 0..CHUNK_HEIGHT {
                let category = self.get_category_at(offset_x, offset_y);

                let tile = self
                    .maybe_transition_tile(offset_x, offset_y, dirt_transitions, water_transitions)
                    .unwrap_or_else(|| place_non_transition_tile(category));

                let coords = OffsetCoords { x: offset_x as u8, y: offset_y as u8 };
                chunk.set_tile_at_offset(coords, tile);
            }
        }

        chunk
    }

    fn maybe_transition_tile(
        &self, offset_x: i32, offset_y: i32, dirt_transitions: &TransitionTiles, water_transitions: &TransitionTiles
    ) -> Option<Tile> {
        None // TODO
    }

    fn get_category_at(&self, offset_x: i32, offset_y: i32) -> TileCategory {
        *self.tile_categories.get(&(offset_x, offset_y)).unwrap_or(&TileCategory::default())
    }
}

#[derive(Clone, Copy)]
pub enum TileCategory {
    Grass,
    Dirt,
    Water
}

impl Default for TileCategory {
    fn default() -> Self {
        TileCategory::Grass
    }
}

pub struct TransitionTiles {
    pub top: Tile,
    pub bottom: Tile,
    pub left: Tile,
    pub right: Tile,
    pub top_left: Tile,
    pub top_right: Tile,
    pub bottom_left: Tile,
    pub bottom_right: Tile,
    pub corner_top_left: Tile,
    pub corner_top_right: Tile,
    pub corner_bottom_left: Tile,
    pub corner_bottom_right: Tile
}

#[cfg(test)]
mod tests {
    #[test]
    fn remove_jutting_tiles() {
        // ..##      ..##
        // .###  ->  ..##
        // ..##      ..##

        // ...##      ...##
        // .####  ->  ...##
        // ...##      ...##

        // ..##
        // .###  ->  no
        // .###  ->  change
        // ..##
    }

    #[test]
    fn remove_unconnected_tiles() {
        // Not allowed:

        // ...
        // .#.
        // ...

        // ....
        // .##.
        // ....

        // ....
        // .##.
        // .#..
        // ....

        // Allowed:

        // ....
        // .##.
        // .##.
        // ....
    }
}
