use macroquad::prelude as quad;

use crate::networking::{ self, PendingConnectionTrait };
use super::State;

const SERVER_ADDRESS: &str = "echo.websocket.org";
const SERVER_PORT: usize = 80;
const SECURE_CONNECTION: bool = false;

pub struct ConnectToServerState {
    pending_connection: networking::PendingConnection,
    text: String
}

impl ConnectToServerState {
    pub fn new() -> Self {
        log::info!("Connecting to '{}' on port {}...", SERVER_ADDRESS, SERVER_PORT);
        ConnectToServerState {
            pending_connection: networking::connect(SERVER_ADDRESS, SERVER_PORT, SECURE_CONNECTION),
            text: String::new()
        }
    }
}

impl State for ConnectToServerState {
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        match self.pending_connection.ready() {
            Ok(Some(connection)) => {
                log::info!("Connection to server established!");

                return Some(Box::new(super::game::GameState::new(connection)));
            }

            Ok(None) => { self.text = "Connecting...".to_string() }

            Err(e) => {
                log::warn!("Failed to connect to server due to error: {}", e);

                self.text = "Failed to connect to server :(".to_string();
            }
        }

        quad::draw_text(&self.text, 0.0, 0.0, 32.0, quad::RED);

        None
    }

    fn title(&self) -> &'static str { "Connect To Server" }
}