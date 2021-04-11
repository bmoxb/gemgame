use std::collections::HashMap;

use shared::maps::{Chunk, OffsetCoords, Tile, CHUNK_HEIGHT, CHUNK_WIDTH};

#[derive(Default)]
pub struct ChunkPlan {
    tile_categories: HashMap<(i32, i32), TileCategory>
}

impl ChunkPlan {
    pub fn set_category_at(&mut self, offset_x: i32, offset_y: i32, category: TileCategory) {
        self.tile_categories.insert((offset_x, offset_y), category);
    }

    pub fn remove_juttting_and_unconnected_tiles(&mut self) {
        unimplemented!() // TODO
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
        let my_category = self.get_category_at(offset_x, offset_y);

        let my_transition_tiles = {
            match my_category {
                TileCategory::Water => water_transitions,
                TileCategory::Dirt => dirt_transitions,
                TileCategory::Grass => return None
            }
        };

        let above = self.get_category_at(offset_x, offset_y + 1) != my_category;
        let below = self.get_category_at(offset_x, offset_y - 1) != my_category;
        let left = self.get_category_at(offset_x - 1, offset_y) != my_category;
        let right = self.get_category_at(offset_x + 1, offset_y) != my_category;

        let transition_tile = match (above, below, left, right) {
            // Straight transition tiles:
            (true, _, false, false) => Some(my_transition_tiles.top),
            (_, true, false, false) => Some(my_transition_tiles.bottom),
            (false, false, true, _) => Some(my_transition_tiles.left),
            (false, false, _, true) => Some(my_transition_tiles.right),
            // Right-angle transition tiles:
            (true, _, true, false) => Some(my_transition_tiles.top_left),
            (true, _, false, true) => Some(my_transition_tiles.top_right),
            (_, true, true, false) => Some(my_transition_tiles.bottom_left),
            (_, true, false, true) => Some(my_transition_tiles.bottom_right),

            _ => None
        };

        transition_tile.or_else(|| {
            let top_left = self.get_category_at(offset_x - 1, offset_y + 1) != my_category;
            let top_right = self.get_category_at(offset_x + 1, offset_y + 1) != my_category;
            let bottom_left = self.get_category_at(offset_x - 1, offset_y - 1) != my_category;
            let bottom_right = self.get_category_at(offset_x + 1, offset_y - 1) != my_category;

            match (top_left, top_right, bottom_left, bottom_right) {
                // Corner tile transitions:
                (true, false, false, _) => Some(my_transition_tiles.corner_top_left),
                (false, true, _, false) => Some(my_transition_tiles.corner_top_right),
                (false, _, true, false) => Some(my_transition_tiles.corner_bottom_left),
                (_, false, false, true) => Some(my_transition_tiles.corner_bottom_right),

                _ => None
            }
        })
    }

    fn get_category_at(&self, offset_x: i32, offset_y: i32) -> TileCategory {
        *self.tile_categories.get(&(offset_x, offset_y)).unwrap_or(&TileCategory::default())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
