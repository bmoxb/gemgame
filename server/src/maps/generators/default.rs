use noise::Seedable;
use rand::{distributions::Distribution, rngs::StdRng, SeedableRng};
use shared::maps::{Chunk, ChunkCoords, Tile, CHUNK_HEIGHT, CHUNK_WIDTH};

use super::{
    chunknoise::ChunkNoise,
    chunkplan::{ChunkPlan, TileCategory}
};

const DIRT_TILE_CHOICES: &[Tile] = &[Tile::Dirt, Tile::Rock, Tile::RockEmerald, Tile::RockRuby, Tile::RockDiamond];
const DIRT_TILE_WEIGHTS: &[usize] = &[600, 15, 10, 5, 1];

const GRASS_TILE_CHOICES: &[Tile] =
    &[Tile::Grass, Tile::FlowerBlue, Tile::FlowersYellowOrange, Tile::FlowerPatch, Tile::Stones, Tile::Shrub];
const GRASS_TILE_WEIGHTS: &[usize] = &[1500, 80, 70, 10, 8, 5];

const NOISE_SAMPLE_POINT_MULTIPLIER: f64 = 0.06;

const NOISE_SAMPLE_VALUE_MULTIPLIER: f64 = 1.0;

/// Default map chunk generator for GemGame. Algorithm is as follows:
/// * Generate Perlin noise for coordinates within the chunk as well as immediately around the chunk (see
///   [`ChunkNoise`]).
/// * Use the noise to determine which category (grass, water, or dirt) each tile will be.
/// * Iterate through tile categories and turn into water all dirt and grass tile categories that have 3 or 4 water tile
///   category neighbours (considering only vertically & hoizontally adjacent - ignore diagonally adjacent).
/// * Iterate through tile categories again and begin placing tiles using the relevant random distributions (see
///   [`super::maybe_transition_tile`] for how transition tiles are placed).
pub struct DefaultGenerator {
    noise_func: noise::OpenSimplex,
    dirt_dist: rand::distributions::WeightedIndex<usize>,
    grass_dist: rand::distributions::WeightedIndex<usize>
}

impl super::Generator for DefaultGenerator {
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
        // Prepare RNG, noise, distributions:

        let chunk_noise = ChunkNoise::new(
            self.noise_func,
            chunk_coords,
            NOISE_SAMPLE_POINT_MULTIPLIER,
            NOISE_SAMPLE_VALUE_MULTIPLIER
        );

        let rng_seed = (chunk_coords.x as u64) ^ (chunk_coords.y as u64);
        let mut rng = StdRng::seed_from_u64(rng_seed);

        // Go through each position in the chunk & identify the tile category based on noise values:

        let mut plan = ChunkPlan::default();

        for offset_x in -1..CHUNK_WIDTH + 2 {
            for offset_y in -1..CHUNK_HEIGHT + 2 {
                let noise_sample = chunk_noise.sample(offset_x, offset_y);

                if should_be_dirt(noise_sample) {
                    plan.set_category_at(offset_x, offset_y, TileCategory::Dirt);
                }
                else if should_be_water(noise_sample) {
                    plan.set_category_at(offset_x, offset_y, TileCategory::Water);
                }
            }
        }

        plan.remove_all_juttting_and_unconnected_tiles();

        // Produce a chunk based on the chunk plan:

        let mut rng_clone = rng.clone();
        plan.to_chunk(&super::DIRT_GRASS_TRANSITION_TILES, &super::WATER_GRASS_TRANSITION_TILES, |category| {
            match category {
                TileCategory::Grass => GRASS_TILE_CHOICES[self.grass_dist.sample(&mut rng_clone)],
                TileCategory::Dirt => DIRT_TILE_CHOICES[self.dirt_dist.sample(&mut rng)],
                TileCategory::Water => Tile::Water // TODO: Add more water tile types.
            }
        })
    }

    fn name(&self) -> &'static str {
        "default"
    }
}

fn should_be_water(noise_sample: f64) -> bool {
    noise_sample <= -0.15
}

fn should_be_dirt(noise_sample: f64) -> bool {
    noise_sample >= 0.3
}
