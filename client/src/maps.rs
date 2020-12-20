use core::maps::{ Coord, Chunk, Chunks };

struct Map {
    /// Chunks that are currently loaded (mapped to by chunk coordinate pairs).
    loaded_chunks: Chunks

    // socket connection, etc.
}

impl core::maps::Map for Map {
    fn chunk_at(&mut self, chunk_x: Coord, chunk_y: Coord) -> &Chunk {
        // TODO: If chunk is in memory then return reference.
        //       Otherwise, fetch the chunk from the server.
        unimplemented!();
    }
}