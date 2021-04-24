pub mod coords;
pub mod entities;

use std::collections::HashMap;

pub use coords::*;
use entities::Entity;
use serde::{Deserialize, Serialize};
use serde_big_array::big_array;

use crate::{
    gems::{Gem, self},
    Id
};

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
    fn loaded_tile_at(&self, coords: TileCoords) -> Option<Tile> {
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

    fn is_tile_loaded(&self, coords: TileCoords) -> bool {
        self.loaded_chunk_at(coords.as_chunk_coords()).is_some()
    }

    fn is_chunk_loaded(&self, coords: ChunkCoords) -> bool {
        self.loaded_chunk_at(coords).is_some()
    }

    fn is_position_free(&self, coords: TileCoords) -> bool {
        !self.is_blocking_tile_at(coords) && !self.is_blocking_entity_at(coords)
    }

    fn is_blocking_tile_at(&self, coords: TileCoords) -> bool {
        self.loaded_tile_at(coords).map(|tile| tile.is_blocking()).unwrap_or(true)
    }

    fn is_blocking_entity_at(&self, coords: TileCoords) -> bool;

    /// Return the loaded chunk at the given chunk coordinates as an optional immutable reference.
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk>;

    /// Return the loaded chunk at the given chunk coordinates as a optional mutable reference.
    fn loaded_chunk_at_mut(&mut self, coords: ChunkCoords) -> Option<&mut Chunk>;

    /// Have this map include the given chunk in its collection of loaded chunks.
    fn add_chunk(&mut self, coords: ChunkCoords, chunk: Chunk);

    fn remove_chunk(&mut self, coords: ChunkCoords) -> Option<Chunk>;

    /// Return the entity with the specified ID as an optional immutable reference.
    fn entity_by_id(&self, id: Id) -> Option<&Entity>;

    /// Return the entity with the specified ID as an optional mutable reference.
    fn entity_by_id_mut(&mut self, id: Id) -> Option<&mut Entity>;

    /// Add an entity to the map. On client side this method is used to add all entities not controlled by the client
    /// (i.e. both players and AI-controlled entities) while on the server side this method is used to add all
    /// player-controlled entities (a separate system is used to manage AI-controlled entities).
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
    pub fn tile_at_offset(&self, offset: OffsetCoords) -> Tile {
        self.tiles[offset.calculate_index()]
    }

    pub fn set_tile_at_offset(&mut self, offset: OffsetCoords, tile: Tile) {
        self.tiles[offset.calculate_index()] = tile;
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk { tiles: [Tile::default(); CHUNK_TILE_COUNT] }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    Grass,
    FlowerPatch,
    Stones,
    Dirt,
    DirtGrassTop,
    DirtGrassBottom,
    DirtGrassLeft,
    DirtGrassRight,
    DirtGrassTopLeft,
    DirtGrassTopRight,
    DirtGrassBottomLeft,
    DirtGrassBottomRight,
    DirtGrassCornerTopLeft,
    DirtGrassCornerTopRight,
    DirtGrassCornerBottomLeft,
    DirtGrassCornerBottomRight,
    Rock,
    RockEmerald,
    RockRuby,
    RockDiamond,
    RockSmashed,
    Shrub,
    FlowerBlue,
    FlowersYellowOrange,
    Water,
    WaterGrassTop,
    WaterGrassBottom,
    WaterGrassLeft,
    WaterGrassRight,
    WaterGrassTopLeft,
    WaterGrassTopRight,
    WaterGrassBottomLeft,
    WaterGrassBottomRight,
    WaterGrassCornerTopLeft,
    WaterGrassCornerTopRight,
    WaterGrassCornerBottomLeft,
    WaterGrassCornerBottomRight
}

impl Tile {
    /// Returns `true` should entities be unable to walk over this tile.
    pub fn is_blocking(&self) -> bool {
        matches!(
            self,
            Tile::Stones
                | Tile::Shrub
                | Tile::Water
                | Tile::WaterGrassTop
                | Tile::WaterGrassCornerTopLeft
                | Tile::WaterGrassCornerTopRight
        )
    }

    /// Returns `true` for a tile that should become [`Tile::RockSmashed`] when an entity walks over it.
    pub fn is_smashable(&self) -> bool {
        matches!(self, Tile::Rock | Tile::RockEmerald | Tile::RockRuby | Tile::RockDiamond)
    }

    pub fn is_grassy(&self) -> bool {
        matches!(self, Tile::Grass | Tile::FlowerPatch | Tile::FlowerBlue | Tile::FlowersYellowOrange)
    }

    pub fn get_entity_movement_frame_changes(&self) -> usize {
        if self.is_smashable() {
            8
        }
        else {
            1
        }
    }

    /// Returns the gem yield for a smashable tile (except [`Tile::Rock`] which is smashable but does not yield any
    /// gems).
    pub fn get_gem_yield(&self) -> Option<gems::Yield> {
        match self {
            Tile::RockEmerald => Some(gems::Yield { gem: Gem::Emerald, minimum_quantity: 3, maximum_quantity: 5 }),
            Tile::RockRuby => Some(gems::Yield { gem: Gem::Ruby, minimum_quantity: 1, maximum_quantity: 3 }),
            Tile::RockDiamond => Some(gems::Yield { gem: Gem::Diamond, minimum_quantity: 1, maximum_quantity: 1 }),
            _ => None
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Grass
    }
}
