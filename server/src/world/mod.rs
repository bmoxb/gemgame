pub mod maps;

use core::maps::ChunkCoords;
use std::{collections::HashMap, fs, io, path::PathBuf};

use crate::Shared;

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: HashMap<String, maps::ServerMap> //players: Vec<entities::Entity>
}

impl World {
    /// Create a new game world instance. Any world data that already exists at the given path will not be removed. This
    /// function does not need to be asynchronous as it is only to be run before beginning to listen for incoming
    /// connections.
    pub fn new(directory: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&directory)?;

        let mut x = World { directory, loaded_maps: HashMap::new() };

        // TODO: Temporary:
        x.loaded_maps.insert(
            "surface".to_string(),
            maps::ServerMap::new(x.directory.join("surface"), Box::new(maps::generators::DefaultGenerator), 0).unwrap()
        );

        Ok(x)
    }

    pub fn get_map_mut(&mut self, title: &str) -> Option<&mut maps::ServerMap> { self.loaded_maps.get_mut(title) }
}
