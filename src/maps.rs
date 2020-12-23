use std::{ cmp, time, collections::HashMap };

use serde::{ Serialize, Deserialize };

use serde_big_array::big_array;

// TODO: Remove this workaround when const generics are properly stablised.
big_array! { BigArray; }

type Coord = i32;

/// How many tiles wide a chunk is.
const CHUNK_WIDTH: Coord = 16;
/// How many tiles high a chunk is.
const CHUNK_HEIGHT: Coord = 16;
/// Total number of tiles contained in a chunk.
const CHUNK_TILE_COUNT: usize = CHUNK_WIDTH as usize * CHUNK_HEIGHT as usize;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TileCoords { x: Coord, y: Coord }

impl  TileCoords {
    /// Identify the coordinates of the chunk that the tile at these tile
    /// coordinates would be found in.
    fn as_chunk_coords(&self) -> ChunkCoords {
        let chunk_x = self.x / CHUNK_WIDTH;
        let chunk_y = self.y / CHUNK_HEIGHT;

        ChunkCoords {
            x: if self.x >= 0 || self.x % CHUNK_WIDTH == 0 { chunk_x } else { chunk_x - 1 },
            y: if self.y >= 0 || self.y % CHUNK_HEIGHT == 0 { chunk_y } else { chunk_y - 1 }
        }
    }

    /// Identify the offset from its containing chunk that the specified tile
    /// would be found at.
    fn as_chunk_offset_coords(&self) -> OffsetCoords {
        let offset_x = self.x % CHUNK_WIDTH;
        let offset_y = self.y % CHUNK_HEIGHT;

        OffsetCoords {
            x: if self.x >= 0 || offset_x == 0 { offset_x } else { CHUNK_WIDTH + offset_x },
            y: if self.y >= 0 || offset_y == 0 { offset_y } else { CHUNK_HEIGHT + offset_y }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChunkCoords { x: Coord, y: Coord }

#[derive(Debug, Clone, PartialEq)]
pub struct OffsetCoords { x: Coord, y: Coord }

pub trait Map {
    fn loaded_tile_at(&mut self, pos: TileCoords) -> Option<&Tile> {
        let chunk = self.loaded_chunk_at(pos.as_chunk_coords())?;
        Some(chunk.tile_at_offset(pos.as_chunk_offset_coords()))
    }

    /// Returns the chunk at the given chunk coordinates. This method is not
    /// implemented by default as the means by which a chunk is obtained differs
    /// between client and server: on the client side a chunk is obtained either
    /// from the local cache of chunks or by fetching it from the sever, while
    /// on the sever side chunks are read from disk or newly generated.
    fn loaded_chunk_at(&mut self, pos: ChunkCoords) -> Option<&Chunk>;
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
        offset.x = cmp::max(0, cmp::min(offset.x, CHUNK_WIDTH - 1));
        offset.y = cmp::max(0, cmp::min(offset.y, CHUNK_HEIGHT - 1));

        &self.tiles[(offset.y * CHUNK_WIDTH + offset.x) as usize]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Tile {
    Ground
}

impl Default for Tile {
    fn default() -> Self { Tile::Ground }
}



#[cfg(test)]
mod test {
    use super::{ TileCoords, ChunkCoords, OffsetCoords };

    #[test]
    fn tile_coords_to_chunk_coords() {
        let test_data = &[
            (TileCoords { x: 0, y: 0 }, ChunkCoords { x: 0, y: 0 }),
            (TileCoords { x: 12, y: -14 }, ChunkCoords { x: 0, y: -1 }),
            (TileCoords { x: -14, y: 14 }, ChunkCoords { x: -1, y: 0 }),
            (TileCoords { x: -3, y: -2 }, ChunkCoords { x: -1, y: -1 }),
            (TileCoords { x: -34, y: -19 }, ChunkCoords { x: -3, y: -2 }),
            (TileCoords { x: -16, y: -17 }, ChunkCoords { x: -1, y: -2 }),
            (TileCoords { x: -33, y: -32 }, ChunkCoords { x: -3, y: -2 })
        ];
        for (tile, chunk) in test_data {
            assert_eq!(tile.as_chunk_coords(), *chunk);
        }
    }

    #[test]
    fn tile_coords_to_chunk_offset_coords() {
        let test_data = &[
            (TileCoords { x: 0, y: 0 }, OffsetCoords { x: 0, y: 0 }),
            (TileCoords { x: 8, y: 6 }, OffsetCoords { x: 8, y: 6 }),
            (TileCoords { x: 12, y: -14 }, OffsetCoords { x: 12, y: 2 }),
            (TileCoords { x: -13, y: 14 }, OffsetCoords { x: 3, y: 14 }),
            (TileCoords { x: -3, y: -2 }, OffsetCoords { x: 13, y: 14 }),
            (TileCoords { x: -34, y: -19 }, OffsetCoords { x: 14, y: 13 }),
            (TileCoords { x: -16, y: -17 }, OffsetCoords { x: 0, y: 15 }),
            (TileCoords { x: -33, y: -32 }, OffsetCoords { x: 15, y: 0 })
        ];
        for (tile, offset) in test_data {
            assert_eq!(tile.as_chunk_offset_coords(), *offset);
        }
    }
}