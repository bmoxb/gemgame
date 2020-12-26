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

    pub fn tile_at(&self, pos: TileCoords, connection: &mut networking::Connection) -> networking::Result<Option<&Tile>> {
        let tile_option = self.loaded_tile_at(&pos);

        if tile_option.is_none() { request_chunk(pos.as_chunk_coords(), connection)? }

        Ok(tile_option)
    }

    pub fn chunk_at(&self, pos: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<Option<&Chunk>> {
        let chunk_option = self.loaded_chunk_at(&pos);

        if chunk_option.is_none() { request_chunk(pos, connection)? }

        Ok(chunk_option)
    }

    pub fn provide_chunk(&mut self, pos: ChunkCoords, chunk: Chunk, connection: &mut networking::Connection) -> networking::Result<()> {
        // TODO: Unload chunk(s) should too many be loaded already.

        log::debug!("Loaded chunk: {}", pos);
        self.loaded_chunks.insert(pos, chunk);

        Ok(())
    }

    fn unload_chunk(&mut self, pos: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<()> {
        if let Some(_) = self.loaded_chunks.remove(&pos) {
            log::debug!("Unloaded chunk: {}", pos);

            connection.send(&messages::ToServer::ChunkUnloadedLocally(pos))?;
        }
        Ok(())
    }
}

impl Map for ClientMap {
    fn loaded_chunk_at(&self, pos: &ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(pos)
    }
}

fn request_chunk(pos: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<()> {
    connection.send(&messages::ToServer::RequestChunk(pos))
}