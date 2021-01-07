pub mod entities;
pub mod maps;
pub mod messages;

//pub mod items;

/// Version of this client/server build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default port used for WebSocket communications between the client and server applications.
pub const WEBSOCKET_CONNECTION_PORT: u16 = 5678;
