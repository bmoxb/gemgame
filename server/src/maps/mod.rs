pub mod chunks;
pub mod entities;
pub mod generators;

use std::{
    collections::{HashMap, HashSet},
    fmt, io,
    path::PathBuf
};

use generators::Generator;
use serde::{Deserialize, Serialize};
use shared::{
    maps::{
        entities::{Direction, Entity},
        Chunk, ChunkCoords, Chunks, Map, Tile, TileCoords
    },
    Id
};
use tokio::io::AsyncReadExt;

const CONFIG_FILE_NAME: &str = "map.json";

pub struct ServerMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Path to the directory containing map data.
    directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    generator: Box<dyn Generator + Send>,

    /// Seed used by the generator.
    seed: u32,

    /// Keeps track of how many remote clients have each chunk loaded.
    chunk_usage: HashMap<ChunkCoords, usize>,

    /// Player-controlled entities mapped to entity IDs.
    player_entities: HashMap<Id, Entity>,

    /// Chunk coordinates mapped to sets of entity IDs. This hash map exists to allow the efficient look up of which
    /// entities exists in which chunks.
    chunk_coords_to_player_ids: HashMap<ChunkCoords, HashSet<Id>>
}

impl ServerMap {
    pub fn new(directory: PathBuf, generator: Box<dyn Generator + Send>, seed: u32) -> Self {
        ServerMap {
            loaded_chunks: HashMap::new(),
            directory,
            generator,
            seed,
            chunk_usage: HashMap::new(),
            player_entities: HashMap::new(),
            chunk_coords_to_player_ids: HashMap::new()
        }
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

    /// Move an entity in a specified direction. This method checks if the desintation position is already occupied or
    /// a blocking tile - if it is then `None` is returned (`None` is also returned should an entity with the specified
    /// ID not be found). If the movement is deemed okay to go ahead, the entity's old position and new position (i.e.
    /// position after the movement is applied) are returned. The hash map that keeps track of which entities reside in
    /// which chunks is updated also.
    pub fn move_entity_towards(&mut self, entity_id: Id, direction: Direction) -> Option<(TileCoords, TileCoords)> {
        if self.player_entities.contains_key(&entity_id) {
            let (old_pos, new_pos, new_pos_is_free) = {
                let entity = self.entity_by_id(entity_id).unwrap();

                let new_pos = direction.apply(entity.pos);
                (entity.pos, new_pos, self.is_position_free(new_pos))
            };

            let entity_mut = self.player_entities.get_mut(&entity_id).unwrap();

            if new_pos_is_free {
                // Apply the new position & direction:
                entity_mut.pos = new_pos;
                entity_mut.direction = direction;

                let old_pos_chunk_coords = old_pos.as_chunk_coords();
                let new_pos_chunk_coords = new_pos.as_chunk_coords();

                // Check if the entity is moving across chunk boundaries:
                if old_pos_chunk_coords != new_pos_chunk_coords {
                    self.chunk_coords_to_player_ids.entry(old_pos_chunk_coords).and_modify(|x| {
                        x.remove(&entity_id);
                    });
                    self.chunk_coords_to_player_ids.entry(new_pos_chunk_coords).or_default().insert(entity_id);
                }

                Some((old_pos, new_pos))
            }
            else {
                None // Movement not allowed.
            }
        }
        else {
            None // Entity with that ID not found.
        }
    }

    /// Get all entity IDs and entities in the chunk at the given chunk coordinates.
    pub fn entities_in_chunk(&self, coords: ChunkCoords) -> Vec<(Id, Entity)> {
        let mut entities = Vec::new();

        if let Some(set) = self.chunk_coords_to_player_ids.get(&coords) {
            for entity_id in set.iter() {
                if let Some(entity) = self.player_entities.get(entity_id) {
                    entities.push((*entity_id, entity.clone()));
                }
            }
        }

        entities
    }

    pub fn chunk_in_use(&mut self, coords: ChunkCoords) {
        *self.chunk_usage.entry(coords).or_default() += 1;
    }

    pub fn chunk_not_in_use(&mut self, coords: ChunkCoords) {
        let entry = self.chunk_usage.entry(coords).or_default();
        *entry -= 1;

        // If no clients have the chunk loaded, then unloaded the chunk on the server:
        if *entry == 0 {
            self.remove_chunk(coords);
        }
    }
}

impl Map for ServerMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(&coords)
    }

    fn loaded_chunk_at_mut(&mut self, coords: ChunkCoords) -> Option<&mut Chunk> {
        self.loaded_chunks.get_mut(&coords)
    }

    fn add_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) {
        self.loaded_chunks.insert(coords, chunk);
        self.chunk_coords_to_player_ids.insert(coords, HashSet::new());
    }

    fn remove_chunk(&mut self, coords: ChunkCoords) -> Option<Chunk> {
        log::debug!("Chunk at {} unloaded", coords);

        self.chunk_coords_to_player_ids.remove(&coords);
        self.loaded_chunks.remove(&coords)
    }

    fn entity_by_id(&self, id: Id) -> Option<&Entity> {
        self.player_entities.get(&id)
    }

    fn add_entity(&mut self, id: Id, entity: Entity) {
        let chunk_coords = entity.pos.as_chunk_coords();

        self.chunk_coords_to_player_ids.entry(chunk_coords).or_default().insert(id);
        self.player_entities.insert(id, entity);

        if self.is_chunk_loaded(chunk_coords) {
            log::debug!("Player entity with ID {} added to game map", id);
        }
        else {
            log::warn!("Add entity {} to map yet that entity's position is in an unloaded chunk", id);
        }
    }

    fn remove_entity(&mut self, id: Id) -> Option<Entity> {
        log::debug!("Removing player entity with ID {} from game map", id);
        let opt = self.player_entities.remove(&id);

        // Remove the association between the entity and the chunk that entity was in:
        if let Some(entity) = &opt {
            self.chunk_coords_to_player_ids.entry(entity.pos.as_chunk_coords()).and_modify(|x| {
                x.remove(&id);
            });
        }

        opt
    }

    fn is_blocking_entity_at(&self, coords: TileCoords) -> bool {
        // First identify all entities in the chunk that the specified coordinates are in:
        if let Some(entity_ids_in_chunk) = self.chunk_coords_to_player_ids.get(&coords.as_chunk_coords()) {
            // Iterate through the entities in that chunk, checking each entity's position:
            for entity_id in entity_ids_in_chunk {
                if let Some(entity) = self.entity_by_id(*entity_id) {
                    if entity.pos == coords {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MapConfig {
    #[serde(rename = "generator")]
    generator_name: String,
    seed: u32
}

/// Represents a change made to the game map (tiles and entities). This enum is used by client tasks to inform other
/// tasks of changes made to the game map.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Modification {
    TileChanged(TileCoords, Tile),

    EntityMoved {
        /// The ID of the entity that moved.
        entity_id: Id,
        /// The previous position of the entity (i.e. before the movement that this message describes).
        old_position: TileCoords,
        /// The new position of the entity that moved.
        new_position: TileCoords,
        /// The direction in which the movement occurred.
        direction: Direction
    },

    /// Indicates a new entity has been added to the map (i.e. a player just connected).
    EntityAdded(Id),

    /// Indicates that the entity with the specified ID has been removed from the map (i.e. a player just
    /// disconnected). The coordinates of the chunk that the entity was positioned in are included so that each
    /// task can decide whether to inform their client of the entity's removal based on their loaded chunks.
    EntityRemoved(Id, ChunkCoords)
}

impl fmt::Display for Modification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Modification::TileChanged(position, change_to) => {
                write!(f, "tile changed at {} to {:?}", position, change_to)
            }
            Modification::EntityMoved { entity_id, old_position, new_position, direction } => {
                write!(
                    f,
                    "entity {} moved from {} to {} in direction {}",
                    entity_id, old_position, new_position, direction
                )
            }
            Modification::EntityAdded(id) => write!(f, "entity {} added to map", id),
            Modification::EntityRemoved(id, coords) => {
                write!(f, "entity {} in chunk at {} removed from map", id, coords)
            }
        }
    }
}

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
