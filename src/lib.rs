pub mod id;
pub mod messages;
pub mod world;

pub use id::Id;

/// Version of this client/server build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default port used for WebSocket communications between the client and server applications.
pub const WEBSOCKET_CONNECTION_PORT: u16 = 5678;
