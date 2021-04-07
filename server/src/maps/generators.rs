use noise::Seedable;
use rand::{distributions::Distribution, SeedableRng};
use shared::maps::{Chunk, ChunkCoords, OffsetCoords, Tile, CHUNK_HEIGHT, CHUNK_TILE_COUNT, CHUNK_WIDTH};

pub trait Generator {
    fn new(seed: u32) -> Self
    where Self: Sized;

    fn generate(&self, chunk_coords: ChunkCoords) -> Chunk;

    fn name(&self) -> &'static str;
}

const NOISE_SAMPLE_POINT_MULTIPLIER: f64 = 0.06;

const NOISE_SAMPLE_VALUE_MULTIPLER: f64 = 1.0;

const DIRT_TILE_CHOICES: &[Tile] = &[Tile::Dirt, Tile::Rock, Tile::RockEmerald, Tile::RockRuby, Tile::RockDiamond];
const DIRT_TILE_WEIGHTS: &[usize] = &[600, 15, 10, 5, 1];

const GRASS_TILE_CHOICES: &[Tile] =
    &[Tile::Grass, Tile::FlowerBlue, Tile::FlowersYellowOrange, Tile::FlowerPatch, Tile::Stones, Tile::Shrub];
const GRASS_TILE_WEIGHTS: &[usize] = &[1500, 80, 70, 10, 8, 5];

pub struct DefaultGenerator {
    noise_gen: noise::OpenSimplex
}

impl Generator for DefaultGenerator {
    fn new(seed: u32) -> Self {
        let noise_gen = noise::OpenSimplex::new();
        noise_gen.set_seed(seed);

        DefaultGenerator { noise_gen }
    }

    fn generate(&self, chunk_coords: ChunkCoords) -> Chunk {
        // Create an empty chunk to modify:
        let mut chunk = Chunk::new([Tile::Dirt; CHUNK_TILE_COUNT]);

        // Prepare RNG and distributions:

        let rng_seed = (chunk_coords.x as u64) ^ (chunk_coords.y as u64);
        let mut rng = rand::rngs::StdRng::seed_from_u64(rng_seed);

        let dirt_dist = rand::distributions::WeightedIndex::new(DIRT_TILE_WEIGHTS).unwrap();
        let grass_dist = rand::distributions::WeightedIndex::new(GRASS_TILE_WEIGHTS).unwrap();

        // Go through each position in the chunk:

        for offset_x in 0..CHUNK_WIDTH {
            for offset_y in 0..CHUNK_HEIGHT {
                // Sample the noise function:
                let noise_sample = sample_noise(&self.noise_gen, chunk_coords, offset_x, offset_y);

                let tile = {
                    if noise_sample > 0.3 {
                        // Select a dirt tile type:
                        DIRT_TILE_CHOICES[dirt_dist.sample(&mut rng)]
                    }
                    else if noise_sample < -0.15 {
                        Tile::Water
                    }
                    else {
                        // Select a grass tile type:
                        GRASS_TILE_CHOICES[grass_dist.sample(&mut rng)]
                    }
                };
                chunk.set_tile_at_offset(OffsetCoords { x: offset_x as u8, y: offset_y as u8 }, tile);
            }
        }

        // TODO: Transition tiles between grass, dirt, water...

        chunk
    }

    fn name(&self) -> &'static str {
        "default"
    }
}

fn sample_noise(gen: &dyn noise::NoiseFn<[f64; 2]>, chunk_coords: ChunkCoords, offset_x: i32, offset_y: i32) -> f64 {
    let noise_sample_point = [
        (((chunk_coords.x * CHUNK_WIDTH) + offset_x) as f64) * NOISE_SAMPLE_POINT_MULTIPLIER,
        (((chunk_coords.y * CHUNK_HEIGHT) + offset_y) as f64) * NOISE_SAMPLE_POINT_MULTIPLIER
    ];

    (gen.get(noise_sample_point) * NOISE_SAMPLE_VALUE_MULTIPLER).clamp(-1.0, 1.0)
}
