pub mod entities;
pub mod rendering;

use std::collections::{HashMap, HashSet};

use shared::{
    maps::{entities::Entities, Chunk, ChunkCoords, Chunks, Map, Tile, TileCoords},
    messages
};

use crate::networking::{self, Connection, ConnectionTrait};

pub struct ClientMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks,
    /// Set of coordinate pairs for the chunks that are needed (i.e. chunks that are not already loaded but were needed
    /// to fulfill a call to [`chunk_at`] or [`tile_at`]). When a needed chunk is requested from the sever then its
    /// coordinates are added to the [`requested_chunks`] set. A chunks's coordinates are not removed from this set
    /// until the chunk itself is actually recevied.
    needed_chunks: HashSet<ChunkCoords>,
    /// Set of coordinate pairs for chunks that have been requested from the server but have not yet been received. A
    /// chunks's coordinates are remove from both this set and [`needed_chunks`] when the chunk itself is received from
    /// the server.
    requested_chunks: HashSet<ChunkCoords>,
    /// All entities (except this client's player entity) that are on this map and within currently loaded chunks.
    entities: Entities
}

impl ClientMap {
    pub fn new() -> Self {
        ClientMap {
            loaded_chunks: HashMap::new(),
            needed_chunks: HashSet::new(),
            requested_chunks: HashSet::new(),
            entities: HashMap::new()
        }
    }

    /// Attempt to get the tile at the specified tile coordinates.
    pub fn tile_at(&mut self, coords: TileCoords) -> Option<&Tile> {
        if !self.is_tile_loaded(coords) {
            let chunk_coords = coords.as_chunk_coords();
            let was_not_present = self.needed_chunks.insert(chunk_coords);

            if was_not_present {
                log::trace!(
                    "Added chunk at {} to list of needed chunks as it contained requested tile at {}",
                    chunk_coords,
                    coords
                );
            }
        }

        self.loaded_tile_at(coords)
    }

    pub fn request_needed_chunks_from_server(&mut self, ws: &mut Connection) -> networking::Result<()> {
        for coords in &self.needed_chunks {
            if !self.requested_chunks.contains(coords) {
                ws.send(&messages::ToServer::RequestChunk(*coords))?;
                self.requested_chunks.insert(*coords);
            }
        }

        Ok(())
    }

    pub fn is_position_free(&mut self, coords: TileCoords) -> bool {
        let tile_blocking = self.tile_at(coords).map_or(true, |tile| tile.is_blocking());
        let entity_blocking = false; // TODO

        !tile_blocking && !entity_blocking
    }

    pub fn apply_modification(&mut self, modification: messages::MapModification) {
        match modification {
            messages::MapModification::TileChanged { pos, change_to } => {
                if self.is_tile_loaded(pos) {
                    self.set_loaded_tile_at(pos, change_to);
                }
                else {
                    log::warn!("Told by server to change tile at {} to {:?} yet the chunk that tile is contained in is not loaded", pos, change_to);
                }
            }

            messages::MapModification::EntityMoved => {
                unimplemented!()
            }
        }
    }
}

impl Map for ClientMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> { self.loaded_chunks.get(&coords) }

    fn loaded_chunk_at_mut(&mut self, coords: ChunkCoords) -> Option<&mut Chunk> { self.loaded_chunks.get_mut(&coords) }

    fn provide_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) {
        // TODO: Unload chunk(s) should too many be loaded already?

        self.needed_chunks.remove(&coords);
        self.requested_chunks.remove(&coords);

        self.loaded_chunks.insert(coords, chunk);
    }
}
