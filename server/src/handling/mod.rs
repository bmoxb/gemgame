mod tests;

use std::{convert::Into, net::SocketAddr};

use rand::Rng;
use shared::{
    items::{self, Item},
    maps::{ChunkCoords, Map, TileCoords},
    messages, Id
};
use thiserror::Error;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::tungstenite;

use crate::{
    maps::{self, entities, EntityMovement, ServerMap},
    networking::{self, Connection},
    Shared
};

const MAX_LOADED_CHUNKS_PER_CLIENT: usize = 12;

/// Creates a new [`Handler`] instance and then calls its [`Handler::handle`] method.
pub async fn handle_connection(
    stream: TcpStream, address: SocketAddr, game_map: Shared<ServerMap>, db_pool: sqlx::PgPool,
    map_changes_sender: broadcast::Sender<maps::Modification>,
    map_changes_receiver: broadcast::Receiver<maps::Modification>
) {
    let mut handler = Handler {
        address,
        game_map,
        db_pool,
        map_changes_sender,
        map_changes_receiver,
        remote_loaded_chunk_coords: Vec::new()
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
    db_pool: sqlx::PgPool,
    map_changes_sender: broadcast::Sender<maps::Modification>,
    map_changes_receiver: broadcast::Receiver<maps::Modification>,
    /// Set used to track of the coordinates of chunks that this handler's remote client has loaded. Stored as a vector
    /// so that chunk coordinate pairs can stored in from least to most recently loaded.
    remote_loaded_chunk_coords: Vec<ChunkCoords>
}

impl Handler {
    /// Handle a connection with the client connected via the given TCP/IP stream.
    async fn handle(&mut self, stream: TcpStream) {
        // Perform the WebSocket handshake:

        match Connection::new(stream).await {
            Ok(ws) => {
                self.log("Performed WebSocket handshake successfully");

                // Handle the connection should the handshake complete successfully:

                if let Err(err) = self.handle_websocket_connection(ws).await {
                    // Display a debug log message if connection is closed without performing the closing handshake. Log
                    // any other error messages at full error log level:
                    if matches!(
                        err,
                        Error::Network(networking::Error::Tungstenite(tungstenite::Error::Protocol(
                            tungstenite::error::ProtocolError::ResetWithoutClosingHandshake
                        )))
                    ) {
                        self.log("Connection closed without performing the closing handshake");
                    }
                    else {
                        self.log_error(&err.to_string());
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
                let mut db = self.db_pool.acquire().await?;

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
                .await?;

            for msg in chunks_and_entities {
                ws.send(&msg).await?;
            }

            // Place this client's player entity on the game map:
            self.game_map.lock().add_entity(player_id, player_entity);

            // Inform other tasks that a new entity now exists on the game map:
            self.map_changes_sender.send(maps::Modification::EntityAdded(player_id)).unwrap();
            self.map_changes_receiver.recv().await.unwrap();

            // Begin main connection loop:
            let result = self.handle_established_connection(&mut ws, player_id).await;

            // Ensure the game map knows that this client's loaded chunks are no longer needed by this task:
            for coords in &self.remote_loaded_chunk_coords {
                self.chunk_not_needed(*coords).await?;
            }

            // Remove this client's player entity from the game world and update database with changes to said entity:
            let entity_option = self.game_map.lock().remove_entity(player_id);
            if let Some(player_entity) = entity_option {
                {
                    let mut db = self.db_pool.acquire().await?;
                    entities::update_database_for_player(&player_entity, client_id, &mut db).await?;
                }

                // Inform other tasks that an entity has been removed from the game map:
                let modification_msg =
                    maps::Modification::EntityRemoved(player_id, player_entity.pos.as_chunk_coords());
                self.map_changes_sender.send(modification_msg).unwrap();
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

                        let responses = self.handle_message(msg, player_id).await?;

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
    async fn handle_message(&mut self, msg: messages::ToServer, player_id: Id) -> Result<Vec<messages::FromServer>> {
        match msg {
            messages::ToServer::Hello { .. } => {
                self.log_warn(&format!("Received unexpected 'hello' message: {}", msg));
                Ok(vec![])
            }

            messages::ToServer::MoveMyEntity { request_number, direction } => {
                let mut responses = Vec::new();

                // TODO: Prevent player exceeding movement rate.

                let movement_option = self.game_map.lock().move_entity_towards(player_id, direction);

                if let Some(EntityMovement { old_position, new_position, smashed_tile_option }) = movement_option {
                    // If moving into a new chunk, ensure chunks adjacent to the destination chunk are loaded and create
                    // message(s) to provide them to the client:
                    if old_position.as_chunk_coords() != new_position.as_chunk_coords() {
                        let msgs = self
                            .provide_chunks_at_and_surrounding_with_entities(new_position.as_chunk_coords(), player_id)
                            .await?;

                        responses.extend(msgs);
                    }

                    // Inform other tasks of the entity's movement:
                    self.map_changes_sender
                        .send(maps::Modification::EntityMoved {
                            entity_id: player_id,
                            old_position,
                            new_position,
                            direction
                        })
                        .unwrap();

                    // Confirm to the remote client that the movement could go ahead:
                    responses.push(messages::FromServer::YourEntityMoved { request_number, new_position });

                    if let Some(smashed_tile) = smashed_tile_option {
                        self.log(&format!("Smashed tile {:?} at {}", smashed_tile, new_position));

                        // If the smashed tile yields gems, calculate a quantity within the determined range, provide
                        // that quantity of gems to the player on the server side, and send a message to the remote
                        // client informing them of how many more gems they now have:

                        if let Some(gem_yield) = smashed_tile.get_gem_yield() {
                            // Random gem quantity within the range specified by the yield specific by the tile type:
                            let quantity_increase = rand::thread_rng()
                                .gen_range(gem_yield.minimum_quantity..(gem_yield.maximum_quantity + 1));

                            // Increase gem quantity on the server side:
                            if let Some(entity) = self.game_map.lock().entity_by_id_mut(player_id) {
                                entity.gem_collection.increase_quantity(gem_yield.gem, quantity_increase);
                            }

                            // Produce message to send to the remote client to inform them of how many more gems they
                            // now have:
                            responses.push(messages::FromServer::YouCollectedGems {
                                gem_type: gem_yield.gem,
                                quantity_increase
                            });

                            self.log(&format!(
                                "Obtained an additional {} gems of type {:?}",
                                quantity_increase, gem_yield.gem
                            ));
                        }
                    }
                }

                if responses.is_empty() {
                    // The `responses` vector will only be empty if the movement was not allowed. In that case, inform
                    // the remote client:

                    let new_position = self.game_map.lock().entity_by_id(player_id).unwrap().pos;
                    Ok(vec![messages::FromServer::YourEntityMoved { request_number, new_position }])
                }
                else {
                    // The `responses` vector will be populated only if the movement could go ahead. If it did then a
                    // message will be sent to all tasks informing them of the entity movement. That message isn't
                    // however relevant to the task that sent it so immediately receive and discard:
                    self.map_changes_receiver.recv().await.unwrap();

                    Ok(responses)
                }
            }

            messages::ToServer::PlaceBomb => {
                // Get player's position & check if the player actually posses a bomb to place:
                let (can_place_bomb, pos) = self
                    .game_map
                    .lock()
                    .entity_by_id(player_id)
                    .map(|player| (player.item_inventory.has_how_many(items::QuantitativeItem::Bomb) >= 1, player.pos))
                    .unwrap_or((false, TileCoords { x: 0, y: 0 }));

                if can_place_bomb {
                    // Place the bomb (server-side):
                    self.game_map.lock().set_bomb_at(pos, player_id);

                    // Inform other tasks that a bomb has been placed:
                    self.map_changes_sender.send(maps::Modification::BombPlaced(pos, player_id)).unwrap();

                    // The client that placed the bomb obviously does not need to be informed by the server that a bomb
                    // has been placed so immediately discarded map modification message on this task:
                    self.map_changes_receiver.recv().await.unwrap();

                    // Remove the placed bomb from the player's inventory:
                    if let Some(player) = self.game_map.lock().entity_by_id_mut(player_id) {
                        player.item_inventory.take_quantity(items::QuantitativeItem::Bomb, 1);
                    }
                }

                Ok(vec![])
            }

            messages::ToServer::DetonateBombs => {
                {
                    let mut map = self.game_map.lock();

                    let coords = map.entity_by_id(player_id).map(|e| e.pos.as_chunk_coords()).unwrap_or_default();
                    map.take_bombs_placed_by_in_and_around_chunk(player_id, coords);
                }

                self.map_changes_sender.send(maps::Modification::BombsDetonated(player_id)).unwrap();
                self.map_changes_receiver.recv().await.unwrap();

                Ok(vec![])
            }

            messages::ToServer::PurchaseSingleItem(item) => {
                let (cost_gem, cost_quantity) = item.get_price();

                if let Some(entity) = self.game_map.lock().entity_by_id_mut(player_id) {
                    // If the player has enough gems...
                    if entity.gem_collection.get_quantity(cost_gem) >= cost_quantity {
                        // Remove the required number of gems:
                        entity.gem_collection.decrease_quantity(cost_gem, cost_quantity);
                        // Give them their item:
                        entity.item_inventory.give(item);
                    }
                }

                Ok(vec![])
            }

            messages::ToServer::PurchaseItemQuantity { item, quantity } => {
                let (cost_gem, single_cost_quantity) = item.get_price();
                let total_cost_quantity = single_cost_quantity * quantity;

                if let Some(entity) = self.game_map.lock().entity_by_id_mut(player_id) {
                    // If the player has enough gems for the specified quantity of items...
                    if entity.gem_collection.get_quantity(cost_gem) >= total_cost_quantity {
                        // Remove the spent gems:
                        entity.gem_collection.decrease_quantity(cost_gem, total_cost_quantity);
                        // Give the player their quantity of items:
                        entity.item_inventory.give_quantity(item, quantity);
                    }
                }

                Ok(vec![])
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

            maps::Modification::EntityMoved { entity_id, old_position, new_position, direction } => {
                let was_in_loaded = self.remote_loaded_chunk_coords.contains(&old_position.as_chunk_coords());
                let is_in_loaded = self.remote_loaded_chunk_coords.contains(&new_position.as_chunk_coords());

                if was_in_loaded && is_in_loaded {
                    // Entity moving within the bounds of the client's loaded chunks:
                    Some(messages::FromServer::MoveEntity(entity_id, new_position, direction))
                }
                else if was_in_loaded {
                    // Entity moved out of the client's loaded chunks:
                    Some(messages::FromServer::ShouldUnloadEntity(entity_id))
                }
                else if is_in_loaded {
                    // Entity just moved into the client's loaded chunks:
                    self.game_map
                        .lock()
                        .entity_by_id(entity_id)
                        .map(|entity| messages::FromServer::ProvideEntity(entity_id, entity.clone()))
                }
                else {
                    None
                }
            }

            maps::Modification::EntityAdded(entity_id) => {
                self.game_map.lock().entity_by_id(entity_id).and_then(|entity| {
                    self.remote_loaded_chunk_coords
                        .contains(&entity.pos.as_chunk_coords())
                        .then(|| messages::FromServer::ProvideEntity(entity_id, entity.clone()))
                })
            }

            maps::Modification::EntityRemoved(entity_id, chunk_coords) => self
                .remote_loaded_chunk_coords
                .contains(&chunk_coords)
                .then(|| messages::FromServer::ShouldUnloadEntity(entity_id)),

            maps::Modification::BombPlaced(position, placed_by_entity_id) => self
                .remote_loaded_chunk_coords
                .contains(&position.as_chunk_coords())
                .then(|| messages::FromServer::BombPlaced { placed_by_entity_id, position }),

            maps::Modification::BombsDetonated(placed_by_entity_id) => {
                self.game_map.lock().entity_by_id(placed_by_entity_id).map(|entity| {
                    messages::FromServer::BombsDetonated {
                        placed_by_entity_id,
                        in_and_around_chunk_coords: entity.pos.as_chunk_coords()
                    }
                })
            }
        }
    }

    /// Will begin by ensuring the chunk at the specified coordinates is loaded (i.e. if not already in-memory within
    /// the game map object, it will either be loaded from disk or newly generated before being added to the game map).
    /// Messages will then be created to provide the remote client with the chunk as well as any entities in said chunk.
    /// This method will add the given chunk coordinates to the set of remote loaded chunk coordinates however it is
    /// the responsiblity of the caller to actually send the messages returned over the network.
    async fn provide_chunk_with_entities(
        &mut self, coords: ChunkCoords, player_id: Id
    ) -> Result<Vec<messages::FromServer>> {
        let mut msgs = Vec::new();

        // Load new chunk & entities:

        let search = self.remote_loaded_chunk_coords.iter().position(|x| coords == *x);

        if let Some(index) = search {
            // Remote client already has the chunk with those chunk coordinates loaded. Ensure those coordinates are
            // positioned at the end of `remote_loaded_chunk_coords` so that those coordinates will not be unloaded yet:

            self.remote_loaded_chunk_coords.remove(index);
            self.remote_loaded_chunk_coords.push(coords);
        }
        else {
            // The remote client does not already have the chunk loaded so prepare messages to provide the client with
            // the chunks and any entities in that chunk:

            let chunk =
                maps::chunks::get_or_load_or_generate_chunk(self.db_pool.acquire().await?, &self.game_map, coords)
                    .await;
            msgs.push(messages::FromServer::ProvideChunk(coords, chunk));

            // Get entities in the chunk but filter out this task's own player entity:
            let entities_in_chunk =
                self.game_map.lock().entities_in_chunk(coords).into_iter().filter(|(id, _)| *id != player_id);

            for (entity_id, entity) in entities_in_chunk {
                msgs.push(messages::FromServer::ProvideEntity(entity_id, entity));
            }

            self.remote_loaded_chunk_coords.push(coords);
            self.game_map.lock().chunk_in_use(coords);
        }

        // If too many chunks now loaded, unload oldest chunk & entities:

        if self.remote_loaded_chunk_coords.len() > MAX_LOADED_CHUNKS_PER_CLIENT {
            let coords = self.remote_loaded_chunk_coords.remove(0);

            for (entity_id, _) in self.game_map.lock().entities_in_chunk(coords).into_iter() {
                msgs.push(messages::FromServer::ShouldUnloadEntity(entity_id));
            }

            msgs.push(messages::FromServer::ShouldUnloadChunk(coords));
            self.chunk_not_needed(coords).await?;
        }

        Ok(msgs)
    }

    /// Call [`Self::provide_chunk_with_entities`] for the specified chunk coordinates as well as the 9 surrounding set
    /// of coordinates.
    async fn provide_chunks_at_and_surrounding_with_entities(
        &mut self, coords: ChunkCoords, player_id: Id
    ) -> Result<Vec<messages::FromServer>> {
        let mut msgs = Vec::new();

        for x_offset in -1..2 {
            for y_offset in -1..2 {
                let msg = self
                    .provide_chunk_with_entities(
                        ChunkCoords { x: coords.x + x_offset, y: coords.y + y_offset },
                        player_id
                    )
                    .await?;

                msgs.extend(msg);
            }
        }

        Ok(msgs)
    }

    /// Informs the game map that the chunk at the specified chunk coordinates is no longer loaded by this task's
    /// remote client. If it is found that the chunk is at that point not loaded by any clients, then it is saved to
    /// the database and removed from the server's loaded chunks collection.
    async fn chunk_not_needed(&self, coords: ChunkCoords) -> maps::chunks::Result<()> {
        let unloaded_chunk_option = self.game_map.lock().chunk_not_in_use(coords);

        if let Some(unloaded_chunk) = unloaded_chunk_option {
            maps::chunks::save_chunk(self.db_pool.acquire().await?, coords, &unloaded_chunk).await?;
        }

        Ok(())
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
    #[error("Networking error - {0}")]
    Network(#[from] networking::Error),
    #[error("Database error - {0}")]
    Database(#[from] sqlx::Error),
    #[error("Chunk access error - {0}")]
    Chunk(#[from] maps::chunks::Error)
}

type Result<T> = std::result::Result<T, Error>;
