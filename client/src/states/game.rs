use macroquad::prelude as quad;
use shared::{
    maps::{entities::Direction, Map},
    messages
};

use super::State;
use crate::{
    maps::{self, entities::MyEntity},
    networking::{self, ConnectionTrait},
    rendering, AssetManager, TextureKey
};

pub struct GameState {
    /// Connection with the remote server.
    connection: networking::Connection,
    /// This client's player character entity.
    my_entity: MyEntity,
    /// The current world map that the player entity is in.
    map: maps::ClientMap,
    /// The rendering system used to draw the game map to the screen.
    map_renderer: rendering::maps::Renderer
}

impl GameState {
    pub fn new(connection: networking::Connection, my_entity: MyEntity) -> Self {
        GameState {
            connection,
            my_entity,
            map: maps::ClientMap::new(),
            map_renderer: rendering::maps::Renderer::new(0.08, 16)
        }
    }
}

impl GameState {
    fn handle_message_from_server(&mut self, msg: messages::FromServer) {
        match msg {
            messages::FromServer::Welcome { .. } => {
                log::warn!("Unexpectedly received 'welcome' message from server");
                unimplemented!()
            }

            messages::FromServer::ProvideChunk(coords, chunk) => {
                self.map.add_chunk(coords, chunk);
            }

            messages::FromServer::ShouldUnloadChunk(coords) => {
                self.map.remove_chunk(coords);
            }

            messages::FromServer::ChangeTile(coords, tile) => {
                if self.map.is_tile_loaded(coords) {
                    self.map.set_loaded_tile_at(coords, tile);
                }
                else {
                    log::warn!(
                        "Told by server to change tile at {} to {:?} yet that tile's chunk is not loaded",
                        coords,
                        tile
                    );
                }
            }

            messages::FromServer::YourEntityMoved { request_number, new_position } => {
                self.my_entity.received_movement_reconciliation(request_number, new_position, &mut self.map_renderer);
            }

            messages::FromServer::MoveEntity(id, pos) => {
                self.map.set_remote_entity_position(id, pos, &mut self.map_renderer);
            }

            messages::FromServer::ProvideEntity(id, entity) => {
                self.map.add_entity(id, entity);
            }

            messages::FromServer::ShouldUnloadEntity(id) => {
                self.map.remove_entity(id);
            }
        }
    }
}

impl State for GameState {
    fn required_textures(&self) -> &[TextureKey] {
        &[TextureKey::Tiles, TextureKey::Entities]
    }

    fn update_and_draw(&mut self, assets: &AssetManager, delta: f32) -> Option<Box<dyn State>> {
        // Rendering:

        self.map_renderer.draw(&self.map, assets, delta);
        //self.ui_renderer.draw(...);

        // Player entity updates/input handling:

        self.my_entity.update(delta);

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
            // TODO: Don't just unwrap.
            self.my_entity
                .move_towards_checked(direction, &mut self.map, &mut self.connection, &mut self.map_renderer)
                .unwrap();
        }

        // Networking:

        match self.connection.receive::<messages::FromServer>() {
            Ok(msg_option) => {
                if let Some(msg) = msg_option {
                    log::info!("Received message from server: {}", msg);

                    self.handle_message_from_server(msg);
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

    fn title(&self) -> &'static str {
        "Game"
    }
}
