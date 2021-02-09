use std::{collections::HashSet, convert::Into, net::SocketAddr};

use shared::{
    maps::{ChunkCoords, Map},
    messages, Id
};
use thiserror::Error;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::tungstenite;

use crate::{
    maps::{self, entities::PlayerEntity, ServerMap},
    networking::{self, Connection},
    Shared
};

/// Handle a connection with an individual client. This function is called concurrently as a Tokio task.
pub async fn handle_connection(
    stream: TcpStream, addr: SocketAddr, map: Shared<ServerMap>, db_pool: sqlx::SqlitePool,
    map_changes_receiver: broadcast::Receiver<maps::Modification>
) {
    // Perform the WebSocket handshake:

    match Connection::new(stream).await {
        Ok(ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            // Handle the connection should the handshake complete successfully:

            match handle_websocket_connection(ws, &addr, map, db_pool, map_changes_receiver).await {
                Ok(_) => {}

                Err(Error::NetworkError(e)) => match e {
                    networking::Error::MessageNotBinary(msg) => {
                        log::error!("Message from {} is not binary: {}", addr, msg);
                    }

                    networking::Error::EncodingFailure(bincode_err) => {
                        log::error!(
                            "Failed to communicate with client {} due to encoding error: {}",
                            addr,
                            bincode_err
                        );
                    }

                    networking::Error::NetworkError(network_err) => match network_err {
                        tungstenite::Error::Protocol(vioation) if vioation.contains("closing handshake") => {
                            log::debug!("Client {} closed connection without performing the closing handshake", addr);
                        }

                        other => {
                            log::error!("Failed to communicate with client {} due to network error: {}", addr, other);
                        }
                    }
                },

                Err(Error::DatabaseError(e)) => {
                    log::error!("Handling client {} resulted in database error: {}", addr, e);
                }
            }

            log::info!("Client disconnected: {}", addr);
        }

        Err(e) => {
            log::warn!("Failed to perform WebSocket handshake with {} - {}", addr, e);
        }
    }
}

/// This function is to be called after the WebSocket connection handshake finishes. It is the role of this function to
/// complete the exchange of 'hello' and 'welcome' messages between client and server before passing control onto the
/// [`handle_established_connection`] function.
async fn handle_websocket_connection(
    mut ws: Connection, addr: &SocketAddr, map: Shared<ServerMap>, db_pool: sqlx::SqlitePool,
    map_changes_receiver: broadcast::Receiver<maps::Modification>
) -> Result<()> {
    // Expect a 'hello' message from the client:

    if let Some(messages::ToServer::Hello { client_id_option }) = ws.receive().await? {
        let (client_id, player_id, player_entity) = {
            let mut db = db_pool.acquire().await.unwrap();

            if let Some(client_id) = client_id_option {
                log::debug!("Client {} has existing ID: {}", addr, client_id);

                // Get the client their existing player entity (if any) from database:

                if let Some((entity_id, entity)) = PlayerEntity::from_database(client_id, &mut db).await? {
                    (client_id, entity_id, entity)
                }
                else {
                    log::warn!(
                        "Client {} provided {} for which an associate entity could not be found in the database",
                        addr,
                        client_id
                    );

                    let (entity_id, entity) = PlayerEntity::new_to_database(client_id, &mut db).await?;
                    (client_id, entity_id, entity)
                }
            }
            else {
                let new_id = crate::id::generate_random();
                log::debug!("Generated new ID {} for client {}", new_id, addr);

                // Create a new entity for this client and insert into the database:

                let (new_entity_id, new_entity) = PlayerEntity::new_to_database(new_id, &mut db).await?;
                (new_id, new_entity_id, new_entity)
            }
        };

        // Send a 'welcome' message to the client:

        ws.send(&messages::FromServer::Welcome {
            version: shared::VERSION.to_string(),
            your_client_id: client_id,
            your_entity_with_id: (player_id, player_entity.inner_entity_cloned())
        })
        .await?;

        // Place this client's player entity in the game world:

        map.lock().unwrap().add_player_entity(player_id, player_entity);

        // Begin main connection loop:

        let result = handle_established_connection(&mut ws, client_id, player_id, &map, map_changes_receiver).await;

        // Remove this client's player entity from the game world and update database with changes to said entity:

        let entity_option = map.lock().unwrap().remove_player_entity(player_id);
        if let Some(player_entity) = entity_option {
            let mut db = db_pool.acquire().await.unwrap();
            player_entity.update_database(client_id, &mut db).await?;
        }

        result
    }
    else {
        log::warn!("Client {} failed to send 'hello' message after establishing a WebSocket connection", addr);
        ws.close().await.map_err(Into::into)
    }
}

