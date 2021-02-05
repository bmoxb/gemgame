use macroquad::prelude as quad;
use shared::{
    messages,
    world::{entities::Direction, maps::Map}
};

use super::State;
use crate::{
    networking::{self, ConnectionTrait},
    world::{entities::PlayerEntity, maps},
    AssetManager, TextureKey
};

pub struct GameState {
    /// Connection with the remote server.
    connection: networking::Connection,
    /// The player character entity.
    player_entity: PlayerEntity,
    /// The current world map that the player entity is in.
    map: maps::ClientMap,
    /// The rendering system used to draw the current world map to the screen.
    map_renderer: maps::rendering::Renderer
}

impl GameState {
    pub fn new(connection: networking::Connection, player_entity: PlayerEntity) -> Self {
        GameState {
            connection,
            map: maps::ClientMap::new(),
            map_renderer: maps::rendering::Renderer::new(0.08, 16),
            player_entity
        }
    }
}

impl GameState {
    fn handle_message_from_server(&mut self, msg: messages::FromServer) -> networking::Result<()> {
        match msg {
            messages::FromServer::Welcome { .. } => {
                log::warn!("Unexpectedly received 'welcome' message from server");
                unimplemented!()
            }

            messages::FromServer::ProvideChunk(coords, chunk) => {
                log::debug!("Chunk loaded from server: {}", coords);

                self.map.provide_chunk(coords, chunk);
                Ok(())
            }

            messages::FromServer::UpdateTile(_coords, _tile) => {
                unimplemented!() // TODO
            }

            messages::FromServer::YourEntityMoved { request_number, new_position } => {
                self.player_entity.received_movement_reconciliation(request_number, new_position);
                Ok(())
            }

            messages::FromServer::EntityMoved(_entity_id, _coords) => {
                unimplemented!() // TODO
            }

            messages::FromServer::ProvideEntity(_entity_id, _entity) => {
                unimplemented!() // TODO
            }
        }
    }
}

impl State for GameState {
    fn required_textures(&self) -> &[TextureKey] { &[TextureKey::Tiles, TextureKey::Entities] }

    fn update_and_draw(&mut self, assets: &AssetManager, delta: f32) -> Option<Box<dyn State>> {
        // Map updates:

        self.map_renderer.draw(&mut self.map, assets.texture(TextureKey::Tiles), assets.texture(TextureKey::Entities));

        self.map.request_needed_chunks_from_server(&mut self.connection).unwrap(); // TODO

        // Player entity updates/input handling:

        self.player_entity.update(delta);

        let direction_option = {
            if quad::is_key_down(quad::KeyCode::W) {
                Some(Direction::Up)
            }
            else if quad::is_key_down(quad::KeyCode::A) {
                Some(Direction::Left)
            }
            else if quad::is_key_down(quad::KeyCode::S) {
                Some(Direction::Down)
            }
            else if quad::is_key_down(quad::KeyCode::D) {
                Some(Direction::Right)
            }
            else {
                None
            }
        };

        if let Some(direction) = direction_option {
            self.player_entity.move_towards(direction, &mut self.map, &mut self.connection).unwrap(); // TODO
        }

        // Networking:

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
