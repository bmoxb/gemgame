mod config;
mod chunks;
mod generators;
use generators::Generator;

use core::maps::{ ChunkCoords, Chunk, Chunks, Map };

use std::{ io, path::PathBuf, collections::HashMap };

pub struct ServerMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// The generator to be used when new chunks must be made.
    generator: Box<dyn Generator + Send>,

    /// Seed used by the generator.
    seed: u32

    // players, entities, etc.
}

impl ServerMap {
    /// Create a new map with a given generator and seed.
    pub fn new(generator: Box<dyn Generator + Send>, seed: u32) -> io::Result<Self> {
        Ok(ServerMap {
            loaded_chunks: HashMap::new(),
            generator, seed
        })
    }
}

impl Map for ServerMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(&coords)
    }
}

#[derive(Debug)]
pub enum Error {
    DoesNotExist(PathBuf),
    AccessFailure(io::Error),
    EncodingFailure(Box<dyn std::error::Error>),
    InvalidGenerator(String)
}

pub type Result<T> = std::result::Result<T, Error>;