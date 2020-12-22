mod networking;
mod maps;

use networking::{ Connection, PendingConnection };

use macroquad::prelude as quad;

#[macroquad::main("Client")]
async fn main() {
    let pending = networking::connect("ws://echo.websocket.org", 80);

    loop {
        let potential_connection = pending.ready();

        match potential_connection {
            Err(e) => match e {
                networking::Error::NotYetConnected => {
                    quad::clear_background(quad::GRAY);
                    quad::draw_text("Connection pending...", 0.0, 0.0, 20.0, quad::RED);
                    quad::next_frame().await;
                }
                _ => panic!() // TODO: Retry connection...
            }

            Ok(mut connection) => {
                let _ = connection.send(&core::messages::ToServer::RequestChunk(0, 0));

                loop {
                    quad::clear_background(quad::GRAY);
                    quad::draw_text("Connected!", 0.0, 0.0, 20.0, quad::GREEN);
                    quad::next_frame().await;
                }
            }
        }
    }
}