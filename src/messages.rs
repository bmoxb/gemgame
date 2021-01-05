//! Contains (de)serialisable enumerations that the server and client
//! applications may communicate by means of.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::maps;

/// Message sent from the client to the server over the WebSocket protocol.
#[derive(Serialize, Deserialize)]
pub enum ToServer {
    /// Indicate to the server that this client would like the data for the
    /// chunk at the specified chunk coordinates. Should the client have a valid
    /// reason for wanting this chunk (e.g. the client's player character is
    /// moving towards the requested chunk) then the server will response with
    /// [`FromServer::ProvideChunk`] with the chunk data.
    RequestChunk(maps::ChunkCoords),

    /// Inform the server that this client has unloaded a chunk. This is done so
    /// that the server knows that it does not need to send
    /// [`FromServer::UpdateTile`] messages for tiles in the specified chunk
    /// to this client (the server keeps track of what chunks it believes
    /// each client has currently loaded).
    ChunkUnloadedLocally(maps::ChunkCoords)
}

impl fmt::Display for ToServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToServer::RequestChunk(coords) => write!(f, "request chunk at {}", coords),
            ToServer::ChunkUnloadedLocally(coords) => {
                write!(f, "chunk at {} has been unloaded locally", coords)
            }
        }
    }
}

/// Message sent from the server to the client over the WebSocket protocol.
#[derive(Serialize, Deserialize)]
pub enum FromServer {
    /// Indicate the client the version of the server being connected to. This
    /// should be the first message sent after the completion of the WebSocket
    /// connection handshake.
    Welcome { version: String },

    /// Provide chunk data to a client so it may store it locally. Chunks are
    /// provided when requested by the client.
    ProvideChunk(maps::ChunkCoords, maps::Chunk),

    /// Whenever a tile in a chunk is modified, the server sends a message
    /// about the changed to each client that it believes has the chunk in
    /// question loaded.
    UpdateTile(maps::TileCoords, maps::Tile)
}

impl fmt::Display for FromServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FromServer::Welcome { version } => {
                write!(f, "welcome to server running version '{}'", version)
            }
            FromServer::ProvideChunk(coords, _) => write!(f, "provide chunk at {}", coords),
            FromServer::UpdateTile(coords, _) => write!(f, "update tile at {}", coords)
        }
    }
}
