use std::{ net, thread };
use tungstenite as ws;

const PORT: usize = 8080;

fn main() {
    let server = net::TcpListener::bind(format!("127.0.0.1:{}", PORT));

    for stream in server.incoming() {}
}