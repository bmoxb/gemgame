use std::{ cmp, time, collections::HashMap };

use serde::{ Serialize, Deserialize };

use serde_big_array::big_array;

// TODO: Remove this workaround when const generics are properly stablised.
big_array! { BigArray; }

/// Type alias for a coordinate type.
pub type Coord = i32;

/// How many tiles wide a chunk is.
const CHUNK_WIDTH: Coord = 8;
/// How many tiles high a chunk is.
const CHUNK_HEIGHT: Coord = 8;
/// Total number of tiles contained in a chunk.
const CHUNK_TILE_COUNT: usize = (CHUNK_WIDTH * CHUNK_HEIGHT) as usize;

pub trait Map {
    fn tile_at(&mut self, x: Coord, y: Coord) -> &Tile {
        let (chunk_x, chunk_y) = tile_coords_to_chunk_coords(x, y);
        let (offset_x, offset_y) = tile_coords_to_chunk_offset_coords(x, y);

        let chunk = self.chunk_at(chunk_x, chunk_y);
        chunk.tile_at_offset(offset_x, offset_y)
    }

    //fn set_tile_at(&mut self, x: Coord, y: Coord, tile: Tile);

    /// Returns the chunk at the given chunk coordinates. This method is not
    /// implemented by default as the means by which a chunk is obtained differs
    /// between client and server: on the client side a chunk is obtained either
    /// from the local cache of chunks or by fetching it from the sever, while
    /// on the sever side chunks are read from disk or newly generated.
    fn chunk_at(&mut self, chunk_x: Coord, chunk_y: Coord) -> &Chunk;
}

/// Type alias for a hash map that maps chunk coordinates to chunks.
pub type Chunks = HashMap<(Coord, Coord), Chunk>;

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
    fn tile_at_offset(&self, mut offset_x: Coord, mut offset_y: Coord) -> &Tile {
        // Ensure offset coordinates are within the chunk's bounds:
        offset_x = cmp::max(0, cmp::min(offset_x, CHUNK_WIDTH - 1));
        offset_y = cmp::max(0, cmp::min(offset_y, CHUNK_HEIGHT - 1));

        &self.tiles[(offset_y * CHUNK_WIDTH + offset_x) as usize]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Tile {
    Ground
}

impl Default for Tile {
    fn default() -> Self { Tile::Ground }
}

/// Identify the coordinates of the chunk that the specified tile would be found
/// in.
const fn tile_coords_to_chunk_coords(x: Coord, y: Coord) -> (Coord, Coord) {
    let chunk_x = x / CHUNK_WIDTH;
    let chunk_y = y / CHUNK_HEIGHT;
    (
        if x >= 0 || x % CHUNK_WIDTH == 0 { chunk_x } else { chunk_x - 1 },
        if y >= 0 || y % CHUNK_HEIGHT == 0 { chunk_y } else { chunk_y - 1 }
    )
}

/// Identify the offset from its containing chunk that the specified tile would
/// be found at.
const fn tile_coords_to_chunk_offset_coords(x: Coord, y: Coord) -> (Coord, Coord) {
    let offset_x = x % CHUNK_WIDTH;
    let offset_y = y % CHUNK_HEIGHT;
    (
        if x >= 0 || offset_x == 0 { offset_x } else { CHUNK_WIDTH + offset_x },
        if y >= 0 || offset_y == 0 { offset_y } else { CHUNK_HEIGHT + offset_y }
    )
}



#[cfg(test)]
mod test {
    #[test]
    fn tile_coords_to_chunk_coords() {
        let test_data = &[
            ((0, 0), (0, 0)),
            ((12, -14), (0, -1)),
            ((-14, 14), (-1, 0)),
            ((-3, -2), (-1, -1)),
            ((-34, -19), (-3, -2)),
            ((-16, -17), (-1, -2)),
            ((-33, -32), (-3, -2))
        ];
        for ((in_x, in_y), out) in test_data {
            assert_eq!(super::tile_coords_to_chunk_coords(*in_x, *in_y), *out);
        }
    }

    #[test]
    fn tile_coords_to_chunk_offset_coords() {
        let test_data = &[
            ((0, 0), (0, 0)),
            ((8, 6), (8, 6)),
            ((12, -14), (12, 2)),
            ((-13, 14), (3, 14)),
            ((-3, -2), (13, 14)),
            ((-34, -19), (14, 13)),
            ((-16, -17), (0, 15)),
            ((-33, -32), (15, 0))
        ];
        for ((in_x, in_y), out) in test_data {
            assert_eq!(super::tile_coords_to_chunk_offset_coords(*in_x, *in_y), *out);
        }
    }
}