pub mod chunks;
pub mod entities;
pub mod generators;

use std::{collections::HashMap, io, path::PathBuf};

use entities::PlayerEntity;
use generators::Generator;
use serde::{Deserialize, Serialize};
use shared::{
    maps::{Chunk, ChunkCoords, Chunks, Map, Tile, TileCoords},
    Id
};
use tokio::io::AsyncReadExt;

const CONFIG_FILE_NAME: &str = "map.json";

pub struct ServerMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Path to the directory containing map data.
    pub directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    pub generator: Box<dyn Generator + Send>,

    /// Seed used by the generator.
    seed: u32,

    /// Player-controlled entities mapped to entity IDs.
    player_entities: HashMap<Id, PlayerEntity>
}

impl ServerMap {
    pub fn new(directory: PathBuf, generator: Box<dyn Generator + Send>, seed: u32) -> Self {
        ServerMap { loaded_chunks: HashMap::new(), directory, generator, seed, player_entities: HashMap::new() }
    }

    /// Attempt to load a map from the specified directory. If unsuccessful, create a new map with appropriate defaults
    /// set.
    pub async fn try_load(directory: PathBuf) -> Self {
        // TODO: Use timestamp as seed.
        ServerMap::load(directory.clone()).await.unwrap_or_else(|e| {
            log::debug!("Could not load existing map from directory '{}' due to error: {}", directory.display(), e);
            ServerMap::new(directory, Box::new(generators::DefaultGenerator), 0)
        })
    }

    /// Load an existing map from the specified directory.
    pub async fn load(directory: PathBuf) -> Result<Self> {
        let config_file_path = directory.join(CONFIG_FILE_NAME);

        log::debug!("Attempting to load map configuration file: {}", config_file_path.display());

        if let Ok(mut file) = tokio::fs::File::open(&config_file_path).await {
            let mut buffer = Vec::new();

            match file.read_to_end(&mut buffer).await {
                Ok(_) => match serde_json::from_slice::<MapConfig>(buffer.as_slice()) {
                    Ok(config) => {
                        log::trace!("Map configuration struct: {:?}", config);

                        if let Some(generator) = generators::by_name(&config.generator_name, config.seed) {
                            log::debug!("Loaded map configuration from file: {}", config_file_path.display());

                            Ok(ServerMap::new(directory, generator, config.seed))
                        }
                        else {
                            log::warn!(
                                "Generator specified in map configuration file '{}' does not exist: {}",
                                config_file_path.display(),
                                config.generator_name
                            );
                            Err(Error::InvalidGenerator(config.generator_name))
                        }
                    }

                    Err(json_error) => {
                        log::warn!(
                            "Failed decode JSON map configuration from file '{}' - {}",
                            config_file_path.display(),
                            json_error
                        );
                        Err(Error::EncodingFailure(Box::new(json_error)))
                    }
                },

                Err(io_error) => {
                    log::warn!(
                        "Failed to read map configuration from file '{}' - {}",
                        config_file_path.display(),
                        io_error
                    );
                    Err(Error::AccessFailure(io_error))
                }
            }
        }
        else {
            Err(Error::DoesNotExist(config_file_path))
        }
    }

    pub async fn save_all(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.directory).await?;
        // TODO: Save map config as well.
        self.save_loaded_chunks().await
    }

    async fn save_loaded_chunks(&self) -> Result<()> {
        let mut success = Ok(());

        for (coords, chunk) in &self.loaded_chunks {
            success = success.and(chunks::save_chunk(&self.directory, *coords, chunk).await);
        }

        success
    }

    pub fn add_player_entity(&mut self, id: Id, entity: PlayerEntity) {
        log::debug!("Player entity with ID {} added to game world", id);
        self.player_entities.insert(id, entity);
    }

    pub fn player_entity_by_id(&mut self, id: Id) -> Option<&mut PlayerEntity> { self.player_entities.get_mut(&id) }

    pub fn remove_player_entity(&mut self, id: Id) -> Option<PlayerEntity> {
        let entity = self.player_entities.remove(&id);
        if entity.is_some() {
            log::debug!("Player entity with ID {} removed from game world", id);
        }
        entity
    }
}

impl Map for ServerMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> { self.loaded_chunks.get(&coords) }

    fn provide_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) { self.loaded_chunks.insert(coords, chunk); }
}

#[derive(Debug, Serialize, Deserialize)]
struct MapConfig {
    #[serde(rename = "generator")]
    generator_name: String,
    seed: u32
}

/// Structure representing a change made to the game map.
#[derive(Copy, Clone)]
pub enum Modification {
    TileChanged {
        /// Position of the tile tile to be modified.
        pos: TileCoords,
        /// What the tile at the specified coordinates should be changed to.
        change_to: Tile
    },

    EntityMoved /* { ... } */
}

// TODO: Derive error macro...
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("file/directory '{0}' does not exist")]
    DoesNotExist(PathBuf),
    #[error("failed to access file/directory due to IO error - {0}")]
    AccessFailure(#[from] io::Error),
    #[error("failed due to bincode (de)serialisation error - {0}")]
    EncodingFailure(Box<dyn std::error::Error>),
    #[error("generator string '{0}' is invalid")]
    InvalidGenerator(String)
}

pub type Result<T> = std::result::Result<T, Error>;
