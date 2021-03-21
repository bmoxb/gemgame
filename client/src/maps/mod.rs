pub mod entities;

use std::collections::HashMap;

use shared::{
    maps::{
        entities::{Entities, Entity},
        Chunk, ChunkCoords, Chunks, Map, TileCoords
    },
    Id
};

use super::rendering;

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

    pub fn set_remote_entity_position(
        &mut self, id: Id, new_pos: TileCoords, renderer: &mut rendering::maps::Renderer
    ) {
        if let Some(entity) = self.entities.get_mut(&id) {
            // Update renderer:
            renderer.remote_entity_moved(id, entity.pos, new_pos, entity.movement_time());

            // Set position:
            entity.pos = new_pos;
        }
        else {
            log::warn!("Cannot set position of entity {} as it is not loaded", id);
        }
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

    fn add_entity(&mut self, id: Id, entity: Entity) {
        self.entities.insert(id, entity);
        log::info!("Entity with ID {} added to game map", id);
    }

    fn remove_entity(&mut self, id: Id) -> Option<Entity> {
        log::info!("Removing entity with ID {} from game map", id);
        self.entities.remove(&id)
    }
}
