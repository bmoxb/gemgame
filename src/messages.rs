//! Contains (de)serialisable enumerations that the server and client
//! applications may communicate by.

use crate::maps;

use serde::{ Serialize, Deserialize };

/// Message sent from the client to the server over the WebSocket protocol.
#[derive(Debug, Serialize, Deserialize)]
pub enum ToServer {
    RequestChunk(maps::Coord, maps::Coord)
}

/// Message sent from the server to the client over the WebSocket protocol.
#[derive(Serialize, Deserialize)]
pub enum FromServer {
    ProvideChunk(maps::Chunk)
}