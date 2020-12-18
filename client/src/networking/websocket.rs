use sapp_jsutils::JsObject;

extern "C" {
    fn ws_connect(addr: JsObject);
    fn ws_is_connected() -> i32;
    fn ws_send(buffer: JsObject);
}

pub struct WebSocket;

impl super::Socket for WebSocket {
    fn connect(addr: &str, port: usize) -> Result<Self, std::io::Error> {
        let obj = JsObject::string(&format!("ws://{}:{}", addr, port));
        unsafe { ws_connect(obj) };

        Ok(WebSocket {})
    }

    fn is_connected(&self) -> bool {
        unsafe { ws_is_connected() == 1 }
    }

    fn send(&mut self, data: &[u8]) {
        let obj = JsObject::buffer(data);
        unsafe { ws_send(obj) };
    }
}