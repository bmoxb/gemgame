use shared::maps::{Chunk, ChunkCoords, Tile, CHUNK_TILE_COUNT};

pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&self, coords: ChunkCoords) -> Chunk;
}

pub struct DefaultGenerator;

impl Generator for DefaultGenerator {
    fn name(&self) -> &'static str {
        "default"
    }

    fn generate(&self, _coords: ChunkCoords) -> Chunk {
        Chunk::new([Tile::default(); CHUNK_TILE_COUNT])
    }
}
