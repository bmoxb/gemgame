use std::io;

use sapp_jsutils::JsObject;

use super::{Error, Result};

extern "C" {
    fn ws_connect(addr: JsObject);
    fn ws_connection_status() -> i32;
    fn ws_send(buffer: JsObject);
    fn ws_receive() -> JsObject;
}

const IO_ERROR_MSG: &str = "Please see the browser console for error message";

pub struct PendingConnection;

impl super::PendingConnectionTrait<Connection> for PendingConnection {
    fn new(full_url: String) -> Self {
        let obj = JsObject::string(&full_url);
        unsafe { ws_connect(obj) };

        PendingConnection
    }

    fn ready(&self) -> Result<Option<Connection>> { ConnectionStatus::result(None, Some(Connection)) }
}

/// WebSocket connection relying on the web browser's JavaScript API.
pub struct Connection;

impl super::ConnectionTrait for Connection {
    fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let obj = JsObject::buffer(bytes.as_slice());
        unsafe { ws_send(obj) };

        ConnectionStatus::result((), ())
    }

    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>> {
        let data = unsafe { ws_receive() };

        if data.is_nil() {
            ConnectionStatus::result(None, None)
        }
        else {
            let mut buffer = Vec::new();
            data.to_byte_buffer(&mut buffer);
            Ok(Some(buffer))
        }
    }
}

enum ConnectionStatus {
    Pending,
    Ok,
    Closed,
    Error
}

impl ConnectionStatus {
    /// Call on JavaScript code that identifies the current statis of the WebSocket connection.
    fn identify() -> ConnectionStatus {
        match unsafe { ws_connection_status() } {
            0 => ConnectionStatus::Pending,
            1 => ConnectionStatus::Ok,
            2 => ConnectionStatus::Closed,
            _ => ConnectionStatus::Error
        }
    }

    /// Return a `Result` value based on the current connection status.
    fn result<T>(pending_value: T, ok_value: T) -> Result<T> {
        match ConnectionStatus::identify() {
            ConnectionStatus::Pending => Ok(pending_value),
            ConnectionStatus::Ok => Ok(ok_value),
            ConnectionStatus::Closed => Err(Error::ConnectionClosed),
            ConnectionStatus::Error => Err(io::Error::new(io::ErrorKind::Other, IO_ERROR_MSG).into())
        }
    }
}
