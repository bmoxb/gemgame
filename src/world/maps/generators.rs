use noise::Seedable;

use array_macro::array;

use super::{ Coord, Chunk };

pub fn by_name(name: &str, seed: u32) -> Option<Box<dyn Generator>> {
    match name {
        "surface" => Some(Box::new(SurfaceGenerator::new(seed))),
        _ => None
    }
}

pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&self, chunk_x: Coord, chunk_y: Coord) -> Chunk;
}

pub struct SurfaceGenerator {
    noise_gen: noise::Perlin
}

impl SurfaceGenerator {
    pub fn new(seed: u32) -> Self {
        let noise_gen = noise::Perlin::new();
        noise_gen.set_seed(seed);

        SurfaceGenerator { noise_gen }
    }
}

impl Generator for SurfaceGenerator {
    fn name(&self) -> &'static str { "surface" }

    fn generate(&self, chunk_x: Coord, chunk_y: Coord) -> Chunk {
        let tiles = array![super::Tile::default(); super::CHUNK_TILE_COUNT];
        Chunk::new(tiles) // TODO: Proper chunk generation.
    }
}