//! Contains (de)serialisable enumerations that the server and client
//! applications may communicate by means of.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{entities::Entity, maps, Id};

/// Message sent from the client to the server over the WebSocket protocol.
#[derive(Serialize, Deserialize)]
pub enum ToServer {
    /// This must be the first message sent by a client to the server after establishing a WebSocket connection.
    Hello {
        /// If this player has played before then a client ID value will be sent so that they may continue playing as
        /// their pre-existing character. If this player has never played before (or have cleared their browser
        /// cookies) then this field should be `None` (but note that a 'hello' message must still be the first
        /// message sent by the client).
        my_client_id: Option<Id>
    },

    /// Indicate to the server that this client would like the data for the chunk at the specified chunk coordinates.
    /// Should the client have a valid reason for wanting this chunk (e.g. the client's player character is moving
    /// towards the requested chunk) then the server will response with [`FromServer::ProvideChunk`] with the chunk
    /// data.
    RequestChunk(maps::ChunkCoords),

    /// Inform the server that this client has unloaded a chunk. This is done so that the server knows that it does not
    /// need to send [`FromServer::UpdateTile`] messages for tiles in the specified chunk to this client (the server
    /// keeps track of what chunks it believes each client has currently loaded).
    ChunkUnloadedLocally(maps::ChunkCoords)
}

impl fmt::Display for ToServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToServer::Hello { my_client_id } => match my_client_id {
                Some(id) => write!(f, "hello as existing client {}", id),
                None => write!(f, "hello as new client")
            },
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
    /// Response to a `ToServer::Hello` message. This should be the first message sent from the server to each client.
    Welcome {
        /// The version of the game that the server is running. If this does not match the client's version then the
        /// client should close the connection.
        version: String,
        your_client_id: Id,
        your_entity: Entity
    },

    /// Provide chunk data to a client so it may store it locally. Chunks are provided when requested by the client.
    ProvideChunk(maps::ChunkCoords, maps::Chunk),

    /// Whenever a tile in a chunk is modified, the server sends a message about the changed to each client that it
    /// believes has the chunk in question loaded.
    UpdateTile(maps::TileCoords, maps::Tile)
}

impl fmt::Display for FromServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FromServer::Welcome { version, your_client_id, your_entity } => {
                write!(
                    f,
                    "welcome client {} to server running version '{}' and provide entity {}",
                    your_client_id, version, your_entity
                )
            }
            FromServer::ProvideChunk(coords, _) => write!(f, "provide chunk at {}", coords),
            FromServer::UpdateTile(coords, _) => write!(f, "update tile at {}", coords)
        }
    }
}
