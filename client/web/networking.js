var socket;
var connected = 0;

function register_plugin(importObject) {
    importObject.env.ws_connect = function(addr) {
        socket = new WebSocket(consume_js_object(addr));
        socket.binaryType = "arraybuffer";

        console.log("WebSocket object created");

        socket.onopen = function() {
            console.log("Connection established");
            connected = 1;
        }
    };

    importObject.env.ws_is_connected = function() {
        return connected;
    }

    importObject.env.ws_send = function(data) {
        socket.send(consume_js_object(data).buffer);
    };
}

function on_init() {}

miniquad_add_plugin({ register_plugin, on_init });