pub mod entities;
pub mod maps;

use std::{collections::HashMap, fs, io, path::PathBuf};

use shared::{
    world::{
        entities::Entities,
        maps::{Tile, TileCoords}
    },
    Id
};

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: HashMap<Id, maps::ServerMap>,

    /// Player-controlled entities mapped to entity IDs.
    player_entities: Entities
}

impl World {
    /// Create a new game world instance. Any world data that already exists at the given path will not be removed. This
    /// function does not need to be asynchronous as it is only to be run before beginning to listen for incoming
    /// connections.
    pub fn new(directory: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&directory)?;

        let mut x = World { directory, loaded_maps: HashMap::new(), player_entities: HashMap::new() };

        // TODO: Temporary:
        x.loaded_maps.insert(
            crate::id::generate_with_timestamp(),
            maps::ServerMap::new(x.directory.join("surface"), Box::new(maps::generators::DefaultGenerator), 0).unwrap()
        );

        Ok(x)
    }

    pub fn add_player_entity(&mut self) {}

    pub fn player_entity_by_id(&mut self) {}

    pub fn remove_player_entity(&mut self) {}
}

/// Structure indicating a change made to the game world.
#[derive(Copy, Clone)]
pub struct Modification {
    /// The ID of the map to be modified.
    map_id: Id,
    /// Position of the tile tile to be modified.
    pos: TileCoords,
    /// What the tile at the specified coordinates should be changed to.
    change_to: Tile
}
