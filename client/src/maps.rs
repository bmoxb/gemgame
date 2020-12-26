use crate::networking::{ self, ConnectionTrait };

use core::{
    messages,
    maps::{ Map, Tile, Chunk, Chunks, ChunkCoords, TileCoords }
};

struct ClientMap {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks

    // socket connection, etc.
}

impl ClientMap {
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
}

impl Map for ClientMap {
    fn loaded_chunk_at(&self, pos: &ChunkCoords) -> Option<&Chunk> {
        self.loaded_chunks.get(pos)
    }
}

fn request_chunk(pos: ChunkCoords, connection: &mut networking::Connection) -> networking::Result<()> {
    connection.send(&messages::ToServer::RequestChunk(pos))
}