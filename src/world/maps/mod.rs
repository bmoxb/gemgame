pub mod generators;

use std::{ path::PathBuf, collections::HashMap, fs, fmt };

use super::{ Coord, entities::Entity, load_json };

use generators::Generator;

const CHUNK_WIDTH: Coord = 16;
const CHUNK_HEIGHT: Coord = 16;

const MAP_JSON_FILE: &'static str = "map.json";

pub struct Map {
    // Path to the directory containing map data.
    directory: PathBuf,

    /// The generator to be used when new chunks must be made.
    generator: Box<dyn Generator>,

    /// The currently loaded chunks that this map is comprised of mapped to
    /// chunk coordinates.
    loaded_chunks: HashMap<(Coord, Coord), Chunk>,

    /// Entities currently on this map.
    entities: Vec<Entity>
}

impl Map {
    /// Create a new map which will store its data to the specified directory
    /// and will be generated by the given generator.
    pub fn new(directory: PathBuf, generator: Box<dyn Generator>) -> Self {
        Map {
            directory, generator,
            loaded_chunks: HashMap::new(),
            entities: Vec::new()
        }
    }

    /// Attempt to load an existing map from the directory specified. This method
    /// relies on the [`load_json`] helper function.
    pub fn load(directory: PathBuf, seed: u32) -> Option<Self> {
        load_json("map", directory, MAP_JSON_FILE, |json, directory| {
            let generator_name = match json["generator"].as_str() {
                Some(value) => {
                    log::debug!("Generator name specified in JSON: {}", value);
                    value
                }
                None => {
                    log::warn!("Map '{}' does not have a generator specified - assuming 'surface' generator",
                                directory.display());
                    "surface"
                }
            };

            let generator = match generators::by_name(generator_name, seed) {
                Some(gen) => {
                    log::debug!("Generator specified: {}", gen.name());
                    gen
                }
                None => {
                    log::warn!("Map generator with name '{}' does not exist", generator_name);
                    Box::new(generators::SurfaceGenerator::new(seed))
                }
            };

            Map {
                directory, generator,
                loaded_chunks: HashMap::new(),
                entities: Vec::new()
            }
        })
    }

    /// Save this map to the filesystem. Will return `true` if able to save map
    /// data successfully.
    pub fn save(&self) -> bool {
        let map_file_path = self.directory.join(MAP_JSON_FILE);

        let data = serde_json::json!({
            "generator": self.generator.name()
        }).to_string();

        match fs::write(&map_file_path, data) {
            Ok(_) => {
                log::info!("Saved map: {}", self);
                true
            }

            Err(e) => {
                log::warn!("Failed to write map JSON file '{}' due to IO error: {}",
                           map_file_path.display(), e);
                false
            }
        }
    }

    /// Get a reference to the tile at the given coordinates. If the coordinates
    /// are for a tile in a chunk that has not been loaded, then it will be
    /// loaded. In the case of a chunk that has not yet been generated, it will
    /// be generated using this map's generator.
    pub fn tile_at(&mut self, x: Coord, y: Coord) -> &Tile {
        let chunk = self.chunk_at(x, y);

        let (offset_x, offset_y) = tile_coords_to_chunk_offset_coords(x, y);

        chunk.tile_at_offset(offset_x, offset_y)
    }

    /// Returns the map chunk at the given tile coordinates. If a chunk at those
    /// coordinates is not loaded, then the chunk will be read from disk. If
    /// chunk data does not exist then a new chunk is created.
    fn chunk_at(&mut self, x: Coord, y: Coord) -> &Chunk {
        let (chunk_x, chunk_y) = tile_coords_to_chunk_coords(x, y);

        if self.is_chunk_loaded(chunk_x, chunk_y) {
            log::debug!("Chunk ({}, {}) which contains tile at ({}, {}) is already loaded",
                        chunk_x, chunk_y, x, y);
        }
        else {
            if self.load_chunk(chunk_x, chunk_y) {
                log::debug!("Loaded chunk ({}, {}) as it contains requested tile ({}, {})",
                            chunk_x, chunk_y, x, y);
            }
            else {
                self.generate_and_load_chunk(chunk_x, chunk_y);
                log::info!("Generated chunk ({}, {})", chunk_x, chunk_y);
            }
        }

        self.get_loaded_chunk(chunk_x, chunk_y).unwrap()
    }

