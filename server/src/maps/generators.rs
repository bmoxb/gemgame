use shared::maps::{Chunk, ChunkCoords, OffsetCoords, Tile, CHUNK_TILE_COUNT};

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
        let mut c = Chunk::new([Tile::Dirt; CHUNK_TILE_COUNT]);
        c.set_tile_at_offset(OffsetCoords { x: 3, y: 3 }, Tile::Rock);
        c
    }
}
