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
    fn loaded_tile_at(&self, pos: &TileCoords) -> Option<&Tile> {
        let chunk = self.loaded_chunk_at(&pos.as_chunk_coords())?;
        Some(chunk.tile_at_offset(pos.as_chunk_offset_coords()))
    }

    /// Returns the chunk at the given chunk coordinates. This method is not
    /// implemented by default as the means by which a chunk is obtained differs
    /// between client and server: on the client side a chunk is obtained either
    /// from the local cache of chunks or by fetching it from the sever, while
    /// on the sever side chunks are read from disk or newly generated.
    fn loaded_chunk_at(&self, pos: &ChunkCoords) -> Option<&Chunk>;
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