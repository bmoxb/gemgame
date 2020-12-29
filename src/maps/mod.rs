pub mod coords;
pub use coords::*;

use std::{ cmp, time, collections::HashMap };

use serde::{ Serialize, Deserialize };

use serde_big_array::big_array;

// TODO: Remove this workaround when const generics are properly stablised.
big_array! { BigArray; }

/// How many tiles wide a chunk is.
const CHUNK_WIDTH: i32 = 16;
/// How many tiles high a chunk is.
const CHUNK_HEIGHT: i32 = 16;
/// Total number of tiles contained in a chunk.
const CHUNK_TILE_COUNT: usize = CHUNK_WIDTH as usize * CHUNK_HEIGHT as usize;

pub trait Map {
    /// Fetch the tile at the given tile coordinates assuming it is in a chunk
    /// that is already loaded.
    fn loaded_tile_at(&self, coords: TileCoords) -> Option<&Tile> {
        let chunk = self.loaded_chunk_at(coords.as_chunk_coords())?;
        Some(chunk.tile_at_offset(coords.as_chunk_offset_coords()))
    }

    fn is_tile_loaded(&self, coords: TileCoords) -> bool {
        self.loaded_chunk_at(coords.as_chunk_coords()).is_some()
    }

    /// Returns the chunk at the given chunk coordinates assuming it is already
    /// loaded.
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk>;

    fn is_chunk_loaded(&self, coords: ChunkCoords) -> bool {
        self.loaded_chunk_at(coords).is_some()
    }
}

/// Type alias for a hash map that maps chunk coordinates to chunks.
pub type Chunks = HashMap<ChunkCoords, Chunk>;

/// 1Area of tiles on a map. As maps are infinite, chunks are generated, loaded,
/// and unloaded dynamically as necessary.
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    /// The tiles that this chunk is comprised of.
    #[serde(with = "BigArray")]
    tiles: [Tile; CHUNK_TILE_COUNT],

    /// The instant at which this chunk was last used.
    #[serde(skip, default = "time::Instant::now")]
    last_access_instant: time::Instant
}

impl Chunk {
    fn tile_at_offset(&self, mut offset: OffsetCoords) -> &Tile {
        // Ensure offset coordinates are within the chunk's bounds:
        offset.x = cmp::max(0, cmp::min(offset.x, CHUNK_WIDTH as u8 - 1));
        offset.y = cmp::max(0, cmp::min(offset.y, CHUNK_HEIGHT as u8 - 1));

        &self.tiles[(offset.y as i32 * CHUNK_WIDTH + offset.x as i32) as usize]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Tile {
    Ground
}

impl Default for Tile {
    fn default() -> Self { Tile::Ground }
}