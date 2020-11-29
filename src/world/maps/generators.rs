use super::{ Coord, Chunk };

fn generator_by_name(name: &str, seed: u32) -> Option<Box<dyn Generator>> {
    match name {
        "overworld" => Some(Box::new(OverworldGenerator::new(seed))),
        _ => None
    }
}

pub trait Generator {
    fn name(&self) -> &'static str;
    fn generate(&self, chunk_x: Coord, chunk_y: Coord) -> Chunk;
}

pub struct OverworldGenerator {
    seed: u32
}

impl OverworldGenerator {
    pub fn new(seed: u32) -> Self { OverworldGenerator { seed } }
}

impl Generator for OverworldGenerator {
    fn name(&self) -> &'static str { "overworld" }

    fn generate(&self, chunk_x: Coord, chunk_y: Coord) -> Chunk {
        unimplemented!()
    }
}