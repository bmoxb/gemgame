use crate::{ Shared, Connection, Connections, world::{ maps, World } };

use std::{
    net::SocketAddr,
    sync::{ Arc, Mutex, MutexGuard }
};

use core::{ messages, maps::Map };

use futures_util::{ SinkExt, StreamExt };
use tokio::net::TcpStream;
use tokio_tungstenite::{ tungstenite, WebSocketStream };

/// Handle a connection with an individual client. This function is called
/// concurrently as a Tokio task.
pub async fn handle_connection(stream: TcpStream, addr: SocketAddr, connections: Shared<Connections>, world: Shared<World>) {
    // Perform the WebSocket handshake:

    match tokio_tungstenite::accept_async(stream).await {
        Ok(mut ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            // Register this connection between threads:

            connections.lock().unwrap().insert(
                addr,
                Connection {
                    current_map_key: "surface".to_string()
                }
            );

            // Inform the client of the server's version:

            let welcome_msg = messages::FromServer::Welcome { version: core::VERSION.to_string() };
            let encoded = bincode::serialize(&welcome_msg).unwrap();
            ws.send(tungstenite::Message::Binary(encoded)).await.unwrap(); // TODO

            // Wait for messages over the connection:

            while let Some(ws_msg_option) = ws.next().await {
                match ws_msg_option {
                    Ok(tungstenite::Message::Binary(data)) => {
                        log::trace!("Binary data from client {} - {:?}", addr, data);

                        // Deserialise the message:

                        match bincode::deserialize::<messages::ToServer>(data.as_slice()) {
                            Ok(msg) => {
                                log::debug!("Message from client {} - {}", addr, msg);

                                let response = handle_message(msg, &mut ws, &addr, &connections, &world).await;

                                log::debug!("Response to client {} - {}", addr, response);

                                // TODO: Serialise and send response.
                            }

                            Err(bincode_error) => {
                                log::warn!("Failed to decode message from client {} - {}",
                                           addr, bincode_error);
                            }
                        }
                    }

                    Ok(tungstenite::Message::Close(_)) => {
                        log::debug!("Closing message from client {}", addr);

                        let _ = ws.close(None).await;
                        break;
                    }

                    Ok(not_binary_msg) => {
                        log::warn!("Message from {} is not binary: {}", addr, not_binary_msg);
                    }

                    Err(tungstenite::Error::Protocol(vioation)) if vioation.contains("closing handshake") => {
                        log::debug!("Client {} closed connection without performing the closing handshake", addr);
                        break;
                    }

                    Err(ws_error) => {
                        log::warn!("Failed to receive message from client {} - {}",
                                   addr, ws_error);
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

async fn handle_message(msg: messages::ToServer, ws: &mut WebSocketStream<TcpStream>, addr: &SocketAddr, connections: &Shared<Connections>, world: &Shared<World>) -> messages::FromServer {
    match msg {
        messages::ToServer::RequestChunk(coords) => {
            let loaded_chunk_option = with_current_map(connections, addr, world, |map| map.loaded_chunk_at(coords).cloned());

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

        messages::ToServer::ChunkUnloadedLocally(coords) => {
            unimplemented!()
        }
    }
}

fn with_current_map<T>(connections: &Shared<Connections>, addr: &SocketAddr, world: &Shared<World>, func: impl Fn(&mut maps::ServerMap) -> T) -> T {
    let connections_lock = connections.lock().unwrap();
    let map_key = &connections_lock.get(addr).unwrap().current_map_key;

    let mut world_lock = world.lock().unwrap();
    let map = world_lock.get_map_mut(map_key).unwrap(); // TODO: Key may be invalid.

    func(map)
}