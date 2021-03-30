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
use sqlx::Row;

use crate::db_query_from_file;

pub struct ServerMap {
    /// Seed used by the generator.
    seed: i32,

    /// The generator to be used when new chunks are generated.
    generator: Box<dyn Generator + Send>,

    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,

    /// Keeps track of how many remote clients have each chunk loaded.
    chunk_usage: HashMap<ChunkCoords, usize>,

    /// Player-controlled entities mapped to entity IDs.
    player_entities: HashMap<Id, Entity>,

    /// Chunk coordinates mapped to sets of entity IDs. This hash map exists to allow the efficient look up of which
    /// entities exists in which chunks.
    chunk_coords_to_player_ids: HashMap<ChunkCoords, HashSet<Id>>
}

impl ServerMap {
    pub async fn load_or_new(db_pool: &sqlx::PgPool) -> sqlx::Result<Self> {
        let existing_map_option = db_query_from_file!("map/select row")
            .map(|row: sqlx::postgres::PgRow| ServerMap::new_with_default_generator(row.get("seed")))
            .fetch_optional(db_pool)
            .await?;

        if let Some(existing_map) = existing_map_option {
            log::debug!("Existing map loaded from database");

            sqlx::Result::Ok(existing_map)
        }
        else {
            let new_map = ServerMap::new_with_default_generator(0); // TODO: Random seed.

            db_query_from_file!("map/create row").bind(new_map.seed).execute(db_pool).await.map(|_| {
                log::debug!("Inserted newly generated map into database");

                new_map
            })
        }
    }

    pub fn new(seed: i32, generator: Box<dyn Generator + Send>) -> Self {
        ServerMap {
            seed,
            generator,
            loaded_chunks: HashMap::new(),
            chunk_usage: HashMap::new(),
            player_entities: HashMap::new(),
            chunk_coords_to_player_ids: HashMap::new()
        }
    }

    pub fn new_with_default_generator(seed: i32) -> Self {
        ServerMap::new(seed, Box::new(generators::DefaultGenerator))
    }

    pub async fn save_loaded_chunks(&self, db_pool: &sqlx::PgPool) -> Result<()> {
        let mut success = Ok(());

        let mut db = db_pool.acquire().await.unwrap();

        for (coords, chunk) in &self.loaded_chunks {
            success = success.and(chunks::save_chunk(&mut db, *coords, chunk).await);
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

    /// To be called by a client task whenever their remote client is provided with a certain chunk.
    pub fn chunk_in_use(&mut self, coords: ChunkCoords) {
        *self.chunk_usage.entry(coords).or_default() += 1;
    }

    /// To be called by a client task whenever their remote client is told to unload a certain chunk.
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