    /// Check if the chunk at the given chunk coordinates is loaded.
    fn is_chunk_loaded(&self, chunk_x: Coord, chunk_y: Coord) -> bool {
        self.loaded_chunks.contains_key(&(chunk_x, chunk_y))
    }

    /// Load the chunk at the given chunk coordinates by reading chunk data from
    /// the appropriate file. Will return `false` if the file containing the
    /// chunk data could not be found (likely implies chunk has not yet been
    /// generated).
    fn load_chunk(&mut self, chunk_x: Coord, chunk_y: Coord) -> bool {
        // TODO: Read chunk data from file.
        // self.loaded_chunks.insert((chunk_x, chunk_y), chunk);

        false
    }

    fn unload_chunk(&mut self, chunk_x: Coord, chunk_y: Coord) {
        // TODO: Save chunk data to file.

        self.loaded_chunks.remove(&(chunk_x, chunk_y));
    }

    /// Will generate a new chunk at the given chunk coordinates using this map's
    /// generator. The newly generated chunk will be inserted into the
    /// `self.loaded_chunks` but will not be saved to file until it is unloaded
    /// (see [`Self::unload_chunk`]).
    fn generate_and_load_chunk(&mut self, chunk_x: Coord, chunk_y: Coord) {
        let chunk = self.generator.generate(chunk_x, chunk_y);
        self.loaded_chunks.insert((chunk_x, chunk_y), chunk);
    }

    fn get_loaded_chunk(&self, chunk_x: Coord, chunk_y: Coord) -> Option<&Chunk> {
        self.loaded_chunks.get(&(chunk_x, chunk_y))
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}' (generator: {}, loaded chunks: {})", self.directory.display(),
               self.generator.name(), self.loaded_chunks.len())
    }
}

pub struct Chunk {
    /// The tiles that this chunk is comprised of.
    tiles: [Tile; (CHUNK_WIDTH * CHUNK_HEIGHT) as usize]
}

impl Chunk {
    fn new(tiles: [Tile; (CHUNK_WIDTH * CHUNK_HEIGHT) as usize]) -> Self {
        Chunk { tiles }
    }

    fn tile_at_offset(&self, mut x: Coord, mut y: Coord) -> &Tile {
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

#[cfg(test)]
mod test {
    #[test]
    fn tile_coords_to_chunk_coords() {
        assert_eq!(super::tile_coords_to_chunk_coords(0, 0), (0, 0));
        assert_eq!(super::tile_coords_to_chunk_coords(8, 6), (0, 0));
        assert_eq!(super::tile_coords_to_chunk_coords(12, -14), (0, -1));
        assert_eq!(super::tile_coords_to_chunk_coords(-13, 14), (-1, 0));
        assert_eq!(super::tile_coords_to_chunk_coords(-3, -2), (-1, -1));
        assert_eq!(super::tile_coords_to_chunk_coords(-34, -19), (-3, -2));
    }

    #[test]
    fn tile_coords_to_chunk_offset_coords() {
        assert_eq!(super::tile_coords_to_chunk_offset_coords(0, 0), (0, 0));
        assert_eq!(super::tile_coords_to_chunk_offset_coords(8, 6), (8, 6));
        assert_eq!(super::tile_coords_to_chunk_offset_coords(12, -14), (12, 2));
        assert_eq!(super::tile_coords_to_chunk_offset_coords(-13, 14), (3, 14));
        assert_eq!(super::tile_coords_to_chunk_offset_coords(-3, -2), (13, 14));
        assert_eq!(super::tile_coords_to_chunk_offset_coords(-34, -19), (14, 13));
    }
}