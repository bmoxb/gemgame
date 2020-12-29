mod maps;

use std::{ io, path::PathBuf, collections::HashMap };

use serde::{ Serialize, Deserialize };

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: HashMap<String, maps::ServerMap>

    //players: Vec<entities::Entity>
}

#[derive(Serialize, Deserialize, Debug)]
struct WorldConfig { /* ... */ }

const MAPS_DIRECTORY: &'static str = "maps/";
const ENTITIES_DIRECTORY: &'static str = "entities/";

impl World {
    pub async fn load_or_new(directory: PathBuf) -> Option<Self> {
        unimplemented!()
    }

    /// Create a new game world.
    pub async fn new(directory: PathBuf) -> io::Result<Self> {
        tokio::fs::create_dir_all(directory.join(MAPS_DIRECTORY)).await?;
        tokio::fs::create_dir_all(directory.join(ENTITIES_DIRECTORY)).await?;

        Ok(World {
            directory,
            loaded_maps: HashMap::new()
        })
    }

    /// Load an existing game world.
    pub async fn load(directory: PathBuf) -> Option<Self> {
        unimplemented!()
    }

    pub async fn load_map(&mut self, title: String) -> Option<()> { // TODO: Use Result type.
        let map_directory = self.directory.join(MAPS_DIRECTORY).join(&title);
        let map = maps::ServerMap::load(map_directory).await?;

        // TODO: Check if already loaded?
        self.loaded_maps.insert(title, map);

        Some(())
    }
}