mod networking;
mod maps;

use networking::PendingConnection;

use macroquad::prelude as quad;

const SERVER_ADDRESS: &str = "echo.websocket.org";
const SERVER_PORT: usize = 80;

#[macroquad::main("Client")]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pretty_env_logger::init(); // Only have logging when build for desktop.

    log::info!("Connecting to '{}' on port {}...", SERVER_ADDRESS, SERVER_PORT);
    let pending = networking::connect(networking::Protocol::WebSocket, SERVER_ADDRESS, SERVER_PORT);

    loop {
        match pending.ready() {
            Err(e) => match e {
                networking::Error::NotYetConnected => {
                    log::trace!("Connection pending...");

                    quad::draw_text("Connecting...", 0.0, 0.0, 32.0, quad::ORANGE);
                    quad::next_frame().await;
                }
                _ => {
                    log::warn!("Failed to connect to server!");

                    quad::clear_background(quad::BLACK);
                    loop {
                        quad::draw_text("Failed to connect to server :(", 0.0, 0.0, 32.0, quad::RED);
                        quad::next_frame().await;
                    }
                }
            }

            Ok(connection) => {
                log::info!("Connection established! Beginning game loop...");
                game_loop(connection).await;
                break;
            }
        }
    }
}

async fn game_loop(mut connection: impl networking::Connection) {
    loop {
        quad::clear_background(quad::BLACK);
        quad::draw_text("Connected!", 0.0, 0.0, 32.0, quad::GREEN);
        quad::next_frame().await;
    }
}