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
