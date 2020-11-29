use std::{ path::PathBuf, collections::HashMap };

use super::{ Coord, entities::Entity };

const CHUNK_WIDTH: Coord = 16;
const CHUNK_HEIGHT: Coord = 16;

pub struct Map {
    /// The currently loaded chunks that this map is comprised of mapped to
    /// chunk coordinates.
    loaded_chunks: HashMap<(Coord, Coord), Chunk>,

    /// Path to the directory containing map data.
    directory: PathBuf,

    /// Entities currently on this map.
    entities: Vec<Entity>
}

impl Map {
    /// Get a reference to the tile at the given coordinates.
    pub fn at(&self, x: Coord, y: Coord) -> &Tile {
        let chunk = self.chunk_at(x, y);

        let (offset_x, offset_y) = tile_coords_to_chunk_offset_coords(x, y);

        &chunk.at_offset(offset_x, offset_y)
    }

    /// Returns the map chunk at the given tile coordinates. If a chunk at those
    /// coordinates is not loaded, then the chunk will be read from disk. If
    /// chunk data does not exist then a new chunk is created.
    fn chunk_at(&self, x: Coord, y: Coord) -> &Chunk {
        let chunk_coords = tile_coords_to_chunk_coords(x, y);

        match self.loaded_chunks.get(&chunk_coords) {
            Some(chunk) => chunk,

            None => {
                unimplemented!()
            }
        }
    }
}

pub struct Chunk {
    /// The tiles that this chunk is comprised of.
    tiles: [Tile; (CHUNK_WIDTH * CHUNK_HEIGHT) as usize],
    /// How long this chunk has been loaded for in milliseconds.
    lifetime: f32
}

impl Chunk {
    fn update(&mut self, delta: f32) {
        self.lifetime += delta;
    }

    fn at_offset(&self, mut x: Coord, mut y: Coord) -> &Tile {
        if x < 0 || x >= CHUNK_WIDTH {
            log::warn!("Chunk x-offset is out of bounds: {}", x);
            x = 0;
        }
        if y < 0 || y >= CHUNK_HEIGHT {
            log::warn!("Chunk y-offset is out of bounds: {}", y);
            y = 0;
        }

        &self.tiles[(y * CHUNK_WIDTH + x) as usize]
    }
}

fn tile_coords_to_chunk_coords(x: Coord, y: Coord) -> (Coord, Coord) {
    let chunk_x = x / CHUNK_WIDTH;
    let chunk_y = y / CHUNK_HEIGHT;

    (
        if x >= 0 { chunk_x } else { chunk_x - 1 },
        if y >= 0 { chunk_y } else { chunk_y - 1 }
    )
}

fn tile_coords_to_chunk_offset_coords(x: Coord, y: Coord) -> (Coord, Coord) {
    let offset_x = x % CHUNK_WIDTH;
    let offset_y = y % CHUNK_HEIGHT;

    (
        if x >= 0 { offset_x } else { CHUNK_WIDTH + offset_x },
        if y >= 0 { offset_y } else { CHUNK_HEIGHT + offset_y }
    )
}

pub struct Tile {
    tile_type: TileType,
    blocking: bool
}

enum TileType {}

trait Generator {
    fn generate(&self, seed: u32) -> Map;
}

struct OverworldGenerator {}
//impl Generator for OverworldGenerator {}

#[cfg(test)]
mod test {
    #[test]
    fn tile_coords_to_chunk_coords() {
        assert_eq!(
            super::tile_coords_to_chunk_coords(0, 0),
            (0, 0)
        );

        assert_eq!(
            super::tile_coords_to_chunk_coords(8, 6),
            (0, 0)
        );

        assert_eq!(
            super::tile_coords_to_chunk_coords(12, -14),
            (0, -1)
        );

        assert_eq!(
            super::tile_coords_to_chunk_coords(-13, 14),
            (-1, 0)
        );

        assert_eq!(
            super::tile_coords_to_chunk_coords(-3, -2),
            (-1, -1)
        );

        assert_eq!(
            super::tile_coords_to_chunk_coords(-34, -19),
            (-3, -2)
        );
    }

    #[test]
    fn tile_coords_to_chunk_offset_coords() {
        assert_eq!(
            super::tile_coords_to_chunk_offset_coords(0, 0),
            (0, 0)
        );

        assert_eq!(
            super::tile_coords_to_chunk_offset_coords(8, 6),
            (8, 6)
        );

        assert_eq!(
            super::tile_coords_to_chunk_offset_coords(12, -14),
            (12, 2)
        );

        assert_eq!(
            super::tile_coords_to_chunk_offset_coords(-13, 14),
            (3, 14)
        );

        assert_eq!(
            super::tile_coords_to_chunk_offset_coords(-3, -2),
            (13, 14)
        );

        assert_eq!(
            super::tile_coords_to_chunk_offset_coords(-34, -19),
            (14, 13)
        );
    }
}