use crate::networking::{ self, ConnectionTrait };

use core::{
    messages,
    maps::{ Map, Tile, Chunk, Chunks, ChunkCoords, TileCoords }
};

use std::collections::HashMap;

pub struct ClientMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks
}

impl ClientMap {
    pub fn new() -> Self {
        ClientMap { loaded_chunks: HashMap::new() }
    }

    /// Attempt to get the tile at the specified tile coordinates. This method
    /// will return `Ok(Some(...))` should the chunk that the desired tile is in
    /// be already loaded. If it is not loaded, the necessary chunk will be
    /// requested from the server and either `Ok(None)` or `Err(...)` will be
    /// returned depending on whether or not the chunk request message could be
    /// sent successfully.
    pub fn tile_at(&self, coords: TileCoords, connection: &mut networking::Connection) -> networking::Result<Option<&Tile>> {
        let tile_option = self.loaded_tile_at(coords);

        if tile_option.is_none() { request_chunk(coords.as_chunk_coords(), connection)? }

        Ok(tile_option)
    }

    pub fn chunk_at(&self, coords: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<Option<&Chunk>> {
        let chunk_option = self.loaded_chunk_at(coords);

        if chunk_option.is_none() { request_chunk(coords, connection)? }

        Ok(chunk_option)
    }

    fn unload_chunk(&mut self, coords: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<()> {
        if let Some(_) = self.loaded_chunks.remove(&coords) {
            log::debug!("Unloaded chunk: {}", coords);

            connection.send(&messages::ToServer::ChunkUnloadedLocally(coords))?;
        }

        Ok(())
    }
}

impl Map for ClientMap {
    fn loaded_chunk_at(&self, coords: ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(&coords)
    }

    fn provide_chunk(&mut self, coords: ChunkCoords, chunk: Chunk) {
        // TODO: Unload chunk(s) should too many be loaded already?

        self.loaded_chunks.insert(coords, chunk);
    }
}

/// Request the chunk at the specified coordinates from the server.
fn request_chunk(coords: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<()> {
    connection.send(&messages::ToServer::RequestChunk(coords))
}