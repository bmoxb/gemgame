#[cfg(target_arch = "wasm32")]
mod websocket;

#[cfg(not(target_arch = "wasm32"))]
mod tcpsocket;

use serde::{ Serialize, de::DeserializeOwned };

trait Socket {
    fn connect(addr: &str, port: usize) -> Result<Self, std::io::Error> where Self: Sized;
    fn is_connected(&self) -> bool;
    fn send(&mut self, data: &[u8]);
}

pub struct Connection {
    socket: Box<dyn Socket>
}

impl Connection {
    pub fn new(addr: &str, port: usize) -> Result<Connection, std::io::Error> {
        #[cfg(target_arch = "wasm32")]
        let socket = Box::new(websocket::WebSocket::connect(addr, port).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        let socket = Box::new(tcpsocket::TcpSocket::connect(addr, port)?);

        Ok(Connection { socket })
    }

    pub fn is_connected(&self) -> bool { self.socket.is_connected() }

    /// Send data of a given type that can be encoded in bincode format.
    pub fn send<T: Serialize>(&mut self, data: &T) -> Result<(), bincode::Error> {
        let bytes = bincode::serialize(data)?;
        self.send_bytes(bytes.as_slice());
        Ok(())
    }

    /// Send some bytes.
    fn send_bytes(&mut self, bytes: &[u8]) { self.socket.send(bytes) }

    /// Receive bincode data and deserialise it to the specified type.
    pub fn receive<T: DeserializeOwned>(&mut self) -> Result<T, bincode::Error> {
        let bytes = self.receive_bytes();
        bincode::deserialize(bytes.as_slice())
    }

    /// Receive some bytes.
    fn receive_bytes(&mut self) -> Vec<u8> { unimplemented!() }
}