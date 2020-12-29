use core::maps::{ ChunkCoords, Chunk, Tile, CHUNK_TILE_COUNT };

pub trait Generator {
    fn generate(&self, coords: ChunkCoords) -> Chunk;
}

pub struct DefaultGenerator;

impl Generator for DefaultGenerator {
    fn generate(&self, _: ChunkCoords) -> Chunk {
        Chunk::new([Tile::default(); CHUNK_TILE_COUNT])
    }
}

pub fn by_name(name: &str) -> Option<Box<dyn Generator + Send>> {
    match name {
        "default" => Some(Box::new(DefaultGenerator)),
        _ => None
    }
}