#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(target_arch = "wasm32")]
pub use browser::*;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;

use std::{convert, fmt};

#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;
use serde::{de::DeserializeOwned, Serialize};

pub fn connect(addr: &str, port: u16, secure: bool) -> PendingConnection {
    PendingConnection::new(addr_port_to_url(secure, addr, port))
}

/// Simple helper function that builds a WebSocket URL given an address, port, and a boolean indicating whether the
/// connection will be secure or not.
fn addr_port_to_url(secure: bool, addr: &str, port: u16) -> String {
    format!("{}://{}:{}", if secure { "wss" } else { "ws" }, addr, port)
}

/// Represents a connection that has not yet been fully established (i.e. still performing handshake).
pub trait PendingConnectionTrait<T: ConnectionTrait> {
    /// Establishes an intent to connect to a specified URL (non-blocking).
    fn new(full_url: String) -> Self
    where Self: Sized;

    /// Check if the connection has been established. Will return `Ok(None)` when no errors have been encountered but
    /// the connection is still in the process of being established.
    fn ready(&self) -> Result<Option<T>>;
}

pub trait ConnectionTrait {
    /// Send data of a given type that can be encoded in bincode format.
    fn send<S: Serialize>(&mut self, data: &S) -> Result<()> {
        let bytes = bincode::serialize(data)?;
        self.send_bytes(bytes)
    }

    /// Send some bytes.
    fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<()>;

    /// Attempt to receive some bincode data and deserialise it to the specified
    /// type (non-blocking).
    fn receive<D: DeserializeOwned>(&mut self) -> Result<Option<D>> {
        match self.receive_bytes()? {
            Some(bytes) => match bincode::deserialize(bytes.as_slice()) {
                Ok(value) => Ok(Some(value)),
                Err(e) => Err(e.into())
            },
            None => Ok(None)
        }
    }

    /// Attempt to receive some bytes (non-blocking).
    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>>;
}

#[derive(Debug)]
pub enum Error {
    /// Indicates that the underlying socket has experienced some sort of issue with its connection to the server or
    /// failed to establish a connection in the first place.
    ConnectionError(Box<dyn std::error::Error + Send>),

    /// Occurs when bincode data sent/received over the connection could not be properly (de)serialised.
    BincodeError(bincode::Error),

    /// This error type is returned when the message received from the remote peer is a closing message and the closing
    /// handshake is performed.
    ConnectionClosed
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ConnectionError(e) => write!(f, "Connection error - {}", e),
            Error::BincodeError(e) => write!(f, "(De)serialisation error - {}", e),
            Error::ConnectionClosed => write!(f, "Connection closed (closing handshake performed)")
        }
    }
}

impl convert::From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::ConnectionError(Box::new(e))
    }
}

impl convert::From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::BincodeError(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
