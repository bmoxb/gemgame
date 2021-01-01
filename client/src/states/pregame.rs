use macroquad::prelude as quad;

use super::State;

use crate::networking::{ self, PendingConnectionTrait, ConnectionTrait };

use core::{ messages, WEBSOCKET_CONNECTION_PORT };

const SERVER_ADDRESS: &str = "localhost";
const SECURE_CONNECTION: bool = false;

const CONNECTING_TEXT: &str = "Connecting...";
const FAILED_TEXT: &str = "Failed to connect to server :(";

#[cfg(target_arch = "wasm32")]
const WRONG_VERSION_TEXT: &str = "Please clear your browser cache!";
#[cfg(not(target_arch = "wasm32"))]
const WRONG_VERSION_TEXT: &str = "Please download the latest version of the game!";

pub struct ConnectingState {
    pending_connection: networking::PendingConnection,
    text: &'static str
}

impl ConnectingState {
    pub fn new() -> Self {
        log::info!("Connecting to '{}' on port {}...", SERVER_ADDRESS, WEBSOCKET_CONNECTION_PORT);
        ConnectingState {
            pending_connection: networking::connect(SERVER_ADDRESS, WEBSOCKET_CONNECTION_PORT, SECURE_CONNECTION),
            text: CONNECTING_TEXT
        }
    }
}

impl State for ConnectingState {
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        match self.pending_connection.ready() {
            Ok(connection_option) => if let Some(connection) = connection_option {
                log::info!("Connection to server established!");

                return Some(Box::new(ConnectedState::new(connection)));
            }

            Err(e) => {
                log::error!("Failed to connect to server due to error: {}", e);

                self.text = FAILED_TEXT;
            }
        }

        quad::draw_text(self.text, 0.0, 0.0, 32.0, quad::WHITE);

        None
    }

    fn title(&self) -> &'static str { "Connecting To Server" }
}

struct ConnectedState {
    connection: Option<networking::Connection>,
    text: &'static str
}

impl ConnectedState {
    fn new(connection: networking::Connection) -> Self {
        ConnectedState {
            connection: Some(connection),
            text: CONNECTING_TEXT
        }
    }
}

impl State for ConnectedState {
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        match self.connection.as_mut().unwrap().receive() {
            Ok(msg_option) => if let Some(msg) = msg_option {
                match msg {
                    messages::FromServer::Welcome { version } => {
                        log::debug!("Server version: {}", version);

                        if version == core::VERSION {
                            let taken_connection = self.connection.take().unwrap();
                            let game_state = super::game::GameState::new(taken_connection);

                            return Some(Box::new(game_state));
                        }
                        else {
                            log::error!("Version of server ({}) differs from that of this client ({})",
                                        version, core::VERSION);

                            self.text = WRONG_VERSION_TEXT;
                        }
                    }

                    other_msg => {
                        log::error!("Expected a 'welcome' message from server but instead received: {}",
                                    other_msg);

                        self.text = FAILED_TEXT;
                    }
                }
            }

            Err(e) => {
                log::warn!("Connecting error while waiting to receive a welcome message: {}", e);
                self.text = FAILED_TEXT;
            }
        }

        quad::draw_text(self.text, 0.0, 0.0, 32.0, quad::WHITE);

        None
    }

    fn title(&self) -> &'static str { "Connected To Server" }
}