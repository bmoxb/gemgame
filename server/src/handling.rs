use std::{collections::HashSet, convert::Into, net::SocketAddr};

use shared::{
    maps::{ChunkCoords, Map},
    messages, Id
};
use thiserror::Error;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::tungstenite;

use crate::{
    maps::{self, entities, ServerMap},
    networking::{self, Connection},
    Shared
};

/// Creates a new `Handler` instance and then calls its `Handler::handle` method.
pub async fn handle_connection(
    stream: TcpStream, address: SocketAddr, game_map: Shared<ServerMap>, db_pool: sqlx::Pool<sqlx::Any>,
    map_changes_sender: broadcast::Sender<maps::Modification>,
    map_changes_receiver: broadcast::Receiver<maps::Modification>
) {
    let mut handler = Handler {
        address,
        game_map,
        db_pool,
        map_changes_sender,
        map_changes_receiver,
        remote_loaded_chunk_coords: HashSet::new()
    };

    handler.handle(stream).await;
}

/// Structure containing information required by a connection coroutine/task.
struct Handler {
    /// The address of the remote client.
    address: SocketAddr,
    /// Arc mutex containing the game map.
    game_map: Shared<ServerMap>,
    /// The database connection pool.
    db_pool: sqlx::Pool<sqlx::Any>,
    map_changes_sender: broadcast::Sender<maps::Modification>,
    map_changes_receiver: broadcast::Receiver<maps::Modification>,
    // Set used tos track of the coordinates of chunks that this handler believes its remote client has loaded.
    remote_loaded_chunk_coords: HashSet<ChunkCoords>
}

impl Handler {
    /// Handle a connection with the client connected via the given TCP/IP stream.
    async fn handle(&mut self, stream: TcpStream) {
        // Perform the WebSocket handshake:

        match Connection::new(stream).await {
            Ok(ws) => {
                self.log("Performed WebSocket handshake successfully");

                // Handle the connection should the handshake complete successfully:

                match self.handle_websocket_connection(ws).await {
                    Ok(_) => {}

                    Err(Error::NetworkError(e)) => match e {
                        networking::Error::MessageNotBinary(msg) => {
                            self.log_error(&format!("Message is not binary: {}", msg));
                        }

                        networking::Error::EncodingFailure(bincode_err) => {
                            self.log_error(&format!("Failed to communicate due to encoding error: {}", bincode_err));
                        }

                        networking::Error::NetworkError(network_err) => match network_err {
                            tungstenite::Error::Protocol(vioation) if vioation.contains("closing handshake") => {
                                self.log("Connection closed without performing the closing handshake");
                            }

                            other => {
                                self.log_error(&format!("Failed to communicate due to network error: {}", other));
                            }
                        }
                    },

                    Err(Error::DatabaseError(db_err)) => {
                        self.log_error(&format!("Encountered database error: {}", db_err));
                    }
                }

                log::info!("Client disconnected: {}", self.address);
            }

            Err(e) => {
                self.log_error(&format!("Failed to perform WebSocket handshake due to error: {}", e));
            }
        }
    }

