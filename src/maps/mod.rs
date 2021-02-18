pub mod coords;
pub mod entities;

use std::collections::HashMap;

pub use coords::*;
use entities::Entity;
use serde::{Deserialize, Serialize};
use serde_big_array::big_array;

use crate::Id;

// TODO: Remove this workaround when const generics are properly stablised.
big_array! { BigArray; }

/// How many tiles wide a chunk is.
pub const CHUNK_WIDTH: i32 = 16;

/// How many tiles high a chunk is.
pub const CHUNK_HEIGHT: i32 = 16;

/// Total number of tiles contained in a chunk.
pub const CHUNK_TILE_COUNT: usize = CHUNK_WIDTH as usize * CHUNK_HEIGHT as usize;

pub trait Map {
    /// Fetch the tile at the given tile coordinates assuming it is in a chunk that is already loaded.
    fn loaded_tile_at(&self, coords: TileCoords) -> Option<&Tile> {
        let chunk = self.loaded_chunk_at(coords.as_chunk_coords())?;
        Some(chunk.tile_at_offset(coords.as_chunk_offset_coords()))
    }

    /// Change the tile at the specified tile coordinates assuming it is in a chunk that is already loaded.
    fn set_loaded_tile_at(&mut self, coords: TileCoords, tile: Tile) -> bool {
        if let Some(chunk) = self.loaded_chunk_at_mut(coords.as_chunk_coords()) {
            chunk.set_tile_at_offset(coords.as_chunk_offset_coords(), tile);
            true
        }
        else {
            false
        }
    }

    fn is_tile_loaded(&self, coords: TileCoords) -> bool { self.loaded_chunk_at(coords.as_chunk_coords()).is_some() }

    fn is_chunk_loaded(&self, coords: ChunkCoords) -> bool { self.loaded_chunk_at(coords).is_some() }

    /// Return the loaded chunk at the given chunk coordinates as an optional immutable reference.
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk>;

    /// Return the loaded chunk at the given chunk coordinates as a optional mutable reference.
    fn loaded_chunk_at_mut(&mut self, coords: ChunkCoords) -> Option<&mut Chunk>;

    /// Have this map include the given chunk in its collection of loaded chunks.
    fn provide_chunk(&mut self, coords: ChunkCoords, chunk: Chunk);

    /// Return the entity with the specified ID as an optional reference.
    fn entity_by_id(&self, id: Id) -> Option<&Entity>;

    /// Return the entity with the specified ID as an optional mutable reference.
    fn entity_by_id_mut(&mut self, id: Id) -> Option<&mut Entity>;

    /// Add an entity to the map. On client side this method is used to add all entities not controlled by the client
    /// (i.e. both players and AI-controlled entities) while on the server side this method is used to add all
    /// player-controlled entities (a separate system is used to manage AI-controled entities).
    fn add_entity(&mut self, id: Id, entity: Entity);

    fn remove_entity(&mut self, id: Id) -> Option<Entity>;
}

/// Type alias for a hash map that maps chunk coordinates to chunks.
pub type Chunks = HashMap<ChunkCoords, Chunk>;

/// Area of tiles on a map. As maps are infinite, chunks are generated, loaded, and unloaded dynamically as necessary.
#[derive(Serialize, Deserialize, Clone)]
pub struct Chunk {
    /// The tiles that this chunk is comprised of.
    #[serde(with = "BigArray")]
    tiles: [Tile; CHUNK_TILE_COUNT]
}

impl Chunk {
    pub fn new(tiles: [Tile; CHUNK_TILE_COUNT]) -> Self { Chunk { tiles } }

    pub fn tile_at_offset(&self, offset: OffsetCoords) -> &Tile { &self.tiles[offset.calculate_index()] }

    pub fn set_tile_at_offset(&mut self, offset: OffsetCoords, tile: Tile) {
        self.tiles[offset.calculate_index()] = tile;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Tile {
    Ground
}

impl Tile {
    pub fn is_blocking(&self) -> bool {
        match self {
            Tile::Ground => false
        }
    }
}

impl Default for Tile {
    fn default() -> Self { Tile::Ground }
}
