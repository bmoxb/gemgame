#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
mod desktop;

use std::{ fmt, convert };

use serde::{ Serialize, de::DeserializeOwned };

#[cfg(target_arch = "wasm32")]
pub fn connect(protocol: Protocol, addr: &str, port: usize) -> wasm::PendingConnection {
    wasm::PendingConnection::new(&full_addr(protocol, addr, port))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn connect(protocol: Protocol, addr: &str, port: usize) -> desktop::PendingConnection {
    desktop::PendingConnection::new(&full_addr(protocol, addr, port))
}

pub enum Protocol { WebSocket, WebSocketSecure }

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Protocol::WebSocket => write!(f, "ws"),
            Protocol::WebSocketSecure => write!(f, "wss")
        }
    }
}

fn full_addr(protocol: Protocol, addr: &str, port: usize) -> String {
    format!("{}://{}:{}", protocol, addr, port)
}

pub trait PendingConnection<T: Connection> {
    fn new(url: &str) -> Self where Self: Sized;
    fn ready(&self) -> Result<T>;
}

pub trait Connection {
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
            Some(bytes) => {
                match bincode::deserialize(bytes.as_slice()) {
                    Ok(value) => Ok(Some(value)),
                    Err(e) => Err(e.into())
                }
            }
            None => Ok(None)
        }
    }

    /// Attempt to receive some bytes (non-blocking).
    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>>;
}

#[derive(Debug)]
pub enum Error {
    /// Occurs when an attempt to send/receive data is made despite the
    /// underlying socket still being in the process of establishing a
    /// connection to the server.
    NotYetConnected,

    /// Indicates that the underlying socket has experienced some sort of issue
    /// with its connection to the server or failed to establish a connection in
    /// the first place.
    ConnectionError,

    /// Occurs when bincode data sent/received over the connection could not be
    /// properly (de)serialised.
    BincodeError(bincode::Error)
}

impl convert::From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self { Error::BincodeError(e) }
}

pub type Result<T> = std::result::Result<T, Error>;