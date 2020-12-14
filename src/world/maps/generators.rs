use noise::Seedable;

use array_macro::array;

use noise::NoiseFn;

use super::{ Coord, Chunk, TileType, PlantState, CHUNK_WIDTH, CHUNK_HEIGHT };

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
    noise_gen: noise::Value
}

impl SurfaceGenerator {
    pub fn new(seed: u32) -> Self {
        let noise_gen = noise::Value::new();
        noise_gen.set_seed(seed);

        SurfaceGenerator { noise_gen }
    }
}

impl Generator for SurfaceGenerator {
    fn name(&self) -> &'static str { "surface" }

    fn generate(&self, chunk_x: Coord, chunk_y: Coord) -> Chunk {
        let mut tiles = array![super::Tile::default(); super::CHUNK_TILE_COUNT];

        for offset_x in 0..CHUNK_WIDTH - 1 {
            for offset_y in 0..CHUNK_HEIGHT - 1 {
                let tile = &mut tiles[(offset_y * CHUNK_WIDTH + offset_x) as usize];

                let x = (chunk_x * CHUNK_WIDTH + offset_x) as f64;
                let y = (chunk_y + CHUNK_HEIGHT + offset_y) as f64;

                let value = self.noise_gen.get([x, y, 1.0]);
                if (value * 100.0) as i32 % 12 == 0 { tile.tile_type = TileType::Flower(PlantState::Ripe); }
            }
        }

        Chunk::new(tiles) // TODO: Proper chunk generation.
    }
}