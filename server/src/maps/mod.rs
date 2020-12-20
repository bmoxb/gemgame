mod generators;

use generators::Generator;

use core::maps::{ Coord, Chunk, Chunks };

use std::path::PathBuf;

struct Map {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Path to the directory containing map data.
    directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    generator: Box<dyn Generator>

    // players, entities, etc.
}

impl core::maps::Map for Map {
    fn chunk_at(&mut self, chunk_x: Coord, chunk_y: Coord) -> &Chunk {
        // TODO: If chunk is in memory then return reference.
        //       Otherwise, if chunk on the disk load into memory.
        //       Otherwise, generate a new chunk.
        unimplemented!();
    }
}