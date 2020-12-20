mod networking;
mod maps;

use macroquad::prelude as quad;

#[macroquad::main("Client")]
async fn main() {
    let mut conn = networking::Connection::new("localhost", 8080).unwrap();

    while !conn.is_connected() { quad::next_frame().await; }

    loop {
        quad::clear_background(quad::RED);
        quad::next_frame().await
    }
}