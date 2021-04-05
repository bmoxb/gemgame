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
        let mut c = Chunk::new([Tile::Grass; CHUNK_TILE_COUNT]);
        c.set_tile_at_offset(OffsetCoords { x: 5, y: 5 }, Tile::FlowerBlue);
        c.set_tile_at_offset(OffsetCoords { x: 8, y: 8 }, Tile::FlowersYellowOrange);
        for x in 10..15 {
            c.set_tile_at_offset(OffsetCoords { x, y: 3 }, Tile::Water);
        }
        c
    }
}
