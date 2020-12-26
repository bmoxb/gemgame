use macroquad::prelude as quad;

use core::messages;

use crate::networking::{ self, ConnectionTrait };
use super::State;

pub struct GameState {
    connection: networking::Connection
}

impl GameState {
    pub fn new(connection: networking::Connection) -> Self {
        GameState { connection }
    }
}

impl GameState {
    fn handle_message_from_server(&mut self, msg: messages::FromServer) {
        log::info!("Received message from server: {}", msg);

        match msg {
            messages::FromServer::ProvideChunk(coords, chunk) => unimplemented!(),
            messages::FromServer::UpdateTile(coords, tile) => unimplemented!()
        }
    }
}

impl State for GameState {
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        quad::draw_text(self.title(), 0.0, 0.0, 32.0, quad::GREEN);

        match self.connection.receive::<messages::FromServer>() {
            Ok(msg_option) => if let Some(msg) = msg_option {
                self.handle_message_from_server(msg);
            }
            Err(e) => match e {
                networking::Error::BincodeError(bincode_e) => {
                    log::warn!("Failed to decode message from server due to error: {}",
                               bincode_e);
                }
                networking::Error::ConnectionError(connection_e) => {
                    log::warn!("Failed to receive from server due to connection error: {}",
                               connection_e);
                    // TODO: Attempt to reconnect...
                }
            }
        }

        None
    }

    fn title(&self) -> &'static str { "Game" }
}