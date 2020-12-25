use super::{ Error, Result };

use std::io;

use sapp_jsutils::JsObject;

extern "C" {
    fn ws_connect(addr: JsObject);
    fn ws_connection_status() -> i32;
    fn ws_send(buffer: JsObject);
}

const IO_ERROR_MSG: &'static str = "Please see the browser console for error message";

pub struct PendingConnection;

impl super::PendingConnectionTrait<Connection> for PendingConnection {
    fn new(full_url: String) -> Self {
        let obj = JsObject::string(&full_url);
        unsafe { ws_connect(obj) };

        PendingConnection {}
    }

    fn ready(&self) -> Result<Option<Connection>> {
        let status = unsafe { ws_connection_status() };

        if status > 0 { Ok(Some(Connection {})) } // connected
        else if status < 0 { // error
            Err(io::Error::new(io::ErrorKind::ConnectionRefused, IO_ERROR_MSG).into())
        }
        else { Ok(None) } // still waiting to connect
    }
}

/// WebSocket connection relying on the web browser's JavaScript API.
pub struct Connection;

impl super::ConnectionTrait for Connection {
    fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<()> {
        let obj = JsObject::buffer(bytes.as_slice());
        unsafe { ws_send(obj) };

        if unsafe { ws_connection_status() } > 0 { Ok(()) }
        else { Err(io::Error::new(io::ErrorKind::Other, IO_ERROR_MSG).into()) }
    }

    fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>> { unimplemented!() }
}