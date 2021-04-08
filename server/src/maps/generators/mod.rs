mod noisegen;

use noise::Seedable;
use noisegen::ChunkNoise;
use rand::{distributions::Distribution, rngs::StdRng, SeedableRng};
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
    noise_func: noise::OpenSimplex,
    dirt_dist: rand::distributions::WeightedIndex<usize>,
    grass_dist: rand::distributions::WeightedIndex<usize>
}

impl Generator for DefaultGenerator {
    fn new(seed: u32) -> Self {
        let noise_func = noise::OpenSimplex::new();
        noise_func.set_seed(seed);

        DefaultGenerator {
            noise_func,
            dirt_dist: rand::distributions::WeightedIndex::new(DIRT_TILE_WEIGHTS).unwrap(),
            grass_dist: rand::distributions::WeightedIndex::new(GRASS_TILE_WEIGHTS).unwrap()
        }
    }

    fn generate(&self, chunk_coords: ChunkCoords) -> Chunk {
        // Create an empty chunk to modify:
        let mut chunk = Chunk::new([Tile::Dirt; CHUNK_TILE_COUNT]);

        // Prepare RNG, noise, distributions:

        let chunk_noise =
            ChunkNoise::new(self.noise_func, chunk_coords, NOISE_SAMPLE_POINT_MULTIPLIER, NOISE_SAMPLE_VALUE_MULTIPLER);

        let rng_seed = (chunk_coords.x as u64) ^ (chunk_coords.y as u64);
        let mut rng = StdRng::seed_from_u64(rng_seed);

        // Go through each position in the chunk:

        for offset_x in 0..CHUNK_WIDTH {
            for offset_y in 0..CHUNK_HEIGHT {
                let tile = {
                    let noise_sample = chunk_noise.sample(offset_x, offset_y);

                    if Self::should_be_dirt(noise_sample) {
                        DIRT_TILE_CHOICES[self.dirt_dist.sample(&mut rng)]
                    }
                    else if Self::should_be_water(noise_sample) {
                        Tile::Water
                    }
                    else {
                        self.place_grass_transition_tiles_around(offset_x, offset_y, &mut chunk, &chunk_noise);

                        GRASS_TILE_CHOICES[self.grass_dist.sample(&mut rng)]
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

impl DefaultGenerator {
    fn place_grass_transition_tiles_around(
        &self, offset_x: i32, offset_y: i32, chunk: &mut Chunk, chunk_noise: &ChunkNoise
    ) {
        //unimplemented!()
    }

    fn should_be_dirt(noise_sample: f64) -> bool {
        noise_sample >= 0.3
    }

    fn should_be_water(noise_sample: f64) -> bool {
        noise_sample <= -0.15
    }
}
