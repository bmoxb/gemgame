use std::{convert::Into, net::SocketAddr};

use shared::{messages, Id};
use thiserror::Error;
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::tungstenite;

use crate::{
    networking::{self, Connection},
    world::{self, entities::PlayerEntity, World},
    ReceivedOn, Shared
};

/// Handle a connection with an individual client. This function is called concurrently as a Tokio task.
pub async fn handle_connection(
    stream: TcpStream, addr: SocketAddr, world: Shared<World>, db_pool: sqlx::SqlitePool,
    world_changes_receiver: broadcast::Receiver<world::Modification>
) {
    // Perform the WebSocket handshake:

    match Connection::new(stream).await {
        Ok(ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            // Handle the connection should the handshake complete successfully:

            match handle_websocket_connection(ws, &addr, world, db_pool, world_changes_receiver).await {
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
    mut ws: Connection, addr: &SocketAddr, world: Shared<World>, db_pool: sqlx::SqlitePool,
    world_changes_receiver: broadcast::Receiver<world::Modification>
) -> Result<()> {
    // Expect a 'hello' message from the client:

    if let Some(messages::ToServer::Hello { client_id_option }) = ws.receive().await? {
        let (client_id, player_entity) = {
            let mut db = db_pool.acquire().await.unwrap();

            if let Some(client_id) = client_id_option {
                log::debug!("Client {} has existing ID: {}", addr, client_id);

                // Get the client their existing player entity (if any) from database:

                if let Some(entity) = PlayerEntity::from_database(client_id, &mut db).await? {
                    (client_id, entity)
                }
                else {
                    log::warn!(
                        "Client {} provided {} for which an associate entity could not be found in the database",
                        addr,
                        client_id
                    );
                    (client_id, PlayerEntity::new_to_database(client_id, &mut db).await?)
                }
            }
            else {
                let new_id = crate::id::generate_random();
                log::debug!("Generated new ID {} for client {}", new_id, addr);

                // Create a new entity for this client and insert into the database:

                (new_id, PlayerEntity::new_to_database(new_id, &mut db).await?)
            }
        };

        // Send a 'welcome' message to the client:

        ws.send(&messages::FromServer::Welcome {
            version: shared::VERSION.to_string(),
            your_client_id: client_id,
            your_entity: player_entity.inner_entity_cloned()
        })
        .await?;

        let result = handle_established_connection(&mut ws, client_id, world, world_changes_receiver).await;

        // Update database with changes to user's entity:

        let mut db = db_pool.acquire().await.unwrap();
        player_entity.update_database(client_id, &mut db).await?;

        result
    }
    else {
        log::warn!("Client {} failed to send 'hello' message after establishing a WebSocket connection", addr);
        ws.close().await.map_err(Into::into)
    }
}

/// A connection is considered 'established' once the WebSocket handshake and the the exchange of 'hello' & 'welcome'
/// messages have completed.
async fn handle_established_connection(
    ws: &mut Connection, client_id: Id, world: Shared<World>,
    mut world_changes_receiver: broadcast::Receiver<world::Modification>
) -> Result<()> {
    // Wait for incoming messages on both the WebSocket connection and the world modifications channel (or close
    // connection on Ctrl-C signal):

    while let Some(src) = tokio::select!(
        res = ws.receive() => res?.map(ReceivedOn::NetworkConnection),
        res = world_changes_receiver.recv() => Some(ReceivedOn::TokioBroadcast(res)),
        _ = tokio::signal::ctrl_c() => {
            log::debug!("Closing connection with client {} due to Ctrl-C signal", client_id);
            ws.close().await?;
            None
        }
    ) {
        match src {
            ReceivedOn::NetworkConnection(msg) => {
                log::debug!("Message from client {} - {}", client_id, msg);

                // Handle and respond to received message:

                if let Some(response) = handle_message(msg, client_id, &world).await {
                    log::debug!("Response to client {} - {}", client_id, response);
                    ws.send(&response).await?;
                }
            }

            ReceivedOn::TokioBroadcast(modification) => {
                // TODO: Inform client of changes made to the world if affecting chunks that client has loaded.
            }
        }
    }

    Ok(())
}

async fn handle_message(msg: messages::ToServer, client_id: Id, world: &Shared<World>) -> Option<messages::FromServer> {
    match msg {
        messages::ToServer::Hello { .. } => {
            log::warn!("Client {} sent unexpected 'hello' message: {}", client_id, msg);
            None
        }
        messages::ToServer::RequestChunk(coords) => {
            /*
            let loaded_chunk_option =
                with_current_map(connections, addr, world, |map| map.loaded_chunk_at(coords).cloned());

            let chunk = {
                if let Some(loaded_chunk) = loaded_chunk_option {
                    log::debug!("Chunk at {} already loaded", coords);

                    loaded_chunk
                }
                else {
                    let directory = with_current_map(connections, addr, world, |map| map.directory.clone());

                    let new_chunk = maps::chunks::load_chunk(&directory, coords).await.unwrap_or_else(|_| {
                        log::debug!("Generated chunk at {}", coords);

                        with_current_map(connections, addr, world, |map| map.generator.generate(coords))
                    });

                    with_current_map(connections, addr, world, |map| map.provide_chunk(coords, new_chunk.clone()));

                    new_chunk
                }
            };

            messages::FromServer::ProvideChunk(coords, chunk)*/
            unimplemented!()
        }

        messages::ToServer::ChunkUnloadedLocally(_coords) => {
            unimplemented!()
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
