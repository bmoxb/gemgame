use std::collections::HashMap;

use shared::maps::{ChunkCoords, CHUNK_HEIGHT, CHUNK_WIDTH};

/// Generates and stores noise for a chunk at a given set of chunk coordinates using the specified noise function. Also
/// generates noise values for a further one tile around the chunk tiles. This is so that edge tiles can be placed
/// appropriately.
pub struct ChunkNoise {
    data: HashMap<(i32, i32), f64>
}

impl ChunkNoise {
    pub fn new(
        noise_func: impl noise::NoiseFn<[f64; 2]>, chunk_coords: ChunkCoords, sample_point_multiplier: f64,
        sample_value_multiplier: f64
    ) -> Self {
        let mut data = HashMap::new();

        for offset_x in -1..CHUNK_WIDTH + 2 {
            for offset_y in -1..CHUNK_HEIGHT + 2 {
                let sample_point = [
                    ((chunk_coords.x * CHUNK_WIDTH + offset_x) as f64) * sample_point_multiplier,
                    ((chunk_coords.y * CHUNK_HEIGHT + offset_y) as f64) * sample_point_multiplier
                ];
                let value = (noise_func.get(sample_point) * sample_value_multiplier).clamp(-1.0, 1.0);

                data.insert((offset_x, offset_y), value);
            }
        }

        ChunkNoise { data }
    }

    pub fn sample(&self, offset_x: i32, offset_y: i32) -> f64 {
        *self.data.get(&(offset_x, offset_y)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use noise::NoiseFn;

    use super::*;

    fn assert_eq_float(lhs: f64, rhs: f64) {
        assert!((lhs - rhs).abs() < f64::EPSILON);
    }

    #[test]
    fn sample_chunk_noise() {
        let original = noise::OpenSimplex::new();
        let gen = ChunkNoise::new(original, ChunkCoords { x: 0, y: 0 }, 2.0, 0.5);

        let test_data = &[(0, 0), (5, 3), (CHUNK_WIDTH - 1, 10), (8, CHUNK_HEIGHT), (-1, -1)];

        for (x, y) in test_data {
            let original_sample_point = [*x as f64 * 2.0, *y as f64 * 2.0];
            assert_eq_float(gen.sample(*x, *y), original.get(original_sample_point) * 0.5);
        }
    }
}
