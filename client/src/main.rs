mod networking;
mod maps;

use networking::PendingConnection;

use macroquad::prelude as quad;

const SERVER_ADDRESS: &str = "echo.websocket.org";
const SERVER_PORT: usize = 80;

#[macroquad::main("Client")]
async fn main() {
    let pending = networking::connect(networking::Protocol::WebSocket, SERVER_ADDRESS, SERVER_PORT);

    let mut connecting_text: &str;

    loop {
        let potential_connection = pending.ready();

        match potential_connection {
            Err(e) => match e {
                networking::Error::NotYetConnected => connecting_text = "Connecting...",
                _ => connecting_text = "Failed to connect to server :("
            }

            Ok(connection) => {
                game_loop(connection).await;
                break;
            }
        }

        quad::clear_background(quad::BLACK);
        quad::draw_text(connecting_text, 0.0, 0.0, 32.0, quad::ORANGE);
        quad::next_frame().await;
    }
}

async fn game_loop(mut connection: impl networking::Connection) {
    let _ = connection.send(&core::messages::ToServer::RequestChunk(0, 0));

    loop {
        quad::clear_background(quad::BLACK);
        quad::draw_text("Connected!", 0.0, 0.0, 32.0, quad::GREEN);
        quad::next_frame().await;
    }
}