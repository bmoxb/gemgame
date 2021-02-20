use shared::maps::{Chunk, ChunkCoords, Tile, CHUNK_TILE_COUNT};

pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&self, coords: ChunkCoords) -> Chunk;
}

const DEFAULT_GENERATOR: &str = "default";

pub struct DefaultGenerator;

impl Generator for DefaultGenerator {
    fn name(&self) -> &'static str { DEFAULT_GENERATOR }

    fn generate(&self, _coords: ChunkCoords) -> Chunk { Chunk::new([Tile::default(); CHUNK_TILE_COUNT]) }
}

pub fn by_name(name: &str, _seed: u32) -> Option<Box<dyn Generator + Send>> {
    match name {
        DEFAULT_GENERATOR => Some(Box::new(DefaultGenerator)),
        _ => None
    }
}
