use macroquad::prelude as quad;

use core::{ maps::ChunkCoords, messages };

use crate::{
    maps,
    networking::{ self, ConnectionTrait }
};

use super::State;

pub struct GameState {
    connection: networking::Connection,
    map: maps::ClientMap
}

impl GameState {
    pub fn new(connection: networking::Connection) -> Self {
        let mut x = GameState { connection, map: maps::ClientMap::new() };
        // TODO: Temporary:
        x.connection.send(&messages::ToServer::RequestChunk(ChunkCoords { x: 0, y: 0 })).unwrap();
        x
    }
}

impl GameState {
    fn handle_message_from_server(&mut self, msg: messages::FromServer) -> networking::Result<()> {
        match msg {
            messages::FromServer::Welcome { version } => {
                log::warn!("Unexpectedly received 'welcome' message from server");
                unimplemented!()
            }

            messages::FromServer::ProvideChunk(coords, chunk) => {
                self.map.provide_chunk(coords, chunk, &mut self.connection)
            }

            messages::FromServer::UpdateTile(coords, tile) => unimplemented!() // TODO
        }
    }
}

impl State for GameState {
    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        quad::draw_text(self.title(), 0.0, 0.0, 32.0, quad::GREEN);

        match self.connection.receive::<messages::FromServer>() {
            Ok(msg_option) => if let Some(msg) = msg_option {
                log::info!("Received message from server: {}", msg);

                let result = self.handle_message_from_server(msg);

                if let Err(e) = result {
                    log::warn!("Error occurred during the handling of message from server: {}", e);
                }
            }
            Err(e) => {
                match e {
                    networking::Error::BincodeError(bincode_error) => {
                        log::warn!("Failed to decode message from server due to error: {}",
                                   bincode_error);
                    }
                    networking::Error::ConnectionError(connection_error) => {
                        log::warn!("Failed to receive from server due to connection error: {}",
                                   connection_error);
                    }

                    networking::Error::ConnectionClosed => {
                        log::error!("Connection closed by the server");
                    }
                }
                panic!() // TODO
            }
        }

        None
    }

    fn title(&self) -> &'static str { "Game" }
}