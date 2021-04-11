mod chunknoise;
mod chunkplan;
pub mod default;

use chunkplan::TransitionTiles;
pub use default::DefaultGenerator;
use shared::maps::{Chunk, ChunkCoords, Tile};

pub trait Generator {
    fn new(seed: u32) -> Self
    where Self: Sized;

    fn generate(&self, chunk_coords: ChunkCoords) -> Chunk;

    fn name(&self) -> &'static str;
}

const DIRT_GRASS_TRANSITION_TILES: TransitionTiles = TransitionTiles {
    top: Tile::DirtGrassTop,
    bottom: Tile::DirtGrassBottom,
    left: Tile::DirtGrassLeft,
    right: Tile::DirtGrassRight,
    top_left: Tile::DirtGrassTopLeft,
    top_right: Tile::DirtGrassTopRight,
    bottom_left: Tile::DirtGrassBottomLeft,
    bottom_right: Tile::DirtGrassBottomRight,
    corner_top_left: Tile::DirtGrassCornerTopLeft,
    corner_top_right: Tile::DirtGrassCornerTopRight,
    corner_bottom_left: Tile::DirtGrassCornerBottomLeft,
    corner_bottom_right: Tile::DirtGrassCornerBottomRight
};

const WATER_GRASS_TRANSITION_TILES: TransitionTiles = TransitionTiles {
    top: Tile::WaterGrassTop,
    bottom: Tile::WaterGrassBottom,
    left: Tile::WaterGrassLeft,
    right: Tile::WaterGrassRight,
    top_left: Tile::WaterGrassTopLeft,
    top_right: Tile::WaterGrassTopRight,
    bottom_left: Tile::WaterGrassBottomLeft,
    bottom_right: Tile::WaterGrassBottomRight,
    corner_top_left: Tile::WaterGrassCornerTopLeft,
    corner_top_right: Tile::WaterGrassCornerTopRight,
    corner_bottom_left: Tile::WaterGrassCornerBottomLeft,
    corner_bottom_right: Tile::WaterGrassCornerBottomRight
};
/*
fn maybe_transition_tile(
    offset_x: i32, offset_y: i32, chunk_noise: &ChunkNoise, should_be_other_tile_at: fn(i32, i32, &ChunkNoise) -> bool,
    jut_replacement_tile: Tile, transition_tiles: &TransitionTiles
) -> Option<Tile> {
    let above = should_be_other_tile_at(offset_x, offset_y + 1, chunk_noise);
    let below = should_be_other_tile_at(offset_x, offset_y - 1, chunk_noise);
    let left = should_be_other_tile_at(offset_x - 1, offset_y, chunk_noise);
    let right = should_be_other_tile_at(offset_x + 1, offset_y, chunk_noise);

    let transition_tile = match (above, below, left, right) {
        // Straight transition tiles:
        (true, _, false, false) => Some(transition_tiles.top),
        (_, true, false, false) => Some(transition_tiles.bottom),
        (false, false, true, _) => Some(transition_tiles.left),
        (false, false, _, true) => Some(transition_tiles.right),
        // Right-angle transition tiles:
        (true, _, true, false) => Some(transition_tiles.top_left),
        (true, _, false, true) => Some(transition_tiles.top_right),
        (_, true, true, false) => Some(transition_tiles.bottom_left),
        (_, true, false, true) => Some(transition_tiles.bottom_right),

        _ => None
    };

    transition_tile.or_else(|| {
        let top_left = should_be_other_tile_at(offset_x - 1, offset_y + 1, chunk_noise);
        let top_right = should_be_other_tile_at(offset_x + 1, offset_y + 1, chunk_noise);
        let bottom_left = should_be_other_tile_at(offset_x - 1, offset_y - 1, chunk_noise);
        let bottom_right = should_be_other_tile_at(offset_x + 1, offset_y - 1, chunk_noise);

        match (top_left, top_right, bottom_left, bottom_right) {
            // Corner tile transitions:
            (true, false, false, _) => Some(transition_tiles.corner_top_left),
            (false, true, _, false) => Some(transition_tiles.corner_top_right),
            (false, _, true, false) => Some(transition_tiles.corner_bottom_left),
            (_, false, false, true) => Some(transition_tiles.corner_bottom_right),

            _ => None
        }
    })
}
*/
