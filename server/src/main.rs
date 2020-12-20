mod maps;

use std::net;

const PORT: usize = 8080;

fn main() {
    let server = net::TcpListener::bind(format!("127.0.0.1:{}", PORT)).unwrap();

    for stream in server.incoming() {}
}