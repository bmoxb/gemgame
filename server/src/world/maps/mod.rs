pub mod generators;

use std::{collections::HashMap, io, path::PathBuf};

use generators::Generator;
use shared::world::maps::{Chunk, ChunkCoords, Chunks, Map};

pub struct ServerMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Path to the directory containing map data.
    pub directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    pub generator: Box<dyn Generator + Send>,

    /// Seed used by the generator.
    seed: u32
}

impl ServerMap {
    /// Create a new map at a specified path with a given generator and seed.
    pub fn new(directory: PathBuf, generator: Box<dyn Generator + Send>, seed: u32) -> io::Result<Self> {
        Ok(ServerMap { loaded_chunks: HashMap::new(), directory, generator, seed })
    }
}

impl Map for ServerMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> { self.loaded_chunks.get(&coords) }

    fn provide_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) { self.loaded_chunks.insert(coords, chunk); }
}

#[derive(Debug)]
pub enum Error {
    DoesNotExist(PathBuf),
    AccessFailure(io::Error),
    EncodingFailure(Box<dyn std::error::Error>),
    InvalidGenerator(String)
}

pub type Result<T> = std::result::Result<T, Error>;