/// A connection is considered 'established' once the WebSocket handshake and the exchange of 'hello' & 'welcome'
/// messages have completed.
async fn handle_established_connection(
    ws: &mut Connection, client_id: Id, player_id: Id, map: &Shared<ServerMap>,
    mut map_changes_receiver: broadcast::Receiver<maps::Modification>
) -> Result<()> {
    // The server keeps track of the  coordinates of chunks that it believes each remote client has loaded using a hash
    // set:
    let mut remote_loaded_chunk_coords = HashSet::new();

    loop {
        // Wait for incoming messages on both the WebSocket connection and the world modifications channel (or close
        // connection on Ctrl-C signal):
        tokio::select!(
            res = ws.receive() => {
                if let Some(msg) = res? {
                    log::debug!("Message from client {} - {}", client_id, msg);

                    // Handle and respond to received message:

                    if let Some(response) = handle_message(msg, client_id, player_id, &map, &mut remote_loaded_chunk_coords).await {
                        log::debug!("Response to client {} - {}", client_id, response);
                        ws.send(&response).await?;
                    }
                }
                else {
                    log::debug!("Connection closed by client {}", client_id);
                    break;
                }
            }

            res = map_changes_receiver.recv() => {
                match res {
                    Ok(modification) => {
                        let response = handle_map_change(modification, &remote_loaded_chunk_coords).await;

                        log::debug!("Informing client {} of change to game world - todo", client_id);
                        // TODO: ws.send(...).await?;
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

async fn handle_message(
    msg: messages::ToServer, client_id: Id, player_id: Id, map: &Shared<ServerMap>,
    remote_loaded_chunk_coords: &mut HashSet<ChunkCoords>
) -> Option<messages::FromServer> {
    match msg {
        messages::ToServer::Hello { .. } => {
            log::warn!("Client {} sent unexpected 'hello' message: {}", client_id, msg);
            None
        }
        messages::ToServer::RequestChunk(coords) => {
            let loaded_chunk_option = map.lock().unwrap().loaded_chunk_at(coords).cloned();

            let chunk = {
                if let Some(loaded_chunk) = loaded_chunk_option {
                    log::debug!("Chunk at {} already loaded", coords);

                    loaded_chunk
                }
                else {
                    // Chunk is not already in memory so needs to either be read from disk or newly generated before
                    // being loaded into the map.

                    let directory = map.lock().unwrap().directory.clone();

                    let new_chunk = maps::chunks::load_chunk(&directory, coords).await.unwrap_or_else(|_| {
                        let generator = &map.lock().unwrap().generator;

                        log::debug!(
                            "Chunk at {} could not be found on disk so will be newly generated using generator '{}'",
                            coords,
                            generator.name()
                        );

                        generator.generate(coords)
                    });

                    // Add the new chunk to map's loaded chunks:
                    map.lock().unwrap().provide_chunk(coords, new_chunk.clone());

                    new_chunk
                }
            };

            // Keep track of which chunks the remote client has loaded:
            remote_loaded_chunk_coords.insert(coords);

            Some(messages::FromServer::ProvideChunk(coords, chunk))
        }

        messages::ToServer::ChunkUnloadedLocally(coords) => {
            log::debug!("Client has locally unloaded chunk at {}", coords);
            remote_loaded_chunk_coords.remove(&coords);
            None
        }

        messages::ToServer::MoveMyEntity { request_number, direction } => {
            if let Some(player) = map.lock().unwrap().player_entity_by_id(player_id) {
                // TODO: Check for blocking tiles/entities...
                // TODO: Prevent player exceeding movement rate...

                // Modify player coordinates:
                player.move_towards_unchecked(direction);

                // TODO: Broadcast change on world modification channel...

                Some(messages::FromServer::YourEntityMoved { request_number, new_position: player.position() })
            }
            else {
                None
            }
        }
    }
}

async fn handle_map_change(
    modification: maps::Modification, remote_loaded_chunk_coords: &HashSet<ChunkCoords>
) -> messages::FromServer {
    unimplemented!()
}

#[derive(Error, Debug)]
enum Error {
    #[error("Networking error")]
    NetworkError(#[from] networking::Error),
    #[error("Database error")]
    DatabaseError(#[from] sqlx::Error)
}

type Result<T> = std::result::Result<T, Error>;
