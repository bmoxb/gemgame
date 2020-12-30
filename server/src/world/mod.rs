mod maps;

use std::{ io, fs, path::PathBuf, collections::HashMap };

use serde::{ Serialize, Deserialize };

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: HashMap<String, maps::ServerMap>

    //players: Vec<entities::Entity>
}

impl World {
    /// Create a new game world instance. Any world data that already exists at
    /// the given path will not be removed.
    ///
    /// This function does not need to be asynchronous as it is only to be run
    /// before beginning to listen for incoming connections.
    pub fn new(directory: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(directory)?;

        Ok(World {
            directory,
            loaded_maps: HashMap::new()
        })
    }

    pub async fn load_map(&mut self, title: String) -> Option<()> { // TODO: Use Result type.
        let map = maps::ServerMap::load(self.directory.clone()).await?;

        // TODO: Check if already loaded?
        self.loaded_maps.insert(title, map);

        Some(())
    }
}