    /// This function is to be called after the WebSocket connection handshake finishes. It is the role of this function
    /// to complete the exchange of 'hello' and 'welcome' messages between client and server before passing control onto
    /// the [`Self::handle_established_connection`] method.
    async fn handle_websocket_connection(&mut self, mut ws: Connection) -> Result<()> {
        // Expect a 'hello' message from the client:

        if let Some(messages::ToServer::Hello { client_id_option }) = ws.receive().await? {
            let (client_id, player_id, player_entity) = {
                let mut db = self.db_pool.acquire().await.unwrap();

                if let Some(client_id) = client_id_option {
                    self.log(&format!("Existing client ID provided: {}", client_id));

                    // Get the client their existing player entity (if any) from database:

                    if let Some((entity_id, entity)) = entities::player_from_database(client_id, &mut db).await? {
                        (client_id, entity_id, entity)
                    }
                    else {
                        self.log_warn(&format!(
                            "Could not find in database a player entity associated with client ID {}",
                            client_id
                        ));

                        let (entity_id, entity) = entities::new_player_in_database(client_id, &mut db).await?;
                        (client_id, entity_id, entity)
                    }
                }
                else {
                    let new_id = crate::id::generate_random();
                    self.log(&format!("Generated new client ID {}", new_id));

                    // Create a new entity for this client and insert into the database:

                    let (new_entity_id, new_entity) = entities::new_player_in_database(new_id, &mut db).await?;
                    (new_id, new_entity_id, new_entity)
                }
            };

            // Send a 'welcome' message to the client:
            ws.send(&messages::FromServer::Welcome {
                version: shared::VERSION.to_string(),
                your_client_id: client_id,
                your_entity_with_id: (player_id, player_entity.clone())
            })
            .await?;

            // Provide all the chunks surrounding the player entity plus any entities that may be in those chunks:

            let chunks_and_entities = self
                .provide_chunks_at_and_surrounding_with_entities(player_entity.pos.as_chunk_coords(), player_id)
                .await;

            for msg in chunks_and_entities {
                ws.send(&msg).await?;
            }

            // Place this client's player entity on the game map:
            self.game_map.lock().unwrap().add_entity(player_id, player_entity);

            // Inform other tasks that a new entity now exists on the game map:
            self.map_changes_sender.send(maps::Modification::EntityAdded(player_id)).unwrap();
            self.map_changes_receiver.recv().await.unwrap();

            // Begin main connection loop:
            let result = self.handle_established_connection(&mut ws, player_id).await;

            // Remove this client's player entity from the game world and update database with changes to said entity:
            let entity_option = self.game_map.lock().unwrap().remove_entity(player_id);
            if let Some(player_entity) = entity_option {
                {
                    let mut db = self.db_pool.acquire().await.unwrap();
                    entities::update_database_for_player(&player_entity, client_id, &mut db).await?;
                }

                // Inform other tasks that an entity has been removed from the game map:
                let modification_msg =
                    maps::Modification::EntityRemoved(player_id, player_entity.pos.as_chunk_coords());
                self.map_changes_sender.send(modification_msg).unwrap();
                self.map_changes_receiver.recv().await.unwrap();
            }

            result
        }
        else {
            self.log_error("Did not receive 'hello' message after establishing a WebSocket connection");
            ws.close().await.map_err(Into::into)
        }
    }

    /// A connection is considered 'established' once the WebSocket handshake and the exchange of 'hello' & 'welcome'
    /// messages have completed.
    async fn handle_established_connection(&mut self, ws: &mut Connection, player_id: Id) -> Result<()> {
        loop {
            // Wait for incoming messages on both the WebSocket connection and the world modifications channel (or close
            // connection on Ctrl-C signal):
            tokio::select!(
                res = ws.receive() => {
                    if let Some(msg) = res? {
                        self.log(&format!("Message received: {}", msg));

                        // Handle and respond to received message:

                        let responses = self.handle_message(msg, player_id).await;

                        for response in responses {
                            self.log(&format!("Response message: {}", response));
                            ws.send(&response).await?;
                        }
                    }
                    else {
                        // Client closed connection.
                        break;
                    }
                }

                res = self.map_changes_receiver.recv() => {
                    match res {
                        Ok(modification) => {
                            if let Some(response) = self.handle_map_change(modification).await {
                                self.log(&format!("Informing client of change to game world: {}", response));
                                ws.send(&response).await?;
                            }
                        }

                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            self.log_warn(&format!("Skipped {} messages on the map modification channel", skipped));
                        }

                        Err(channel_err) => {
                            self.log_error(&format!("Failed to receive on map modification channel: {}", channel_err));
                            break;
                        }
                    }
                }

                _ = tokio::signal::ctrl_c() => {
                    self.log("Closing connection due to Ctrl-C signal");
                    ws.close().await?;
                    break;
                }
            );
        }

