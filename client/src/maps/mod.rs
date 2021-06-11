pub mod entities;
pub mod rendering;

use std::collections::HashMap;

pub use rendering::MapRenderer;
use shared::{
    maps::{
        entities::{Direction, Entities, Entity},
        Chunk, ChunkCoords, Chunks, Map, Tile, TileCoords
    },
    Id
};

pub struct ClientMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,
    /// All entities (except this client's player entity) that are on this map and within currently loaded chunks.
    entities: Entities
}

impl ClientMap {
    pub fn new() -> Self {
        ClientMap { loaded_chunks: HashMap::new(), entities: HashMap::new() }
    }

    pub fn move_remote_entity(
        &mut self, id: Id, new_pos: TileCoords, direction: Direction, renderer: &mut MapRenderer
    ) {
        let dest_tile = self.loaded_tile_at(new_pos).unwrap_or_default();

        if let Some(entity) = self.entities.get_mut(&id) {
            // Update renderer:
            renderer.remote_entity_moved(
                id,
                new_pos,
                entity.movement_time(dest_tile),
                dest_tile.get_entity_movement_frame_changes()
            );

            // Set position & direction:
            entity.pos = new_pos;
            entity.direction = direction;
        }
        else {
            log::warn!("Cannot set position of entity {} as it is not loaded", id);
        }

        self.some_entity_moved_to(new_pos, renderer);
    }

    /// Handles the changing of certain tiles when entities walk over them (e.g. turning a rock tile into a smashed rock
    /// with an animated transition). Should be called whenever an entity (whether remote or the local player entity)
    /// moves.
    pub fn some_entity_moved_to(&mut self, pos: TileCoords, renderer: &mut MapRenderer) {
        let dest_tile_smashable = self.loaded_tile_at(pos).map(|t| t.is_smashable()).unwrap_or(false);

        if dest_tile_smashable {
            self.set_loaded_tile_at(pos, Tile::RockSmashed);
            renderer.rock_tile_smashed(pos);
        }
    }

    pub fn get_loaded_chunk_coords(&self) -> impl Iterator<Item = ChunkCoords> + '_ {
        self.loaded_chunks.keys().copied()
    }
}

impl Map for ClientMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(&coords)
    }

    fn loaded_chunk_at_mut(&mut self, coords: ChunkCoords) -> Option<&mut Chunk> {
        self.loaded_chunks.get_mut(&coords)
    }

    fn add_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) {
        self.loaded_chunks.insert(coords, chunk);
    }

    fn remove_chunk(&mut self, coords: ChunkCoords) -> Option<Chunk> {
        self.loaded_chunks.remove(&coords)
    }

    fn is_blocking_entity_at(&self, coords: TileCoords) -> bool {
        // Determining if there are blocking entities like this is O(n) so may need a better solution in future for
        // instances where many entities are together in a small area.

        self.entities.values().any(|entity| entity.pos == coords)
    }

    fn entity_by_id(&self, id: Id) -> Option<&Entity> {
        self.entities.get(&id)
    }

    fn entity_by_id_mut(&mut self, id: Id) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    fn add_entity(&mut self, id: Id, entity: Entity) {
        self.entities.insert(id, entity);
        log::info!("Entity with ID {} added to game map", id);
    }

    fn remove_entity(&mut self, id: Id) -> Option<Entity> {
        log::info!("Removing entity with ID {} from game map", id);
        self.entities.remove(&id)
    }
}
