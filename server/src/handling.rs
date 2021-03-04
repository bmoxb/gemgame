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
    stream: TcpStream, address: SocketAddr, game_map: Shared<ServerMap>, db_pool: sqlx::SqlitePool,
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
    db_pool: sqlx::SqlitePool,
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
                log::debug!("Performed WebSocket handshake successfully with: {}", self.address);

                // Handle the connection should the handshake complete successfully:

                match self.handle_websocket_connection(ws).await {
                    Ok(_) => {}

                    Err(Error::NetworkError(e)) => match e {
                        networking::Error::MessageNotBinary(msg) => {
                            log::error!("Message from {} is not binary: {}", self.address, msg);
                        }

                        networking::Error::EncodingFailure(bincode_err) => {
                            log::error!(
                                "Failed to communicate with client {} due to encoding error: {}",
                                self.address,
                                bincode_err
                            );
                        }

                        networking::Error::NetworkError(network_err) => match network_err {
                            tungstenite::Error::Protocol(vioation) if vioation.contains("closing handshake") => {
                                log::debug!(
                                    "Client {} closed connection without performing the closing handshake",
                                    self.address
                                );
                            }

                            other => {
                                log::error!(
                                    "Failed to communicate with client {} due to network error: {}",
                                    self.address,
                                    other
                                );
                            }
                        }
                    },

                    Err(Error::DatabaseError(e)) => {
                        log::error!("Handling client {} resulted in database error: {}", self.address, e);
                    }
                }

                log::info!("Client disconnected: {}", self.address);
            }

            Err(e) => {
                log::warn!("Failed to perform WebSocket handshake with {} - {}", self.address, e);
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
                    log::debug!("Client {} has existing ID: {}", self.address, client_id);

                    // Get the client their existing player entity (if any) from database:

                    if let Some((entity_id, entity)) = entities::player_from_database(client_id, &mut db).await? {
                        (client_id, entity_id, entity)
                    }
                    else {
                        log::warn!(
                            "Client {} provided {} for which an associate entity could not be found in the database",
                            self.address,
                            client_id
                        );

                        let (entity_id, entity) = entities::new_player_in_database(client_id, &mut db).await?;
                        (client_id, entity_id, entity)
                    }
                }
                else {
                    let new_id = crate::id::generate_random();
                    log::debug!("Generated new ID {} for client {}", new_id, self.address);

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

            // Place this client's player entity on the game map:
            self.game_map.lock().unwrap().add_entity(player_id, player_entity);

            // Inform other tasks that a new entity now exists on the game map:
            self.map_changes_sender.send(maps::Modification::EntityAdded(player_id)).unwrap();
            self.map_changes_receiver.recv().await.unwrap();

            // Begin main connection loop:
            let result = self.handle_established_connection(&mut ws, client_id, player_id).await;

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
            log::warn!(
                "Client {} failed to send 'hello' message after establishing a WebSocket connection",
                self.address
            );
            ws.close().await.map_err(Into::into)
        }
    }

    /// A connection is considered 'established' once the WebSocket handshake and the exchange of 'hello' & 'welcome'
    /// messages have completed.
    async fn handle_established_connection(&mut self, ws: &mut Connection, client_id: Id, player_id: Id) -> Result<()> {
        loop {
            // Wait for incoming messages on both the WebSocket connection and the world modifications channel (or close
            // connection on Ctrl-C signal):
            tokio::select!(
                res = ws.receive() => {
                    if let Some(msg) = res? {
                        log::debug!("Message from client {} - {}", client_id, msg);

                        // Handle and respond to received message:

                        let responses = self.handle_message(msg, client_id, player_id).await;

                        for response in responses {
                            log::debug!("Response to client {} - {}", client_id, response);
                            ws.send(&response).await?;
                        }
                    }
                    else {
                        log::debug!("Connection closed by client {}", client_id);
                        break;
                    }
                }

                res = self.map_changes_receiver.recv() => {
                    match res {
                        Ok(modification) => {
                            if let Some(response) = self.handle_map_change(modification).await {
                                log::debug!("Informing client {} of change to game world - {}", client_id, response);
                                ws.send(&response).await?;
                            }
                        }

                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            log::warn!("Task for client {} skipped {} messages on the map modification channel", client_id, skipped);
                        }

                        Err(channel_err) => {
                            log::warn!("Failed to receive on map modification channel: {}", channel_err);
                            break;
                        }
                    }
                }

                _ = tokio::signal::ctrl_c() => {
                    log::debug!("Closing connection with client {} due to Ctrl-C signal", client_id);
                    ws.close().await?;
                    break;
                }
            );
        }

        Ok(())
    }

    /// Produces message(s) that are to be sent to the client in the response to the message they sent to the server.
    async fn handle_message(
        &mut self, msg: messages::ToServer, client_id: Id, player_id: Id
    ) -> Vec<messages::FromServer> {
        match msg {
            messages::ToServer::Hello { .. } => {
                log::warn!("Client {} sent unexpected 'hello' message: {}", client_id, msg);
                vec![]
            }

            messages::ToServer::MoveMyEntity { request_number, direction } => {
                let responses = {
                    // TODO: Prevent player exceeding movement rate.

                    let movement_option = self.game_map.lock().unwrap().move_entity_towards(player_id, direction);

                    if let Some((old_position, new_position)) = movement_option {
                        // Inform other tasks of the entity's movement:

                        let broadcast_msg =
                            maps::Modification::EntityMoved { entity_id: player_id, old_position, new_position };

                        self.map_changes_sender.send(broadcast_msg).unwrap();

                        vec![messages::FromServer::YourEntityMoved { request_number, new_position }]
                    }
                    else {
                        vec![]
                    }
                };

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
                    self.make_provide_entity_msg_if_in_loaded_chunk(entity_id)
                }
                else {
                    None
                }
            }

            maps::Modification::EntityAdded(id) => self.make_provide_entity_msg_if_in_loaded_chunk(id),

            maps::Modification::EntityRemoved(id, chunk_coords) => self
                .remote_loaded_chunk_coords
                .contains(&chunk_coords)
                .then(|| messages::FromServer::ShouldUnloadEntity(id))
        }
    }

    /// Produces a `messages::FromServer::ProvideEntity` message provided the entity with the specified ID is within one
    /// of the remote client's loaded chunks.
    fn make_provide_entity_msg_if_in_loaded_chunk(&self, entity_id: Id) -> Option<messages::FromServer> {
        if let Some(entity) = self.game_map.lock().unwrap().entity_by_id(entity_id) {
            self.remote_loaded_chunk_coords
                .contains(&entity.pos.as_chunk_coords())
                .then(|| messages::FromServer::ProvideEntity(entity_id, entity.clone()))
        }
        else {
            log::warn!(
                "Message on map modification channel refers to entity {} yet an entity with that ID could not be found",
                entity_id
            );
            None
        }
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
