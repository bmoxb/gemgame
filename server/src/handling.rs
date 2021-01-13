use std::net::SocketAddr;

use shared::{messages, world::maps::Map, Id};
use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::tungstenite;

use crate::{
    generate_id,
    networking::{self, Connection},
    world::{self, maps, World},
    Shared
};

/// Handle a connection with an individual client. This function is called concurrently as a Tokio task.
pub async fn handle_connection(
    stream: TcpStream, addr: SocketAddr, world: Shared<World>,
    world_changes_receiver: broadcast::Receiver<world::Modification>
) {
    // Perform the WebSocket handshake:

    match Connection::new(stream).await {
        Ok(ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            // Handle the connection should the handshake complete successfully:

            match handle_websocket_connection(ws, &addr, world, world_changes_receiver).await {
                Ok(_) => {}

                Err(e) => match e {
                    networking::Error::MessageNotBinary(msg) => {
                        log::warn!("Message from {} is not binary: {}", addr, msg);
                    }

                    networking::Error::EncodingFailure(bincode_err) => {
                        log::warn!("Failed to communicate with client {} due to encoding error: {}", addr, bincode_err);
                    }

                    networking::Error::NetworkError(network_err) => match network_err {
                        tungstenite::Error::Protocol(vioation) if vioation.contains("closing handshake") => {
                            log::debug!("Client {} closed connection without performing the closing handshake", addr);
                        }

                        other => {
                            log::warn!("Failed to communicate with client {} due to network error: {}", addr, other);
                        }
                    }
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
    mut ws: Connection, addr: &SocketAddr, world: Shared<World>,
    world_changes_receiver: broadcast::Receiver<world::Modification>
) -> networking::Result<()> {
    // Expect a 'hello' message from the client:

    if let Some(messages::ToServer::Hello { client_id_option }) = ws.receive().await? {
        /*let (client_id, entity) = {
            if let Some(client_id) = client_id_option {
                log::debug!("Client {} has existing ID: {}", addr, client_id);
                // TODO: Get the client their existing player entity (if any) from database.
            }
            else {
                let new_id = generate_id();
                log::debug!("Generated new ID {} for client {}", new_id, addr);
                // TODO: Create a new player entity for this client and insert into database.
            }
        };*/
        let client_id = Id::new(0);

        // Send a 'welcome' message to the client:
        // TODO: Send the welcome message.

        let result = handle_established_connection(&mut ws, client_id, world, world_changes_receiver).await;

        result
    }
    else {
        log::warn!("Client {} failed to send 'hello' message after establishing a WebSocket connection", addr);
        ws.close().await
    }
}

/// A connection is considered 'established' once the WebSocket handshake and the the exchange of 'hello' & 'welcome'
/// messages have completed.
async fn handle_established_connection(
    ws: &mut Connection, client_id: Id, world: Shared<World>,
    world_changes_receiver: broadcast::Receiver<world::Modification>
) -> networking::Result<()> {
    // Wait for incoming messages (or close connection on Ctrl-C signal):

    while let Some(msg) = tokio::select!(
        res = ws.receive() => res?,
        _ = tokio::signal::ctrl_c() => {
            log::debug!("Closing connection with client {} due to Ctrl-C signal", client_id);
            ws.close().await?;
            None
        }
    ) {
        log::debug!("Message from client {} - {}", client_id, msg);

        // Handle and respond to received message:

        let response = handle_message(msg, client_id, &world).await;
        log::debug!("Response to client {} - {}", client_id, response);

        ws.send(&response).await?;
    }

    Ok(())
}

async fn handle_message(msg: messages::ToServer, client_id: Id, world: &Shared<World>) -> messages::FromServer {
    match msg {
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
