use shared::maps::{Chunk, ChunkCoords, Tile, CHUNK_TILE_COUNT};

pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&self, coords: ChunkCoords) -> Chunk;
}

pub struct DefaultGenerator;

impl DefaultGenerator {
    const NAME: &'static str = "default";
}

impl Generator for DefaultGenerator {
    fn name(&self) -> &'static str { DefaultGenerator::NAME }

    fn generate(&self, _: ChunkCoords) -> Chunk { Chunk::new([Tile::default(); CHUNK_TILE_COUNT]) }
}

pub fn by_name(name: &str) -> Option<Box<dyn Generator + Send>> {
    match name {
        DefaultGenerator::NAME => Some(Box::new(DefaultGenerator)),
        _ => None
    }
}
