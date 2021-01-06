use std::net::SocketAddr;

use shared::{maps::Map, messages};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;

use crate::{
    networking::{self, Connection},
    world::{maps, World},
    ConnectionRecord, ConnectionRecords, Shared
};

/// Handle a connection with an individual client. This function is called concurrently as a Tokio task.
pub async fn handle_connection(
    stream: TcpStream, addr: SocketAddr, connections: Shared<ConnectionRecords>, world: Shared<World>
) {
    // Perform the WebSocket handshake:

    match Connection::new(stream).await {
        Ok(mut ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            // Register this connection between threads:

            connections.lock().unwrap().insert(addr, ConnectionRecord { current_map_key: "surface".to_string() });

            match handle_websocket_connection(&mut ws, &addr, &connections, &world).await {
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

            // Remove this connection from the shared connections hash map:

            connections.lock().unwrap().remove(&addr);

            log::info!("Client disconnected: {}", addr);
        }

        Err(e) => {
            log::warn!("Failed to perform WebSocket handshake with {} - {}", addr, e);
        }
    }
}

async fn handle_websocket_connection(
    ws: &mut Connection, addr: &SocketAddr, connections: &Shared<ConnectionRecords>, world: &Shared<World>
) -> networking::Result<()> {
    // Inform the client of the server's version:

    let welcome_msg = messages::FromServer::Welcome { version: shared::VERSION.to_string() };
    ws.send(&welcome_msg).await?;

    // Wait for incoming messages:

    while let Some(msg) = ws.receive().await? {
        log::debug!("Message from client {} - {}", addr, msg);

        let response = handle_message(msg, addr, connections, world).await;
        log::debug!("Response to client {} - {}", addr, response);

        ws.send(&response).await?;
    }

    Ok(())
}

async fn handle_message(
    msg: messages::ToServer, addr: &SocketAddr, connections: &Shared<ConnectionRecords>, world: &Shared<World>
) -> messages::FromServer {
    match msg {
        messages::ToServer::RequestChunk(coords) => {
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

            messages::FromServer::ProvideChunk(coords, chunk)
        }

        messages::ToServer::ChunkUnloadedLocally(_coords) => {
            unimplemented!()
        }
    }
}

fn with_current_map<T>(
    connections: &Shared<ConnectionRecords>, addr: &SocketAddr, world: &Shared<World>,
    func: impl Fn(&mut maps::ServerMap) -> T
) -> T {
    let connections_lock = connections.lock().unwrap();
    let map_key = &connections_lock.get(addr).unwrap().current_map_key;

    let mut world_lock = world.lock().unwrap();
    let map = world_lock.get_map_mut(map_key).unwrap(); // TODO: Key may be invalid.

    func(map)
}
