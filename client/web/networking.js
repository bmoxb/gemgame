var socket;
var status = 0;

function register_plugin(importObject) {
    importObject.env.ws_connect = function(addr) {
        socket = new WebSocket(consume_js_object(addr));
        socket.binaryType = "arraybuffer";

        socket.onopen = function() { status = 1; }
        socket.onerror = function(e) { status = -1; }
    };

    importObject.env.ws_connection_status = function() {
        return status;
    }

    importObject.env.ws_send = function(data) {
        socket.send(consume_js_object(data).buffer);
    };
}

function on_init() {}

miniquad_add_plugin({ register_plugin, on_init });