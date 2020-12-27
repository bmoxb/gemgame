use super::{ CHUNK_WIDTH, CHUNK_HEIGHT };

use std::{ fmt, hash::Hash };

use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct TileCoords { pub x: i32, pub y: i32 }

impl TileCoords {
    /// Identify the coordinates of the chunk that the tile at these tile
    /// coordinates would be found in.
    pub fn as_chunk_coords(&self) -> ChunkCoords {
        let chunk_x = self.x / CHUNK_WIDTH;
        let chunk_y = self.y / CHUNK_HEIGHT;

        ChunkCoords {
            x: if self.x >= 0 || self.x % CHUNK_WIDTH == 0 { chunk_x } else { chunk_x - 1 },
            y: if self.y >= 0 || self.y % CHUNK_HEIGHT == 0 { chunk_y } else { chunk_y - 1 }
        }
    }

    /// Identify the offset from its containing chunk that the specified tile
    /// would be found at.
    pub fn as_chunk_offset_coords(&self) -> OffsetCoords {
        let offset_x = self.x % CHUNK_WIDTH;
        let offset_y = self.y % CHUNK_HEIGHT;

        OffsetCoords {
            x: (if self.x >= 0 || offset_x == 0 { offset_x } else { CHUNK_WIDTH + offset_x }) as u8,
            y: (if self.y >= 0 || offset_y == 0 { offset_y } else { CHUNK_HEIGHT + offset_y }) as u8
        }
    }
}

impl fmt::Display for TileCoords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "tile coordinates ({}, {})", self.x, self.y)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoords { pub x: i32, pub y: i32 }

impl fmt::Display for ChunkCoords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "chunk coordinates ({}, {})", self.x, self.y)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OffsetCoords { pub x: u8, pub y: u8 }

impl fmt::Display for OffsetCoords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "chunk offset coordinates ({}, {})", self.x, self.y)
    }
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