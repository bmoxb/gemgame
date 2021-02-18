//! Contains (de)serialisable enumerations that the server and client
//! applications may communicate by means of.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    maps::{
        self,
        entities::{self, Entity}
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
    MoveMyEntity {
        /// In order to facilitate client side prediction of movement and reconciliation with the server afterwards
        /// regardless of connection speed, requests to move a player entity are incrementally numbered.
        request_number: u32,
        direction: entities::Direction
    }
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
            ToServer::MoveMyEntity { request_number, direction } => {
                write!(f, "move my player entity {} (request #{})", direction, request_number)
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
        /// The ID assigned to the client.
        your_client_id: Id,
        /// The entity ID and player entity that the client controls.
        your_entity_with_id: (Id, Entity)
    },

    /// Provide chunk data to a client so it may store it locally. Chunks are provided when requested by the client.
    ProvideChunk(maps::ChunkCoords, maps::Chunk),

    /// Whenever a tile is modified or an entity moves in a chunk, the server sends a message about the changed to each
    /// client that it believes has the chunk in question loaded.
    MapModified(MapModification),

    /// Inform a client that their player entity's position has changed. This is most frequently sent as a response to
    /// a client sending a [`ToServer::MoveMyEntity`] message.
    YourEntityMoved { request_number: u32, new_position: maps::TileCoords },

    /// Provide a client with some entity. This message is sent to a client whenever one of the following occurs:
    /// * The client requests a chunk which has an entity present in it.
    /// * An entity in a chunk not loaded by the client moves into a chunk that is.
    /// * An entity that would be in one of the client's loaded chunks is added to the map (typically when a new player
    ///   connects).
    ProvideEntity(Id, Entity),

    /// Instruct the client to unload the entity with the specified ID. This message is sent whenever:
    /// * An entity in one of the client's loaded chunks moves into a chunk that is not loaded by that client.
    /// * An entity in one of the chunks loaded by the client is removed from the map (typically as a result of a
    ///   player disconnecting).
    ShouldUnloadEntity(Id)
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
            FromServer::ProvideChunk(coords, _chunk) => write!(f, "provide chunk at {}", coords),
            FromServer::MapModified(modification) => write!(f, "map modified - {}", modification),
            FromServer::YourEntityMoved { request_number, new_position } => {
                write!(f, "your entity has moved to {} (request #{})", new_position, request_number)
            }
            FromServer::ProvideEntity(id, entity) => write!(f, "provide entity {} - {}", entity, id),
            FromServer::ShouldUnloadEntity(id) => write!(f, "should unload entity {}", id)
        }
    }
}

/// Represents a change made to the game map (tiles and entities).
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum MapModification {
    TileChanged {
        /// Position of the tile tile to be modified.
        position: maps::TileCoords,
        /// What the tile at the specified coordinates should be changed to.
        change_to: maps::Tile
    },

    EntityMoved {
        /// The ID of the entity that moved.
        entity_id: Id,
        /// The previous position of the entity (i.e. before the movement that this message describes).
        old_position: maps::TileCoords,
        /// The new position of the entity that moved.
        new_position: maps::TileCoords
    },

    /// Indicates a new entity has been added to the map (in the case of a player entity, this means that a player just
    /// connected). This variant is used internally by the server and so can be ignored by the client application.
    EntityAdded(Id),

    /// Indicates that the entity with the specified ID has been removed from the map (in the case of a player entity,
    /// this means that a player just disconnected). This variant is used internally by the server and so can be
    /// ignored by the client application.
    EntityRemoved(Id)
}

impl fmt::Display for MapModification {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapModification::TileChanged { position, change_to } => {
                write!(f, "tile changed at {} to {:?}", position, change_to)
            }
            MapModification::EntityMoved { entity_id, old_position, new_position } => {
                write!(f, "entity {} moved from {} to {}", entity_id, old_position, new_position)
            }

            MapModification::EntityAdded(id, entity) => write!(f, "entity {} - {} added to map", id, entity),
            MapModification::EntityRemoved(id) => write!(f, "entity {} removed from map", id)
        }
    }
}
