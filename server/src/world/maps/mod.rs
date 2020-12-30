pub mod generators;

use generators::Generator;

use core::maps::{ ChunkCoords, Chunk, Chunks, Map };

use std::{ io, path::PathBuf, collections::HashMap };

use serde::{ Serialize, Deserialize };

use tokio::io::{ AsyncReadExt, AsyncWriteExt };

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
    pub async fn load(directory: PathBuf) -> Result<Self> {
        let config_file_path = directory.join(MAP_CONFIG_FILE_NAME);

        log::debug!("Attempting to load map configuration file: {}", config_file_path.display());

        if let Ok(mut file) = tokio::fs::File::open(&config_file_path).await {
            let mut buffer = Vec::new();
            match file.read_to_end(&mut buffer).await {
                Ok(_) => match serde_json::from_slice::<ServerMapConfig>(buffer.as_slice()) {
                    Ok(config) => {
                        log::trace!("Map configuration struct: {:?}", config);

                        if let Some(generator) = generators::by_name(&config.generator_name) {
                            log::debug!("Loaded map configuration from file: {}",
                                        config_file_path.display());

                            Ok(ServerMap::new(directory, generator, config.seed).await.unwrap())
                        }
                        else {
                            log::warn!("Generator specified in map configuration file '{}' does not exist: {}",
                                       config_file_path.display(), config.generator_name);
                            Err(Error::InvalidGenerator(config.generator_name))
                        }
                    }

                    Err(json_error) => {
                        log::warn!("Failed decode JSON map configuration from file '{}' - {}",
                                   config_file_path.display(), json_error);
                        Err(Error::EncodingFailure(Box::new(json_error)))
                    }
                }

                Err(io_error) => {
                    log::warn!("Failed to read map configuration from file '{}' - {}",
                               config_file_path.display(), io_error);
                    Err(Error::AccessFailure(io_error))
                }
            }
        }
        else { Err(Error::DoesNotExist(config_file_path)) }
    }

    /// Save this map's configuration and its currently loaded chunks.
    pub async fn save_all(&self) -> Result<()> {
        let config_saved = self.save_config().await;
        let chunks_saved = self.save_loaded_chunks().await;

        config_saved.and(chunks_saved)
    }

    /// Save this map's configuration (i.e. map generation seed, type of generator
    /// used, etc.)
    pub async fn save_config(&self) -> Result<()> {
        let config_file_path = self.directory.join(MAP_CONFIG_FILE_NAME);

        log::debug!("Attempting to save map configuration file: {}", config_file_path.display());

        match tokio::fs::File::create(&config_file_path).await {
            Ok(mut file) => {
                let config = ServerMapConfig {
                    generator_name: self.generator.name().to_string(),
                    seed: self.seed
                };
                let config_json = serde_json::to_string(&config).unwrap();

                log::trace!("Map configuration JSON: {}", config_json);

                match file.write_all(config_json.as_bytes()).await {
                    Ok(_) => {
                        log::debug!("Saved map configuration to file: {}",
                                    config_file_path.display());
                        Ok(())
                    }

                    Err(write_error) => {
                        log::warn!("Failed to write map configuration to file '{}' - {}",
                                   config_file_path.display(), write_error);
                        Err(Error::AccessFailure(write_error))
                    }
                }
            }

            Err(create_error) => {
                log::warn!("Failed to create/open map configuration file '{}' - {}",
                           config_file_path.display(), create_error);
                Err(Error::AccessFailure(create_error))
            }
        }
    }

    /// Save all of this map's chunks that are currently loaded.
    pub async fn save_loaded_chunks(&self) -> Result<()> {
        let mut success = Ok(());

        for (coords, chunk) in self.loaded_chunks.iter() {
            success = success.and(self.save_chunk_to_filesystem(*coords, chunk).await);
        }

        success
    }

    /// Fetch from memory/read from the filesystem/newly generate the chunk at
    /// the specified coordinates.
    pub async fn chunk_at(&mut self, coords: ChunkCoords) -> &Chunk {
        if !self.is_chunk_loaded(coords) {
            let new_chunk = self.load_chunk_from_filesystem(coords).await
                                .unwrap_or_else(|_| self.generate_new_chunk(coords));

            self.loaded_chunks.insert(coords, new_chunk);
        }

        self.loaded_chunk_at(coords).unwrap()
    }

    /// Attempt to asynchronously read data from the file system for the chunk
    /// at the specified coordinates. Note that this method will *not* insert
    /// the loaded chunk into the `loaded_chunks` hash map.
    async fn load_chunk_from_filesystem(&self, coords: ChunkCoords) -> Result<Chunk> {
        let chunk_file_path = self.directory.join(chunk_file_name(coords));

        log::trace!("Attempting to load chunk at {} from file: {}", coords, chunk_file_path.display());

        if let Ok(mut file) = tokio::fs::File::open(&chunk_file_path).await {
            let mut buffer = Vec::new();
            match file.read_to_end(&mut buffer).await {
                Ok(_) => match bincode::deserialize(buffer.as_slice()) {
                    Ok(chunk) => {
                        log::debug!("Loaded chunk at {} from file: {}", coords,
                                    chunk_file_path.display());
                        Ok(chunk)
                    }

                    Err(bincode_error) => {
                        log::warn!("Failed to decode chunk data read from file '{}' - {}",
                                   chunk_file_path.display(), bincode_error);
                        Err(Error::EncodingFailure(bincode_error))
                    }
                }

                Err(read_error) => {
                    log::warn!("Failed to read chunk data from file '{}' - {}",
                               chunk_file_path.display(), read_error);
                    Err(Error::AccessFailure(read_error))
                }
            }
        }
        else { Err(Error::DoesNotExist(chunk_file_path)) }
    }

    async fn save_chunk_to_filesystem(&self, coords: ChunkCoords, chunk: &Chunk) -> Result<()> {
        let chunk_file_path = self.directory.join(chunk_file_name(coords));

        log::trace!("Attempting to save chunk at {} to file: {}", coords, chunk_file_path.display());

        match tokio::fs::File::create(&chunk_file_path).await {
            Ok(mut file) => match bincode::serialize(chunk) {
                Ok(data) => match file.write(data.as_slice()).await {
                    Ok(_) => {
                        log::debug!("Saved chunk at {} to file: {}", coords,
                                    chunk_file_path.display());
                        Ok(())
                    }

                    Err(write_error) => {
                        log::warn!("Failed to write chunk data to file '{}' - {}",
                                   chunk_file_path.display(), write_error);
                        Err(Error::AccessFailure(write_error))
                    }
                }

                Err(bincode_error) => {
                    log::warn!("Failed to encode chunk data for {} - {}",
                               coords, bincode_error);
                    Err(Error::EncodingFailure(bincode_error))
                }
            }

            Err(create_error) => {
                log::warn!("Failed to create/open chunk data file '{}' - {}",
                           chunk_file_path.display(), create_error);
                Err(Error::AccessFailure(create_error))
            }
        }
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

fn chunk_file_name(coords: ChunkCoords) -> String {
    format!("{}_{}.chunk", coords.x, coords.y)
}

#[derive(Debug)]
pub enum Error {
    DoesNotExist(PathBuf),
    AccessFailure(io::Error),
    EncodingFailure(Box<dyn std::error::Error>),
    InvalidGenerator(String)
}

pub type Result<T> = std::result::Result<T, Error>;