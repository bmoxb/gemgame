mod maps;

use std::{ io, path::PathBuf };

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: Vec<maps::ServerMap>,

    //players: Vec<entities::Entity>
}

const MAPS_DIRECTORY: &'static str = "maps/";
const ENTITIES_DIRECTORY: &'static str = "entities/";

impl World {
    /// Create a new instance of a game world. Directories for storing world
    /// data will be created should they not already exist. Any existing world
    /// data will *not* be deleted.
    pub async fn new(directory: PathBuf) -> io::Result<Self> {
        tokio::fs::create_dir_all(directory.join(MAPS_DIRECTORY)).await?;
        tokio::fs::create_dir_all(directory.join(ENTITIES_DIRECTORY)).await?;

        Ok(World {
            directory,
            loaded_maps: Vec::new()
        })
    }
}