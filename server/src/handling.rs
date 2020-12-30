use super::Shared;

use std::{
    net::SocketAddr,
    sync::{ Arc, Mutex }
};

use core::messages;

use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{ tungstenite, WebSocketStream };

/// Handle a connection with an individual client. This function is called
/// concurrently as a Tokio task.
pub async fn handle_connection(stream: TcpStream, addr: SocketAddr, shared: Arc<Mutex<Shared>>) {
    match tokio_tungstenite::accept_async(stream).await {
        Ok(mut ws) => {
            log::debug!("Performed WebSocket handshake successfully with: {}", addr);

            while let Some(ws_msg_option) = ws.next().await {
                match ws_msg_option {
                    Ok(tungstenite::Message::Binary(data)) => {
                        log::trace!("Binary data from client {} - {:?}", addr, data);

                        match bincode::deserialize::<messages::ToServer>(data.as_slice()) {
                            Ok(msg) => {
                                log::debug!("Message from client {} - {}", addr, msg);

                                handle_message(msg, &mut ws, &addr, &shared).await;
                            }

                            Err(bincode_error) => {
                                log::warn!("Failed to decode message from client {} - {}",
                                           addr, bincode_error);
                            }
                        }
                    }

                    Ok(not_binary_msg) => {
                        log::warn!("Message from {} is not binary: {}", addr, not_binary_msg);
                    }

                    Err(tungstenite::Error::ConnectionClosed) => { break; }

                    Err(ws_error) => {
                        log::warn!("Failed to receive message from client {} - {}",
                                   addr, ws_error);
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

async fn handle_message(msg: messages::ToServer, ws: &mut WebSocketStream<TcpStream>, addr: &SocketAddr, shared: &Arc<Mutex<Shared>>) {
    match msg {
        messages::ToServer::RequestChunk(coords) => {
            // shared.lock().unwrap().game_world.chunk_at(coords);
        }

        messages::ToServer::ChunkUnloadedLocally(coords) => {
            // ...
        }
    }
}