        Ok(())
    }

    /// Produces message(s) that are to be sent to the client in the response to the message they sent to the server.
    async fn handle_message(&mut self, msg: messages::ToServer, player_id: Id) -> Vec<messages::FromServer> {
        match msg {
            messages::ToServer::Hello { .. } => {
                self.log_warn(&format!("Received unexpected 'hello' message: {}", msg));
                vec![]
            }

            messages::ToServer::MoveMyEntity { request_number, direction } => {
                let mut responses = Vec::new();

                // TODO: Prevent player exceeding movement rate.

                let movement_option = self.game_map.lock().unwrap().move_entity_towards(player_id, direction);

                if let Some((old_position, new_position)) = movement_option {
                    // If moving into a new chunk, ensure adjacent chunks are loaded:

                    if old_position.as_chunk_coords() != new_position.as_chunk_coords() {
                        let msgs = self
                            .provide_chunks_at_and_surrounding_with_entities(new_position.as_chunk_coords(), player_id)
                            .await;

                        responses.extend(msgs);
                    }

                    // Inform other tasks of the entity's movement:

                    let broadcast_msg =
                        maps::Modification::EntityMoved { entity_id: player_id, old_position, new_position };

                    self.map_changes_sender.send(broadcast_msg).unwrap();

                    // Inform our remote client of the server's decision regarding their player entity's new position:
                    responses.push(messages::FromServer::YourEntityMoved { request_number, new_position });
                }

                if !responses.is_empty() {
                    // The broadcast isn't relevant to the task that sent it so immediately receive and discard:
                    self.map_changes_receiver.recv().await.unwrap();
                }

                responses
            }
        }
    }

    /// May produce a message that is to be sent to the client based on map modification messages received from other
    /// connection handling tasks.
    async fn handle_map_change(&mut self, modification: maps::Modification) -> Option<messages::FromServer> {
        match modification {
            maps::Modification::TileChanged(position, tile) => {
                let is_position_loaded = self.remote_loaded_chunk_coords.contains(&position.as_chunk_coords());
                is_position_loaded.then(|| messages::FromServer::ChangeTile(position, tile))
            }

            maps::Modification::EntityMoved { entity_id, old_position, new_position } => {
                let was_in_loaded = self.remote_loaded_chunk_coords.contains(&old_position.as_chunk_coords());
                let is_in_loaded = self.remote_loaded_chunk_coords.contains(&new_position.as_chunk_coords());

                if was_in_loaded && is_in_loaded {
                    // Entity moving within the bounds of the client's loaded chunks:
                    Some(messages::FromServer::MoveEntity(entity_id, new_position))
                }
                else if was_in_loaded {
                    // Entity moved out of the client's loaded chunks:
                    Some(messages::FromServer::ShouldUnloadEntity(entity_id))
                }
                else if is_in_loaded {
                    // Entity just moved into the client's loaded chunks:
                    self.game_map
                        .lock()
                        .unwrap()
                        .entity_by_id(entity_id)
                        .map(|entity| messages::FromServer::ProvideEntity(entity_id, entity.clone()))
                }
                else {
                    None
                }
            }

            maps::Modification::EntityAdded(entity_id) => {
                self.game_map.lock().unwrap().entity_by_id(entity_id).and_then(|entity| {
                    self.remote_loaded_chunk_coords
                        .contains(&entity.pos.as_chunk_coords())
                        .then(|| messages::FromServer::ProvideEntity(entity_id, entity.clone()))
                })
            }

            maps::Modification::EntityRemoved(entity_id, chunk_coords) => self
                .remote_loaded_chunk_coords
                .contains(&chunk_coords)
                .then(|| messages::FromServer::ShouldUnloadEntity(entity_id))
        }
    }

    /// Will begin by ensuring the chunk at the specified coordinates is loaded (i.e. if not already in-memory within
    /// the game map object, it will either be loaded from disk or newly generated before being added to the game map).
    /// Messages will then be created to provide the remote client with the chunk as well as any entities in said chunk.
    /// This method will add the given chunk coordinates to the set of remote loaded chunk coordinates however it is
    /// the responsiblity of the caller to actually send the messages returned over the network.
    async fn provide_chunk_with_entities(&mut self, coords: ChunkCoords, player_id: Id) -> Vec<messages::FromServer> {
        let mut msgs = Vec::new();

        if !self.remote_loaded_chunk_coords.contains(&coords) {
            let chunk = maps::chunks::get_or_load_or_generate_chunk(&self.game_map, coords).await;
            msgs.push(messages::FromServer::ProvideChunk(coords, chunk));

            // Get entities in the chunk but filter out this task's own player entity:
            let entities_in_chunk =
                self.game_map.lock().unwrap().entities_in_chunk(coords).into_iter().filter(|(id, _)| *id != player_id);

            for (entity_id, entity) in entities_in_chunk {
                msgs.push(messages::FromServer::ProvideEntity(entity_id, entity));
            }

            self.remote_loaded_chunk_coords.insert(coords);
        }

        msgs
    }

    /// Call `Self::provide_chunk_with_entities` for the specified chunk coordinates as well as the 9 surrounding set
    /// of coordinates.
    async fn provide_chunks_at_and_surrounding_with_entities(
        &mut self, coords: ChunkCoords, player_id: Id
    ) -> Vec<messages::FromServer> {
        let mut msgs = Vec::new();

        for x_offset in -1..2 {
            for y_offset in -1..2 {
                let msg = self
                    .provide_chunk_with_entities(
                        ChunkCoords { x: coords.x + x_offset, y: coords.y + y_offset },
                        player_id
                    )
                    .await;

                msgs.extend(msg);
            }
        }

        msgs
    }

    fn log(&self, msg: &str) {
        log::debug!("Handler for client {} -- {}", self.address, msg);
    }

    fn log_warn(&self, msg: &str) {
        log::warn!("Handler for client {} -- {}", self.address, msg);
    }

    fn log_error(&self, msg: &str) {
        log::error!("Handler for client {} -- {}", self.address, msg);
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("Networking error")]
    NetworkError(#[from] networking::Error),
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error)
}

