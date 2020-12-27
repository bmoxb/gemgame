use core::maps::{ ChunkCoords, Chunk };

pub trait Generator {
    fn generate(&self, coords: &ChunkCoords) -> Chunk;
}