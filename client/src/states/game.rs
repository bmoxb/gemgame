use macroquad::prelude as quad;
use shared::{
    maps::{ChunkCoords, Map},
    messages
};

use super::State;
use crate::{
    maps,
    networking::{self, ConnectionTrait},
    TextureKey
};

pub struct GameState {
    connection: networking::Connection,
    map: maps::ClientMap,
    map_renderer: maps::rendering::Renderer
}

impl GameState {
    pub fn new(connection: networking::Connection) -> Self {
        GameState { connection, map: maps::ClientMap::new(), map_renderer: maps::rendering::Renderer::new(0.1) }
    }
}

impl GameState {
    fn handle_message_from_server(&mut self, msg: messages::FromServer) -> networking::Result<()> {
        match msg {
            messages::FromServer::Welcome { version: _ } => {
                log::warn!("Unexpectedly received 'welcome' message from server");
                unimplemented!()
            }

            messages::FromServer::ProvideChunk(coords, chunk) => {
                log::debug!("Chunk loaded from server: {}", coords);

                self.map.provide_chunk(coords, chunk);
                Ok(())
            }

            messages::FromServer::UpdateTile(coords, tile) => unimplemented!() // TODO
        }
    }
}

impl State for GameState {
    fn required_textures(&self) -> &[TextureKey] { &[TextureKey::Tiles] }

    fn update_and_draw(&mut self, delta: f32) -> Option<Box<dyn State>> {
        quad::draw_text(self.title(), 0.0, 0.0, 32.0, quad::GREEN);

        self.map_renderer.draw(&mut self.map);

        self.map.request_needed_chunks_from_server(&mut self.connection).unwrap(); // TODO

        match self.connection.receive::<messages::FromServer>() {
            Ok(msg_option) => {
                if let Some(msg) = msg_option {
                    log::info!("Received message from server: {}", msg);

                    let result = self.handle_message_from_server(msg);

                    if let Err(e) = result {
                        log::warn!("Error occurred during the handling of message from server: {}", e);
                    }
                }
            }

            Err(e) => {
                match e {
                    networking::Error::BincodeError(bincode_error) => {
                        log::warn!("Failed to decode message from server due to error: {}", bincode_error);
                    }
                    networking::Error::ConnectionError(connection_error) => {
                        log::warn!("Failed to receive from server due to connection error: {}", connection_error);
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
