mod networking;
mod maps;

use networking::PendingConnection;

use macroquad::prelude as quad;

const SERVER_ADDRESS: &str = "echo.websocket.org";
const SERVER_PORT: usize = 80;
const SECURE_CONNECTION: bool = false;

#[macroquad::main("Client")]
async fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    pretty_env_logger::init(); // Only have logging when build for desktop.

    log::info!("Connecting to '{}' on port {}...", SERVER_ADDRESS, SERVER_PORT);
    let pending = networking::connect(SERVER_ADDRESS, SERVER_PORT, SECURE_CONNECTION);

    loop {
        match pending.ready() {
            Ok(Some(connection)) => {
                log::info!("Connection established! Beginning game loop...");
                game_loop(connection).await;
                break;
            }

            Ok(None) => {
                log::trace!("Connection pending...");

                quad::draw_text("Connecting...", 0.0, 0.0, 32.0, quad::ORANGE);
                quad::next_frame().await;
            }

            Err(e) => {
                log::warn!("Failed to connect to server: {}", e);

                quad::clear_background(quad::BLACK);
                loop {
                    quad::draw_text("Failed to connect to server :(", 0.0, 0.0, 32.0, quad::RED);
                    quad::next_frame().await;
                }
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