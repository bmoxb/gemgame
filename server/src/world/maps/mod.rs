pub mod generators;

use generators::Generator;

use core::maps::{ ChunkCoords, Chunk, Chunks, Map };

use std::{ io, path::PathBuf, collections::HashMap };

use serde::{ Serialize, Deserialize };

use tokio::io::AsyncReadExt;

pub struct ServerMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Path to the directory containing map data.
    directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    generator: Box<dyn Generator + Send>,

    /// Seed used by the generator.
    seed: u32

    // players, entities, etc.
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerMapConfig {
    #[serde(rename = "generator")]
    generator_name: String,
    seed: u32
}

const MAP_CONFIG_FILE_NAME: &'static str = "map.json";

impl ServerMap {
    /// Create a new map at a specified path with a given generator and seed.
    pub async fn new(directory: PathBuf, generator: Box<dyn Generator + Send>, seed: u32) -> io::Result<Self> {
        tokio::fs::create_dir_all(&directory).await?;

        Ok(ServerMap {
            loaded_chunks: HashMap::new(),
            directory, generator, seed
        })
    }

    /// Attempt to load an existing map.
    pub async fn load(directory: PathBuf) -> LoadResult<Self> {
        let config_file_path = directory.join(MAP_CONFIG_FILE_NAME);

        log::debug!("Attempting to load map configuration file: {}", config_file_path.display());

        if let Ok(mut file) = tokio::fs::File::open(&config_file_path).await {
            let mut buffer = Vec::new();
            match file.read_buf(&mut buffer).await {
                Ok(_) => match serde_json::from_slice::<ServerMapConfig>(buffer.as_slice()) {
                    Ok(config) => {
                        log::trace!("Map configuration: {:?}", config);

                        if let Some(generator) = generators::by_name(&config.generator_name) {
                            log::debug!("Loaded map configuration from file: {}",
                                        config_file_path.display());

                            Ok(ServerMap::new(directory, generator, config.seed).await.unwrap())
                        }
                        else {
                            log::warn!("Generator specified in map configuration file '{}' does not exist: {}",
                                       config_file_path.display(), config.generator_name);
                            Err(LoadError::InvalidGenerator(config.generator_name))
                        }
                    }

                    Err(json_error) => {
                        log::warn!("Failed decode JSON map configuration from file '{}' - {}",
                                   config_file_path.display(), json_error);
                        Err(LoadError::CouldNotDecode(Box::new(json_error)))
                    }
                }

                Err(io_error) => {
                    log::warn!("Failed to read map configuration from file '{}' - {}",
                               config_file_path.display(), io_error);
                    Err(LoadError::CouldNotRead(io_error))
                }
            }
        }
        else { Err(LoadError::DoesNotExist(config_file_path)) }
    }

    /// Fetch from memory/read from the filesystem/newly generate the chunk at
    /// the specified coordinates.
    async fn chunk_at(&mut self, coords: ChunkCoords) -> &Chunk {
        if !self.is_chunk_loaded(coords) {
            let new_chunk = self.read_chunk_from_filesystem(coords).await
                                .unwrap_or_else(|_| self.generate_new_chunk(coords));

            self.loaded_chunks.insert(coords, new_chunk);
        }

        self.loaded_chunk_at(coords).unwrap()
    }

    /// Attempt to asynchronously read data from the file system for the chunk
    /// at the specified coordinates. Note that this method will *not* insert
    /// the loaded chunk into the `loaded_chunks` hash map.
    async fn read_chunk_from_filesystem(&self, coords: ChunkCoords) -> LoadResult<Chunk> {
        let chunk_file_path = self.directory.join(format!("{}_{}.chunk", coords.x, coords.y));

        log::trace!("Attempting to load chunk at {} from file: {}", coords, chunk_file_path.display());

        if let Ok(mut file) = tokio::fs::File::open(&chunk_file_path).await {
            let mut buffer = Vec::new();
            match file.read_buf(&mut buffer).await {
                Ok(_) => match bincode::deserialize(buffer.as_slice()) {
                    Ok(chunk) => {
                        log::debug!("Loaded chunk from file: {}", chunk_file_path.display());

                        Ok(chunk)
                    }

                    Err(bincode_error) => {
                        log::warn!("Failed to decode chunk data read from file '{}' - {}",
                                   chunk_file_path.display(), bincode_error);
                        Err(LoadError::CouldNotDecode(bincode_error))
                    }
                }

                Err(io_error) => {
                    log::warn!("Failed to read chunk data from file '{}' - {}",
                               chunk_file_path.display(), io_error);
                    Err(LoadError::CouldNotRead(io_error))
                }
            }
        }
        else { Err(LoadError::DoesNotExist(chunk_file_path)) }
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

#[derive(Debug)]
pub enum LoadError {
    DoesNotExist(PathBuf),
    CouldNotRead(io::Error),
    CouldNotDecode(Box<dyn std::error::Error>),
    InvalidGenerator(String)
}

pub type LoadResult<T> = std::result::Result<T, LoadError>;