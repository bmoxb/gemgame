const NIL = -1;

const PENDING = 0;
const OK = 1;
const CLOSED = 2;
const ERROR = -1;

var socket;
var status = PENDING;
var received = [];

function register_plugin(importObject) {
    importObject.env.ws_connect = function(addr) {
        socket = new WebSocket(consume_js_object(addr));
        socket.binaryType = "arraybuffer";

        socket.onopen = function() { status = OK; }

        socket.onerror = function(e) { status = ERROR; }

        socket.onclose = function(e) { status = CLOSED; }

        socket.onmessage = function(msg) {
            received.push(new Uint8Array(msg.data));
        }
    };

    importObject.env.ws_connection_status = function() {
        return status;
    };

    importObject.env.ws_send = function(data) {
        socket.send(consume_js_object(data).buffer);
    };

    importObject.env.ws_receive = function() {
        if(received.length > 0) {
            return js_object(received.shift())
        }
        return NIL;
    }

    importObject.env.cookie_get = function(key) {
        var pair = document.cookie.split(";")
                           .find(pair => pair.trim().startsWith(consume_js_object(key)) );

        if(pair) {
            var value = pair.split("=")[1];
            return js_object(value);
        }
        else { return NIL; }
    }

    importObject.env.cookie_set = function(key, value) {
        document.cookie = consume_js_object(key) + "="
                        + consume_js_object(value)
                        + "; expires=Tue, 01 Jan 2030 00:00:00 GMT";
    }
}

miniquad_add_plugin({ register_plugin, function(){} });