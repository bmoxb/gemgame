use core::maps::{ ChunkCoords, Chunk };

pub trait Generator {
    fn generate(&self, coords: ChunkCoords) -> Chunk;
}

pub fn by_name(name: &str) -> Option<Box<dyn Generator + Send>> {
    match name {
        _ => None
    }
}