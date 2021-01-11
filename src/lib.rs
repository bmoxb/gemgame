pub mod entities;
pub mod maps;
pub mod messages;

/// Version of this client/server build.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default port used for WebSocket communications between the client and server applications.
pub const WEBSOCKET_CONNECTION_PORT: u16 = 5678;

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Id {
    value: u64
}

impl Id {
    pub fn new(value: u64) -> Self { Id { value } }

    pub fn encode(&self) -> String {
        format!("{:X}", self.value)
    }

    pub fn decode(s: &str) -> Option<Self> {
        s.parse().ok().map(Id::new)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04X}-{:04X}-{:04X}-{:04X}",
            self.value >> 48,
            (self.value >> 32) & 0xFFFF,
            (self.value >> 16) & 0xFFFF,
            self.value & 0xFFFF
        )
    }
}
