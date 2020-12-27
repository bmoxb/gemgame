mod maps;

use std::path::{ PathBuf };

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: Vec<maps::ServerMap>
}

impl World {
    pub fn load_or_new(directory: PathBuf) -> Self {
        //unimplemented!()
        World::new(directory)
    }

    pub fn load(directory: PathBuf) -> Option<Self> { unimplemented!() }

    pub fn new(directory: PathBuf) -> Self {
        World {
            directory,
            loaded_maps: Vec::new()
        }
    }
}