type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        path::PathBuf,
        sync::{Arc, Mutex}
    };

    use shared::maps::{
        entities::{Direction, Entity},
        Chunk, ChunkCoords, Tile, TileCoords, CHUNK_TILE_COUNT, CHUNK_WIDTH
    };

    use super::*;

    async fn make_test_handler() -> Handler {
        let (map_changes_sender, map_changes_receiver) = broadcast::channel(5);

        let handler = super::Handler {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
            db_pool: sqlx::any::AnyPoolOptions::new().connect("sqlite::memory:").await.unwrap(),
            game_map: Arc::new(Mutex::new(ServerMap::new(
                PathBuf::from("/tmp"),
                Box::new(maps::generators::DefaultGenerator),
                0
            ))),
            map_changes_sender,
            map_changes_receiver,
            remote_loaded_chunk_coords: HashSet::new()
        };

        handler
    }

    impl Handler {
        fn add_test_entity(&mut self, pos: TileCoords) -> Id {
            let entity_id = crate::id::generate_with_timestamp();

            self.game_map
                .lock()
                .unwrap()
                .add_entity(entity_id, Entity { pos, name: "test".to_string(), variety: Default::default() });

            entity_id
        }
    }

    /// Ensure that no response is provided when an unexpected (i.e. sent after connection establishment) 'hello'
    /// message is recevied.
    #[tokio::test(flavor = "multi_thread")]
    async fn handle_unexpected_hello_msg() {
        let mut handler = make_test_handler().await;

        let id = crate::id::generate_random();
        let msg = messages::ToServer::Hello { client_id_option: None };

        assert!(handler.handle_message(msg, id).await.is_empty());
    }

    /// Ensure that a 'move my entity' message causes the entity's position on the game map to be appropriately
    /// updated, a response message is created, and that a message on the map changes broadcast channel is sent to
    /// inform other tasks of the change.
    #[tokio::test(flavor = "multi_thread")]
    async fn handle_move_my_entity() {
        let mut handler = make_test_handler().await;
        let mut other_map_changes_receiver = handler.map_changes_sender.subscribe();

        let player_id = handler.add_test_entity(TileCoords { x: 5, y: 5 });

        handler
            .game_map
            .lock()
            .unwrap()
            .add_chunk(ChunkCoords { x: 0, y: 0 }, Chunk::new([Tile::Ground; CHUNK_TILE_COUNT]));

        let msg = messages::ToServer::MoveMyEntity { request_number: 0, direction: Direction::Right };

        let responses = handler.handle_message(msg, player_id).await;

        // Ensure the response message is correct:
        assert_eq!(responses.len(), 1);
        assert!(matches!(
            responses[0],
            messages::FromServer::YourEntityMoved { request_number: 0, new_position: TileCoords { x: 6, y: 5 } }
        ));

        // A message should not be broadcast to this task's map changes recevier:
        assert!(matches!(handler.map_changes_receiver.try_recv(), Err(broadcast::error::TryRecvError::Empty)));

        // A message should have been sent to all the map changes receviers of other tasks:
        let change = other_map_changes_receiver.recv().await.unwrap();

        assert!(matches!(
            change,
            maps::Modification::EntityMoved {
                old_position: TileCoords { x: 5, y: 5 },
                new_position: TileCoords { x: 6, y: 5 },
                entity_id
            } if entity_id == player_id
        ));
    }

    /// Ensure that a 'move my entity' message that would fail due to a blocking tile or entity being in the way does
    /// not modify the player entity's position, and does *not* send a message on the map modifications channel.
    #[tokio::test(flavor = "multi_thread")]
    async fn handle_move_my_entity_blocking() {
        // TODO
    }

    /// Ensure that the appropriate map modification channel message is sent when some entity moves within this task's
    /// remote client's loaded chunks.
    #[tokio::test(flavor = "multi_thread")]
    async fn handle_entity_moved_change_within_loaded_chunk() {
        let mut handler = make_test_handler().await;
        handler.remote_loaded_chunk_coords.insert(ChunkCoords { x: 0, y: 0 });

        let entity_id = handler.add_test_entity(TileCoords { x: 5, y: 5 });
        let modification = maps::Modification::EntityMoved {
            entity_id,
            old_position: TileCoords { x: 5, y: 5 },
            new_position: TileCoords { x: 6, y: 5 }
        };

        let response = handler.handle_map_change(modification).await.unwrap();

        assert!(matches!(
            response,
            messages::FromServer::MoveEntity(id, TileCoords { x: 6, y: 5 }) if id == entity_id
        ));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn handle_entity_moved_change_into_loaded_chunk() {
        let mut handler = make_test_handler().await;
        handler.remote_loaded_chunk_coords.insert(ChunkCoords { x: 0, y: 0 });

        let entity_id = handler.add_test_entity(TileCoords { x: CHUNK_WIDTH, y: 5 });

        let modification = maps::Modification::EntityMoved {
            entity_id,
            old_position: TileCoords { x: CHUNK_WIDTH, y: 5 }, // chunk at 1, 0
            new_position: TileCoords { x: CHUNK_WIDTH - 1, y: 5 }  // chunk at 0, 0
        };

        let response = handler.handle_map_change(modification).await.unwrap();

        assert!(matches!(
            response,
            messages::FromServer::ProvideEntity(id, _) if id == entity_id
        ));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn handle_entity_moved_change_leaving_loaded_chunk() {
        let mut handler = make_test_handler().await;
        handler.remote_loaded_chunk_coords.insert(ChunkCoords { x: 0, y: 0 });

        let entity_id = handler.add_test_entity(TileCoords { x: 0, y: 5 });

        let modification = maps::Modification::EntityMoved {
            entity_id,
            old_position: TileCoords { x: 0, y: 5 },
            new_position: TileCoords { x: -1, y: 5 }
        };

        let response = handler.handle_map_change(modification).await.unwrap();

        assert!(matches!(
            response,
            messages::FromServer::ShouldUnloadEntity(id) if id == entity_id
        ));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn handle_entity_moved_change_outside_loaded_chunk() {
        let mut handler = make_test_handler().await;

        let entity_id = handler.add_test_entity(TileCoords { x: 12, y: 13 });

        let modification = maps::Modification::EntityMoved {
            entity_id,
            old_position: TileCoords { x: 12, y: 13 },
            new_position: TileCoords { x: 13, y: 13 }
        };

        assert!(handler.handle_map_change(modification).await.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn handle_entity_added_change_in_my_loaded_chunks() {
        // TODO
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn handle_entity_removed_change_in_my_loaded_chunks() {
        // TODO
    }
}
