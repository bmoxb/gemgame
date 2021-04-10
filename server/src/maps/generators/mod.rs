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
                        self.dirt_or_transition_tile(offset_x, offset_y, &mut rng, &chunk_noise)
                    }
                    else if Self::should_be_water(noise_sample) {
                        Tile::Water
                    }
                    else {
                        self.grass_or_transition_tile(offset_x, offset_y, &mut rng, &chunk_noise)
                    }
                };

                chunk.set_tile_at_offset(OffsetCoords { x: offset_x as u8, y: offset_y as u8 }, tile);
            }
        }

        chunk
    }

    fn name(&self) -> &'static str {
        "default"
    }
}

impl DefaultGenerator {
    fn dirt_or_transition_tile(
        &self, offset_x: i32, offset_y: i32, rng: &mut StdRng, chunk_noise: &ChunkNoise
    ) -> Tile {
        let above = Self::should_be_grass_at(offset_x, offset_y + 1, chunk_noise);
        let below = Self::should_be_grass_at(offset_x, offset_y - 1, chunk_noise);
        let left = Self::should_be_grass_at(offset_x - 1, offset_y, chunk_noise);
        let right = Self::should_be_grass_at(offset_x + 1, offset_y, chunk_noise);

        // If a dirt tile is surrounded by grass tiles on any 3 sides, replace it with a grass tile:
        if (above ^ below ^ left ^ right) & ((above & below) | (left & right)) {
            return Tile::Grass;
        }

        let transition_tile = match (above, below, left, right) {
            // Straight transition tiles:
            (true, _, false, false) => Some(Tile::DirtGrassTop),
            (_, true, false, false) => Some(Tile::DirtGrassBottom),
            (false, false, true, _) => Some(Tile::DirtGrassLeft),
            (false, false, _, true) => Some(Tile::DirtGrassRight),
            // Right-angle transition tiles:
            (true, _, true, false) => Some(Tile::DirtGrassTopLeft),
            (true, _, false, true) => Some(Tile::DirtGrassTopRight),
            (_, true, true, false) => Some(Tile::DirtGrassBottomLeft),
            (_, true, false, true) => Some(Tile::DirtGrassBottomRight),

            _ => None
        };

        transition_tile
            .or_else(|| {
                let top_left = Self::should_be_grass_at(offset_x - 1, offset_y + 1, chunk_noise);
                let top_right = Self::should_be_grass_at(offset_x + 1, offset_y + 1, chunk_noise);
                let bottom_left = Self::should_be_grass_at(offset_x - 1, offset_y - 1, chunk_noise);
                let bottom_right = Self::should_be_grass_at(offset_x + 1, offset_y - 1, chunk_noise);

                match (top_left, top_right, bottom_left, bottom_right) {
                    // Corner tile transitions:
                    (true, false, false, _) => Some(Tile::DirtGrassCornerTopLeft),
                    (false, true, _, false) => Some(Tile::DirtGrassCornerTopRight),
                    (false, _, true, false) => Some(Tile::DirtGrassCornerBottomLeft),
                    (_, false, false, true) => Some(Tile::DirtGrassCornerBottomRight),
                    // Straight transition tiles (accounting for positions where a single dirt tile would jut out but is
                    // replaced with a grass tile instead):
                    (true, true, ..) => Some(Tile::DirtGrassTop),
                    (_, _, true, true) => Some(Tile::DirtGrassBottom),
                    (true, _, true, _) => Some(Tile::DirtGrassLeft),
                    (_, true, _, true) => Some(Tile::DirtGrassRight),
                    _ => None
                }
            })
            .unwrap_or_else(|| DIRT_TILE_CHOICES[self.dirt_dist.sample(rng)])
    }

    fn grass_or_transition_tile(
        &self, offset_x: i32, offset_y: i32, rng: &mut StdRng, chunk_noise: &ChunkNoise
    ) -> Tile {
        GRASS_TILE_CHOICES[self.grass_dist.sample(rng)]
    }

    fn should_be_grass_at(offset_x: i32, offset_y: i32, chunk_noise: &ChunkNoise) -> bool {
        let sample = chunk_noise.sample(offset_x, offset_y);
        !Self::should_be_dirt(sample) && !Self::should_be_water(sample)
    }

    fn should_be_dirt(noise_sample: f64) -> bool {
        noise_sample >= 0.3
    }

    fn should_be_water(noise_sample: f64) -> bool {
        noise_sample <= -0.15
    }
}
