//! Contains (de)serialisable enumerations that the server and client applications may communicate with.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    gems, items,
    maps::{
        self,
        entities::{self, Entity}
    },
    Id
};

/// Message sent from the client to the server over the WebSocket protocol.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToServer {
    /// This must be the first message sent by a client to the server after establishing a WebSocket connection.
    Hello {
        /// If this player has played before then a client ID value will be sent so that they may continue playing as
        /// their pre-existing character. If this player has never played before (or have cleared their browser
        /// cookies) then this field should be `None` (but note that a 'hello' message must still be the first
        /// message sent by the client).
        client_id_option: Option<Id>
    },

    /// Inform the server that the player has moved their player entity. The server will respond with a
    /// [`FromServer::YourEntityMoved`] message to inform the client of their player entity's new position.
    MoveMyEntity {
        /// In order to facilitate client side prediction of movement and reconciliation with the server afterwards
        /// regardless of connection speed, requests to move a player entity are incrementally numbered.
        request_number: u32,
        direction: entities::Direction
    },

    /// Attempt to place a bomb at the player entity's position. The client is expected to ensure that their player
    /// actually has a bomb to place before sending this message.
    PlaceBomb,

    /// Have the server detonate all of the player's placed bombs that are within the 9 chunks they are in and
    /// surrounded by.
    DetonateBombs,

    /// Indicate that the player wishes to purchase the given item (of type [`items::BoolItem`]). The client should
    /// only send this message if it believes the player has enough gems to do so (if they do not then the server
    /// will silently ignore the message).
    PurchaseSingleItem(items::BoolItem),

    /// Inform the server that the player wishes the purchase the specified quantity of the given item (of type
    /// [`items::QuantitativeItem`]). The server will ignore the message if the player does have enough gems to
    /// complete the purchase.
    PurchaseItemQuantity { item: items::QuantitativeItem, quantity: u32 }
}

impl fmt::Display for ToServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ToServer::Hello { client_id_option } => match client_id_option {
                Some(id) => write!(f, "hello as existing client {}", id),
                None => write!(f, "hello as new client")
            },
            ToServer::MoveMyEntity { request_number, direction } => {
                write!(f, "move my player entity {} (request #{})", direction, request_number)
            }
            ToServer::PlaceBomb => write!(f, "place bomb"),
            ToServer::DetonateBombs => write!(f, "detonate bombs"),
            ToServer::PurchaseSingleItem(item) => write!(f, "purchase {:?}", item),
            ToServer::PurchaseItemQuantity { item, quantity } => write!(f, "purchase {} of {:?}", quantity, item)
        }
    }
}

/// Message sent from the server to the client over the WebSocket protocol.
#[derive(Serialize, Deserialize)]
pub enum FromServer {
    /// Response to a [`ToServer::Hello`] message. This should be the first message sent from the server to each
    /// client.
    Welcome {
        /// The version of the game that the server is running. If this does not match the client's version then the
        /// client should close the connection.
        version: String,
        /// The ID assigned to the client.
        your_client_id: Id,
        /// The entity ID and player entity that the client controls.
        your_entity_with_id: (Id, Entity)
    },

    /// Provide chunk data to a client so it may store it locally. Chunks are provided automatically based on the
    /// position of a client's player entity.
    ProvideChunk(maps::ChunkCoords, maps::Chunk),

    /// Indicate to a client that they should unload the chunk at the specified coordinates. This message is sent when
    /// a client's player entity moves far outside a loaded chunk.
    ShouldUnloadChunk(maps::ChunkCoords),

    /// Whenever a map tile is change, this message to sent to all clients that the server believes has loaded the
    /// chunk that the modified tile is contained in.
    ChangeTile(maps::TileCoords, maps::Tile),

    /// Inform a client that their player entity's position has changed. This is most frequently sent as a response to
    /// a client sending a [`ToServer::MoveMyEntity`] message.
    YourEntityMoved { request_number: u32, new_position: maps::TileCoords },

    /// Inform a client that an entity that is not the player entity that they control has moved within the bounds of
    /// that client's loaded chunks.
    MoveEntity(Id, maps::TileCoords, entities::Direction),

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
    ShouldUnloadEntity(Id),

    /// Inform the client that some entity (not their own) placed a bomb down within the client's loaded chunks.
    BombPlaced { placed_by_entity_id: Id, position: maps::TileCoords },

    /// Inform the client that all the bombs placed by the entity with the given ID in and surrounded the specified
    /// chunk have now detonated.
    BombsDetonated { placed_by_entity_id: Id, in_and_around_chunk_coords: maps::ChunkCoords },

    /// Informs the client of the type and quantity of gems they received after their entity smashed a rock.
    YouCollectedGems { gem_type: gems::Gem, quantity_increase: u32 }
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
            FromServer::ShouldUnloadChunk(coords) => write!(f, "should unload chunk at {}", coords),
            FromServer::ChangeTile(coords, tile) => write!(f, "change tile at {} to {:?}", coords, tile),
            FromServer::YourEntityMoved { request_number, new_position } => {
                write!(f, "your entity moved to {} (request #{})", new_position, request_number)
            }
            FromServer::MoveEntity(id, pos, direction) => {
                write!(f, "move entity {} to {} in direction {}", id, pos, direction)
            }
            FromServer::ProvideEntity(id, entity) => write!(f, "provide entity {} - {}", entity, id),
            FromServer::ShouldUnloadEntity(id) => write!(f, "should unload entity {}", id),
            FromServer::BombPlaced { placed_by_entity_id, position } => {
                write!(f, "bomb placed at {} by entity {}", position, placed_by_entity_id)
            }
            FromServer::BombsDetonated { placed_by_entity_id, in_and_around_chunk_coords } => {
                write!(
                    f,
                    "bombs placed entity {} in and around chunk at {} detonated",
                    placed_by_entity_id, in_and_around_chunk_coords
                )
            }
            FromServer::YouCollectedGems { gem_type, quantity_increase } => {
                write!(f, "you collected {} gems of type {:?}", quantity_increase, gem_type)
            }
        }
    }
}
