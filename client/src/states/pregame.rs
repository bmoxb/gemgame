use macroquad::prelude as quad;
use shared::{messages, WEBSOCKET_PORT};

use super::State;
use crate::{
    maps::entities::MyEntity,
    networking::{self, ConnectionTrait, PendingConnectionTrait},
    sessions, AssetManager
};

const WEBSOCKET_ADDRESS: &str = "gemgame.mblack.dev";
const WEBSOCKET_SECURE: bool = false; // TODO: Set up TLS for WebSocket connections.

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
        log::info!("Connecting to '{}' on port {}...", WEBSOCKET_ADDRESS, WEBSOCKET_PORT);
        ConnectingState {
            pending_connection: networking::connect(WEBSOCKET_ADDRESS, WEBSOCKET_PORT, WEBSOCKET_SECURE),
            text: CONNECTING_TEXT
        }
    }
}

impl State for ConnectingState {
    fn update_and_draw(&mut self, _assets: &AssetManager, _delta: f32) -> Option<Box<dyn State>> {
        match self.pending_connection.ready() {
            Ok(connection_option) => {
                if let Some(connection) = connection_option {
                    log::info!("Connection to server established!");

                    return Some(Box::new(ConnectedState::new(connection)));
                }
            }

            Err(e) => {
                log::error!("Failed to connect to server due to error: {}", e);

                self.text = FAILED_TEXT;
            }
        }

        quad::draw_text(self.text, 0.0, 0.0, 32.0, quad::WHITE);

        None
    }

    fn title(&self) -> &'static str {
        "Connecting To Server"
    }
}

struct ConnectedState {
    connection: Option<networking::Connection>,
    text: &'static str
}

impl ConnectedState {
    fn new(mut connection: networking::Connection) -> Self {
        let hello_msg = messages::ToServer::Hello { client_id_option: sessions::retrieve_client_id() };

        let text = match connection.send(&hello_msg) {
            Ok(_) => {
                log::debug!("Sent 'hello' message to server: {}", hello_msg);
                CONNECTING_TEXT
            }

            Err(e) => {
                log::error!("Failed to send 'hello' message due to error: {}", e);
                FAILED_TEXT
            }
        };

        ConnectedState { connection: Some(connection), text }
    }
}

impl State for ConnectedState {
    fn update_and_draw(&mut self, _assets: &AssetManager, _delta: f32) -> Option<Box<dyn State>> {
        match self.connection.as_mut().unwrap().receive() {
            Ok(msg_option) => {
                if let Some(msg) = msg_option {
                    match msg {
                        messages::FromServer::Welcome {
                            version,
                            your_client_id,
                            your_entity_with_id: (entity_id, entity)
                        } => {
                            log::debug!("Server version: {}", version);

                            // Check version aligns with that of the version:

                            if version == shared::VERSION {
                                // Save the client ID (browser local storage):

                                log::debug!("Given client ID: {}", your_client_id);

                                sessions::store_client_id(your_client_id);

                                // Enter the main game state:

                                log::debug!("Given player entity: {} - {}", entity, entity_id);

                                let my_entity = MyEntity::new(entity_id, entity);
                                let taken_connection = self.connection.take().unwrap();
                                let game_state = super::game::GameState::new(taken_connection, my_entity);

                                return Some(Box::new(game_state));
                            }
                            else {
                                log::error!(
                                    "Version of server ({}) differs from that of this client ({})",
                                    version,
                                    shared::VERSION
                                );

                                self.text = WRONG_VERSION_TEXT;
                            }
                        }

                        other_msg => {
                            log::error!("Expected a 'welcome' message from server but instead received: {}", other_msg);

                            self.text = FAILED_TEXT;
                        }
                    }
                }
            }

            Err(e) => {
                log::error!("Connecting error while waiting to receive a 'welcome' message: {}", e);
                self.text = FAILED_TEXT;
            }
        }

        quad::draw_text(self.text, 0.0, 0.0, 32.0, quad::WHITE);

        None
    }

    fn title(&self) -> &'static str {
        "Connected To Server"
    }
}
