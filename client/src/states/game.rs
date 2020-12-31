use macroquad::prelude as quad;

use core::messages;

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
        GameState { connection, map: maps::ClientMap::new() }
    }
}

impl GameState {
    fn handle_message_from_server(&mut self, msg: messages::FromServer) {
        log::info!("Received message from server: {}", msg);

        match msg {
            messages::FromServer::ProvideChunk(coords, chunk) => {
                self.map.provide_chunk(coords, chunk, &mut self.connection).unwrap(); // TODO
            }
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