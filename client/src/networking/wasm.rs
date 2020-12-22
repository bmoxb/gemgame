use super::{ Error, Result };

use std::io;

use sapp_jsutils::JsObject;

extern "C" {
    fn ws_connect(addr: JsObject);
    fn ws_connection_status() -> i32;
    fn ws_send(buffer: JsObject);
}

pub struct PendingConnection;

impl super::PendingConnection<Connection> for PendingConnection {
    fn new(url: &str) -> Self {
        let obj = JsObject::string(url);
        unsafe { ws_connect(obj) };

        PendingConnection {}
    }

    fn ready(&self) -> Result<Connection> {
        let status = unsafe { ws_connection_status() };

        if status > 0 { Ok(Connection {}) } // connected
        else if status < 0 { Err(Error::ConnectionError) } // error
        else { Err(Error::NotYetConnected) } // waiting to connect
    }
}

/// WebSocket connection relying on the web browser's JavaScript API.
pub struct Connection;

impl super::Connection for Connection {
    fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let obj = JsObject::buffer(bytes.as_slice());
        unsafe { ws_send(obj) };

        if unsafe { ws_connection_status() } > 0 { Ok(()) }
        else { Err(Error::ConnectionError) }
    }

    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>> { unimplemented!() }
}