use serde::{ Serialize, Deserialize };

/// Message sent from the client to the server over the WebSocket protocol.
#[derive(Debug, Serialize, Deserialize)]
pub enum ToServer {}

/// Message sent from the server to the client over the WebSocket protocol.
#[derive(Debug, Serialize, Deserialize)]
pub enum FromServer {}