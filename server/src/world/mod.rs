pub mod entities;
pub mod maps;

use entities::PlayerEntity;

use std::{collections::HashMap, fs, io, path::PathBuf};

use shared::{
    world::maps::{Tile, TileCoords},
    Id
};

pub struct World {
    /// Directory containing world data.
    directory: PathBuf,

    /// Currently-loaded maps.
    loaded_maps: HashMap<Id, maps::ServerMap>,

    /// Player-controlled entities mapped to entity IDs.
    player_entities: HashMap<Id, PlayerEntity>
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

    pub fn add_player_entity(&mut self, id: Id, entity: PlayerEntity) {
        log::debug!("Player entity with ID {} added to game world", id);
        self.player_entities.insert(id, entity);
    }

    pub fn player_entity_by_id(&mut self, id: Id) -> Option<&mut PlayerEntity> {
        self.player_entities.get_mut(&id)
    }

    pub fn remove_player_entity(&mut self, id: Id) -> Option<PlayerEntity> {
        let entity = self.player_entities.remove(&id);
        if entity.is_some() {
            log::debug!("Player entity with ID {} removed from game world", id);
        }
        entity
    }
}

/// Structure indicating a change made to the game world.
#[derive(Copy, Clone)]
pub struct Modification {
    /// The ID of the map affected by this modification.
    map_id: Id,
    mod_type: ModificationType
}

#[derive(Copy, Clone)]
pub enum ModificationType {
    TileChanged {
        /// Position of the tile tile to be modified.
        pos: TileCoords,
        /// What the tile at the specified coordinates should be changed to.
        change_to: Tile
    },

    EntityMoved
}
