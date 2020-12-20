use core::maps::{ Coord, Chunk };

pub trait Generator {
    fn generate(&self, chunk_x: Coord, chunk_y: Coord) -> Chunk;
}