//! Contains (de)serialisable enumerations that the server and client
//! applications may communicate by means of.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    world::{
        entities::{self, Entity},
        maps
    },
    Id
};

/// Message sent from the client to the server over the WebSocket protocol.
#[derive(Serialize, Deserialize)]
pub enum ToServer {
    /// This must be the first message sent by a client to the server after establishing a WebSocket connection.
    Hello {
        /// If this player has played before then a client ID value will be sent so that they may continue playing as
        /// their pre-existing character. If this player has never played before (or have cleared their browser
        /// cookies) then this field should be `None` (but note that a 'hello' message must still be the first
        /// message sent by the client).
        client_id_option: Option<Id>
    },

    /// Indicate to the server that this client would like the data for the chunk at the specified chunk coordinates.
    /// Should the client have a valid reason for wanting this chunk (e.g. the client's player character is moving
    /// towards the requested chunk) then the server will response with [`FromServer::ProvideChunk`] with the chunk
    /// data.
    /// TODO: Remove the need to request chunks - have server provide them automatically based on player's position.
    RequestChunk(maps::ChunkCoords),

    /// Inform the server that this client has unloaded a chunk. This is done so that the server knows that it does not
    /// need to send [`FromServer::UpdateTile`] messages for tiles in the specified chunk to this client (the server
    /// keeps track of what chunks it believes each client has currently loaded).
    ChunkUnloadedLocally(maps::ChunkCoords),

    /// Inform the server that the player has moved their player entity. The server will respond with a
    /// [`FromServer::YourEntityMoved`] message to inform the client of their player entity's new position.
    MoveMyEntity(entities::Direction)
}

impl fmt::Display for ToServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToServer::Hello { client_id_option } => match client_id_option {
                Some(id) => write!(f, "hello as existing client {}", id),
                None => write!(f, "hello as new client")
            },
            ToServer::RequestChunk(coords) => write!(f, "request chunk at {}", coords),
            ToServer::ChunkUnloadedLocally(coords) => write!(f, "chunk at {} has been unloaded locally", coords),
            ToServer::MoveMyEntity(direction) => write!(f, "move my player entity {}", direction)
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
        /// The ID assigned to the client.
        your_client_id: Id,
        /// The entity ID and player entity that the client controls.
        your_entity_with_id: (Id, Entity)
    },

    /// Provide chunk data to a client so it may store it locally. Chunks are provided when requested by the client.
    ProvideChunk(maps::ChunkCoords, maps::Chunk),

    /// Whenever a tile in a chunk is modified, the server sends a message about the changed to each client that it
    /// believes has the chunk in question loaded.
    UpdateTile(maps::TileCoords, maps::Tile),

    /// Inform a client that their player entity's position has changed. This is most frequently sent as a response to
    /// a client sending a [`ToServer::MoveMyEntity`] message.
    YourEntityMoved(maps::TileCoords),

        /// Inform the client that the entity with the specified ID has moved to the specified coordinates. This message is
    /// only sent to clients with a player entity on the same map as and in close proximity (i.e. chunk loaded) to the
    /// moved entity.
    /// TODO: Remember to consider that clients must be informed of entities that are crossing chunk borders out of
    /// TODO: the client's currently loaded chunks in addition to entities moving within the bounds of loaded chunks.
    EntityMoved(Id, maps::TileCoords),

    /// Provide a client with some entity. This is done when the client's player entity comes to be in close proximity
    /// to another entity that the server believes is not already loaded by the client.
    ProvideEntity(Id, Entity)
}

impl fmt::Display for FromServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FromServer::Welcome { version, your_client_id, your_entity_with_id: (entity_id, entity) } => {
                write!(
                    f,
                    "welcome client {} to server running version '{}' and provide entity {} - {}",
                    your_client_id, version, entity, entity_id
                )
            }
            FromServer::ProvideChunk(coords, _) => write!(f, "provide chunk at {}", coords),
            FromServer::UpdateTile(coords, _) => write!(f, "update tile at {}", coords),
            FromServer::ProvideEntity(id, entity) => write!(f, "provide entity {} - {}", entity, id),
            FromServer::YourEntityMoved(coords) => write!(f, "your entity has moved to {}", coords),
            FromServer::EntityMoved(id, coords) => write!(f, "entity with ID {} moved to {}", id, coords)
        }
    }
}
