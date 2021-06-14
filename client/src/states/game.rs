use macroquad::prelude as quad;
use shared::{
    maps::{entities::Direction, Map},
    messages
};

use super::State;
use crate::{
    maps::{self, entities::MyEntity, MapRenderer},
    networking::{self, ConnectionTrait},
    ui::{self, Ui},
    AssetManager, TextureKey
};

pub struct GameState {
    /// Connection with the remote server.
    connection: networking::Connection,
    /// This client's player character entity.
    my_entity: MyEntity,
    /// The current world map that the player entity is in.
    map: maps::ClientMap,
    /// The rendering system used to draw the game map to the screen.
    map_renderer: MapRenderer,
    /// User interface.
    ui: Ui
}

impl GameState {
    pub fn new(connection: networking::Connection, my_entity: MyEntity) -> Self {
        let my_entity_pos = my_entity.get_pos();
        GameState {
            connection,
            my_entity,
            map: maps::ClientMap::new(),
            map_renderer: MapRenderer::new(0.1, my_entity_pos),
            ui: Ui::new(0.12)
        }
    }

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

            messages::FromServer::MoveEntity(id, pos, direction) => {
                self.map.move_remote_entity(id, pos, direction, &mut self.map_renderer);
            }

            messages::FromServer::ProvideEntity(id, entity) => {
                self.map_renderer.add_remote_entity(id, entity.pos);
                self.map.add_entity(id, entity);
            }

            messages::FromServer::ShouldUnloadEntity(id) => {
                self.map_renderer.remove_remote_entity(id);
                self.map.remove_entity(id);
            }

            messages::FromServer::BombPlaced { placed_by_entity_id, position } => {
                self.map.set_bomb_at(position, placed_by_entity_id);
            }

            messages::FromServer::BombsDetonated { placed_by_entity_id: _, in_and_around_chunk_coords: _ } => {
                unimplemented!()
            } // TODO

            messages::FromServer::YouCollectedGems { gem_type, quantity_increase } => {
                self.my_entity.obtained_gems(gem_type, quantity_increase);
            }
        }
    }
}

impl State for GameState {
    fn required_textures(&self) -> &[TextureKey] {
        &[TextureKey::Tiles, TextureKey::Entities, TextureKey::Bombs, TextureKey::Ui]
    }

    fn update_and_draw(&mut self, assets: &AssetManager, delta: f32) -> Option<Box<dyn State>> {
        // Update UI:
        self.ui.update(&mut self.my_entity, &mut self.map, &mut self.map_renderer, &mut self.connection).unwrap(); // TODO: Don't unwrap.

        // Rendering:

        self.map_renderer.draw(&self.map, self.my_entity.get_contained_entity(), assets, delta);
        self.ui.draw(assets);

        #[cfg(debug_assertions)]
        ui::draw_debug_text(
            28.0,
            quad::DARKPURPLE,
            assets,
            self.my_entity.get_contained_entity(),
            self.map.get_loaded_chunk_coords()
        );

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
                    networking::Error::Bincode(bincode_error) => {
                        log::warn!("Failed to decode message from server due to error: {}", bincode_error);
                    }
                    networking::Error::Connection(connection_error) => {
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
