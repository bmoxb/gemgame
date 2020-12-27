pub mod generators;

use generators::Generator;

use core::maps::{ ChunkCoords, Chunk, Chunks, Map };

use std::{ path::PathBuf, collections::HashMap };

use tokio::io::AsyncReadExt;

pub struct ServerMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Path to the directory containing map data.
    directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    generator: Box<dyn Generator + Send>

    // players, entities, etc.
}

impl ServerMap {
    pub fn new(directory: PathBuf, generator: Box<dyn Generator + Send>) -> Self {
        ServerMap {
            loaded_chunks: HashMap::new(),
            directory, generator
        }
    }

    /// Fetch/read from the filesystem/newly generate the chunk at the specified
    /// coordinates
    async fn chunk_at(&mut self, coords: ChunkCoords) -> &Chunk {
        if !self.is_chunk_loaded(coords) {
            let new_chunk = self.read_chunk_from_filesystem(coords).await
                                .unwrap_or_else(|| self.generate_new_chunk(coords));

            self.loaded_chunks.insert(coords, new_chunk);
        }

        self.loaded_chunk_at(coords).unwrap()
    }

    /// Attempt to asynchronously read data from the file system for the chunk
    /// at the specified coordinates. Note that this method will *not* insert
    /// the loaded chunk into the `loaded_chunks` hash map.
    async fn read_chunk_from_filesystem(&self, coords: ChunkCoords) -> Option<Chunk> {
        let chunk_file_path = self.directory.join(format!("{}_{}.chunk", coords.x, coords.y));

        log::debug!("Attempting to load chunk at {} from file: {}", coords, chunk_file_path.display());

        if let Ok(mut file) = tokio::fs::File::open(&chunk_file_path).await {
            log::trace!("Opened chunk file: {}", chunk_file_path.display());

            let mut buffer = Vec::new();
            match file.read_buf(&mut buffer).await {
                Ok(_) => match bincode::deserialize(buffer.as_slice()) {
                    Ok(chunk) => {
                        log::info!("Loaded chunk from file: {}", chunk_file_path.display());

                        return Some(chunk);
                    }

                    Err(bincode_error) => log::warn!("Failed to decode chunk data read from file '{}' - {}",
                                         chunk_file_path.display(), bincode_error)
                }

                Err(io_error) => log::warn!("Failed to read chunk data from file '{}' - {}",
                                     chunk_file_path.display(), io_error)
            }
        }

        None
    }

    /// Generate a new chunk by passing the specified chunk coordinates to this
    /// map's generator. Note that this method will *not* insert the generated
    /// chunk into the `loaded_chunks` hash map.
    fn generate_new_chunk(&self, coords: ChunkCoords) -> Chunk {
        log::info!("Generating new chunk at: {}", coords);

        self.generator.generate(coords)
    }
}

impl Map for ServerMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(&coords)
    }
}